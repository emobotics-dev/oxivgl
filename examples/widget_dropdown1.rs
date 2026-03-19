#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Dropdown 1 — Simple drop-down list
//!
//! A centered dropdown with fruit options and a label showing the selected
//! item, updated via VALUE_CHANGED event.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{register_event_on, View},
    widgets::{Align, Dropdown, Label, Screen, WidgetError},
};

struct WidgetDropdown1 {
    dd: Dropdown<'static>,
    label: Label<'static>,
}

impl View for WidgetDropdown1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let dd = Dropdown::new(&screen)?;
        dd.set_options(
            "Apple\n\
             Banana\n\
             Orange\n\
             Cherry\n\
             Grape\n\
             Raspberry\n\
             Melon\n\
             Lemon\n\
             Nuts",
        );
        dd.align(Align::TopMid, 0, 20);
        dd.bubble_events();

        let label = Label::new(&screen)?;
        label.text("Apple");
        label.align(Align::BottomMid, 0, -20);

        Ok(Self { dd, label })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.dd.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == EventCode::VALUE_CHANGED {
            let mut buf = [0u8; 32];
            if let Some(text) = self.dd.get_selected_str(&mut buf) {
                self.label.text(text);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetDropdown1);
