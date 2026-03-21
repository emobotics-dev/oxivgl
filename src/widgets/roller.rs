// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL roller (scroll-wheel picker) widget.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Roller, RollerMode, Screen};
///
/// let screen = Screen::active().unwrap();
/// let roller = Roller::new(&screen).unwrap();
/// roller.set_options("Jan\nFeb\nMar", RollerMode::Normal);
/// roller.set_visible_row_count(3);
/// roller.center();
/// ```
#[derive(Debug)]
pub struct Roller<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Roller<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Roller<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Roller scrolling mode.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum RollerMode {
    /// Roller stops at the first and last option.
    Normal = lv_roller_mode_t_LV_ROLLER_MODE_NORMAL,
    /// Roller wraps around infinitely.
    Infinite = lv_roller_mode_t_LV_ROLLER_MODE_INFINITE,
}

impl<'p> Roller<'p> {
    /// Create a new roller widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_roller_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Roller { obj: Obj::from_raw(handle) }) }
    }

    /// Set roller options as newline-separated string.
    pub fn set_options(&self, opts: &str, mode: RollerMode) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Roller handle cannot be null");
        let mut buf = Vec::with_capacity(opts.len() + 1);
        buf.extend_from_slice(opts.as_bytes());
        buf.push(0);
        // SAFETY: handle non-null; buf NUL-terminated. LVGL copies internally.
        unsafe { lv_roller_set_options(self.obj.handle(), buf.as_ptr() as *const c_char, mode as u32) };
        self
    }

    /// Set the number of visible rows.
    pub fn set_visible_row_count(&self, rows: u32) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Roller handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_roller_set_visible_row_count(self.obj.handle(), rows) };
        self
    }

    /// Set the selected item (0-based index).
    pub fn set_selected(&self, idx: u32, anim: bool) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Roller handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_roller_set_selected(self.obj.handle(), idx, anim) };
        self
    }

    /// Get the currently selected item index.
    pub fn get_selected(&self) -> u32 {
        assert_ne!(self.obj.handle(), null_mut(), "Roller handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_roller_get_selected(self.obj.handle()) }
    }
}
