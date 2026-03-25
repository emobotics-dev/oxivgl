#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Keyboard 1 — Keyboard with two textareas
//!
//! Two textareas at top (left and right) with placeholder text. A keyboard
//! at the bottom is linked to the first textarea by default. Clicking a
//! textarea switches the keyboard to it.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::View,
    widgets::{Align, Keyboard, Screen, Textarea, WidgetError},
};

struct WidgetKeyboard1 {
    _screen: Screen,
    ta1: Textarea<'static>,
    ta2: Textarea<'static>,
    kb: Keyboard<'static>,
}

impl View for WidgetKeyboard1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let kb = Keyboard::new(&screen)?;
        kb.align(Align::BottomMid, 0, 0);

        let ta1 = Textarea::new(&screen)?;
        ta1.size(140, 80);
        ta1.align(Align::TopLeft, 10, 10);
        ta1.set_placeholder_text("Hello");
        ta1.bubble_events();

        let ta2 = Textarea::new(&screen)?;
        ta2.size(140, 80);
        ta2.align(Align::TopRight, -10, 10);
        ta2.set_placeholder_text("Hello");
        ta2.bubble_events();

        kb.set_textarea(&ta1);

        Ok(Self { _screen: screen, ta1, ta2, kb })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.ta1, EventCode::FOCUSED) {
            self.kb.set_textarea(&self.ta1);
        } else if event.matches(&self.ta2, EventCode::FOCUSED) {
            self.kb.set_textarea(&self.ta2);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetKeyboard1);
