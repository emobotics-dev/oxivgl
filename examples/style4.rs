#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 4 — Outline

extern crate alloc;

use oxivgl::{
    style::{palette_lighten, palette_main, Palette, Selector, Style, StyleBuilder},
    view::View,
    widgets::{Obj, Screen, WidgetError},
};

struct Style4 {
    _obj: Obj<'static>,
    _style: Style,
}

impl View for Style4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut builder = StyleBuilder::new();
        builder
            .radius(5)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 1))
            .outline_width(2)
            .outline_color(palette_main(Palette::Blue))
            .outline_pad(8);
        let style = builder.build();

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, Selector::DEFAULT);
        obj.center();

        Ok(Self {
            _obj: obj,
            _style: style,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style4);
