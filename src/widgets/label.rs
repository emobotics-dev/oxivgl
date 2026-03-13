// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use heapless::CString;
use lvgl_rust_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// LVGL text label widget.
///
/// Text is limited to 19 UTF-8 bytes (plus NUL) per [`text`](Label::text) call
/// due to the internal `heapless::CString::<20>` buffer.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Label, Screen};
///
/// let screen = Screen::active().unwrap();
/// let label = Label::new(&screen).unwrap();
/// label.text("Hello\0").unwrap().align(Align::Center, 0, 0);
/// ```
#[derive(Debug)]
pub struct Label<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Label<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Label<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Label<'p> {
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_label_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Label {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Set label text. `text` must include a NUL terminator and be ≤ 20 bytes
    /// total.
    pub fn text(&self, text: &str) -> Result<&Self, WidgetError> {
        let c_str = CString::<20>::from_bytes_with_nul(text.as_bytes())?;
        let c_ptr = c_str.as_ptr() as *mut c_char;
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        assert_ne!(c_ptr, null_mut(), "CString pointer cannot be null");
        // SAFETY: handle and c_ptr non-null (asserted above); c_str is a valid
        // NUL-terminated C string backed by the local CString buffer, alive for
        // the duration of the call.
        unsafe { lv_label_set_text(self.obj.handle(), c_ptr) };
        Ok(self)
    }

    /// Set label text without requiring a NUL terminator. Truncates at 63 bytes.
    /// LVGL copies the string internally.
    pub fn set_text(&self, text: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        let bytes = text.as_bytes();
        let len = bytes.len().min(63);
        let mut buf = [0u8; 64];
        buf[..len].copy_from_slice(&bytes[..len]);
        unsafe { lv_label_set_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set label text from a pre-NUL-terminated byte literal.
    /// Panics in debug mode if the last byte is not `\0`.
    ///
    /// Use for long or static strings: `label.set_text_static(b"Hello world\0");`
    pub fn set_text_static(&self, text: &[u8]) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        debug_assert!(text.last() == Some(&0), "text must be NUL-terminated");
        unsafe { lv_label_set_text(self.obj.handle(), text.as_ptr() as *const c_char) };
        self
    }
}
