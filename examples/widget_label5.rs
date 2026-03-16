#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 5 — Circular scroll
//!
//! Label with scroll-circular long mode — text scrolls in a continuous loop.

use oxivgl::{
    view::View,
    widgets::{Align, Label, LabelLongMode, Screen, WidgetError},
};

struct WidgetLabel5 {
    _label: Label<'static>,
}

impl View for WidgetLabel5 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label = Label::new(&screen)?;
        label.set_long_mode(LabelLongMode::ScrollCircular);
        label.width(150);
        label.text("It is a circularly scrolling text. ");
        label.align(Align::Center, 0, 0);

        Ok(Self { _label: label })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetLabel5);
