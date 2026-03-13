#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 2 — Button

use oxivgl::{
    view::View,
    widgets::{Button, Label, Screen, WidgetError},
};

struct GettingStarted2 {
    _btn: Button<'static>,
    _label: Label<'static>,
}

impl View for GettingStarted2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let btn = Button::new(&screen)?;
        btn.pos(10, 10).size(120, 50);

        let label = Label::new(&btn)?;
        label.text("Button\0")?.center();

        Ok(Self {
            _btn: btn,
            _label: label,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted2);
