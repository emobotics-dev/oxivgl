#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Button 1 — Click and toggle buttons
//!
//! Two buttons: a standard button that logs clicks and a checkable toggle
//! button that logs state changes.

use oxivgl::{
    view::View,
    widgets::{Align, Button, Event, EventCode, Label, ObjFlag, Screen, WidgetError},
};

struct WidgetButton1 {
    btn1: Button<'static>,
    btn2: Button<'static>,
    _label1: Label<'static>,
    _label2: Label<'static>,
}

impl View for WidgetButton1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let btn1 = Button::new(&screen)?;
        btn1.align(Align::Center, 0, -40);
        btn1.remove_flag(ObjFlag::PRESS_LOCK);
        btn1.bubble_events();

        let label1 = Label::new(&btn1)?;
        label1.text("Button").center();

        let btn2 = Button::new(&screen)?;
        btn2.align(Align::Center, 0, 40);
        btn2.add_flag(ObjFlag::CHECKABLE);
        btn2.bubble_events();

        let label2 = Label::new(&btn2)?;
        label2.text("Toggle").center();

        Ok(Self {
            btn1,
            btn2,
            _label1: label1,
            _label2: label2,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.btn1, EventCode::CLICKED) {
            // Button clicked
        }
        if event.matches(&self.btn2, EventCode::VALUE_CHANGED) {
            // Toggle changed
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetButton1);
