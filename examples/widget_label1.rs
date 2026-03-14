#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 1 — Line wrap and scrolling
//!
//! Two labels: one with word-wrap (centered), one with circular scrolling.

use oxivgl::{
    view::View,
    widgets::{Align, Label, LabelLongMode, Screen, TextAlign, WidgetError},
};

struct WidgetLabel1 {
    _label1: Label<'static>,
    _label2: Label<'static>,
}

impl View for WidgetLabel1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label1 = Label::new(&screen)?;
        label1.set_long_mode(LabelLongMode::Wrap);
        label1.text_long(
            "Words of a label, align the lines to the center \
             and wrap long text automatically.",
        );
        label1.width(150);
        label1.text_align(TextAlign::Center);
        label1.align(Align::Center, 0, -40);

        let label2 = Label::new(&screen)?;
        label2.set_long_mode(LabelLongMode::ScrollCircular);
        label2.width(150);
        label2.text("It is a circularly scrolling text. ");
        label2.align(Align::Center, 0, 40);

        Ok(Self {
            _label1: label1,
            _label2: label2,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetLabel1);
