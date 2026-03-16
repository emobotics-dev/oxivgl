#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Obj 1 — Base objects with custom styles
//!
//! Two base objects: a plain one and one with a blue shadow style.

use oxivgl::{
    view::View,
    widgets::{
        palette_main, Align, Obj, Palette, Screen, Selector, Style, StyleBuilder, WidgetError,
    },
};

struct WidgetObj1 {
    _obj1: Obj<'static>,
    _obj2: Obj<'static>,
    _style_shadow: Style,
}

impl View for WidgetObj1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let obj1 = Obj::new(&screen)?;
        obj1.size(100, 50);
        obj1.align(Align::Center, -60, -30);

        let mut style_shadow = StyleBuilder::new();
        style_shadow
            .shadow_width(10)
            .shadow_spread(2)
            .shadow_color(palette_main(Palette::Blue));
        let style_shadow = style_shadow.build();

        let obj2 = Obj::new(&screen)?;
        obj2.add_style(&style_shadow, Selector::DEFAULT);
        obj2.align(Align::Center, 60, 30);

        Ok(Self {
            _obj1: obj1,
            _obj2: obj2,
            _style_shadow: style_shadow,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetObj1);
