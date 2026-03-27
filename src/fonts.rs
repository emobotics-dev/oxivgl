// SPDX-License-Identifier: MIT OR Apache-2.0
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::addr_of;

use oxivgl_sys::{lv_font_glyph_dsc_t, lv_font_get_glyph_dsc_fmt_txt, lv_font_t};

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
    /// importing `oxivgl_sys` directly:
    ///
    /// ```no_run
    /// use oxivgl::fonts::Font;
    /// unsafe extern "C" { static my_font: u8; }
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

/// LVGL built-in DejaVu 16 pt with Persian/Hebrew glyphs.
pub static DEJAVU_16_PERSIAN_HEBREW: Font =
    Font(addr_of!(oxivgl_sys::lv_font_dejavu_16_persian_hebrew));


/// LVGL built-in Source Han Sans SC 14 pt with CJK glyphs.
pub static SOURCE_HAN_SANS_SC_14_CJK: Font =
    Font(addr_of!(oxivgl_sys::lv_font_source_han_sans_sc_14_cjk));

/// LVGL built-in Source Han Sans SC 16 pt with CJK glyphs.
pub static SOURCE_HAN_SANS_SC_16_CJK: Font =
    Font(addr_of!(oxivgl_sys::lv_font_source_han_sans_sc_16_cjk));

/// Fixed-width font derived from an existing LVGL font.
///
/// Clones the source font's `lv_font_t` and overrides the `get_glyph_dsc`
/// callback so every glyph uses the same advance width, producing a
/// monospaced appearance from a proportional font.
///
/// Must be placed in a `static` because LVGL stores the font pointer.
///
/// # Example
///
/// ```ignore
/// use oxivgl::fonts::{FixedWidthFont, MONTSERRAT_20};
///
/// static MONO_FONT: FixedWidthFont = FixedWidthFont::new();
///
/// // In View::create():
/// let font = MONO_FONT.init(MONTSERRAT_20, 20);
/// label.text_font(font);
/// ```
pub struct FixedWidthFont {
    inner: UnsafeCell<MaybeUninit<lv_font_t>>,
}

// SAFETY: init() must be called from the LVGL task (single-threaded).
// After init the font data is effectively immutable (read-only by LVGL).
unsafe impl Send for FixedWidthFont {}
unsafe impl Sync for FixedWidthFont {}

impl FixedWidthFont {
    /// Create an uninitialised placeholder. Call [`init`](Self::init) once
    /// before use.
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::zeroed()),
        }
    }

    /// Initialise by cloning `source` and setting a fixed glyph advance
    /// width of `advance_w` pixels. Returns a [`Font`] handle suitable
    /// for `text_font()` / `style_text_font()`.
    ///
    /// Must be called exactly once, from the LVGL task, before any widget
    /// references the returned font.
    pub fn init(&self, source: Font, advance_w: u16) -> Font {
        // SAFETY: source.as_ptr() points to a valid static lv_font_t.
        // We copy the entire struct, then override the callback and
        // store advance_w in user_data.
        unsafe {
            let font_ptr = self.inner.get();
            core::ptr::copy_nonoverlapping(source.as_ptr(), (*font_ptr).as_mut_ptr(), 1);
            let font = (*font_ptr).as_mut_ptr();
            (*font).get_glyph_dsc = Some(fixed_width_get_glyph_dsc);
            (*font).user_data = advance_w as usize as *mut core::ffi::c_void;
            Font((*font_ptr).as_ptr())
        }
    }
}

/// Custom `get_glyph_dsc` callback that delegates to the original
/// format-text decoder, then forces a fixed advance width and centres the
/// glyph horizontally.
///
/// # Safety
/// Called by LVGL internally. `font` must point to a valid cloned
/// `lv_font_t` whose `user_data` stores the desired advance width.
unsafe extern "C" fn fixed_width_get_glyph_dsc(
    font: *const lv_font_t,
    dsc: *mut lv_font_glyph_dsc_t,
    letter: u32,
    letter_next: u32,
) -> bool {
    // SAFETY: font points to a valid lv_font_t inside a FixedWidthFont.
    // lv_font_get_glyph_dsc_fmt_txt is the standard LVGL glyph decoder.
    unsafe {
        let ret = lv_font_get_glyph_dsc_fmt_txt(font, dsc, letter, letter_next);
        if !ret {
            return false;
        }
        let adv = (*font).user_data as usize as u16;
        (*dsc).adv_w = adv;
        (*dsc).ofs_x = (adv as i16 - (*dsc).box_w as i16) / 2;
        true
    }
}

// SAFETY: All lv_font_montserrat_* are valid static fonts compiled into the
// binary (enabled via LV_FONT_MONTSERRAT_* in lv_conf.h). LTO removes unused.

/// LVGL built-in Montserrat 8 pt.
pub static MONTSERRAT_8: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_8));
/// LVGL built-in Montserrat 10 pt.
pub static MONTSERRAT_10: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_10));
/// LVGL built-in Montserrat 12 pt.
pub static MONTSERRAT_12: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_12));
/// LVGL built-in Montserrat 14 pt.
pub static MONTSERRAT_14: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_14));
/// LVGL built-in Montserrat 16 pt.
pub static MONTSERRAT_16: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_16));
/// LVGL built-in Montserrat 18 pt.
pub static MONTSERRAT_18: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_18));
/// LVGL built-in Montserrat 20 pt.
pub static MONTSERRAT_20: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_20));
/// LVGL built-in Montserrat 22 pt.
pub static MONTSERRAT_22: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_22));
/// LVGL built-in Montserrat 24 pt.
pub static MONTSERRAT_24: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_24));
/// LVGL built-in Montserrat 26 pt.
pub static MONTSERRAT_26: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_26));
/// LVGL built-in Montserrat 28 pt.
pub static MONTSERRAT_28: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_28));
/// LVGL built-in Montserrat 30 pt.
pub static MONTSERRAT_30: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_30));
/// LVGL built-in Montserrat 32 pt.
pub static MONTSERRAT_32: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_32));
/// LVGL built-in Montserrat 34 pt.
pub static MONTSERRAT_34: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_34));
/// LVGL built-in Montserrat 36 pt.
pub static MONTSERRAT_36: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_36));
/// LVGL built-in Montserrat 38 pt.
pub static MONTSERRAT_38: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_38));
/// LVGL built-in Montserrat 40 pt.
pub static MONTSERRAT_40: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_40));
/// LVGL built-in Montserrat 42 pt.
pub static MONTSERRAT_42: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_42));
/// LVGL built-in Montserrat 44 pt.
pub static MONTSERRAT_44: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_44));
/// LVGL built-in Montserrat 46 pt.
pub static MONTSERRAT_46: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_46));
/// LVGL built-in Montserrat 48 pt.
pub static MONTSERRAT_48: Font = Font(addr_of!(oxivgl_sys::lv_font_montserrat_48));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_as_ptr_nonnull() {
        assert!(!MONTSERRAT_12.as_ptr().is_null());
        assert!(!MONTSERRAT_14.as_ptr().is_null());
        assert!(!MONTSERRAT_20.as_ptr().is_null());
        assert!(!MONTSERRAT_48.as_ptr().is_null());
    }

    #[test]
    fn font_copy_clone() {
        let f = MONTSERRAT_12;
        let g = f;
        assert_eq!(f.as_ptr(), g.as_ptr());
        let h = f;
        assert_eq!(f.as_ptr(), h.as_ptr());
    }

    #[test]
    fn font_debug_fmt() {
        let s = format!("{:?}", MONTSERRAT_14);
        assert!(!s.is_empty());
    }

    #[test]
    fn font_from_extern() {
        // from_extern wraps the same address; round-trip should match.
        let ptr = MONTSERRAT_12.as_ptr() as *const ();
        let f = unsafe { Font::from_extern(ptr) };
        assert_eq!(f.as_ptr(), MONTSERRAT_12.as_ptr());
    }

    #[test]
    fn fixed_width_font_new_is_const() {
        static FW: FixedWidthFont = FixedWidthFont::new();
        // Just verify it compiles as a const static.
        let _ = &FW;
    }
}
