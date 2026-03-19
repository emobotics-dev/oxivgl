#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Textarea 3 — Clock format auto-insert
//!
//! A textarea restricted to digits and ':', max 5 characters.
//! After typing two digits, ':' is auto-inserted. A numeric keyboard
//! is shown below.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{register_event_on, View},
    widgets::{Keyboard, KeyboardMode, Screen, Textarea, WidgetError},
};

struct WidgetTextarea3 {
    ta: Textarea<'static>,
    _kb: Keyboard<'static>,
}

impl View for WidgetTextarea3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let ta = Textarea::new(&screen)?;
        ta.set_accepted_chars(c"0123456789:");
        ta.set_max_length(5);
        ta.set_one_line(true);
        ta.set_text("");
        ta.bubble_events();

        let kb = Keyboard::new(&screen)?;
        kb.size(320, 120);
        kb.set_mode(KeyboardMode::Number);
        kb.set_textarea(&ta);

        Ok(Self { ta, _kb: kb })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.ta.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.ta, EventCode::VALUE_CHANGED) {
            if let Some(txt) = self.ta.get_text() {
                let bytes = txt.as_bytes();
                if bytes.len() >= 2
                    && bytes[0].is_ascii_digit()
                    && bytes[1].is_ascii_digit()
                    && bytes.get(2).copied() != Some(b':')
                {
                    self.ta.set_cursor_pos(2);
                    self.ta.add_char(':');
                }
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetTextarea3);
