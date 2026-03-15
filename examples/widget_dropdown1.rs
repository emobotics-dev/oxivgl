#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Dropdown 1 — Simple drop-down list
//!
//! A centered dropdown with ten fruit options.

use oxivgl::{
    view::View,
    widgets::{Align, Dropdown, Screen, WidgetError},
};

struct WidgetDropdown1 {
    _dd: Dropdown<'static>,
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

        Ok(Self { _dd: dd })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetDropdown1);
