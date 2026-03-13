#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 12 — Local styles

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{palette_lighten, palette_main, Obj, Palette, Screen, Selector, Style, WidgetError},
};

struct Style12 {
    _obj: Obj<'static>,
    _style: Box<Style>,
}

impl View for Style12 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style = Box::new(Style::new());
        style
            .bg_color(palette_main(Palette::Green))
            .border_color(palette_lighten(Palette::Green, 3))
            .border_width(3);

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, Selector::DEFAULT);
        obj.style_bg_color(palette_main(Palette::Orange), Selector::DEFAULT);
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

oxivgl_examples_common::example_main!(Style12);
