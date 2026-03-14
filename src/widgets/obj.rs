// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_void, marker::PhantomData, ptr::null_mut};

use lvgl_rust_sys::*;

use super::WidgetError;

/// 3×3 affine transform matrix.
///
/// Chain operations via builder-style methods. Requires
/// `LV_DRAW_TRANSFORM_USE_MATRIX = 1` and `LV_USE_FLOAT = 1` in `lv_conf.h`.
///
/// ```ignore
/// let mut m = Matrix::identity();
/// m.scale(0.5, 0.5).rotate(45.0);
/// obj.set_transform(&m);
/// ```
pub struct Matrix {
    inner: lv_matrix_t,
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
#[derive(Clone, Copy, Debug)]
pub enum Part {
    /// Main background rectangle (`LV_PART_MAIN = 0x000000`).
    Main = 0x000000,
    /// Indicator (e.g. filled arc, slider thumb, `LV_PART_INDICATOR =
    /// 0x020000`).
    Indicator = 0x020000,
    /// Grab handle (`LV_PART_KNOB = 0x030000`).
    Knob = 0x030000,
    /// Repeated sub-elements such as tick marks (`LV_PART_ITEMS = 0x050000`).
    Items = 0x050000,
    /// Scrollbar part (`LV_PART_SCROLLBAR = 0x010000`).
    Scrollbar = lvgl_rust_sys::lv_part_t_LV_PART_SCROLLBAR,
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

/// Type-safe wrapper for `lv_flex_flow_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum FlexFlow {
    /// Horizontal left to right.
    Row = 0,
    /// Vertical top to bottom.
    Column = 1,
    /// Row with wrapping.
    RowWrap = 4,
    /// Row, right to left.
    RowReverse = 8,
    /// Row with wrapping, reversed.
    RowWrapReverse = 12,
    /// Column with wrapping.
    ColumnWrap = 5,
    /// Column, bottom to top.
    ColumnReverse = 9,
    /// Column with wrapping, reversed.
    ColumnWrapReverse = 13,
}

/// Type-safe wrapper for `lv_flex_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum FlexAlign {
    /// Align to start.
    Start = 0,
    /// Align to end.
    End = 1,
    /// Center alignment.
    Center = 2,
    /// Equal space around all items.
    SpaceEvenly = 3,
    /// Equal space around each item.
    SpaceAround = 4,
    /// Equal space between items.
    SpaceBetween = 5,
}

/// Type-safe wrapper for `lv_grid_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum GridAlign {
    /// Align to start.
    Start = 0,
    /// Center alignment.
    Center = 1,
    /// Align to end.
    End = 2,
    /// Stretch to fill cell.
    Stretch = 3,
    /// Equal space around all items.
    SpaceEvenly = 4,
    /// Equal space around each item.
    SpaceAround = 5,
    /// Equal space between items.
    SpaceBetween = 6,
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
            // SAFETY: handle non-null (checked); Obj is non-Clone, so this is the unique
            // owner.
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
    pub fn from_raw(ptr: *mut lv_obj_t) -> Self {
        Obj {
            handle: ptr,
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
    /// # Panics
    ///
    /// The LVGL SW renderer does not clip the transformed bounding box to
    /// display bounds. If the transformed object extends outside the screen
    /// (e.g. scaled up while positioned near an edge), the renderer may
    /// write out of bounds. **Always [`center()`](Self::center) or
    /// position the object so that its transformed extents stay within the
    /// display.**
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
    pub fn add_state(&self, state: super::ObjState) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_add_state(self.handle, state.0) };
        self
    }

    /// Remove an object state.
    pub fn remove_state(&self, state: super::ObjState) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_remove_state(self.handle, state.0) };
        self
    }

    /// Check if the object has the given state.
    pub fn has_state(&self, state: super::ObjState) -> bool {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_has_state(self.handle, state.0) }
    }

    /// Add an object flag (e.g. clickable, scrollable).
    pub fn add_flag(&self, flag: super::ObjFlag) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_add_flag(self.handle, flag.0) };
        self
    }

    /// Remove an object flag.
    pub fn remove_flag(&self, flag: super::ObjFlag) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_remove_flag(self.handle, flag.0) };
        self
    }

    /// Remove the SCROLLABLE flag (convenience).
    pub fn remove_scrollable(&self) -> &Self {
        self.remove_flag(super::ObjFlag::SCROLLABLE)
    }

    /// Remove the CLICKABLE flag (convenience).
    pub fn remove_clickable(&self) -> &Self {
        self.remove_flag(super::ObjFlag::CLICKABLE)
    }

    /// Set the scrollbar display mode.
    pub fn set_scrollbar_mode(&self, mode: super::ScrollbarMode) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_scrollbar_mode(self.handle, mode as lv_scrollbar_mode_t) };
        self
    }

    /// Set horizontal scroll snap alignment.
    pub fn set_scroll_snap_x(&self, snap: super::ScrollSnap) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_scroll_snap_x(self.handle, snap as lv_scroll_snap_t) };
        self
    }

    /// Set vertical scroll snap alignment.
    pub fn set_scroll_snap_y(&self, snap: super::ScrollSnap) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_scroll_snap_y(self.handle, snap as lv_scroll_snap_t) };
        self
    }

    /// Set allowed scroll direction(s).
    pub fn set_scroll_dir(&self, dir: super::ScrollDir) -> &Self {
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
    pub fn send_event(&self, code: super::EventCode) -> &Self {
        assert_ne!(self.handle, null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_send_event(self.handle, code.0, core::ptr::null_mut()) };
        self
    }

    // ── Events ───────────────────────────────────────────────────────────

    /// Add an event callback. `cb` is an `extern "C"` function pointer.
    /// `filter`: use `EventCode::ALL` to receive all events,
    /// or a specific code like `EventCode::CLICKED`.
    /// `user_data`: arbitrary pointer passed to the callback; pass `core::ptr::null_mut()` if unused.
    pub fn on_event(
        &self,
        cb: unsafe extern "C" fn(*mut lv_event_t),
        filter: super::EventCode,
        user_data: *mut c_void,
    ) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null; cb is a valid extern "C" fn pointer.
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
    pub fn on(&self, code: super::EventCode, cb: fn(&super::event::Event)) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");

        unsafe extern "C" fn trampoline(e: *mut lv_event_t) {
            // SAFETY: user_data was set to a fn pointer in on(); transmute
            // back. fn pointers are pointer-sized.
            unsafe {
                let cb_ptr = lv_event_get_user_data(e) as *const ();
                let cb: fn(&super::event::Event) = core::mem::transmute(cb_ptr);
                let event = super::event::Event::from_raw(e);
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
        self.add_flag(super::ObjFlag::EVENT_BUBBLE)
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
