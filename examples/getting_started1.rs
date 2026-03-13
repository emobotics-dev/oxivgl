#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 1 — Hello World

use oxivgl::{
    view::View,
    widgets::{Align, Label, Screen, WidgetError},
};

struct GettingStarted1 {
    _label: Label<'static>,
}

impl View for GettingStarted1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.bg_color(0x003a57).bg_opa(255);
        screen.text_color(0xffffff);

        let label = Label::new(&screen)?;
        label.text("Hello world\0")?.align(Align::Center, 0, 0);

        Ok(Self { _label: label })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted1);
