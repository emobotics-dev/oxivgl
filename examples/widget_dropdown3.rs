#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Dropdown 3 — Menu-style dropdown
//!
//! Dropdown with fixed "Menu" button text and no selected-item highlight.

extern crate alloc;

use oxivgl::{
    view::View,
    widgets::{Align, Dropdown, Screen, WidgetError},
};

struct WidgetDropdown3 {
    _dd: Dropdown<'static>,
}

impl View for WidgetDropdown3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let dd = Dropdown::new(&screen)?;
        dd.set_options("New project\nNew file\nSave\nSave as ...\nOpen project\nRecent projects\nPreferences\nExit");
        dd.set_text(c"Menu");
        dd.set_selected_highlight(false);
        dd.align(Align::TopLeft, 10, 10);

        Ok(Self { _dd: dd })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetDropdown3);
