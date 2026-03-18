#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Streak — Short-click streak counting
//!
//! Button reports short-clicked (with streak count), single-clicked,
//! double-clicked, and triple-clicked events to labels.

use oxivgl::{
    enums::EventCode,
    event::Event,
    indev::Indev,
    view::View,
    widgets::{Button, Label, Screen, WidgetError},
};

struct EventStreak {
    btn: Button<'static>,
    btn_label: Label<'static>,
    info_label: Label<'static>,
}

impl View for EventStreak {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let info_label = Label::new(&screen)?;
        info_label.text("No events yet");

        let btn = Button::new(&screen)?;
        btn.size(100, 50).center();

        let btn_label = Label::new(&btn)?;
        btn_label.text("Click me!").center();

        Ok(Self { btn, btn_label, info_label })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.btn, EventCode::SHORT_CLICKED) {
            if let Some(indev) = Indev::active() {
                let cnt = indev.short_click_streak();
                let mut buf = heapless::String::<32>::new();
                let _ = core::fmt::Write::write_fmt(
                    &mut buf,
                    format_args!("Short click streak: {}", cnt),
                );
                self.info_label.text(&buf);
            }
        } else if event.matches(&self.btn, EventCode::SINGLE_CLICKED) {
            self.btn_label.text("Single clicked");
        } else if event.matches(&self.btn, EventCode::DOUBLE_CLICKED) {
            self.btn_label.text("Double clicked");
        } else if event.matches(&self.btn, EventCode::TRIPLE_CLICKED) {
            self.btn_label.text("Triple clicked");
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(EventStreak);
