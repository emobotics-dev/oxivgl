// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// LVGL slider widget (native range 0–100 by default).
///
/// Use [`on_event`](Obj::on_event) with `lv_event_code_t_LV_EVENT_VALUE_CHANGED`
/// to react to slider movement.
#[derive(Debug)]
pub struct Slider<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Slider<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Slider<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Slider<'p> {
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via LvglDriver.
        let handle = unsafe { lv_slider_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Slider {
                obj: Obj::from_raw(handle),
            })
        }
    }

    /// Returns the current slider value (native LVGL integer range).
    pub fn get_value(&self) -> i32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_get_value(self.obj.handle()) }
    }

    /// Sets the slider range (min and max values).
    pub fn set_range(&self, min: i32, max: i32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        unsafe { lv_slider_set_range(self.obj.handle(), min, max) };
        self
    }

    /// Sets the slider value (native LVGL integer range).
    pub fn set_value(&self, val: i32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_set_value(self.obj.handle(), val, false) };
        self
    }
}
