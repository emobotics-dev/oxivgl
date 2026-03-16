#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Button — Handle multiple events
//!
//! TODO: Hardware target (fire27) has no touch screen yet — button events
//! require an input device to trigger. The GUI is fully wired; only the
//! physical input is missing.

use oxivgl::{
    view::View,
    widgets::{Button, Event, EventCode, Label, Screen, WidgetError},
};

struct EventButton {
    btn: Button<'static>,
    _btn_label: Label<'static>,
    info_label: Label<'static>,
}

impl View for EventButton {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // TODO: No touch input on fire27 hardware — button events won't fire
        // until an input device is connected.
        #[cfg(target_arch = "xtensa")]
        oxivgl_examples_common::warn!(
            "event_button: no touch input — button events require input device"
        );

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

    fn on_event(&mut self, event: &Event) {
        if event.target_handle() != self.btn.handle() {
            return;
        }
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
