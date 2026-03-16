#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 2 — Text shadow
//!
//! Fake shadow effect: identical text rendered twice, offset by 2 pixels,
//! with reduced opacity for the shadow layer.

use oxivgl::{
    style::{color_black, Selector, Style, StyleBuilder},
    view::View,
    widgets::{Align, Label, Screen, WidgetError},
};

struct WidgetLabel2 {
    _shadow_label: Label<'static>,
    _main_label: Label<'static>,
    _style_shadow: Style,
}

impl View for WidgetLabel2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style_shadow = StyleBuilder::new();
        style_shadow.text_opa(76); // ~LV_OPA_30
        style_shadow.text_color(color_black());
        let style_shadow = style_shadow.build();

        let shadow_label = Label::new(&screen)?;
        shadow_label.add_style(&style_shadow, Selector::DEFAULT);

        let main_label = Label::new(&screen)?;
        main_label.text_long(
            "A simple method to create\nshadows on a text.\n\
             It even works with\n\nnewlines     and spaces.",
        );

        // Copy shadow text from main label (use same string)
        shadow_label.text_long(
            "A simple method to create\nshadows on a text.\n\
             It even works with\n\nnewlines     and spaces.",
        );

        main_label.align(Align::Center, 0, 0);
        shadow_label.align_to(&main_label, Align::TopLeft, 2, 2);

        Ok(Self {
            _shadow_label: shadow_label,
            _main_label: main_label,
            _style_shadow: style_shadow,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetLabel2);
