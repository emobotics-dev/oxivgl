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
        DrawDscBase {
            part: Part::from_raw(base.part),
            id1: base.id1,
            id2: base.id2,
        }
    }

    /// Label draw descriptor, if this task draws a label.
    ///
    /// Returns `None` if the task is not a label draw operation.
    pub fn label_dsc(&self) -> Option<DrawLabelDsc> {
        // SAFETY: ptr valid during callback; returns null if not a label task.
        let dsc = unsafe { lv_draw_task_get_label_dsc(self.ptr) };
        if dsc.is_null() {
            None
        } else {
            Some(DrawLabelDsc { ptr: dsc })
        }
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
        if base.layer.is_null() {
            None
        } else {
            Some(Layer::from_raw(base.layer))
        }
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

// ── Area utilities ────────────────────────────────────────────────────────────

impl Area {
    /// Align this area relative to `base` by `align`, then offset by `(ofs_x, ofs_y)`.
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
/// Valid only during a draw event callback. Obtain via [`Event::layer`](crate::event::Event::layer).
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
        local_dsc.set_text_local(0);
        let area_lv: lv_area_t = area.into();
        // SAFETY: ptr valid during callback; local_dsc.text points to buf on this stack frame.
        unsafe { lv_draw_label(self.ptr, &local_dsc, &area_lv) };
    }
}

// ── DrawRectDsc ───────────────────────────────────────────────────────────────

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

// ── DrawLabelDscOwned ─────────────────────────────────────────────────────────

/// Owned LVGL label draw descriptor.
///
/// Initialised via `lv_draw_label_dsc_init`. Pass to [`Layer::draw_label`].
pub struct DrawLabelDscOwned {
    inner: lv_draw_label_dsc_t,
}

impl DrawLabelDscOwned {
    /// Create with default font (`lv_font_montserrat_14`) and LVGL defaults.
    pub fn default_font() -> Self {
        // SAFETY: zeroed is a valid starting state; init fills required fields.
        let mut inner = unsafe { core::mem::zeroed() };
        unsafe { lv_draw_label_dsc_init(&mut inner) };
        // SAFETY: lv_font_montserrat_14 is a 'static LVGL global; pointer is always valid.
        inner.font = unsafe { &lv_font_montserrat_14 };
        Self { inner }
    }

    /// Set text color.
    pub fn set_color(&mut self, color: lv_color_t) -> &mut Self {
        self.inner.color = color;
        self
    }

    /// Measure pixel size of `text` using this descriptor's current font and spacing.
    ///
    /// Returns `(width, height)`.
    pub fn text_size(&self, text: &str) -> (i32, i32) {
        let mut buf = [0u8; 64];
        let len = text.len().min(buf.len() - 1);
        buf[..len].copy_from_slice(&text.as_bytes()[..len]);
        buf[len] = 0;
        let mut size: lv_point_t = unsafe { core::mem::zeroed() };
        // SAFETY: font pointer valid (set in default_font); buf is null-terminated stack data.
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

    /// Raw pointer to the inner descriptor. Used by [`CanvasLayer::draw_label`].
    pub(crate) fn as_ptr(&self) -> *const lv_draw_label_dsc_t {
        &self.inner
    }
}
