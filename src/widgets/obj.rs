// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{cell::RefCell, ffi::c_void, marker::PhantomData, ptr::null_mut};

use lvgl_rust_sys::*;

use super::WidgetError;

/// 3×3 affine transform matrix.
///
/// Chain operations via builder-style methods. Requires
/// `LV_DRAW_TRANSFORM_USE_MATRIX = 1` and `LV_USE_FLOAT = 1` in `lv_conf.h`.
///
/// ```no_run
/// use oxivgl::widgets::Matrix;
///
/// let mut m = Matrix::identity();
/// m.scale(0.5, 0.5).rotate(45.0);
/// // Apply with: obj.set_transform(&m);
/// ```
pub struct Matrix {
    inner: lv_matrix_t,
}

impl core::fmt::Debug for Matrix {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Matrix").finish_non_exhaustive()
    }
}

impl Matrix {
    /// Create an identity matrix (no transform).
    pub fn identity() -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_matrix_t>() };
        // SAFETY: inner is a valid zeroed lv_matrix_t.
        unsafe { lv_matrix_identity(&mut inner) };
        Self { inner }
    }

    /// Apply uniform or non-uniform scale.
    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        // SAFETY: inner was initialized by lv_matrix_identity.
        unsafe { lv_matrix_scale(&mut self.inner, sx, sy) };
        self
    }

    /// Apply rotation in degrees.
    pub fn rotate(&mut self, degrees: f32) -> &mut Self {
        // SAFETY: inner was initialized by lv_matrix_identity.
        unsafe { lv_matrix_rotate(&mut self.inner, degrees) };
        self
    }

    /// Raw pointer for passing to LVGL.
    pub(crate) fn as_ptr(&self) -> *const lv_matrix_t {
        &self.inner
    }
}

/// Type-safe selector for an LVGL style part (maps to `lv_part_t`).
///
/// Used with style-setter methods such as [`Obj::line_width`] to target a
/// specific sub-part of a widget (e.g. the indicator arc vs. the background
/// track).
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// Main background rectangle (`LV_PART_MAIN = 0x000000`).
    Main = 0x000000,
    /// Indicator (e.g. filled arc, slider thumb, `LV_PART_INDICATOR =
    /// 0x020000`).
    Indicator = 0x020000,
    /// Grab handle (`LV_PART_KNOB = 0x030000`).
    Knob = 0x030000,
    /// Selected item highlight, e.g. roller selected row (`LV_PART_SELECTED = 0x040000`).
    Selected = 0x040000,
    /// Repeated sub-elements such as tick marks (`LV_PART_ITEMS = 0x050000`).
    Items = 0x050000,
    /// Text cursor (e.g. textarea cursor, `LV_PART_CURSOR = 0x060000`).
    Cursor = 0x060000,
    /// Scrollbar part (`LV_PART_SCROLLBAR = 0x010000`).
    Scrollbar = lvgl_rust_sys::lv_part_t_LV_PART_SCROLLBAR,
}

impl Part {
    /// Convert a raw `lv_part_t` value to a `Part` enum.
    /// Unknown values map to `Main`.
    pub fn from_raw(raw: u32) -> Self {
        match raw {
            0x000000 => Part::Main,
            0x010000 => Part::Scrollbar,
            0x020000 => Part::Indicator,
            0x030000 => Part::Knob,
            0x040000 => Part::Selected,
            0x050000 => Part::Items,
            0x060000 => Part::Cursor,
            _ => Part::Main,
        }
    }
}

/// Type-safe wrapper for `lv_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Align {
    /// LVGL default alignment.
    Default = 0,
    /// Top-left corner.
    TopLeft = 1,
    /// Top center.
    TopMid = 2,
    /// Top-right corner.
    TopRight = 3,
    /// Bottom-left corner.
    BottomLeft = 4,
    /// Bottom center.
    BottomMid = 5,
    /// Bottom-right corner.
    BottomRight = 6,
    /// Left center.
    LeftMid = 7,
    /// Right center.
    RightMid = 8,
    /// Centered in parent.
    Center = 9,
    /// Outside top-left.
    OutTopLeft = 10,
    /// Outside top center.
    OutTopMid = 11,
    /// Outside top-right.
    OutTopRight = 12,
    /// Outside bottom-left.
    OutBottomLeft = 13,
    /// Outside bottom center.
    OutBottomMid = 14,
    /// Outside bottom-right.
    OutBottomRight = 15,
    /// Outside left-top.
    OutLeftTop = 16,
    /// Outside left center.
    OutLeftMid = 17,
    /// Outside left-bottom.
    OutLeftBottom = 18,
    /// Outside right-top.
    OutRightTop = 19,
    /// Outside right center.
    OutRightMid = 20,
    /// Outside right-bottom.
    OutRightBottom = 21,
}

/// Type-safe wrapper for `lv_text_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum TextAlign {
    /// Auto (based on text direction).
    Auto = 0,
    /// Left-aligned.
    Left = 1,
    /// Center-aligned.
    Center = 2,
    /// Right-aligned.
    Right = 3,
}

/// Type-safe wrapper for `lv_base_dir_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum BaseDir {
    /// Left to right.
    Ltr = 0,
    /// Right to left.
    Rtl = 1,
    /// Auto-detect from content.
    Auto = 2,
}

/// Implemented by any type that wraps an LVGL object handle.
///
/// Allows widget constructors to accept any [`Obj`], [`Screen`](super::Screen),
/// or other widget as a parent without exposing raw pointers.
pub trait AsLvHandle {
    /// Return the raw `lv_obj_t` pointer. Must be non-null for any live widget.
    fn lv_handle(&self) -> *mut lv_obj_t;
}

/// Owning wrapper around an `lv_obj_t`. Calls `lv_obj_delete` on drop.
///
/// All LVGL widget types wrap an `Obj` and `Deref` to it for style/layout
/// methods. Style-setter methods return `&Self` to allow chaining.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Obj, Screen, Align};
///
/// let screen = Screen::active().unwrap();
/// let label = oxivgl::widgets::Label::new(&screen).unwrap();
/// label.align(Align::Center, 0, 0).bg_color(0x112233).bg_opa(128);
/// ```
pub struct Obj<'p> {
    handle: *mut lv_obj_t,
    pub(super) _styles: RefCell<Vec<crate::style::Style>>,
    _parent: PhantomData<&'p lv_obj_t>,
}

impl<'p> core::fmt::Debug for Obj<'p> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Obj").field("handle", &self.handle).finish()
    }
}

impl<'p> Drop for Obj<'p> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: handle non-null; Obj is non-Clone so this is the unique owner.
            // lv_obj_delete (LVGL v9.3, lv_obj.c) calls lv_obj_remove_style_all
            // (lv_obj.c:521) and lv_anim_delete(obj, NULL) (lv_obj.c:525) internally,
            // so all style and animation back-references are cleared before Rust
            // drops _styles and any live Anim.
            // Re-verify these call sites when upgrading LVGL.
            unsafe { lv_obj_delete(self.handle) };
        }
    }
}

impl<'p> AsLvHandle for Obj<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.handle
    }
}

impl<'p> Obj<'p> {
    /// Create a new base object as a child of `parent`.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        // SAFETY: parent.lv_handle() is a valid non-null LVGL object; lv_init() was
        // called.
        let handle = unsafe { lv_obj_create(parent.lv_handle()) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Obj::from_raw(handle))
        }
    }

    /// Wrap a raw LVGL pointer. `ptr` must be non-null and owned by the caller.
    pub(crate) fn from_raw(ptr: *mut lv_obj_t) -> Self {
        Obj {
            handle: ptr,
            _styles: RefCell::new(Vec::new()),
            _parent: PhantomData,
        }
    }

    /// Return the raw `lv_obj_t` pointer.
    pub fn handle(&self) -> *mut lv_obj_t {
        self.handle
    }

    // ── Position / size ──────────────────────────────────────────────────

    /// Set alignment relative to parent with X/Y offset.
    pub fn align(&self, alignment: Align, x_offset: i32, y_offset: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_align(self.handle, alignment as lv_align_t, x_offset, y_offset) };
        self
    }

    /// Set X position.
    pub fn x(&self, x: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_x(self.handle, x) };
        self
    }

    /// Set Y position.
    pub fn y(&self, y: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_y(self.handle, y) };
        self
    }

    /// Set width and height.
    pub fn size(&self, w: i32, h: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_size(self.handle, w, h) };
        self
    }

    /// Set width.
    pub fn width(&self, w: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_width(self.handle, w) };
        self
    }

    /// Set height.
    pub fn height(&self, h: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_height(self.handle, h) };
        self
    }

    /// Set X and Y position.
    pub fn pos(&self, x: i32, y: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_pos(self.handle, x, y) };
        self
    }

    /// Center in parent.
    pub fn center(&self) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_center(self.handle) };
        self
    }

    /// Position this object relative to `base` using `lv_obj_align_to`.
    pub fn align_to(&self, base: &impl AsLvHandle, align: Align, x: i32, y: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle and base.lv_handle() non-null (asserted / guaranteed by AsLvHandle).
        unsafe { lv_obj_align_to(self.handle, base.lv_handle(), align as lv_align_t, x, y) };
        self
    }

    /// Apply a 3×3 matrix transform (scale, rotate, skew).
    ///
    /// Requires `LV_DRAW_TRANSFORM_USE_MATRIX = 1` in `lv_conf.h`.
    ///
    /// # Partial rendering caveat
    ///
    /// `refr_obj_matrix` inverse-transforms the render band's clip area and
    /// draws directly into the band buffer. With partial rendering (small
    /// band buffers, e.g. 40 lines), the inverse-transformed coordinates
    /// can exceed the buffer bounds, causing a crash
    /// (`LoadProhibited` / SIGSEGV). This happens because
    /// `refr_check_obj_clip_overflow` only checks style-based rotation, not
    /// matrix transforms set via this method.
    ///
    /// **Safe only when the display uses a full-screen buffer** (host) or
    /// the transformed bounding box fits entirely within a single render
    /// band (very small objects). For embedded targets with partial
    /// rendering, prefer [`StyleBuilder::transform_rotation`](crate::style::StyleBuilder::transform_rotation) — it allocates an
    /// intermediate layer but handles band clipping correctly.
    pub fn set_transform(&self, matrix: &Matrix) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null, matrix pointer valid.
        unsafe { lv_obj_set_transform(self.handle, matrix.as_ptr()) };
        self
    }

    /// Remove any matrix transform from this object.
    pub fn reset_transform(&self) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_obj_reset_transform(self.handle) };
        self
    }

    // ── Getters ──────────────────────────────────────────────────────────

    /// Get current X position after layout.
    pub fn get_x(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_get_x(self.handle) }
    }

    /// Get current Y position after layout.
    pub fn get_y(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_get_y(self.handle) }
    }

    /// Get X position as set by the user (before alignment resolution).
    pub fn get_x_aligned(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        unsafe { lv_obj_get_x_aligned(self.handle) }
    }

    /// Get Y position as set by the user (before alignment resolution).
    pub fn get_y_aligned(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        unsafe { lv_obj_get_y_aligned(self.handle) }
    }

    /// Get current width after layout.
    pub fn get_width(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_get_width(self.handle) }
    }

    /// Get current height after layout.
    pub fn get_height(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_get_height(self.handle) }
    }

    // ── State / flags ────────────────────────────────────────────────────

    /// Add an object state (e.g. checked, pressed).
    pub fn add_state(&self, state: crate::enums::ObjState) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_add_state(self.handle, state.0) };
        self
    }

    /// Remove an object state.
    pub fn remove_state(&self, state: crate::enums::ObjState) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_remove_state(self.handle, state.0) };
        self
    }

    /// Check if the object has the given state.
    pub fn has_state(&self, state: crate::enums::ObjState) -> bool {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_has_state(self.handle, state.0) }
    }

    /// Add an object flag (e.g. clickable, scrollable).
    pub fn add_flag(&self, flag: crate::enums::ObjFlag) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_add_flag(self.handle, flag.0) };
        self
    }

    /// Remove an object flag.
    pub fn remove_flag(&self, flag: crate::enums::ObjFlag) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_remove_flag(self.handle, flag.0) };
        self
    }

    /// Check if the object has the given flag.
    pub fn has_flag(&self, flag: crate::enums::ObjFlag) -> bool {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_has_flag(self.handle, flag.0) }
    }

    /// Remove the SCROLLABLE flag (convenience).
    pub fn remove_scrollable(&self) -> &Self {
        self.remove_flag(crate::enums::ObjFlag::SCROLLABLE)
    }

    /// Remove the CLICKABLE flag (convenience).
    pub fn remove_clickable(&self) -> &Self {
        self.remove_flag(crate::enums::ObjFlag::CLICKABLE)
    }

    /// Set the scrollbar display mode.
    pub fn set_scrollbar_mode(&self, mode: crate::enums::ScrollbarMode) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_scrollbar_mode(self.handle, mode as lv_scrollbar_mode_t) };
        self
    }

    /// Set horizontal scroll snap alignment.
    pub fn set_scroll_snap_x(&self, snap: crate::enums::ScrollSnap) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_scroll_snap_x(self.handle, snap as lv_scroll_snap_t) };
        self
    }

    /// Set vertical scroll snap alignment.
    pub fn set_scroll_snap_y(&self, snap: crate::enums::ScrollSnap) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_scroll_snap_y(self.handle, snap as lv_scroll_snap_t) };
        self
    }

    /// Set allowed scroll direction(s).
    pub fn set_scroll_dir(&self, dir: crate::enums::ScrollDir) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_scroll_dir(self.handle, dir.0 as lv_dir_t) };
        self
    }

    /// Scroll to an absolute position with optional animation.
    pub fn scroll_to(&self, x: i32, y: i32, anim: bool) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_scroll_to(self.handle, x, y, anim) };
        self
    }

    /// Scroll this child into view within its parent.
    pub fn scroll_to_view(&self, anim: bool) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_scroll_to_view(self.handle, anim) };
        self
    }

    /// Update snap alignment after children are added.
    pub fn update_snap(&self, anim: bool) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_update_snap(self.handle, anim) };
        self
    }

    /// Get the current horizontal scroll position.
    pub fn get_scroll_x(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_get_scroll_x(self.handle) }
    }

    /// Get the current vertical scroll position.
    pub fn get_scroll_y(&self) -> i32 {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_get_scroll_y(self.handle) }
    }

    /// Get the number of children.
    pub fn get_child_count(&self) -> u32 {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_get_child_count(self.handle) }
    }

    /// Move this object to the foreground (on top of siblings).
    pub fn move_foreground(&self) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_move_foreground(self.handle) };
        self
    }

    /// Send an event to this object programmatically.
    pub fn send_event(&self, code: crate::enums::EventCode) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_send_event(self.handle, code.0, core::ptr::null_mut()) };
        self
    }

    // ── Events ───────────────────────────────────────────────────────────

    /// Add a raw event callback with user data.
    ///
    /// Prefer [`on`](Self::on) for stateless callbacks. Use this only when
    /// you need to pass context via `user_data`.
    ///
    /// # Safety
    ///
    /// `user_data` must remain valid for the entire lifetime of this widget.
    /// LVGL stores the pointer in the event handler list
    /// (`lv_obj_add_event_cb`). Passing a dangling pointer causes UB when
    /// the event fires.
    pub unsafe fn on_event(
        &self,
        cb: unsafe extern "C" fn(*mut lv_event_t),
        filter: crate::enums::EventCode,
        user_data: *mut c_void,
    ) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null; cb is a valid extern "C" fn pointer.
        // Caller guarantees user_data validity per the Safety contract above.
        unsafe { lv_obj_add_event_cb(self.handle, Some(cb), filter.0, user_data) };
        self
    }

    /// Register a simple per-widget event callback (no View state access).
    ///
    /// ```ignore
    /// btn.on(EventCode::CLICKED, |_event| {
    ///     // handle click — no access to View fields
    /// });
    /// ```
    ///
    /// For handlers that need View state, use [`View::on_event`](crate::view::View::on_event)
    /// with event bubbling instead.
    pub fn on(&self, code: crate::enums::EventCode, cb: fn(&crate::event::Event)) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");

        unsafe extern "C" fn trampoline(e: *mut lv_event_t) {
            // SAFETY: user_data was set to a fn pointer in on(); transmute
            // back. fn pointers are pointer-sized.
            unsafe {
                let cb_ptr = lv_event_get_user_data(e) as *const ();
                let cb: fn(&crate::event::Event) = core::mem::transmute(cb_ptr);
                let event = crate::event::Event::from_raw(e);
                cb(&event);
            }
        }

        // SAFETY: handle non-null; cb is stored as user_data and retrieved by
        // trampoline. fn pointers have the same size as *mut c_void.
        unsafe {
            lv_obj_add_event_cb(
                self.handle,
                Some(trampoline),
                code.0,
                cb as *const () as *mut c_void,
            )
        };
        self
    }

    /// Enable event bubbling on this widget.
    /// Shorthand for `self.add_flag(ObjFlag::EVENT_BUBBLE)`.
    pub fn bubble_events(&self) -> &Self {
        self.add_flag(crate::enums::ObjFlag::EVENT_BUBBLE)
    }

    /// Enable `DRAW_TASK_ADDED` events on this widget.
    /// Required for custom draw hooks in `on_event`.
    pub fn send_draw_task_events(&self) -> &Self {
        self.add_flag(crate::enums::ObjFlag::SEND_DRAW_TASK_EVENTS)
    }

    // ── Children ─────────────────────────────────────────────────────────

    /// Get child widget by index (0-based). Returns `None` if index out of range.
    /// The returned `Child` does NOT own the pointer — LVGL frees it when the parent is deleted.
    pub fn get_child(&self, idx: i32) -> Option<super::Child<Obj<'_>>> {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above); LVGL returns NULL for out-of-range idx.
        let child_ptr = unsafe { lv_obj_get_child(self.handle, idx) };
        if child_ptr.is_null() {
            None
        } else {
            Some(super::Child::new(Obj::from_raw(child_ptr)))
        }
    }
}
