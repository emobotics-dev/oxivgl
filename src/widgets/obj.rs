// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_void, marker::PhantomData, ptr::null_mut};

use lvgl_rust_sys::*;

use super::WidgetError;

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
}

/// Type-safe wrapper for `lv_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Align {
    Default = 0,
    TopLeft = 1,
    TopMid = 2,
    TopRight = 3,
    BottomLeft = 4,
    BottomMid = 5,
    BottomRight = 6,
    LeftMid = 7,
    RightMid = 8,
    Center = 9,
    OutTopLeft = 10,
    OutTopMid = 11,
    OutTopRight = 12,
    OutBottomLeft = 13,
    OutBottomMid = 14,
    OutBottomRight = 15,
    OutLeftTop = 16,
    OutLeftMid = 17,
    OutLeftBottom = 18,
    OutRightTop = 19,
    OutRightMid = 20,
    OutRightBottom = 21,
}

/// Type-safe wrapper for `lv_text_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum TextAlign {
    Auto = 0,
    Left = 1,
    Center = 2,
    Right = 3,
}

/// Implemented by any type that wraps an LVGL object handle.
///
/// Allows widget constructors to accept any [`Obj`], [`Screen`], or other
/// widget as a parent without exposing raw pointers.
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
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        // SAFETY: parent.lv_handle() is a valid non-null LVGL object; lv_init() was
        // called.
        let handle = unsafe { lv_obj_create(parent.lv_handle()) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Obj::from_raw(handle)) }
    }

    /// Wrap a raw LVGL pointer. `ptr` must be non-null and owned by the caller.
    pub fn from_raw(ptr: *mut lv_obj_t) -> Self {
        Obj { handle: ptr, _parent: PhantomData }
    }

    /// Return the raw `lv_obj_t` pointer.
    pub fn handle(&self) -> *mut lv_obj_t {
        self.handle
    }

    pub fn align(&self, alignment: Align, x_offset: i32, y_offset: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above); all lv_obj_* fns safe with valid
        // pointer.
        unsafe { lv_obj_align(self.handle, alignment as lv_align_t, x_offset, y_offset) };
        self
    }

    pub fn x(&self, x: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_x(self.handle, x) };
        self
    }

    pub fn y(&self, y: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_y(self.handle, y) };
        self
    }

    pub fn size(&self, w: i32, h: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_size(self.handle, w, h) };
        self
    }

    pub fn width(&self, w: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_width(self.handle, w) };
        self
    }

    pub fn height(&self, h: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_height(self.handle, h) };
        self
    }

    pub fn bg_color(&self, color: u32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_color(self.handle, lv_color_hex(color), 0) };
        self
    }

    pub fn bg_opa(&self, opa: u8) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_opa(self.handle, opa as lv_opa_t, 0) };
        self
    }

    pub fn border_width(&self, w: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_border_width(self.handle, w, 0) };
        self
    }

    pub fn pad(&self, p: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_all(self.handle, p, 0) };
        self
    }

    pub fn pad_top(&self, p: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_top(self.handle, p, 0) };
        self
    }

    pub fn pad_bottom(&self, p: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_bottom(self.handle, p, 0) };
        self
    }

    pub fn pad_left(&self, p: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_left(self.handle, p, 0) };
        self
    }

    pub fn pad_right(&self, p: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_right(self.handle, p, 0) };
        self
    }

    pub fn remove_scrollable(&self) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_remove_flag(self.handle, lv_obj_flag_t_LV_OBJ_FLAG_SCROLLABLE) };
        self
    }

    pub fn remove_clickable(&self) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_remove_flag(self.handle, lv_obj_flag_t_LV_OBJ_FLAG_CLICKABLE) };
        self
    }

    /// Apply a style to this object for the given selector.
    /// Pass `0` for default state, `lv_state_t_LV_STATE_PRESSED` (= 128) for pressed.
    ///
    /// The `style` must outlive this object (see [`Style`](super::Style) docs).
    pub fn add_style(&self, style: &super::Style, selector: u32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null; style.inner pointer valid for style's lifetime.
        unsafe {
            lv_obj_add_style(self.handle, &style.inner as *const lv_style_t, selector)
        };
        self
    }

    /// Remove all styles from this object.
    pub fn remove_style_all(&self) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_obj_remove_style_all(self.handle) };
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_text_color(self.handle, lv_color_hex(color), 0) };
        self
    }

    pub fn text_font(&self, font: crate::fonts::Font) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        assert_ne!(font.as_ptr(), null_mut(), "Font pointer cannot be null");
        // SAFETY: handle and font pointer non-null (asserted above).
        unsafe { lv_obj_set_style_text_font(self.handle, font.as_ptr(), 0) };
        self
    }

    /// Alias for [`text_font`].
    pub fn font(&self, font: crate::fonts::Font) -> &Self {
        self.text_font(font)
    }

    pub fn text_align(&self, align: TextAlign) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_text_align(self.handle, align as lv_text_align_t, 0) };
        self
    }

    pub fn opa(&self, opa: u8) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_opa(self.handle, opa as lv_opa_t, 0) };
        self
    }

    pub fn pos(&self, x: i32, y: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_pos(self.handle, x, y) };
        self
    }

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

    /// Add an event callback. `cb` is an `extern "C"` function pointer.
    /// `filter`: use `lv_event_code_t_LV_EVENT_ALL` to receive all events,
    /// or a specific code like `lv_event_code_t_LV_EVENT_CLICKED`.
    /// `user_data`: arbitrary pointer passed to the callback; pass `core::ptr::null_mut()` if unused.
    pub fn on_event(
        &self,
        cb: unsafe extern "C" fn(*mut lv_event_t),
        filter: lv_event_code_t,
        user_data: *mut c_void,
    ) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null; cb is a valid extern "C" fn pointer.
        unsafe { lv_obj_add_event_cb(self.handle, Some(cb), filter, user_data) };
        self
    }

    /// Set `lv_obj_set_style_line_width` for the given LVGL style part.
    pub fn line_width(&self, part: Part, width: i32) -> &Self {
        assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_line_width(self.handle, width, part as u32) };
        self
    }
}

/// Non-owning reference to the active LVGL screen. Does **not** delete it on
/// drop.
///
/// Obtain via [`Screen::active()`]. Use as a parent for top-level widgets.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::Screen;
///
/// let screen = Screen::active().expect("LVGL not initialized");
/// screen.bg_color(0x06080f).bg_opa(255).pad_top(6).pad_bottom(6);
/// ```
pub struct Screen {
    handle: *mut lv_obj_t,
}

impl AsLvHandle for Screen {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.handle
    }
}

impl Screen {
    /// Returns `None` if LVGL has no active screen yet.
    pub fn active() -> Option<Self> {
        // SAFETY: lv_screen_active() is safe after lv_init(); NULL result is handled
        // below.
        let handle = unsafe { lv_screen_active() };
        if handle.is_null() { None } else { Some(Screen { handle }) }
    }

    /// Return the raw `lv_obj_t` pointer for this screen.
    pub fn handle(&self) -> *mut lv_obj_t {
        self.handle
    }

    pub fn remove_scrollable(&self) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_remove_flag(self.handle, lv_obj_flag_t_LV_OBJ_FLAG_SCROLLABLE) };
        self
    }

    pub fn bg_color(&self, color: u32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_bg_color(self.handle, lv_color_hex(color), 0) };
        self
    }

    pub fn bg_opa(&self, opa: u8) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_bg_opa(self.handle, opa as lv_opa_t, 0) };
        self
    }

    pub fn pad_top(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_top(self.handle, p, 0) };
        self
    }

    pub fn pad_bottom(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_bottom(self.handle, p, 0) };
        self
    }

    pub fn pad_left(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_left(self.handle, p, 0) };
        self
    }

    pub fn pad_right(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_right(self.handle, p, 0) };
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_text_color(self.handle, lv_color_hex(color), 0) };
        self
    }
}
