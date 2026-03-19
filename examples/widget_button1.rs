#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Button 1 — Click and toggle buttons
//!
//! Two buttons: a standard button with a click counter and a checkable toggle
//! button whose label reflects ON/OFF state.

use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    view::{register_event_on, View},
    widgets::{Align, Button, Label, Screen, WidgetError},
};

struct WidgetButton1 {
    btn1: Button<'static>,
    btn2: Button<'static>,
    label1: Label<'static>,
    label2: Label<'static>,
    cnt: u32,
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
            label1,
            label2,
            cnt: 0,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.btn1.handle());
        register_event_on(self, self.btn2.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.btn1, EventCode::CLICKED) {
            self.cnt += 1;
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Button: {}", self.cnt));
            self.label1.text(&buf);
        }
        if event.matches(&self.btn2, EventCode::VALUE_CHANGED) {
            if self.btn2.has_state(ObjState::CHECKED) {
                self.label2.text("ON");
            } else {
                self.label2.text("OFF");
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetButton1);
