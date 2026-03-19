#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Textarea 2 — Password and text fields with keyboard
//!
//! A password textarea (left) and a plain text textarea (right) with an
//! on-screen keyboard at the bottom. Clicking either textarea switches
//! the keyboard focus.

use oxivgl::{
    enums::EventCode,
    event::Event,
    style::lv_pct,
    view::{register_event_on, View},
    widgets::{Align, Keyboard, Label, Screen, Textarea, WidgetError},
};

struct WidgetTextarea2 {
    pwd_ta: Textarea<'static>,
    text_ta: Textarea<'static>,
    kb: Keyboard<'static>,
    _pwd_label: Label<'static>,
    _text_label: Label<'static>,
}

impl View for WidgetTextarea2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Password textarea
        let pwd_ta = Textarea::new(&screen)?;
        pwd_ta.set_text("");
        pwd_ta.set_password_mode(true);
        pwd_ta.set_one_line(true);
        pwd_ta.width(lv_pct(40));
        pwd_ta.pos(5, 20);
        pwd_ta.bubble_events();

        let pwd_label = Label::new(&screen)?;
        pwd_label.text("Password:");
        pwd_label.align_to(&pwd_ta, Align::OutTopLeft, 0, 0);

        // Plain text textarea
        let text_ta = Textarea::new(&screen)?;
        text_ta.set_one_line(true);
        text_ta.set_password_mode(false);
        text_ta.width(lv_pct(40));
        text_ta.align(Align::TopRight, -5, 20);
        text_ta.bubble_events();

        let text_label = Label::new(&screen)?;
        text_label.text("Text:");
        text_label.align_to(&text_ta, Align::OutTopLeft, 0, 0);

        // Keyboard
        let kb = Keyboard::new(&screen)?;
        kb.size(320, 120);
        kb.set_textarea(&pwd_ta);

        Ok(Self {
            pwd_ta,
            text_ta,
            kb,
            _pwd_label: pwd_label,
            _text_label: text_label,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.pwd_ta.handle());
        register_event_on(self, self.text_ta.handle());
    }

    fn on_event(&mut self, event: &Event) {
        let code = event.code();
        if code == EventCode::CLICKED || code == EventCode::FOCUSED {
            if event.target_handle() == self.pwd_ta.handle() {
                self.kb.set_textarea(&self.pwd_ta);
            } else if event.target_handle() == self.text_ta.handle() {
                self.kb.set_textarea(&self.text_ta);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetTextarea2);
