#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 1 — Start animation on an event

use core::ffi::c_void;
use lvgl_rust_sys::*;
use oxivgl::{
    view::View,
    widgets::{
        anim_path_ease_in, anim_path_overshoot, Label, Screen, Switch, WidgetError,
        LV_EVENT_VALUE_CHANGED, LV_STATE_CHECKED,
    },
};

struct Anim1 {
    _label: Label<'static>,
    _sw: Switch<'static>,
}

unsafe extern "C" fn anim_x_cb(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_x(var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn sw_event_cb(e: *mut lv_event_t) {
    unsafe {
        let sw = lv_event_get_target_obj(e);
        let label = lv_event_get_user_data(e) as *mut lv_obj_t;
        let checked = lv_obj_has_state(sw, LV_STATE_CHECKED);

        let mut a = core::mem::zeroed::<lv_anim_t>();
        lv_anim_init(&mut a);
        lv_anim_set_var(&mut a, label as *mut c_void);
        lv_anim_set_duration(&mut a, 500);
        lv_anim_set_exec_cb(&mut a, Some(anim_x_cb));

        if checked {
            lv_anim_set_values(&mut a, lv_obj_get_x(label), 100);
            lv_anim_set_path_cb(&mut a, Some(anim_path_overshoot));
        } else {
            lv_anim_set_values(&mut a, lv_obj_get_x(label), -lv_obj_get_width(label));
            lv_anim_set_path_cb(&mut a, Some(anim_path_ease_in));
        }
        lv_anim_start(&a);
    }
}

impl View for Anim1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label = Label::new(&screen)?;
        label.text("Hello animations!\0")?.pos(100, 10);

        let sw = Switch::new(&screen)?;
        sw.center();
        sw.add_state(LV_STATE_CHECKED);
        sw.on_event(
            sw_event_cb,
            LV_EVENT_VALUE_CHANGED,
            label.handle() as *mut c_void,
        );

        Ok(Self {
            _label: label,
            _sw: sw,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Anim1);
