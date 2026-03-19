// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ffi::c_char, ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// Static button matrix map.
///
/// LVGL expects a null-terminated array of C string pointers. Use the
/// [`btnmatrix_map!`](crate::btnmatrix_map) macro to create safely.
///
/// LVGL stores the raw pointer (`lv_buttonmatrix_set_map`); the map MUST
/// be `'static` per spec-memory-lifetime §1/§3.
#[repr(transparent)]
pub struct ButtonmatrixMap(pub [*const c_char]);

impl core::fmt::Debug for ButtonmatrixMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ButtonmatrixMap").finish_non_exhaustive()
    }
}

// SAFETY: The contained pointers reference 'static C string literals
// (enforced by the `btnmatrix_map!` macro).
unsafe impl Sync for ButtonmatrixMap {}

/// Create a `&'static` [`ButtonmatrixMap`] from C string literals.
///
/// Use `"\n"` entries to create row breaks. The macro appends a null
/// terminator automatically.
///
/// ```no_run
/// use oxivgl::btnmatrix_map;
/// use oxivgl::widgets::ButtonmatrixMap;
///
/// static MAP: &ButtonmatrixMap = btnmatrix_map!(
///     c"1", c"2", c"3", c"\n",
///     c"4", c"5", c"6"
/// );
/// ```
#[macro_export]
macro_rules! btnmatrix_map {
    ($($label:expr),+ $(,)?) => {
        // SAFETY: ButtonmatrixMap is repr(transparent) over [*const c_char].
        // All pointers come from c"…" literals which are 'static.
        // The array is a const-promoted 'static temporary.
        unsafe {
            &*(&[$($label.as_ptr()),+, ::core::ptr::null()]
                as *const [*const ::core::ffi::c_char]
                as *const $crate::widgets::ButtonmatrixMap)
        }
    };
}

/// LVGL button matrix widget.
///
/// Requires `LV_USE_BUTTONMATRIX = 1` in `lv_conf.h`.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::btnmatrix_map;
/// use oxivgl::widgets::{Align, Buttonmatrix, ButtonmatrixMap, Screen};
///
/// static MAP: &ButtonmatrixMap = btnmatrix_map!(c"A", c"B", c"C");
///
/// let screen = Screen::active().unwrap();
/// let btnm = Buttonmatrix::new(&screen).unwrap();
/// btnm.set_map(MAP);
/// btnm.align(Align::Center, 0, 0);
/// ```
#[derive(Debug)]
pub struct Buttonmatrix<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Buttonmatrix<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Buttonmatrix<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Buttonmatrix<'p> {
    /// Create a new button matrix widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via LvglDriver.
        let handle = unsafe { lv_buttonmatrix_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Buttonmatrix {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Set the button map.
    ///
    /// LVGL stores the raw pointer; the map MUST be `'static`
    /// (spec-memory-lifetime §1/§3). Use [`btnmatrix_map!`](crate::btnmatrix_map).
    pub fn set_map(&self, map: &'static ButtonmatrixMap) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Buttonmatrix handle cannot be null");
        // SAFETY: handle non-null; map is 'static and null-terminated.
        // LVGL stores the pointer; 'static satisfies lifetime (spec §1/§3).
        unsafe {
            lv_buttonmatrix_set_map(self.obj.handle(), map.0.as_ptr() as *const *const c_char)
        };
        self
    }

    /// Get the index of the currently selected (last pressed) button.
    pub fn get_selected_button(&self) -> u32 {
        assert_ne!(self.obj.handle(), null_mut(), "Buttonmatrix handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_buttonmatrix_get_selected_button(self.obj.handle()) }
    }

    /// Get the text of a button by index. Returns `None` if the index is
    /// invalid or the text is not valid UTF-8.
    pub fn get_button_text(&self, btn_id: u32) -> Option<&str> {
        assert_ne!(self.obj.handle(), null_mut(), "Buttonmatrix handle cannot be null");
        // SAFETY: handle non-null; LVGL returns NULL for invalid btn_id.
        let ptr =
            unsafe { lv_buttonmatrix_get_button_text(self.obj.handle(), btn_id) };
        if ptr.is_null() {
            return None;
        }
        // SAFETY: ptr is a valid C string from the button map.
        let cstr = unsafe { core::ffi::CStr::from_ptr(ptr) };
        cstr.to_str().ok()
    }
}
