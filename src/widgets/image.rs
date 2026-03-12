// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL image widget. Wraps [`Obj`](super::obj::Obj) and `Deref`s to it for
/// style methods.
#[derive(Debug)]
pub struct Image<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Image<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Image<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Image<'p> {
    /// Create an image widget as a child of `parent`. Returns
    /// [`WidgetError::LvglNullPointer`] on OOM.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_image_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Image { obj: Obj::from_raw(handle) }) }
    }
}
