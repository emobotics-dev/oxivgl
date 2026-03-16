// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    WidgetError,
};

/// LVGL slider widget (native range 0–100 by default).
///
/// Use [`on_event`](Obj::on_event) with `EventCode::VALUE_CHANGED`
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

/// Slider operating mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum SliderMode {
    /// Normal slider (single value).
    Normal = lv_slider_mode_t_LV_SLIDER_MODE_NORMAL,
    /// Symmetrical from center.
    Symmetrical = lv_slider_mode_t_LV_SLIDER_MODE_SYMMETRICAL,
    /// Range slider (two handles, start + end).
    Range = lv_slider_mode_t_LV_SLIDER_MODE_RANGE,
}

impl<'p> Slider<'p> {
    /// Create a new slider widget.
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

    /// Set slider mode (normal, symmetrical, or range).
    pub fn set_mode(&self, mode: SliderMode) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_set_mode(self.obj.handle(), mode as lv_slider_mode_t) };
        self
    }

    /// Set the start value (left handle in range mode).
    pub fn set_start_value(&self, val: i32) -> &Self {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_set_start_value(self.obj.handle(), val, false) };
        self
    }

    /// Get the left/start value (range mode).
    pub fn get_left_value(&self) -> i32 {
        assert_ne!(
            self.obj.handle(),
            null_mut(),
            "Slider handle cannot be null"
        );
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_slider_get_left_value(self.obj.handle()) }
    }
}
