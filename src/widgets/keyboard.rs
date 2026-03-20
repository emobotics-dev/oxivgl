// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// Keyboard input mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum KeyboardMode {
    /// Lowercase letters (default).
    TextLower = 0,
    /// Uppercase letters.
    TextUpper = 1,
    /// Special characters.
    Special = 2,
    /// Numeric keypad.
    Number = 3,
}

/// LVGL keyboard widget (on-screen virtual keyboard).
///
/// Requires `LV_USE_KEYBOARD = 1` in `lv_conf.h` (which also requires
/// `LV_USE_BUTTONMATRIX` and `LV_USE_TEXTAREA`).
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Keyboard, KeyboardMode, Screen, Textarea};
///
/// let screen = Screen::active().unwrap();
/// let ta = Textarea::new(&screen).unwrap();
/// let kb = Keyboard::new(&screen).unwrap();
/// kb.set_textarea(&ta);
/// kb.set_mode(KeyboardMode::Number);
/// ```
#[derive(Debug)]
pub struct Keyboard<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Keyboard<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Keyboard<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Keyboard<'p> {
    /// Create a new keyboard widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_keyboard_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Keyboard { obj: Obj::from_raw(handle) }) }
    }

    /// Associate the keyboard with a textarea.
    ///
    /// Key presses will be sent to the textarea. Pass a different textarea
    /// to switch focus.
    pub fn set_textarea(&self, ta: &impl AsLvHandle) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Keyboard handle cannot be null");
        // SAFETY: handle non-null; ta.lv_handle() is a valid LVGL textarea object.
        unsafe { lv_keyboard_set_textarea(self.obj.handle(), ta.lv_handle()) };
        self
    }

    /// Set the keyboard mode (letter case, special chars, numbers).
    pub fn set_mode(&self, mode: KeyboardMode) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Keyboard handle cannot be null");
        // SAFETY: handle non-null; mode is a valid lv_keyboard_mode_t value.
        unsafe { lv_keyboard_set_mode(self.obj.handle(), mode as lv_keyboard_mode_t) };
        self
    }
}
