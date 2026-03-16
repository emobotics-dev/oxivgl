#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 1 — Start animation on an event

use oxivgl::{
    anim::{anim_path_ease_in, anim_path_overshoot, anim_set_x, Anim},
    view::View,
    enums::{EventCode, ObjState},
    event::Event,
    widgets::{Label, Screen, Switch, WidgetError},
};

struct Anim1 {
    label: Label<'static>,
    sw: Switch<'static>,
}

impl View for Anim1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label = Label::new(&screen)?;
        label.text("Hello animations!").pos(100, 10);

        let sw = Switch::new(&screen)?;
        sw.center();
        sw.add_state(ObjState::CHECKED);
        sw.bubble_events();

        Ok(Self { label, sw })
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
            a.start();
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Anim1);
