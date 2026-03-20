// SPDX-License-Identifier: MIT OR Apache-2.0
//! Draw task wrappers for `LV_EVENT_DRAW_TASK_ADDED` handlers.
//!
//! All types in this module are **callback-scoped**: valid only during the
//! `DRAW_TASK_ADDED` event callback. Storing them beyond that scope is
//! undefined behaviour (LVGL frees the draw task after the callback returns).
//!
//! See `docs/spec-memory-lifetime.md` §2 for the lifetime table.

use lvgl_rust_sys::*;

use crate::widgets::Part;

/// Rectangle area (x1, y1, x2, y2) — value copy of `lv_area_t`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Area {
    /// Left edge.
    pub x1: i32,
    /// Top edge.
    pub y1: i32,
    /// Right edge.
    pub x2: i32,
    /// Bottom edge.
    pub y2: i32,
}

impl Area {
    /// Width of this area.
    pub fn width(&self) -> i32 {
        self.x2 - self.x1 + 1
    }

    /// Height of this area.
    pub fn height(&self) -> i32 {
        self.y2 - self.y1 + 1
    }

    /// Adjust width symmetrically around center. Distributes the delta
    /// equally on both sides (extra pixel goes to the right).
    pub fn set_width_centered(&mut self, new_w: i32) {
        let old_w = self.width();
        let delta = new_w - old_w;
        self.x1 -= delta / 2;
        self.x2 += (delta + 1) / 2;
    }
}

impl From<lv_area_t> for Area {
    fn from(a: lv_area_t) -> Self {
        Self { x1: a.x1, y1: a.y1, x2: a.x2, y2: a.y2 }
    }
}

impl From<Area> for lv_area_t {
    fn from(a: Area) -> Self {
        Self { x1: a.x1, y1: a.y1, x2: a.x2, y2: a.y2 }
    }
}

/// Non-owning handle to an LVGL draw task.
///
/// Valid only during a `DRAW_TASK_ADDED` event callback — LVGL owns the
/// task and frees it after the callback returns.
pub struct DrawTask {
    ptr: *mut lv_draw_task_t,
}

impl core::fmt::Debug for DrawTask {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DrawTask").finish_non_exhaustive()
    }
}

impl DrawTask {
    /// Create from a raw pointer (called by `Event::draw_task()`).
    pub(crate) fn from_raw(ptr: *mut lv_draw_task_t) -> Self {
        Self { ptr }
    }

    /// Base draw descriptor (part, id1, id2). Returns a value copy.
    pub fn base(&self) -> DrawDscBase {
        // SAFETY: ptr valid during callback; lv_draw_task_get_draw_dsc returns
        // a pointer to the descriptor embedded in the task (lv_draw.c).
        let dsc = unsafe { lv_draw_task_get_draw_dsc(self.ptr) };
        let base = unsafe { &*(dsc as *const lv_draw_dsc_base_t) };
        DrawDscBase { part: Part::from_raw(base.part), id1: base.id1, id2: base.id2 }
    }

    /// Label draw descriptor, if this task draws a label.
    ///
    /// Returns `None` if the task is not a label draw operation.
    pub fn label_dsc(&self) -> Option<DrawLabelDsc> {
        // SAFETY: ptr valid during callback; returns null if not a label task.
        let dsc = unsafe { lv_draw_task_get_label_dsc(self.ptr) };
        if dsc.is_null() { None } else { Some(DrawLabelDsc { ptr: dsc }) }
    }

    /// Current draw area (copy).
    pub fn area(&self) -> Area {
        // SAFETY: ptr valid during callback.
        let task = unsafe { &*self.ptr };
        task.area.into()
    }

    /// Overwrite the draw area.
    pub fn set_area(&self, area: Area) {
        // SAFETY: ptr valid during callback; area is a plain value field.
        let task = unsafe { &mut *self.ptr };
        task.area = area.into();
    }

    /// Get the draw layer from this task's base descriptor.
    ///
    /// Required for `DRAW_TASK_ADDED` handlers that call [`Layer::draw_rect`] /
    /// [`Layer::draw_label`]. Valid only during the callback.
    pub fn layer(&self) -> Option<Layer> {
        // SAFETY: ptr valid during callback; lv_draw_task_get_draw_dsc returns
        // a pointer to the embedded base descriptor (lv_draw.c).
        let dsc = unsafe { lv_draw_task_get_draw_dsc(self.ptr) };
        if dsc.is_null() {
            return None;
        }
        // SAFETY: dsc is a valid lv_draw_dsc_base_t * (first field in every draw dsc).
        let base = unsafe { &*(dsc as *const lv_draw_dsc_base_t) };
        if base.layer.is_null() { None } else { Some(Layer::from_raw(base.layer)) }
    }
}

/// Base draw descriptor fields (value copy — no pointer risk).
#[derive(Clone, Copy, Debug)]
pub struct DrawDscBase {
    /// Which widget part this draw task belongs to.
    pub part: Part,
    /// First identifier (e.g. tick index for scales).
    pub id1: u32,
    /// Second identifier (e.g. tick value for scales).
    pub id2: u32,
}

/// Mutable handle to a label draw descriptor.
///
/// Valid only during the `DRAW_TASK_ADDED` callback. Modifications take
/// effect on the current draw operation.
pub struct DrawLabelDsc {
    ptr: *mut lv_draw_label_dsc_t,
}

impl core::fmt::Debug for DrawLabelDsc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DrawLabelDsc").finish_non_exhaustive()
    }
}

impl DrawLabelDsc {
    /// Current label color.
    pub fn color(&self) -> lv_color_t {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).color }
    }

    /// Set the label color.
    pub fn set_color(&self, color: lv_color_t) {
        // SAFETY: ptr valid during callback; color is a plain value field.
        unsafe { (*self.ptr).color = color };
    }

    /// Current label text, or `None` if the pointer is null.
    pub fn text(&self) -> Option<&str> {
        // SAFETY: ptr valid during callback; text pointer valid for callback duration.
        let text_ptr = unsafe { (*self.ptr).text };
        if text_ptr.is_null() {
            return None;
        }
        // SAFETY: LVGL label text is valid UTF-8 (ASCII subset).
        let cstr = unsafe { core::ffi::CStr::from_ptr(text_ptr) };
        cstr.to_str().ok()
    }

    /// Replace the label text with an LVGL-allocated copy.
    ///
    /// Handles `lv_free`/`lv_strdup`/`text_local` internally. The previous
    /// text is freed if it was locally allocated. LVGL will free the new
    /// text after the draw operation completes.
    pub fn set_text(&self, text: &str) {
        // SAFETY: ptr valid during callback.
        let dsc = unsafe { &mut *self.ptr };

        // Free previous text if locally allocated.
        if dsc.text_local() != 0 {
            unsafe { lv_free(dsc.text as *mut core::ffi::c_void) };
        }

        // Build null-terminated buffer on stack, then lv_strdup.
        let mut buf = [0u8; 32];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;

        // SAFETY: lv_strdup allocates via LVGL's allocator; LVGL frees it
        // when text_local is set.
        dsc.text = unsafe { lv_strdup(buf.as_ptr() as *const core::ffi::c_char) };
        dsc.set_text_local(1);
    }

    /// Current font pointer.
    pub fn font(&self) -> *const lv_font_t {
        // SAFETY: ptr valid during callback.
        unsafe { (*self.ptr).font }
    }

    /// Measure the pixel size of a text string using this descriptor's font.
    ///
    /// Returns `(width, height)`. Uses letter_space=0, line_space=0,
    /// max_width=1000, no flags — matching the LVGL example pattern.
    pub fn text_size(&self, text: &str) -> (i32, i32) {
        let mut buf = [0u8; 32];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        let mut size: lv_point_t = unsafe { core::mem::zeroed() };
        // SAFETY: font pointer valid during callback; buf is null-terminated.
        unsafe {
            lv_text_get_size(
                &mut size,
                buf.as_ptr() as *const core::ffi::c_char,
                (*self.ptr).font,
                0,    // letter_space
                0,    // line_space
                1000, // max_width
                lv_text_flag_t_LV_TEXT_FLAG_NONE,
            );
        }
        (size.x, size.y)
    }
}

// ── Area utilities
// ────────────────────────────────────────────────────────────

impl Area {
    /// Align this area relative to `base` by `align`, then offset by `(ofs_x,
    /// ofs_y)`.
    ///
    /// Equivalent to `lv_area_align(base, self, align, ofs_x, ofs_y)`.
    pub fn align_to_area(&mut self, base: Area, align: crate::widgets::Align, ofs_x: i32, ofs_y: i32) {
        let base_lv: lv_area_t = base.into();
        let mut self_lv: lv_area_t = (*self).into();
        // SAFETY: both areas are valid stack values; lv_area_align writes to self_lv.
        unsafe { lv_area_align(&base_lv, &mut self_lv, align as lv_align_t, ofs_x, ofs_y) };
        *self = self_lv.into();
    }

    /// Set width from the left edge (x2 = x1 + new_w - 1). x1 is unchanged.
    pub fn set_width(&mut self, new_w: i32) {
        self.x2 = self.x1 + new_w - 1;
    }
}

/// Maximum radius constant — produces a circle or fully-rounded corners.
/// Equivalent to `LV_RADIUS_CIRCLE` (0x7FFF).
pub const RADIUS_CIRCLE: i32 = 0x7FFF;

// ── Layer ─────────────────────────────────────────────────────────────────────

/// Non-owning handle to an LVGL draw layer.
///
/// Valid only during a draw event callback. Obtain via
/// [`Event::layer`](crate::event::Event::layer).
pub struct Layer {
    ptr: *mut lv_layer_t,
}

impl core::fmt::Debug for Layer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Layer").finish_non_exhaustive()
    }
}

impl Layer {
    /// Create from a raw LVGL layer pointer (used by `Event::layer()`).
    pub(crate) fn from_raw(ptr: *mut lv_layer_t) -> Self {
        Self { ptr }
    }

    /// Draw a filled rectangle onto this layer.
    ///
    /// The descriptor is passed by reference; `area` is a copy.
    pub fn draw_rect(&self, dsc: &DrawRectDsc, area: Area) {
        let area_lv: lv_area_t = area.into();
        // SAFETY: ptr valid during callback; dsc and area_lv are stack values.
        unsafe { lv_draw_rect(self.ptr, &dsc.inner, &area_lv) };
    }

    /// Draw a text label onto this layer.
    ///
    /// `text` must fit in 63 bytes; longer strings are truncated.
    /// The text pointer is valid only for the duration of this call.
    pub fn draw_label(&self, dsc: &DrawLabelDscOwned, area: Area, text: &str) {
        let mut buf = [0u8; 64];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        // lv_draw_label_dsc_t derives Copy — simple copy is safe.
        let mut local_dsc = dsc.inner;
        local_dsc.text = buf.as_ptr() as *const _;
        local_dsc.set_text_local(1);
        let area_lv: lv_area_t = area.into();
        // SAFETY: ptr valid during callback; text_local=1 means lv_draw_label calls
        // lv_strndup when queuing the task, so buf need only live until this call returns.
        unsafe { lv_draw_label(self.ptr, &local_dsc, &area_lv) };
    }
}

// ── DrawRectDsc
// ───────────────────────────────────────────────────────────────

/// Owned LVGL rectangle draw descriptor.
///
/// Initialised via `lv_draw_rect_dsc_init`. Pass to [`Layer::draw_rect`].
pub struct DrawRectDsc {
    inner: lv_draw_rect_dsc_t,
}

impl DrawRectDsc {
    /// Create with LVGL defaults (`lv_draw_rect_dsc_init`).
    pub fn new() -> Self {
        // SAFETY: zeroed memory is a valid starting state; lv_draw_rect_dsc_init
        // fills all required fields (lv_draw.c).
        let mut inner = unsafe { core::mem::zeroed() };
        unsafe { lv_draw_rect_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Set background color.
    pub fn bg_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.bg_color = color;
        self
    }

    /// Set corner radius. Use `RADIUS_CIRCLE` (0x7FFF) for a full circle.
    pub fn radius(&mut self, r: i32) -> &mut Self {
        self.inner.radius = r;
        self
    }

    /// Set border color.
    pub fn border_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.border_color = color;
        self
    }

    /// Set border width in pixels.
    pub fn border_width(&mut self, w: i32) -> &mut Self {
        self.inner.border_width = w;
        self
    }

    /// Set outline color.
    pub fn outline_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.outline_color = color;
        self
    }

    /// Set outline width in pixels.
    pub fn outline_width(&mut self, w: i32) -> &mut Self {
        self.inner.outline_width = w;
        self
    }

    /// Set gap between the object border and the outline.
    pub fn outline_pad(&mut self, pad: i32) -> &mut Self {
        self.inner.outline_pad = pad;
        self
    }

    /// Raw pointer to the inner descriptor. Used by [`CanvasLayer::draw_rect`].
    pub(crate) fn as_ptr(&self) -> *const lv_draw_rect_dsc_t {
        &self.inner
    }
}

impl Default for DrawRectDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawLabelDscOwned
// ─────────────────────────────────────────────────────────

/// Owned LVGL label draw descriptor.
///
/// Initialised via `lv_draw_label_dsc_init`. Pass to [`Layer::draw_label`].
pub struct DrawLabelDscOwned {
    pub(crate) inner: lv_draw_label_dsc_t,
}

impl DrawLabelDscOwned {
    /// Create with default font (`lv_font_montserrat_14`) and LVGL defaults.
    pub fn default_font() -> Self {
        // SAFETY: zeroed is a valid starting state; init fills required fields.
        let mut inner = unsafe { core::mem::zeroed() };
        unsafe { lv_draw_label_dsc_init(&mut inner) };
        // SAFETY: lv_font_montserrat_14 is a 'static LVGL global; pointer is always
        // valid.
        inner.font = unsafe { &lv_font_montserrat_14 };
        Self { inner }
    }

    /// Set text color.
    pub fn set_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.color = color;
        self
    }

    /// Measure pixel size of `text` using this descriptor's current font and
    /// spacing.
    ///
    /// Returns `(width, height)`.
    pub fn text_size(&self, text: &str) -> (i32, i32) {
        let mut buf = [0u8; 64];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        let mut size: lv_point_t = unsafe { core::mem::zeroed() };
        // SAFETY: font pointer valid (set in default_font); buf is null-terminated
        // stack data.
        unsafe {
            lv_text_get_size(
                &mut size,
                buf.as_ptr() as *const core::ffi::c_char,
                self.inner.font,
                self.inner.letter_space,
                self.inner.line_space,
                0x7FFF,
                lv_text_flag_t_LV_TEXT_FLAG_NONE,
            );
        }
        (size.x, size.y)
    }

    /// Raw pointer to the inner descriptor. Used by
    /// [`CanvasLayer::draw_label`].
    pub(crate) fn as_ptr(&self) -> *const lv_draw_label_dsc_t {
        &self.inner
    }
}

// ── DrawArcDsc
// ────────────────────────────────────────────────────────────────

/// Descriptor for drawing an arc onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawArcDsc {
    inner: lv_draw_arc_dsc_t,
}

impl DrawArcDsc {
    /// Create with LVGL defaults (`lv_draw_arc_dsc_init`).
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; lv_draw_arc_dsc_init fills required
        // fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_arc_dsc_t>() };
        unsafe { lv_draw_arc_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Arc center in canvas coordinates.
    pub fn center(&mut self, x: i32, y: i32) -> &mut Self {
        self.inner.center.x = x;
        self.inner.center.y = y;
        self
    }

    /// Outer radius in pixels.
    pub fn radius(&mut self, r: u16) -> &mut Self {
        self.inner.radius = r;
        self
    }

    /// Start and end angles in degrees (0–360, clockwise from 3 o'clock).
    ///
    /// Angles are `f32` (`lv_value_precise_t`) in LVGL 9.3.
    pub fn angles(&mut self, start: f32, end: f32) -> &mut Self {
        self.inner.start_angle = start;
        self.inner.end_angle = end;
        self
    }

    /// Arc stroke width in pixels.
    pub fn width(&mut self, w: i32) -> &mut Self {
        self.inner.width = w;
        self
    }

    /// Stroke color.
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Overall opacity (0 = transparent, 255 = opaque).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    /// Draw rounded start and end caps.
    pub fn rounded(&mut self, r: bool) -> &mut Self {
        self.inner.set_rounded(r as u8);
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_arc_dsc_t {
        &self.inner
    }
}

impl Default for DrawArcDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawLineDsc
// ───────────────────────────────────────────────────────────────

/// Descriptor for drawing a straight line onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawLineDsc {
    inner: lv_draw_line_dsc_t,
}

impl DrawLineDsc {
    /// Create with LVGL defaults (`lv_draw_line_dsc_init`).
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; lv_draw_line_dsc_init fills
        // required fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_line_dsc_t>() };
        unsafe { lv_draw_line_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Start point. Coordinates are `f32` (`lv_point_precise_t`) in LVGL 9.3.
    pub fn p1(&mut self, x: f32, y: f32) -> &mut Self {
        self.inner.p1.x = x;
        self.inner.p1.y = y;
        self
    }

    /// End point. Coordinates are `f32` (`lv_point_precise_t`) in LVGL 9.3.
    pub fn p2(&mut self, x: f32, y: f32) -> &mut Self {
        self.inner.p2.x = x;
        self.inner.p2.y = y;
        self
    }

    /// Line stroke width in pixels.
    pub fn width(&mut self, w: i32) -> &mut Self {
        self.inner.width = w;
        self
    }

    /// Stroke color.
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Overall opacity (0–255).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    /// Draw a rounded cap at the start.
    pub fn round_start(&mut self, r: bool) -> &mut Self {
        self.inner.set_round_start(r as u8);
        self
    }

    /// Draw a rounded cap at the end.
    pub fn round_end(&mut self, r: bool) -> &mut Self {
        self.inner.set_round_end(r as u8);
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_line_dsc_t {
        &self.inner
    }
}

impl Default for DrawLineDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawTriangleDsc
// ───────────────────────────────────────────────────────────

/// Descriptor for drawing a solid-fill or gradient-fill triangle onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawTriangleDsc {
    inner: lv_draw_triangle_dsc_t,
}

impl DrawTriangleDsc {
    /// Create with LVGL defaults.
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; init fills required fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_triangle_dsc_t>() };
        unsafe { lv_draw_triangle_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Set the three vertex coordinates as `[(x0,y0), (x1,y1), (x2,y2)]`.
    ///
    /// Coordinates are `f32` (`lv_point_precise_t` fields).
    pub fn points(&mut self, pts: [(f32, f32); 3]) -> &mut Self {
        for (i, (x, y)) in pts.iter().enumerate() {
            self.inner.p[i].x = *x;
            self.inner.p[i].y = *y;
        }
        self
    }

    /// Overall opacity (0–255).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    /// Solid fill color (used when no gradient is configured).
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Number of active gradient stops (1 or 2).
    pub fn grad_stops_count(&mut self, n: u8) -> &mut Self {
        self.inner.grad.stops_count = n;
        self
    }

    /// Gradient direction.
    pub fn grad_dir(&mut self, dir: crate::style::GradDir) -> &mut Self {
        self.inner.grad.set_dir(dir as lv_grad_dir_t);
        self
    }

    /// Configure one gradient stop (index 0 or 1).
    ///
    /// - `frac`: position 0–255 (0 = start, 255 = end of gradient axis).
    /// - `opa`: stop opacity 0–255.
    pub fn grad_stop(&mut self, idx: usize, color: lv_color_t, frac: u8, opa: u8) -> &mut Self {
        assert!(idx < 2, "gradient stop index must be 0 or 1");
        self.inner.grad.stops[idx].color = color;
        self.inner.grad.stops[idx].frac = frac;
        self.inner.grad.stops[idx].opa = opa;
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_triangle_dsc_t {
        &self.inner
    }
}

impl Default for DrawTriangleDsc {
    fn default() -> Self {
        Self::new()
    }
}

// ── DrawImageDsc
// ──────────────────────────────────────────────────────────────

/// Descriptor for drawing an image onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
///
/// The `'i` lifetime ties this descriptor to the [`ImageDsc`](crate::draw_buf::ImageDsc)
/// source, which in turn borrows from the originating [`DrawBuf`](crate::draw_buf::DrawBuf).
/// This ensures the pixel data remains valid until `lv_canvas_finish_layer` completes.
pub struct DrawImageDsc<'i> {
    inner: lv_draw_image_dsc_t,
    _img_lifetime: core::marker::PhantomData<&'i ()>,
}

impl<'i> DrawImageDsc<'i> {
    /// Create from an [`ImageDsc`](crate::draw_buf::ImageDsc) obtained via
    /// [`DrawBuf::image_dsc`](crate::draw_buf::DrawBuf::image_dsc).
    ///
    /// `'i` is bound to the `ImageDsc` borrow, which is bound to the `DrawBuf`
    /// lifetime — preventing the pixel buffer from being freed while this
    /// descriptor is in use.
    pub fn from_image_dsc(img: &'i crate::draw_buf::ImageDsc<'_>) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_image_dsc_t>() };
        unsafe { lv_draw_image_dsc_init(&mut inner) };
        // SAFETY: img.inner contains a pointer into a DrawBuf pixel buffer.
        // 'i ties this descriptor to the ImageDsc borrow, which ties to the
        // DrawBuf lifetime — the pixel data is valid until 'i expires.
        inner.src = &img.inner as *const lv_image_dsc_t as *const core::ffi::c_void;
        Self { inner, _img_lifetime: core::marker::PhantomData }
    }

    /// Rotation in 0.1-degree units (e.g. `1200` = 120°).
    pub fn rotation(&mut self, r: i32) -> &mut Self {
        self.inner.rotation = r;
        self
    }

    /// Transform pivot point in canvas coordinates.
    pub fn pivot(&mut self, x: i32, y: i32) -> &mut Self {
        self.inner.pivot.x = x;
        self.inner.pivot.y = y;
        self
    }

    /// Overall opacity (0–255).
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_image_dsc_t {
        &self.inner
    }
}

// ── DrawLetterDsc
// ─────────────────────────────────────────────────────────────

/// Descriptor for drawing a single Unicode glyph onto a
/// [`CanvasLayer`](crate::widgets::CanvasLayer).
///
/// Used by canvas_10 / canvas_11 for per-character animations.
pub struct DrawLetterDsc {
    inner: lv_draw_letter_dsc_t,
}

impl DrawLetterDsc {
    /// Create with LVGL defaults.
    pub fn new() -> Self {
        // SAFETY: zeroed is a valid starting state; init fills required fields.
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_letter_dsc_t>() };
        unsafe { lv_draw_letter_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Unicode code point to render.
    pub fn unicode(&mut self, cp: u32) -> &mut Self {
        self.inner.unicode = cp;
        self
    }

    /// Glyph color.
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Rotation in 0.1-degree units (e.g. `900` = 90°).
    ///
    /// **Embedded note:** non-zero rotation triggers LVGL's vector-font path,
    /// which allocates an internal `ARGB8888` scratch buffer per glyph.
    /// On RAM-constrained targets use `rotation(0)` with an `RGB565` canvas.
    pub fn rotation(&mut self, r: i32) -> &mut Self {
        self.inner.rotation = r;
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_letter_dsc_t {
        &self.inner
    }
}

impl Default for DrawLetterDsc {
    fn default() -> Self {
        Self::new()
    }
}
