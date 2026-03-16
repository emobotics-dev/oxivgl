#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 3 — Border

extern crate alloc;

use oxivgl::{
    style::{
        palette_lighten, palette_main, BorderSide, Palette, Selector, Style, StyleBuilder,
    },
    view::View,
    widgets::{Obj, Screen, WidgetError},
};

struct Style3 {
    _obj: Obj<'static>,
    _style: Style,
}

impl View for Style3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut builder = StyleBuilder::new();
        builder
            .radius(10)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 1))
            .border_color(palette_main(Palette::Blue))
            .border_width(5)
            .border_opa(127)
            .border_side(BorderSide::BOTTOM | BorderSide::RIGHT);
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

oxivgl_examples_common::example_main!(Style3);
