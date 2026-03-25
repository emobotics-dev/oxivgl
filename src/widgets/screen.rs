// SPDX-License-Identifier: MIT OR Apache-2.0
//! [`Screen`] — non-owning reference to the active LVGL screen.

use alloc::vec::Vec;
use core::cell::RefCell;

use lvgl_rust_sys::*;

use super::obj::AsLvHandle;
use crate::{
    layout::{FlexAlign, FlexFlow},
    style::{Selector, Style},
};

/// Non-owning reference to the active LVGL screen. Does **not** delete it on
/// drop.
///
/// Obtain via [`Screen::active()`]. Use as a parent for top-level widgets.
///
/// # Style lifetime
///
/// Styles added via [`add_style`](Screen::add_style) are kept alive by this
/// `Screen` reference. If the `Screen` is dropped while child widgets still
/// reference those styles, the underlying `lv_style_t` memory is freed and
/// LVGL will read dangling pointers. Ensure the `Screen` outlives all styles
/// added to it (typically by storing it in the `View` struct).
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
    /// Rc clones of styles added via `add_style`. Keeps the `lv_style_t`
    /// alive as long as this Screen reference exists (spec §5.1).
    _styles: RefCell<Vec<Style>>,
}

impl core::fmt::Debug for Screen {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Screen").finish_non_exhaustive()
    }
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
        if handle.is_null() { None } else { Some(Screen { handle, _styles: RefCell::new(Vec::new()) }) }
    }

    /// Get the top layer — a global overlay above all screens.
    ///
    /// Returns a non-owning handle. The top layer is owned by LVGL and
    /// must never be deleted.
    ///
    /// **Warning:** Styles added to the returned `Child` handle will **not**
    /// be freed, because `Child` suppresses `Drop`. If you add styles to
    /// the top layer, they leak for the process lifetime.
    pub fn layer_top() -> super::child::Child<super::obj::Obj<'static>> {
        // SAFETY: lv_layer_top() returns a valid global object after lv_init().
        let handle = unsafe { lv_layer_top() };
        assert!(!handle.is_null(), "lv_layer_top returned NULL");
        super::child::Child::new(super::obj::Obj::from_raw(handle))
    }

    /// Return the raw `lv_obj_t` pointer for this screen.
    pub fn handle(&self) -> *mut lv_obj_t {
        self.handle
    }

    /// Remove the scrollable flag from the screen.
    pub fn remove_scrollable(&self) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_remove_flag(self.handle, crate::enums::ObjFlag::SCROLLABLE.0) };
        self
    }

    /// Set background color from RGB hex.
    pub fn bg_color(&self, color: u32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_bg_color(self.handle, lv_color_hex(color), 0) };
        self
    }

    /// Set background opacity (0–255).
    pub fn bg_opa(&self, opa: u8) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_bg_opa(self.handle, opa as lv_opa_t, 0) };
        self
    }

    /// Set top padding.
    pub fn pad_top(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_top(self.handle, p, 0) };
        self
    }

    /// Set bottom padding.
    pub fn pad_bottom(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_bottom(self.handle, p, 0) };
        self
    }

    /// Set left padding.
    pub fn pad_left(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_left(self.handle, p, 0) };
        self
    }

    /// Set right padding.
    pub fn pad_right(&self, p: i32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_pad_right(self.handle, p, 0) };
        self
    }

    /// Set default text color from RGB hex.
    pub fn text_color(&self, color: u32) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_style_text_color(self.handle, lv_color_hex(color), 0) };
        self
    }

    /// Set flex layout flow direction.
    pub fn set_flex_flow(&self, flow: FlexFlow) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe { lv_obj_set_flex_flow(self.handle, flow as lv_flex_flow_t) };
        self
    }

    /// Apply a style for the given selector.
    ///
    /// Clones the `Style` Rc to keep the `lv_style_t` alive for the
    /// screen's lifetime (spec §5.1).
    pub fn add_style(&self, style: &Style, selector: impl Into<Selector>) -> &Self {
        self._styles.borrow_mut().push(style.clone());
        let selector = selector.into().raw();
        // SAFETY: handle non-null (Screen::active() returns None for null).
        // Style Rc clone above keeps the lv_style_t valid.
        unsafe { lv_obj_add_style(self.handle, style.lv_ptr(), selector) };
        self
    }

    /// Set flex alignment (main, cross, track).
    pub fn set_flex_align(&self, main: FlexAlign, cross: FlexAlign, track: FlexAlign) -> &Self {
        // SAFETY: handle non-null (Screen::active() returns None for null).
        unsafe {
            lv_obj_set_flex_align(
                self.handle,
                main as lv_flex_align_t,
                cross as lv_flex_align_t,
                track as lv_flex_align_t,
            )
        };
        self
    }
}
