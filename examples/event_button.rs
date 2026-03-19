#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Button — Handle multiple events
//!
//! A button that displays which event was last triggered (PRESSED, CLICKED,
//! LONG_PRESSED, LONG_PRESSED_REPEAT) in an info label.

use oxivgl::{
    view::{register_event_on, View},
    enums::EventCode,
    event::Event,
    widgets::{Button, Label, Screen, WidgetError},
};

struct EventButton {
    btn: Button<'static>,
    _btn_label: Label<'static>,
    info_label: Label<'static>,
}

impl View for EventButton {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let btn = Button::new(&screen)?;
        btn.size(100, 50).center();
        btn.bubble_events();

        let btn_label = Label::new(&btn)?;
        btn_label.text("Click me!").center();

        let info_label = Label::new(&screen)?;
        info_label.text("The last button event:\nNone");

        Ok(Self {
            btn,
            _btn_label: btn_label,
            info_label,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.btn.handle());
    }

    fn on_event(&mut self, event: &Event) {
        let text = match event.code() {
            EventCode::PRESSED => "The last button event:\nLV_EVENT_PRESSED",
            EventCode::CLICKED => "The last button event:\nLV_EVENT_CLICKED",
            EventCode::LONG_PRESSED => "The last button event:\nLV_EVENT_LONG_PRESSED",
            EventCode::LONG_PRESSED_REPEAT => {
                "The last button event:\nLV_EVENT_LONG_PRESSED_REPEAT"
            }
            _ => return,
        };
        self.info_label.text(text);
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(EventButton);
