#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 1 — Start animation on an event

use oxivgl::{
    view::{Event, View},
    widgets::{
        anim_path_ease_in, anim_path_overshoot, anim_set_x, Anim, Label, Screen, Switch,
        WidgetError, LV_EVENT_VALUE_CHANGED, LV_OBJ_FLAG_EVENT_BUBBLE, LV_STATE_CHECKED,
    },
};

struct Anim1 {
    label: Label<'static>,
    sw: Switch<'static>,
}

impl View for Anim1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label = Label::new(&screen)?;
        label.text("Hello animations!\0")?.pos(100, 10);

        let sw = Switch::new(&screen)?;
        sw.center();
        sw.add_state(LV_STATE_CHECKED);
        sw.add_flag(LV_OBJ_FLAG_EVENT_BUBBLE);

        Ok(Self { label, sw })
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == LV_EVENT_VALUE_CHANGED && event.target_handle() == self.sw.handle() {
            let checked = self.sw.has_state(LV_STATE_CHECKED);

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
