#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget List 1 — File/Connectivity/Exit sections with icon buttons
//!
//! A list with text section headers and icon+text buttons. Clicking a button
//! logs the button text (in the C original; here the View pattern catches
//! CLICKED events on the list via bubbling).

use oxivgl::{
    enums::EventCode,
    event::Event,
    symbols,
    view::View,
    widgets::{Child, Button, List, Screen, WidgetError},
};

struct WidgetList1 {
    list: List<'static>,
    _buttons: [Child<Button<'static>>; 11],
}

impl View for WidgetList1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let list = List::new(&screen)?;
        list.size(180, 220).center();

        // File section
        list.add_text("File");
        let b0 = list.add_button(Some(&symbols::FILE), "New");
        b0.bubble_events();
        let b1 = list.add_button(Some(&symbols::DIRECTORY), "Open");
        b1.bubble_events();
        let b2 = list.add_button(Some(&symbols::SAVE), "Save");
        b2.bubble_events();
        let b3 = list.add_button(Some(&symbols::CLOSE), "Delete");
        b3.bubble_events();
        let b4 = list.add_button(Some(&symbols::EDIT), "Edit");
        b4.bubble_events();

        // Connectivity section
        list.add_text("Connectivity");
        let b5 = list.add_button(Some(&symbols::BLUETOOTH), "Bluetooth");
        b5.bubble_events();
        let b6 = list.add_button(Some(&symbols::GPS), "Navigation");
        b6.bubble_events();
        let b7 = list.add_button(Some(&symbols::USB), "USB");
        b7.bubble_events();
        let b8 = list.add_button(Some(&symbols::BATTERY_FULL), "Battery");
        b8.bubble_events();

        // Exit section
        list.add_text("Exit");
        let b9 = list.add_button(Some(&symbols::OK), "Apply");
        b9.bubble_events();
        let b10 = list.add_button(Some(&symbols::CLOSE), "Close");
        b10.bubble_events();

        Ok(Self {
            list,
            _buttons: [b0, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10],
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == EventCode::CLICKED {
            if let Some(text) = self.list.get_button_text(&event.target()) {
                let _ = text; // C original logs; we just consume
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetList1);
