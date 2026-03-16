#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 5 — Shadow

extern crate alloc;

use oxivgl::{
    view::View,
    widgets::{
        palette_lighten, palette_main, Obj, Palette, Screen, Selector, Style, StyleBuilder,
        WidgetError,
    },
};

struct Style5 {
    _obj: Obj<'static>,
    _style: Style,
}

impl View for Style5 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut builder = StyleBuilder::new();
        builder
            .radius(5)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 1))
            .shadow_width(55)
            .shadow_color(palette_main(Palette::Blue));
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

oxivgl_examples_common::example_main!(Style5);
