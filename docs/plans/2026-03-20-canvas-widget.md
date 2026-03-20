# Canvas Widget Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement LVGL Canvas widget wrapper + examples canvas_1–7, canvas_9–11 (canvas_8 skipped: needs `LV_USE_VECTOR_GRAPHIC`; canvas_6 deferred: needs static image asset).

**Architecture:** Three layers — (1) `DrawBuf` owned buffer type + `Canvas<'p>` wrapper (owns its DrawBuf) with `CanvasLayer` RAII; (2) New draw descriptors (`DrawArcDsc`, `DrawLineDsc`, `DrawTriangleDsc`, `DrawImageDsc`, `DrawLetterDsc`) in `draw.rs` on `CanvasLayer` only (not speculatively on `Layer`); (3) 9 examples + tests + docs. Tasks 3 and 4 are parallel-safe.

**Tech Stack:** Rust edition 2024, no_std/std, LVGL 9.3 via lvgl_rust_sys, SDL2 host testing.

---

## File Map

| Action | Path | Responsibility |
|--------|------|----------------|
| Modify | `conf/lv_conf.h` | Enable `LV_USE_CANVAS 1` |
| Create | `src/draw_buf.rs` | `ColorFormat` newtype + `DrawBuf` (owned `lv_draw_buf_t`) |
| Modify | `src/lib.rs` | `pub mod draw_buf;` |
| Modify | `src/draw.rs` | Add `DrawArcDsc`, `DrawLineDsc`, `DrawTriangleDsc` (with gradient builders), `DrawImageDsc<'i>`, `DrawLetterDsc` |
| Create | `src/widgets/canvas.rs` | `Canvas<'p>` (owns DrawBuf) + `CanvasLayer<'_>` RAII with all draw methods |
| Modify | `src/widgets/mod.rs` | `mod canvas; pub use canvas::{Canvas, CanvasLayer};` |
| Modify | `src/prelude.rs` | Re-export `Canvas`, `DrawBuf`, `ColorFormat`, new draw descriptor types (not `CanvasLayer`) |
| Modify | `src/style.rs` | Add `color_hsv(h, s, v)` |
| Create | `examples/canvas_{1..7,9,10,11}.rs` | Ported examples |
| Modify | `tests/integration.rs` | Canvas integration tests |
| Modify | `tests/leak_check.rs` | `leak_canvas` test |
| Modify | `README.md` | Widget table + example count |
| Modify | `examples/doc/README.md` | Per-example entries + coverage table |

---

## Spec Compliance Notes

Key invariants enforced in this plan:

- **spec-memory-lifetime §1**: `Canvas` owns its `DrawBuf` — no dangling pointer possible.
- **spec-memory-lifetime §1**: `DrawImageDsc<'i>` carries a lifetime tying it to the `lv_image_dsc_t` source — enforced at type level.
- **spec-api-vision §6**: `DrawTriangleDsc` has proper gradient builder methods (`grad_dir`, `grad_stop`, `grad_stops_count`) — no raw `lv_grad_dsc_t` escape hatch.
- **spec-api-vision §7**: New draw methods added to `CanvasLayer` only. `Layer` (event-based drawing) is not modified — no speculative API surface.
- **spec-api-vision §1**: Examples never touch `lvgl_rust_sys` types directly.
- **spec-widget-wrapper §4**: `fill_bg` and `set_px` return `&Self` for chaining.
- **spec-testing §8**: Every task verifies both host and embedded targets.

---

## Task 1 — Enable Canvas + DrawBuf type

**Files:** `conf/lv_conf.h`, `src/draw_buf.rs` (create), `src/lib.rs`

- [ ] Enable canvas in `conf/lv_conf.h`:

```c
#define LV_USE_CANVAS     1
```

- [ ] Create `src/draw_buf.rs`:

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Owned LVGL draw buffer — wraps `lv_draw_buf_t`.

use lvgl_rust_sys::*;

/// LVGL pixel color format.
///
/// Passed to [`DrawBuf::create`] to specify the pixel layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorFormat(pub u32);

impl ColorFormat {
    /// 16-bit RGB (5-6-5), no alpha channel.
    pub const RGB565: Self = Self(lv_color_format_t_LV_COLOR_FORMAT_RGB565 as u32);
    /// 32-bit ARGB (8-8-8-8), with full alpha.
    pub const ARGB8888: Self = Self(lv_color_format_t_LV_COLOR_FORMAT_ARGB8888 as u32);
}

/// Owned LVGL draw buffer. Allocated by LVGL on [`create`](DrawBuf::create) and freed on `Drop`.
///
/// Pass to [`Canvas::new`](crate::widgets::Canvas::new) — `Canvas` takes ownership
/// and ensures the buffer outlives the LVGL canvas object.
pub struct DrawBuf {
    ptr: *mut lv_draw_buf_t,
}

impl DrawBuf {
    /// Allocate a draw buffer of the given dimensions and color format.
    ///
    /// Returns `None` if LVGL allocation fails (OOM).
    pub fn create(w: u32, h: u32, cf: ColorFormat) -> Option<Self> {
        // SAFETY: lv_draw_buf_create allocates and zero-initialises the buffer;
        // returns null on allocation failure. We check before storing.
        let ptr = unsafe { lv_draw_buf_create(w, h, cf.0, 0) };
        if ptr.is_null() { None } else { Some(Self { ptr }) }
    }

    /// Raw LVGL pointer. Valid for the lifetime of this `DrawBuf`.
    pub(crate) fn as_ptr(&self) -> *mut lv_draw_buf_t {
        self.ptr
    }

    /// Convert buffer contents to an `lv_image_dsc_t` for use with [`DrawImageDsc`](crate::draw::DrawImageDsc).
    ///
    /// The returned descriptor borrows from `self`; it is valid only while
    /// this `DrawBuf` is alive.
    pub fn image_dsc(&self) -> lv_image_dsc_t {
        // SAFETY: ptr is a valid lv_draw_buf_t; lv_draw_buf_to_image fills the
        // image descriptor with a pointer into the buffer's pixel data.
        let mut img = unsafe { core::mem::zeroed::<lv_image_dsc_t>() };
        unsafe { lv_draw_buf_to_image(self.ptr, &mut img) };
        img
    }
}

impl Drop for DrawBuf {
    fn drop(&mut self) {
        // SAFETY: ptr was allocated by lv_draw_buf_create and has not been freed.
        // Canvas::drop (via lv_obj_delete) runs before DrawBuf::drop because
        // Canvas is declared after DrawBuf in every View struct (reverse field order).
        unsafe { lv_draw_buf_destroy(self.ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_format_rgb565_value() {
        // Sanity-check that the constant matches the raw binding value.
        assert_eq!(
            ColorFormat::RGB565.0,
            lv_color_format_t_LV_COLOR_FORMAT_RGB565 as u32
        );
    }

    #[test]
    fn color_format_argb8888_value() {
        assert_eq!(
            ColorFormat::ARGB8888.0,
            lv_color_format_t_LV_COLOR_FORMAT_ARGB8888 as u32
        );
    }
}
```

- [ ] Add `pub mod draw_buf;` to `src/lib.rs`.
- [ ] Host build check: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`
- [ ] Embedded build check: `cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04`

---

## Task 2 — Canvas wrapper + CanvasLayer RAII

**Files:** `src/widgets/canvas.rs` (create), `src/widgets/mod.rs`

Key design decisions enforced here:
- `Canvas` owns its `DrawBuf` → buffer always outlives canvas (spec-memory-lifetime §1).
- `fill_bg` and `set_px` return `&Self` for chaining (spec-widget-wrapper §4).
- `CanvasLayer` has `draw_rect` and `draw_label` only at this stage; other draw methods added in Tasks 3 and 4.

- [ ] Create `src/widgets/canvas.rs`:

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canvas widget — off-screen drawing surface.

use core::marker::PhantomData;
use lvgl_rust_sys::*;

use crate::draw::{Area, DrawLabelDscOwned, DrawRectDsc};
use crate::draw_buf::DrawBuf;
use crate::widgets::{AsLvHandle, Obj, WidgetError};

/// Off-screen drawing surface backed by a [`DrawBuf`].
///
/// `Canvas` takes ownership of its draw buffer, ensuring the buffer outlives
/// the LVGL canvas object. Use [`init_layer`](Canvas::init_layer) to batch-draw
/// onto the canvas; drawing is committed when the returned [`CanvasLayer`] is dropped.
///
/// # Example
///
/// ```no_run
/// # use oxivgl::{draw_buf::{ColorFormat, DrawBuf}, widgets::{Canvas, Screen}};
/// # use oxivgl::draw::{Area, DrawRectDsc};
/// # use oxivgl::style::color_make;
/// let screen = Screen::active().unwrap();
/// let buf = DrawBuf::create(100, 100, ColorFormat::ARGB8888).unwrap();
/// let canvas = Canvas::new(&screen, buf).unwrap();
/// canvas.fill_bg(color_make(200, 200, 200), 255);
/// {
///     let mut layer = canvas.init_layer();
///     let mut rdc = DrawRectDsc::new();
///     rdc.bg_color(color_make(255, 0, 0));
///     layer.draw_rect(&rdc, Area { x1: 10, y1: 10, x2: 50, y2: 50 });
/// } // layer dropped → lv_canvas_finish_layer called
/// ```
pub struct Canvas<'p> {
    obj: Obj<'p>,
    draw_buf: DrawBuf,
}

impl<'p> Canvas<'p> {
    /// Create a canvas child of `parent`, backed by `buf`.
    ///
    /// `Canvas` takes ownership of `buf`, ensuring the draw buffer outlives
    /// the LVGL canvas object (spec-memory-lifetime §1).
    pub fn new(parent: &impl AsLvHandle, buf: DrawBuf) -> Result<Self, WidgetError> {
        let handle = parent.lv_handle();
        if handle.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }
        // SAFETY: handle is a valid non-null LVGL object pointer.
        let obj_ptr = unsafe { lv_canvas_create(handle) };
        if obj_ptr.is_null() {
            return Err(WidgetError::LvglNullPointer);
        }
        // SAFETY: obj_ptr is valid; buf.as_ptr() is stored by LVGL. Canvas owns
        // buf so the pointer is valid for the canvas object's entire lifetime.
        unsafe { lv_canvas_set_draw_buf(obj_ptr, buf.as_ptr()) };
        Ok(Self { obj: unsafe { Obj::from_raw(obj_ptr) }, draw_buf: buf })
    }

    /// Borrow the owned draw buffer (e.g. to obtain an [`lv_image_dsc_t`] for
    /// canvas-to-canvas drawing via [`DrawBuf::image_dsc`]).
    pub fn draw_buf(&self) -> &DrawBuf {
        &self.draw_buf
    }

    /// Fill the entire canvas background with a solid color.
    ///
    /// `opa`: 0 = fully transparent, 255 = fully opaque.
    pub fn fill_bg(&self, color: lv_color_t, opa: u8) -> &Self {
        // SAFETY: handle is valid.
        unsafe { lv_canvas_fill_bg(self.lv_handle(), color, opa) };
        self
    }

    /// Set a single pixel. Effective only on ARGB/palette format buffers.
    pub fn set_px(&self, x: i32, y: i32, color: lv_color_t, opa: u8) -> &Self {
        // SAFETY: handle is valid; x/y out-of-bounds are clamped by LVGL.
        unsafe { lv_canvas_set_px(self.lv_handle(), x, y, color, opa) };
        self
    }

    /// Get a pixel's 32-bit ARGB color value.
    pub fn get_px(&self, x: i32, y: i32) -> lv_color32_t {
        // SAFETY: handle is valid.
        unsafe { lv_canvas_get_px(self.lv_handle(), x, y) }
    }

    /// Begin a batch drawing session.
    ///
    /// Returns a [`CanvasLayer`] RAII guard. Draw onto it using the `draw_*`
    /// methods. When the guard is dropped, `lv_canvas_finish_layer` is called
    /// and all queued draw commands are committed synchronously.
    pub fn init_layer(&self) -> CanvasLayer<'_> {
        CanvasLayer::new(self.lv_handle())
    }
}

impl<'p> AsLvHandle for Canvas<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> core::ops::Deref for Canvas<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

/// RAII guard for canvas batch drawing.
///
/// Obtained via [`Canvas::init_layer`]. Draw calls are queued until this guard
/// is dropped, at which point `lv_canvas_finish_layer` flushes all commands
/// synchronously.
///
/// `draw_arc`, `draw_line`, `draw_triangle`, `draw_image`, `draw_letter` are
/// added in Tasks 3 and 4 as their descriptor types become available.
pub struct CanvasLayer<'c> {
    canvas: *mut lv_obj_t,
    layer: lv_layer_t,
    _canvas_lifetime: PhantomData<&'c ()>,
}

impl<'c> CanvasLayer<'c> {
    pub(crate) fn new(canvas: *mut lv_obj_t) -> Self {
        // SAFETY: canvas is a valid lv_canvas object pointer. The layer is
        // stack-allocated here and initialised by lv_canvas_init_layer.
        let mut layer = unsafe { core::mem::zeroed::<lv_layer_t>() };
        unsafe { lv_canvas_init_layer(canvas, &mut layer) };
        Self { canvas, layer, _canvas_lifetime: PhantomData }
    }

    /// Draw a filled/bordered rectangle.
    pub fn draw_rect(&mut self, dsc: &DrawRectDsc, area: Area) {
        let lv_area = lv_area_t::from(area);
        // SAFETY: layer is valid until finish_layer is called in Drop.
        unsafe { lv_draw_rect(&mut self.layer, dsc.as_ptr(), &lv_area) };
    }

    /// Draw a text label.
    pub fn draw_label(&mut self, dsc: &DrawLabelDscOwned, area: Area) {
        let lv_area = lv_area_t::from(area);
        // SAFETY: layer valid; dsc owns its text via lv_strdup.
        unsafe { lv_draw_label(&mut self.layer, dsc.as_ptr(), &lv_area) };
    }
}

impl Drop for CanvasLayer<'_> {
    fn drop(&mut self) {
        // SAFETY: canvas and layer are valid. finish_layer flushes queued draw
        // commands synchronously and invalidates the layer. After this call the
        // layer struct must not be used — we consume it in this Drop.
        unsafe { lv_canvas_finish_layer(self.canvas, &mut self.layer) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::GradDir;

    // Verify ColorFormat is accessible from draw_buf module (import smoke test).
    #[test]
    fn canvas_module_imports_compile() {
        let _ = core::mem::size_of::<Canvas<'_>>();
        let _ = core::mem::size_of::<CanvasLayer<'_>>();
    }
}
```

- [ ] Add to `src/widgets/mod.rs`:

```rust
mod canvas;
pub use canvas::{Canvas, CanvasLayer};
```

- [ ] Host build check.
- [ ] Embedded build check.

---

## Task 3 — DrawArcDsc + DrawLineDsc (parallel with Task 4)

**Files:** `src/draw.rs`, `src/widgets/canvas.rs`

Note: these are added to `CanvasLayer` only — NOT to `Layer` (no event-based example currently needs them; spec-api-vision §7).

- [ ] Add `DrawArcDsc` builder to `src/draw.rs`:

```rust
/// Descriptor for drawing an arc onto a [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawArcDsc {
    inner: lv_draw_arc_dsc_t,
}

impl DrawArcDsc {
    /// Create with LVGL defaults (white, width 1, opa 255, not rounded).
    pub fn new() -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_arc_dsc_t>() };
        // SAFETY: zeroed memory is acceptable initial state; lv_draw_arc_dsc_init
        // sets defaults (lv_draw_arc.c).
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
    pub fn angles(&mut self, start: i32, end: i32) -> &mut Self {
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
        // NOTE: `rounded` is a bitfield in the C struct. The binding may expose
        // `set_rounded(u8)` or a direct bool field — check at compile time.
        self.inner.set_rounded(r as u8);
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_arc_dsc_t {
        &self.inner
    }
}
```

- [ ] Add `DrawLineDsc` builder to `src/draw.rs`:

```rust
/// Descriptor for drawing a straight line onto a [`CanvasLayer`](crate::widgets::CanvasLayer).
pub struct DrawLineDsc {
    inner: lv_draw_line_dsc_t,
}

impl DrawLineDsc {
    /// Create with LVGL defaults (white, width 1, opa 255, square caps).
    pub fn new() -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_line_dsc_t>() };
        // SAFETY: zeroed + lv_draw_line_dsc_init sets defaults.
        unsafe { lv_draw_line_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Start point coordinates.
    pub fn p1(&mut self, x: i32, y: i32) -> &mut Self {
        self.inner.p1.x = x;
        self.inner.p1.y = y;
        self
    }

    /// End point coordinates.
    pub fn p2(&mut self, x: i32, y: i32) -> &mut Self {
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
```

- [ ] Add `draw_arc` and `draw_line` to `CanvasLayer` in `canvas.rs`:

```rust
use crate::draw::{DrawArcDsc, DrawLineDsc};

// inside impl<'c> CanvasLayer<'c>:

/// Draw an arc.
pub fn draw_arc(&mut self, dsc: &DrawArcDsc) {
    // SAFETY: layer valid until Drop; lv_draw_arc reads dsc synchronously.
    unsafe { lv_draw_arc(&mut self.layer, dsc.as_ptr()) };
}

/// Draw a line.
pub fn draw_line(&mut self, dsc: &DrawLineDsc) {
    // SAFETY: layer valid until Drop.
    unsafe { lv_draw_line(&mut self.layer, dsc.as_ptr()) };
}
```

- [ ] Host build check.
- [ ] Embedded build check.

---

## Task 4 — DrawTriangleDsc + DrawImageDsc + DrawLetterDsc (parallel with Task 3)

**Files:** `src/draw.rs`, `src/widgets/canvas.rs`, `src/style.rs`

Key: `DrawTriangleDsc` has proper gradient builder methods (no raw `lv_grad_dsc_t` escape hatch — spec-api-vision §6). `DrawImageDsc<'i>` carries a lifetime tying it to its image source (spec-memory-lifetime §1).

- [ ] Add `DrawTriangleDsc` to `src/draw.rs`:

```rust
use crate::enums::GradDir;

/// Descriptor for drawing a solid-fill or gradient-fill triangle.
pub struct DrawTriangleDsc {
    inner: lv_draw_triangle_dsc_t,
}

impl DrawTriangleDsc {
    /// Create with LVGL defaults (white fill, opa 255, no gradient).
    pub fn new() -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_triangle_dsc_t>() };
        // SAFETY: zeroed + lv_draw_triangle_dsc_init sets defaults.
        unsafe { lv_draw_triangle_dsc_init(&mut inner) };
        Self { inner }
    }

    /// Set the three vertex coordinates as `[(x0,y0), (x1,y1), (x2,y2)]`.
    pub fn points(&mut self, pts: [(i32, i32); 3]) -> &mut Self {
        for (i, (x, y)) in pts.iter().enumerate() {
            self.inner.p[i].x = *x;
            self.inner.p[i].y = *y;
        }
        self
    }

    /// Overall opacity (0–255). Applied on top of any gradient.
    pub fn opa(&mut self, o: u8) -> &mut Self {
        self.inner.opa = o;
        self
    }

    /// Solid fill color. Ignored if a gradient is configured.
    pub fn color(&mut self, c: lv_color_t) -> &mut Self {
        self.inner.color = c;
        self
    }

    /// Number of gradient stops (1 or 2).
    pub fn grad_stops_count(&mut self, n: u8) -> &mut Self {
        self.inner.grad.stops_count = n;
        self
    }

    /// Gradient direction.
    pub fn grad_dir(&mut self, dir: GradDir) -> &mut Self {
        self.inner.grad.dir = dir as u8;
        self
    }

    /// Configure one gradient stop (index 0 or 1).
    ///
    /// - `frac`: position along the gradient axis, 0–255 (0=start, 255=end).
    /// - `opa`: stop opacity (0–255).
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
```

- [ ] Add `DrawImageDsc<'i>` to `src/draw.rs`:

```rust
/// Descriptor for drawing an image onto a [`CanvasLayer`](crate::widgets::CanvasLayer).
///
/// The `'i` lifetime ties this descriptor to the `lv_image_dsc_t` source
/// (obtained via [`DrawBuf::image_dsc`](crate::draw_buf::DrawBuf::image_dsc)).
/// The source must remain alive until the [`CanvasLayer`] guard is dropped
/// (i.e. until `lv_canvas_finish_layer` has completed).
pub struct DrawImageDsc<'i> {
    inner: lv_draw_image_dsc_t,
    _img_lifetime: core::marker::PhantomData<&'i lv_image_dsc_t>,
}

impl<'i> DrawImageDsc<'i> {
    /// Create from an image descriptor obtained via [`DrawBuf::image_dsc`](crate::draw_buf::DrawBuf::image_dsc).
    ///
    /// `img` must remain alive for the duration of the [`CanvasLayer`] that
    /// will consume this descriptor (i.e. until `lv_canvas_finish_layer`).
    pub fn from_image_dsc(img: &'i lv_image_dsc_t) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_image_dsc_t>() };
        // SAFETY: zeroed + init sets defaults (scale 256, opa 255, no rotation).
        unsafe { lv_draw_image_dsc_init(&mut inner) };
        // SAFETY: img is a valid lv_image_dsc_t valid for 'i; the src pointer is
        // stored in inner.src and consumed when lv_draw_image is called before
        // 'i expires (enforced by the lifetime parameter).
        inner.src = img as *const lv_image_dsc_t as *const core::ffi::c_void;
        Self { inner, _img_lifetime: core::marker::PhantomData }
    }

    /// Rotation in 0.1-degree units (e.g. `1200` = 120°, `3600` = 360°).
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
```

- [ ] Add `DrawLetterDsc` to `src/draw.rs`:

```rust
/// Descriptor for drawing a single Unicode glyph onto a [`CanvasLayer`](crate::widgets::CanvasLayer).
///
/// Used by canvas_10 / canvas_11 for per-character animations.
pub struct DrawLetterDsc {
    inner: lv_draw_letter_dsc_t,
}

impl DrawLetterDsc {
    /// Create with LVGL defaults (default font, white, no rotation).
    pub fn new() -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_draw_letter_dsc_t>() };
        // SAFETY: zeroed + lv_draw_letter_dsc_init sets default font and opa.
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
    pub fn rotation(&mut self, r: i32) -> &mut Self {
        self.inner.rotation = r;
        self
    }

    pub(crate) fn as_ptr(&self) -> *const lv_draw_letter_dsc_t {
        &self.inner
    }
}
```

- [ ] Add `draw_triangle`, `draw_image`, `draw_letter` to `CanvasLayer` in `canvas.rs` (NOT to `Layer` — no event-based example currently needs them):

```rust
use crate::draw::{DrawTriangleDsc, DrawImageDsc, DrawLetterDsc};

// inside impl<'c> CanvasLayer<'c>:

/// Draw a filled or gradient-filled triangle.
pub fn draw_triangle(&mut self, dsc: &DrawTriangleDsc) {
    // SAFETY: layer valid until Drop.
    unsafe { lv_draw_triangle(&mut self.layer, dsc.as_ptr()) };
}

/// Draw an image (or rotated canvas snapshot).
///
/// The [`DrawImageDsc`] lifetime ensures the image source lives long enough.
pub fn draw_image<'i>(&mut self, dsc: &DrawImageDsc<'i>, area: Area) {
    let lv_area = lv_area_t::from(area);
    // SAFETY: layer valid; image src pointer in dsc is valid for 'i, and
    // lv_canvas_finish_layer (called in Drop) processes commands before 'i
    // can expire (both live in the same scope).
    unsafe { lv_draw_image(&mut self.layer, dsc.as_ptr(), &lv_area) };
}

/// Draw a single Unicode glyph at position `(x, y)`.
pub fn draw_letter(&mut self, dsc: &DrawLetterDsc, x: i32, y: i32) {
    let pt = lv_point_t { x, y };
    // SAFETY: layer valid; lv_draw_letter reads dsc synchronously.
    unsafe { lv_draw_letter(&mut self.layer, dsc.as_ptr(), &pt) };
}
```

- [ ] Add `color_hsv` to `src/style.rs` (needed by canvas_10/11):

```rust
/// Convert HSV to an LVGL color value.
///
/// - `h`: hue, 0–360
/// - `s`: saturation, 0–100
/// - `v`: value (brightness), 0–100
pub fn color_hsv(h: u16, s: u8, v: u8) -> lv_color_t {
    // SAFETY: pure computation, no LVGL object state involved.
    unsafe { lv_color_hsv_to_rgb(h, s, v) }
}
```

- [ ] If `lv_atan2` is not in the bindings, add to `src/math.rs`:

```rust
/// Integer arctangent. Returns angle in degrees × 10 (tenths of a degree).
pub fn atan2(y: i32, x: i32) -> i32 {
    // SAFETY: pure computation.
    unsafe { lv_atan2(y, x) }
}
```

Verify with: `grep "fn lv_atan2" target/x86_64-unknown-linux-gnu/debug/build/lvgl_rust_sys-*/out/bindings.rs`

- [ ] Host + embedded build checks.

---

## Task 5 — Examples canvas_2, canvas_3, canvas_4 (simple)

**Files:** `examples/canvas_2.rs`, `examples/canvas_3.rs`, `examples/canvas_4.rs`

Note: with `Canvas::new(parent, buf)`, the View struct no longer needs a separate `_buf` field.

- [ ] Create `examples/canvas_2.rs` — ARGB8888 canvas with pixel opacity bands:

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Canvas 2 — Transparent pixels
//!
//! An 80×60 ARGB8888 canvas filled blue, with three horizontal bands of
//! decreasing opacity (50 %, 20 %, 0 %) drawn via `set_px`.

use oxivgl::{
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Canvas, Screen, WidgetError},
};

struct Canvas2 {
    canvas: Canvas<'static>,
}

impl View for Canvas2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let buf = DrawBuf::create(80, 60, ColorFormat::ARGB8888)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(&screen, buf)?;
        canvas.fill_bg(color_make(0, 0, 196), 255);
        for y in 10..20_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 128);
            }
        }
        for y in 20..30_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 51);
            }
        }
        for y in 30..40_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 0);
            }
        }
        canvas.center();
        Ok(Self { canvas })
    }

    fn register_events(&mut self) {}
    fn on_event(&mut self, _: &oxivgl::event::Event) {}
    fn update(&mut self) -> Result<(), WidgetError> { Ok(()) }
}

oxivgl_examples_common::example_main!(Canvas2);
```

- [ ] Create `examples/canvas_3.rs` — 70×70 ARGB8888 canvas, red fill, blue border, green outline via `CanvasLayer::draw_rect`.
- [ ] Create `examples/canvas_4.rs` — 80×30 ARGB8888 canvas, "Hello" in red via `CanvasLayer::draw_label`.
- [ ] Screenshot all three: `./run_host.sh -s canvas_2 canvas_3 canvas_4`
- [ ] Add all three to `ALL_EXAMPLES` in `run_host.sh`.
- [ ] Add entries to `examples/doc/README.md` (title, description, screenshot link, coverage table row).

---

## Task 6 — Examples canvas_5, canvas_7

**Files:** `examples/canvas_5.rs`, `examples/canvas_7.rs`

Depends on Task 3 (`DrawArcDsc`, `DrawLineDsc`).

- [ ] Create `examples/canvas_5.rs` — 50×50 ARGB8888 canvas, red arc (center 25,25; radius 15; width 10; 0°–220°):

```rust
// In create():
let buf = DrawBuf::create(50, 50, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?;
let canvas = Canvas::new(&screen, buf)?;
canvas.fill_bg(color_make(200, 200, 200), 255).center();
{
    let mut layer = canvas.init_layer();
    let mut dsc = DrawArcDsc::new();
    dsc.center(25, 25).radius(15).angles(0, 220).width(10)
       .color(color_make(255, 0, 0)).opa(255);
    layer.draw_arc(&dsc);
}
```

- [ ] Create `examples/canvas_7.rs` — 50×50 ARGB8888 canvas, red line (15,15)→(35,10), width 4, rounded caps.
- [ ] Screenshots: `./run_host.sh -s canvas_5 canvas_7`
- [ ] Add to `ALL_EXAMPLES` and `examples/doc/README.md`.

---

## Task 7 — Example canvas_1 (dual canvas + rotation)

**Files:** `examples/canvas_1.rs`

Depends on Task 4 (`DrawImageDsc`, `DrawBuf::image_dsc`).

- [ ] Create `examples/canvas_1.rs`:
  - View struct: two `Canvas<'static>` fields.
  - First canvas (200×150 RGB565): gradient rect + orange label via `CanvasLayer`.
  - Second canvas (200×150 ARGB8888): rotated snapshot of first:
    ```rust
    let img = self.canvas1.draw_buf().image_dsc();  // lv_image_dsc_t on stack
    let mut layer = self.canvas2.init_layer();
    let mut dsc = DrawImageDsc::from_image_dsc(&img);
    dsc.rotation(120).pivot(100, 75);
    layer.draw_image(&dsc, Area { x1: 0, y1: 0, x2: 199, y2: 149 });
    // layer dropped → finish_layer; img still in scope → safe
    ```
  - Note in doc comment: `rotation = 120` is 12° (LVGL image rotation is in 0.1° units).
- [ ] Screenshot: `./run_host.sh -s canvas_1`
- [ ] Add to `ALL_EXAMPLES` and `examples/doc/README.md`.

---

## Task 8 — Example canvas_9 (gradient triangle)

**Files:** `examples/canvas_9.rs`

Depends on Task 4 (`DrawTriangleDsc` with gradient builder API).

- [ ] Create `examples/canvas_9.rs` — 150×150 ARGB8888 canvas:

```rust
// In create():
let buf = DrawBuf::create(150, 150, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?;
let canvas = Canvas::new(&screen, buf)?;
canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255).center();
{
    let mut layer = canvas.init_layer();
    let mut dsc = DrawTriangleDsc::new();
    dsc.points([(10, 10), (100, 30), (50, 100)])
       .opa(128)
       .grad_stops_count(2)
       .grad_dir(GradDir::Vertical)
       .grad_stop(0, color_make(0xff, 0x00, 0x00), 64, 255)   // red @ 25%
       .grad_stop(1, color_make(0x00, 0x00, 0xff), 192, 0);   // blue @ 75%, transparent
    layer.draw_triangle(&dsc);
}
```

- [ ] Screenshot: `./run_host.sh -s canvas_9`
- [ ] Add to `ALL_EXAMPLES` and `examples/doc/README.md`.

---

## Task 9 — Examples canvas_10, canvas_11 (animated)

**Files:** `examples/canvas_10.rs`, `examples/canvas_11.rs`

Depends on Task 4 (`DrawLetterDsc`, `color_hsv`, `atan2`). Animation driven by `update()` frame counter — no LVGL timer needed (spec-example-porting §4).

- [ ] Create `examples/canvas_10.rs` — animated wavy text:
  - View struct: `canvas: Canvas<'static>`, `counter: i32`.
  - `create()`: 300×200 ARGB8888 canvas, centered.
  - `update()`:

```rust
fn update(&mut self) -> Result<(), WidgetError> {
    use oxivgl::math::{atan2, trigo_sin};
    use oxivgl::style::{color_hsv, color_make};
    const TXT: &[u8] = b"lol~ I'm wavvvvvvving~>>>";
    self.canvas.fill_bg(color_make(255, 255, 255), 255);
    {
        let mut layer = self.canvas.init_layer();
        let mut pre_x = -2_i32 + 10;
        let mut pre_y = trigo_sin(-5_i16) as i32 * 40 / 32767 + 100;
        for (i, &ch) in TXT.iter().enumerate() {
            let angle = i as i32 * 5;
            let x = angle * 2 + 10;
            let y = trigo_sin(((angle + self.counter / 2) * 5) as i16) as i32 * 40 / 32767 + 100;
            let mut dsc = DrawLetterDsc::new();
            dsc.unicode(ch as u32)
               .color(color_hsv((i as u16 * 10) % 360, 100, 100))
               .rotation(atan2(y - pre_y, x - pre_x) * 10);
            layer.draw_letter(&dsc, x, y);
            pre_x = x;
            pre_y = y;
        }
    } // layer dropped → finish_layer
    self.counter += 1;
    Ok(())
}
```

- [ ] Create `examples/canvas_11.rs` — animated windstorm text. Same structure; replace curve with:
  ```
  x = (i * 2 + counter/3) % CANVAS_WIDTH
  y = lv_trigo_sin(i*7 + counter) * 50 / 32767 + CANVAS_HEIGHT/2 + lv_trigo_cos(i*3)*30/32767
  ```
  (translate the C parametric curve from `lv_example_canvas_11.c` verbatim).
- [ ] Screenshots: `./run_host.sh -s canvas_10 canvas_11`
- [ ] Add to `ALL_EXAMPLES` and `examples/doc/README.md`.

---

## Task 10 — Integration + leak tests

**Files:** `tests/integration.rs`, `tests/leak_check.rs`

Integration tests use the new `Canvas::new(parent, buf)` API (Canvas owns buf).

- [ ] Add canvas integration tests to `tests/integration.rs`:

```rust
#[test]
fn canvas_create_and_fill() {
    let _screen = fresh_screen();
    ensure_init();
    let screen = Screen::active().unwrap();
    let buf = DrawBuf::create(50, 50, ColorFormat::RGB565).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(255, 0, 0), 255).size(50, 50).center();
    pump(5);
}

#[test]
fn canvas_set_px_and_get() {
    let _screen = fresh_screen();
    ensure_init();
    let screen = Screen::active().unwrap();
    let buf = DrawBuf::create(10, 10, ColorFormat::ARGB8888).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(0, 0, 0), 255);
    canvas.set_px(5, 5, color_make(255, 255, 255), 255);
    let px = canvas.get_px(5, 5);
    // Verify pixel was set (exact field name depends on lv_color32_t binding layout)
    assert_ne!(px.red, 0, "pixel should be non-zero after set_px");
    pump(2);
}

#[test]
fn canvas_layer_draw_rect() {
    let _screen = fresh_screen();
    ensure_init();
    let screen = Screen::active().unwrap();
    let buf = DrawBuf::create(100, 100, ColorFormat::ARGB8888).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(200, 200, 200), 255);
    {
        let mut layer = canvas.init_layer();
        let mut dsc = DrawRectDsc::new();
        dsc.bg_color(color_make(255, 0, 0)).radius(5);
        layer.draw_rect(&dsc, Area { x1: 10, y1: 10, x2: 50, y2: 50 });
    }
    pump(3);
}

#[test]
fn drawbuf_create_returns_some() {
    let buf = DrawBuf::create(100, 100, ColorFormat::RGB565);
    assert!(buf.is_some());
}

#[test]
fn canvas_draw_buf_accessor() {
    let _screen = fresh_screen();
    ensure_init();
    let screen = Screen::active().unwrap();
    let buf = DrawBuf::create(40, 40, ColorFormat::RGB565).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    // draw_buf() accessor must return a non-null pointer.
    assert!(!canvas.draw_buf().as_ptr().is_null());
}
```

- [ ] Add `leak_canvas` to `tests/leak_check.rs`:

```rust
#[test]
fn leak_canvas() {
    // TODO: determine expected_alloc_count empirically by running once with
    // a large value (e.g. 100) and reading the actual count from the failure message.
    assert_no_leak("Canvas", TODO, |screen| {
        let buf = DrawBuf::create(50, 50, ColorFormat::RGB565).unwrap();
        let canvas = Canvas::new(screen, buf).unwrap();
        canvas.fill_bg(color_make(100, 100, 100), 255);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawRectDsc::new();
            dsc.bg_color(color_make(255, 0, 0));
            layer.draw_rect(&dsc, Area { x1: 5, y1: 5, x2: 45, y2: 45 });
        }
        drop(canvas); // also drops draw_buf owned inside
    });
}
```

- [ ] Run `./run_tests.sh all` — determine `expected_alloc_count` from output, update `TODO`.
- [ ] Run again with correct count to confirm green.

---

## Task 11 — Prelude exports + README updates

**Files:** `src/prelude.rs`, `README.md`, `examples/doc/README.md`

- [ ] Add to `src/prelude.rs` (note: `CanvasLayer` is NOT added — users never construct it):

```rust
// Canvas
pub use crate::draw_buf::{ColorFormat, DrawBuf};
pub use crate::widgets::Canvas;
// New draw descriptor types
pub use crate::draw::{DrawArcDsc, DrawImageDsc, DrawLetterDsc, DrawLineDsc, DrawTriangleDsc};
```

- [ ] Update `README.md`:
  - Add Canvas row to widget table (with `set_px`, `fill_bg`, `init_layer`/`CanvasLayer`).
  - Add new draw descriptor types to draw table.
  - Increment example count by 9 (canvas_1–7 skip 8, canvas_9–11).
  - Update integration and leak test counts.

- [ ] Update `examples/doc/README.md`:
  - Add one entry per new example (title, description, screenshot `![canvas_N](screenshots/canvas_N.png)`).
  - Update Implementation Coverage table: mark canvas_1–11 rows (skip 6, 8).

- [ ] Final checks:
  - `./run_tests.sh all`
  - `LIBCLANG_PATH=/usr/lib64 RUSTDOCFLAGS="-W missing-docs" cargo +nightly doc --target x86_64-unknown-linux-gnu --no-deps 2>&1 | grep "warning:"`
  - `cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04`

- [ ] Commit.

---

## Unresolved Questions

1. **`lv_point_precise_t` field types**: Are `x`/`y` `i32` or `f32` in the generated bindings? Verify at Task 3/4 impl time: `grep "lv_point_precise_t" target/.../bindings.rs`. If `f32`, adjust `DrawLineDsc::p1/p2` and `DrawTriangleDsc::points` accordingly.
2. **Bitfield accessor names**: `rounded` in `lv_draw_arc_dsc_t`, `round_start`/`round_end` in `lv_draw_line_dsc_t` — binding may generate `set_rounded(u8)`, a direct bool, or something else. Verify at compile time.
3. **`lv_atan2` in bindings**: May be absent if LVGL doesn't export it. Task 4 includes fallback wrapper in `math.rs`.
4. **`lv_trigo_sin` on Xtensa**: Confirm it is available in no_std+esp-hal build (should be — it's a core LVGL utility).
5. **canvas_6** (deferred): Needs `lv_image_dsc_t img_star` from LVGL example assets. Check if `lv_example_img_star` is compiled and linked in `lvgl_rust_sys`.
6. **`lv_color32_t` field names** in integration test `canvas_set_px_and_get`: the `red` field is assumed from the LVGL struct layout. Verify the actual field name in the binding.
