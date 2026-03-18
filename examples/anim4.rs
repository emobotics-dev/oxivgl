#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 4 — Animation with timed pause
//!
//! Switch toggles label X animation (overshoot / ease-in).
//! A one-shot 200 ms timer pauses the running animation for 1 s.

use oxivgl::{
    anim::{anim_path_ease_in, anim_path_overshoot, anim_set_x, Anim, AnimHandle},
    enums::{EventCode, ObjState},
    event::Event,
    timer::Timer,
    view::View,
    widgets::{Label, Screen, Switch, WidgetError},
};

struct Anim4 {
    label: Label<'static>,
    sw: Switch<'static>,
    pause_timer: Timer,
    anim_handle: Option<AnimHandle>,
}

impl View for Anim4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label = Label::new(&screen)?;
        label.text("Hello animations!").pos(0, 10);

        let sw = Switch::new(&screen)?;
        sw.center();
        sw.add_state(ObjState::CHECKED);
        sw.bubble_events();

        // One-shot timer, starts paused — armed in on_event.
        let pause_timer = Timer::new(200)?;
        pause_timer.set_repeat_count(1).pause();

        // Intro: slide label from x=0 to x=100 with overshoot.
        let mut a = Anim::new();
        a.set_var(&label)
            .set_values(0, 100)
            .set_duration(500)
            .set_exec_cb(Some(anim_set_x))
            .set_path_cb(Some(anim_path_overshoot));
        let anim_handle = Some(a.start());

        Ok(Self {
            label,
            sw,
            pause_timer,
            anim_handle,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.sw, EventCode::VALUE_CHANGED) {
            let checked = self.sw.has_state(ObjState::CHECKED);

            let mut a = Anim::new();
            a.set_var(&self.label)
                .set_duration(500)
                .set_exec_cb(Some(anim_set_x));

            if checked {
                a.set_values(self.label.get_x(), 100)
                    .set_path_cb(Some(anim_path_overshoot));
            } else {
                a.set_values(self.label.get_x(), -self.label.get_width())
                    .set_path_cb(Some(anim_path_ease_in));
            }

            self.anim_handle = Some(a.start());

            // Arm the one-shot 200 ms timer.
            self.pause_timer.resume().ready();
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        if self.pause_timer.triggered() {
            if let Some(ref handle) = self.anim_handle {
                // SAFETY: timer fires at 200ms, animation duration is 500ms,
                // so the animation is guaranteed to still be running.
                unsafe { handle.pause_for(1000) };
            }
        }
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Anim4);
