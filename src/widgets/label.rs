// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL text label widget.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Align, Label, Screen};
///
/// let screen = Screen::active().unwrap();
/// let label = Label::new(&screen).unwrap();
/// label.text("Hello").align(Align::Center, 0, 0);
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
    /// Create a new label widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_label_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Label { obj: Obj::from_raw(handle) }) }
    }

    /// Set label text. Accepts any `&str` (no NUL terminator required).
    /// LVGL copies the string internally. Truncates at 127 bytes.
    /// For longer text use [`text_long`](Self::text_long).
    pub fn text(&self, s: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        let bytes = s.as_bytes();
        let len = bytes.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&bytes[..len]);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated
        // (zero-initialized, len ≤ 127).
        unsafe { lv_label_set_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set translation tag — the label text will auto-update when the
    /// language changes via [`translation::set_language`](crate::translation::set_language).
    ///
    /// Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.
    pub fn set_translation_tag(&self, tag: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        let len = tag.len().min(127);
        let mut buf = [0u8; 128];
        buf[..len].copy_from_slice(&tag.as_bytes()[..len]);
        // SAFETY: handle non-null; buf is NUL-terminated (zero-initialized).
        // LVGL copies the tag string internally.
        unsafe { lv_label_set_translation_tag(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set label text without the 127-byte limit. Heap-allocates a
    /// NUL-terminated copy. Use [`text`](Self::text) for short UI labels.
    pub fn text_long(&self, s: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        let mut buf = Vec::with_capacity(s.len() + 1);
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
        // SAFETY: handle non-null (asserted above); buf is NUL-terminated.
        // LVGL copies the string internally so buf can be dropped.
        unsafe { lv_label_set_text(self.obj.handle(), buf.as_ptr() as *const c_char) };
        self
    }

    /// Set the label long mode (wrap, scroll, clip, etc.).
    pub fn set_long_mode(&self, mode: LabelLongMode) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Label handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_label_set_long_mode(self.obj.handle(), mode as u32) };
        self
    }
}

/// Label long-mode behaviour.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum LabelLongMode {
    /// Keep the object width, wrap lines and expand height.
    Wrap = lv_label_long_mode_t_LV_LABEL_LONG_MODE_WRAP,
    /// Keep size, write dots at end if text too long.
    Dots = lv_label_long_mode_t_LV_LABEL_LONG_MODE_DOTS,
    /// Keep size, scroll text back and forth.
    Scroll = lv_label_long_mode_t_LV_LABEL_LONG_MODE_SCROLL,
    /// Keep size, scroll text circularly.
    ScrollCircular = lv_label_long_mode_t_LV_LABEL_LONG_MODE_SCROLL_CIRCULAR,
    /// Keep size, clip text.
    Clip = lv_label_long_mode_t_LV_LABEL_LONG_MODE_CLIP,
}
