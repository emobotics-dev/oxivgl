// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use alloc::vec::Vec;
use lvgl_rust_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// LVGL drop-down list widget.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Dropdown, Screen};
///
/// let screen = Screen::active().unwrap();
/// let dd = Dropdown::new(&screen).unwrap();
/// dd.set_options("Apple\nBanana\nOrange");
/// dd.align(Align::Center, 0, 0);
/// ```
#[derive(Debug)]
pub struct Dropdown<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Dropdown<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Dropdown<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

/// Drop-down list open direction.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DdDir {
    /// Open below (default).
    Bottom = lv_dir_t_LV_DIR_BOTTOM,
    /// Open above.
    Top = lv_dir_t_LV_DIR_TOP,
    /// Open to the left.
    Left = lv_dir_t_LV_DIR_LEFT,
    /// Open to the right.
    Right = lv_dir_t_LV_DIR_RIGHT,
}

impl<'p> Dropdown<'p> {
    /// Create a new dropdown widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via LvglDriver.
        let handle = unsafe { lv_dropdown_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Dropdown {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Set dropdown options as newline-separated string.
    /// LVGL copies the string internally.
    pub fn set_options(&self, opts: &str) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        let mut buf = Vec::with_capacity(opts.len() + 1);
        buf.extend_from_slice(opts.as_bytes());
        buf.push(0);
        // SAFETY: handle non-null; buf is NUL-terminated. LVGL copies internally.
        unsafe { lv_dropdown_set_options(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set the open direction.
    pub fn set_dir(&self, dir: DdDir) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_set_dir(self.obj.handle(), dir as lv_dir_t) };
        self
    }

    /// Set the dropdown symbol (typically an arrow icon string).
    pub fn set_symbol(&self, symbol: &str) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        let bytes = symbol.as_bytes();
        let len = bytes.len().min(15);
        let mut buf = [0u8; 16];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null; buf NUL-terminated. LVGL treats symbol as
        // void* (can be string or image pointer).
        unsafe {
            lv_dropdown_set_symbol(self.obj.handle(), buf.as_ptr() as *const core::ffi::c_void)
        };
        self
    }

    /// Set the selected item index.
    pub fn set_selected(&self, idx: u32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_set_selected(self.obj.handle(), idx) };
        self
    }

    /// Get the currently selected item index.
    pub fn get_selected(&self) -> u32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Dropdown handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_dropdown_get_selected(self.obj.handle()) }
    }
}
