// SPDX-License-Identifier: MIT OR Apache-2.0
use core::ptr::addr_of;

use lvgl_rust_sys::lv_font_t;

/// Wrapper around an LVGL font pointer.
#[derive(Copy, Clone, Debug)]
pub struct Font(pub(crate) *const lv_font_t);

// SAFETY: lv_font_t is immutable C data; sharing across threads is safe.
unsafe impl Send for Font {}
unsafe impl Sync for Font {}

impl Font {
    /// # Safety
    /// `ptr` must point to a valid, static LVGL font object.
    pub const unsafe fn from_raw(ptr: *const lv_font_t) -> Self {
        Font(ptr)
    }

    /// Create a [`Font`] from an opaque extern-C symbol address.
    ///
    /// Use this to reference custom fonts compiled from `.c` files without
    /// importing `lvgl_rust_sys` directly:
    ///
    /// ```no_run
    /// extern "C" { static my_font: u8; }
    /// static MY_FONT: Font = unsafe {
    ///     Font::from_extern(core::ptr::addr_of!(my_font) as *const ())
    /// };
    /// ```
    ///
    /// # Safety
    /// `ptr` must be the address of a valid, static `lv_font_t` object in
    /// memory.
    pub const unsafe fn from_extern(ptr: *const ()) -> Self {
        Font(ptr as *const lv_font_t)
    }

    /// Return the raw LVGL font pointer. Valid for the `'static` lifetime of
    /// the font object.
    pub fn as_ptr(self) -> *const lv_font_t {
        self.0
    }
}

/// LVGL built-in Montserrat 12 pt.
// SAFETY: lv_font_montserrat_12 is a valid static font compiled into the
// binary.
pub static MONTSERRAT_12: Font = Font(addr_of!(lvgl_rust_sys::lv_font_montserrat_12));

/// LVGL built-in Montserrat 32 pt.
// SAFETY: lv_font_montserrat_32 is a valid static font compiled into the
// binary.
pub static MONTSERRAT_32: Font = Font(addr_of!(lvgl_rust_sys::lv_font_montserrat_32));
