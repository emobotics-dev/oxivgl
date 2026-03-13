// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// LVGL switch (toggle) widget.
#[derive(Debug)]
pub struct Switch<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Switch<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Switch<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Switch<'p> {
    /// Create a new switch (toggle) widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        let handle = unsafe { lv_switch_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Switch {
                obj: Obj::from_raw(handle),
            })
        }
    }
}
