#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim Timeline 1 — Animation timeline

extern crate alloc;

use alloc::boxed::Box;
use core::ffi::c_void;
use lvgl_rust_sys::*;
use oxivgl::{
    view::View,
    widgets::{
        anim_path_ease_out, anim_path_linear, anim_path_overshoot, Align, Anim, AnimTimeline,
        Button, FlexAlign, FlexFlow, Label, Obj, Screen, Slider, WidgetError,
        ANIM_TIMELINE_PROGRESS_MAX, LV_EVENT_CLICKED, LV_EVENT_VALUE_CHANGED,
        LV_OBJ_FLAG_CHECKABLE, LV_OBJ_FLAG_IGNORE_LAYOUT, LV_SCROLLBAR_MODE_OFF, LV_STATE_CHECKED,
    },
};

const OBJ_WIDTH: i32 = 90;
const OBJ_HEIGHT: i32 = 70;

struct AnimTimeline1 {
    _timeline: Box<AnimTimeline>,
    _btn_start: Button<'static>,
    _btn_pause: Button<'static>,
    _slider: Slider<'static>,
    _obj1: Obj<'static>,
    _obj2: Obj<'static>,
    _obj3: Obj<'static>,
    _label_start: Label<'static>,
    _label_pause: Label<'static>,
}

unsafe extern "C" fn set_width(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_obj_set_width((*a).var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn set_height(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_obj_set_height((*a).var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn set_slider_value(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_slider_set_value((*a).var as *mut lv_obj_t, v, false) };
}

unsafe extern "C" fn btn_start_event_handler(e: *mut lv_event_t) {
    unsafe {
        let btn = lv_event_get_current_target_obj(e);
        let timeline = lv_event_get_user_data(e) as *mut lv_anim_timeline_t;
        let reverse = lv_obj_has_state(btn, LV_STATE_CHECKED);
        lv_anim_timeline_set_reverse(timeline, reverse);
        lv_anim_timeline_start(timeline);
    }
}

unsafe extern "C" fn btn_pause_event_handler(e: *mut lv_event_t) {
    unsafe {
        let timeline = lv_event_get_user_data(e) as *mut lv_anim_timeline_t;
        lv_anim_timeline_pause(timeline);
    }
}

unsafe extern "C" fn slider_prg_event_handler(e: *mut lv_event_t) {
    unsafe {
        let slider = lv_event_get_current_target_obj(e);
        let timeline = lv_event_get_user_data(e) as *mut lv_anim_timeline_t;
        let progress = lv_slider_get_value(slider);
        lv_anim_timeline_set_progress(timeline, progress as u16);
    }
}

impl View for AnimTimeline1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut timeline = Box::new(AnimTimeline::new());
        let tl_ptr = timeline.handle() as *mut c_void;

        // Setup flex on screen — Screen doesn't have Obj methods, use raw
        unsafe {
            lv_obj_set_flex_flow(screen.handle(), FlexFlow::Row as lv_flex_flow_t);
            lv_obj_set_flex_align(
                screen.handle(),
                FlexAlign::SpaceAround as lv_flex_align_t,
                FlexAlign::Center as lv_flex_align_t,
                FlexAlign::Center as lv_flex_align_t,
            );
        }

        // Start button (checkable)
        let btn_start = Button::new(&screen)?;
        btn_start.add_flag(LV_OBJ_FLAG_IGNORE_LAYOUT);
        btn_start.add_flag(LV_OBJ_FLAG_CHECKABLE);
        btn_start.align(Align::TopMid, -100, 20);
        btn_start.on_event(btn_start_event_handler, LV_EVENT_VALUE_CHANGED, tl_ptr);
        let label_start = Label::new(&btn_start)?;
        label_start.text("Start\0")?.center();

        // Pause button
        let btn_pause = Button::new(&screen)?;
        btn_pause.add_flag(LV_OBJ_FLAG_IGNORE_LAYOUT);
        btn_pause.align(Align::TopMid, 100, 20);
        btn_pause.on_event(btn_pause_event_handler, LV_EVENT_CLICKED, tl_ptr);
        let label_pause = Label::new(&btn_pause)?;
        label_pause.text("Pause\0")?.center();

        // Progress slider
        let slider = Slider::new(&screen)?;
        slider.add_flag(LV_OBJ_FLAG_IGNORE_LAYOUT);
        slider.align(Align::BottomMid, 0, -20);
        slider.set_range(0, ANIM_TIMELINE_PROGRESS_MAX as i32);
        slider.on_event(slider_prg_event_handler, LV_EVENT_VALUE_CHANGED, tl_ptr);

        // 3 objects
        let obj1 = Obj::new(&screen)?;
        obj1.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(LV_SCROLLBAR_MODE_OFF);

        let obj2 = Obj::new(&screen)?;
        obj2.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(LV_SCROLLBAR_MODE_OFF);

        let obj3 = Obj::new(&screen)?;
        obj3.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(LV_SCROLLBAR_MODE_OFF);

        // Animations — slider progress
        let mut a_slider = Anim::new();
        a_slider
            .set_var(&slider)
            .set_values(0, ANIM_TIMELINE_PROGRESS_MAX as i32)
            .set_custom_exec_cb(Some(set_slider_value))
            .set_path_cb(Some(anim_path_linear))
            .set_duration(700);

        // obj1 width + height
        let mut a1 = Anim::new();
        a1.set_var(&obj1)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a2 = Anim::new();
        a2.set_var(&obj1)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj2
        let mut a3 = Anim::new();
        a3.set_var(&obj2)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a4 = Anim::new();
        a4.set_var(&obj2)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj3
        let mut a5 = Anim::new();
        a5.set_var(&obj3)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a6 = Anim::new();
        a6.set_var(&obj3)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // Add to timeline
        timeline.add(0, &a_slider);
        timeline.add(0, &a1);
        timeline.add(0, &a2);
        timeline.add(200, &a3);
        timeline.add(200, &a4);
        timeline.add(400, &a5);
        timeline.add(400, &a6);

        timeline.set_progress(ANIM_TIMELINE_PROGRESS_MAX);

        Ok(Self {
            _timeline: timeline,
            _btn_start: btn_start,
            _btn_pause: btn_pause,
            _slider: slider,
            _obj1: obj1,
            _obj2: obj2,
            _obj3: obj3,
            _label_start: label_start,
            _label_pause: label_pause,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(AnimTimeline1);
