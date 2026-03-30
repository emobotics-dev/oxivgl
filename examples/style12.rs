#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 12 — Local styles

extern crate alloc;

use oxivgl::{
    style::{palette_lighten, palette_main, Palette, Selector, Style, StyleBuilder},
    view::View,
    widgets::{Obj, WidgetError},
};

#[derive(Default)]
struct Style12 {
    _obj: Option<Obj<'static>>,
    _style: Option<Style>,
}

impl View for Style12 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        let mut builder = StyleBuilder::new();
        builder
            .bg_color(palette_main(Palette::Green))
            .border_color(palette_lighten(Palette::Green, 3))
            .border_width(3);
        let style = builder.build();

        let obj = Obj::new(container)?;
        obj.add_style(&style, Selector::DEFAULT);
        obj.style_bg_color(palette_main(Palette::Orange), Selector::DEFAULT);
        obj.center();

                self._obj = Some(obj);
        self._style = Some(style);
        Ok(())
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style12::default());
