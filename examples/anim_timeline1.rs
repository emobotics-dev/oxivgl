#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim Timeline 1 — Animation timeline

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    anim::{
        anim_path_ease_out, anim_path_linear, anim_path_overshoot, anim_set_height,
        anim_set_slider_value, anim_set_width, Anim, AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX,
    },
    view::View,
    widgets::{
        Align, Button, Event, EventCode, FlexAlign, FlexFlow, Label, Obj, ObjFlag, ObjState,
        Screen, ScrollbarMode, Slider, WidgetError,
    },
};

const OBJ_WIDTH: i32 = 90;
const OBJ_HEIGHT: i32 = 70;

struct AnimTimeline1 {
    timeline: Box<AnimTimeline>,
    btn_start: Button<'static>,
    btn_pause: Button<'static>,
    slider: Slider<'static>,
    _obj1: Obj<'static>,
    _obj2: Obj<'static>,
    _obj3: Obj<'static>,
    _label_start: Label<'static>,
    _label_pause: Label<'static>,
}

impl View for AnimTimeline1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut timeline = Box::new(AnimTimeline::new());

        screen.set_flex_flow(FlexFlow::Row);
        screen.set_flex_align(FlexAlign::SpaceAround, FlexAlign::Center, FlexAlign::Center);

        // Start button (checkable)
        let btn_start = Button::new(&screen)?;
        btn_start.add_flag(ObjFlag::IGNORE_LAYOUT);
        btn_start.add_flag(ObjFlag::CHECKABLE);
        btn_start.bubble_events();
        btn_start.align(Align::TopMid, -100, 20);
        let label_start = Label::new(&btn_start)?;
        label_start.text("Start").center();

        // Pause button
        let btn_pause = Button::new(&screen)?;
        btn_pause.add_flag(ObjFlag::IGNORE_LAYOUT);
        btn_pause.bubble_events();
        btn_pause.align(Align::TopMid, 100, 20);
        let label_pause = Label::new(&btn_pause)?;
        label_pause.text("Pause").center();

        // Progress slider
        let slider = Slider::new(&screen)?;
        slider.add_flag(ObjFlag::IGNORE_LAYOUT);
        slider.bubble_events();
        slider.align(Align::BottomMid, 0, -20);
        slider.set_range(0, ANIM_TIMELINE_PROGRESS_MAX as i32);

        // 3 objects
        let obj1 = Obj::new(&screen)?;
        obj1.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(ScrollbarMode::Off);

        let obj2 = Obj::new(&screen)?;
        obj2.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(ScrollbarMode::Off);

        let obj3 = Obj::new(&screen)?;
        obj3.size(OBJ_WIDTH, OBJ_HEIGHT)
            .set_scrollbar_mode(ScrollbarMode::Off);

        // Animations — slider progress
        let mut a_slider = Anim::new();
        a_slider
            .set_var(&slider)
            .set_values(0, ANIM_TIMELINE_PROGRESS_MAX as i32)
            .set_custom_exec_cb(Some(anim_set_slider_value))
            .set_path_cb(Some(anim_path_linear))
            .set_duration(700);

        // obj1 width + height
        let mut a1 = Anim::new();
        a1.set_var(&obj1)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(anim_set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a2 = Anim::new();
        a2.set_var(&obj1)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(anim_set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj2
        let mut a3 = Anim::new();
        a3.set_var(&obj2)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(anim_set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a4 = Anim::new();
        a4.set_var(&obj2)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(anim_set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj3
        let mut a5 = Anim::new();
        a5.set_var(&obj3)
            .set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(anim_set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a6 = Anim::new();
        a6.set_var(&obj3)
            .set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(anim_set_height))
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
            timeline,
            btn_start,
            btn_pause,
            slider,
            _obj1: obj1,
            _obj2: obj2,
            _obj3: obj3,
            _label_start: label_start,
            _label_pause: label_pause,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.btn_start, EventCode::VALUE_CHANGED) {
            let reverse = self.btn_start.has_state(ObjState::CHECKED);
            self.timeline.set_reverse(reverse);
            self.timeline.start();
        } else if event.matches(&self.btn_pause, EventCode::CLICKED) {
            self.timeline.pause();
        } else if event.matches(&self.slider, EventCode::VALUE_CHANGED) {
            let progress = self.slider.get_value();
            self.timeline.set_progress(progress as u16);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(AnimTimeline1);
