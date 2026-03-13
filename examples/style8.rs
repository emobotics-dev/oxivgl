#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 8 — Text styles

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        palette_lighten, palette_main, Label, Palette, Screen, Selector, Style, TextDecor,
        WidgetError,
    },
};

struct Style8 {
    _label: Label<'static>,
    _style: Box<Style>,
}

impl View for Style8 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style = Box::new(Style::new());
        style
            .radius(5)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 2))
            .border_width(2)
            .border_color(palette_main(Palette::Blue))
            .pad_all(10)
            .text_color(palette_main(Palette::Blue))
            .text_letter_space(5)
            .text_line_space(20)
            .text_decor(TextDecor::UNDERLINE);

        let label = Label::new(&screen)?;
        label.add_style(&style, Selector::DEFAULT);
        label.text("Text of\na label");
        label.center();

        Ok(Self {
            _label: label,
            _style: style,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style8);
