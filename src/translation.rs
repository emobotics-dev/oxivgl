// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL translation/i18n support.
//!
//! Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.
//!
//! Register static translation packs, then switch languages at runtime.
//! Labels created with [`Label::set_translation_tag`] auto-update when
//! the language changes.
//!
//! # Example
//!
//! ```no_run
//! use oxivgl::translation::{self, StaticCStr as S};
//!
//! static LANGS: [S; 3] = [S::from_cstr(c"en"), S::from_cstr(c"de"), S::NULL];
//! static TAGS: [S; 2] = [S::from_cstr(c"hello"), S::NULL];
//! static TRANS: [S; 2] = [S::from_cstr(c"Hello"), S::from_cstr(c"Hallo")];
//!
//! translation::add_static(&LANGS, &TAGS, &TRANS);
//! translation::set_language(c"de");
//! ```

use core::ffi::{CStr, c_char};
use lvgl_rust_sys::*;

/// A `*const c_char` wrapper that is `Sync` + `Send`.
///
/// Use for `'static` translation arrays. The pointer must refer to a
/// compile-time string literal (`c"..."` or `&'static CStr`).
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct StaticCStr(pub *const c_char);

// SAFETY: the pointer targets compile-time string literals which are
// inherently immutable and 'static — safe to share across threads.
unsafe impl Sync for StaticCStr {}
unsafe impl Send for StaticCStr {}

impl StaticCStr {
    /// Create from a `&'static CStr` literal.
    pub const fn from_cstr(s: &'static CStr) -> Self {
        Self(s.as_ptr())
    }
    /// NULL sentinel for array termination.
    pub const NULL: Self = Self(core::ptr::null());
}

/// Register a translation pack from static NULL-terminated arrays.
///
/// - `languages`: e.g. `&[S::from_cstr(c"en"), S::from_cstr(c"de"), S::NULL]`
/// - `tags`: NULL-terminated tag names
/// - `translations`: flattened by language (tags × languages)
///
/// All arrays must be `'static`. LVGL stores the pointers directly.
pub fn add_static(
    languages: &'static [StaticCStr],
    tags: &'static [StaticCStr],
    translations: &'static [StaticCStr],
) {
    // SAFETY: StaticCStr is repr(transparent) over *const c_char.
    // All pointers are 'static (enforced by the type system + 'static borrow).
    // Arrays are NULL-terminated per API contract.
    unsafe {
        lv_translation_add_static(
            languages.as_ptr().cast(),
            tags.as_ptr().cast(),
            translations.as_ptr().cast(),
        );
    }
}

/// Set the active language. All widgets with translation tags will update.
///
/// `lang` must match one of the registered language strings (e.g. `c"en"`).
pub fn set_language(lang: &CStr) {
    // SAFETY: lang is a valid NUL-terminated CStr.
    unsafe { lv_translation_set_language(lang.as_ptr()) };
}

/// Get the currently active language, or `None` if none set.
pub fn get_language() -> Option<&'static CStr> {
    // SAFETY: lv_translation_get_language returns a static pointer or NULL.
    let ptr = unsafe { lv_translation_get_language() };
    if ptr.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(ptr) })
    }
}
