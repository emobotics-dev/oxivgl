#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 7 — Arc color and width

extern crate alloc;

use oxivgl::{
    view::View,
    widgets::{palette_main, Arc, Palette, Screen, Selector, Style, StyleBuilder, WidgetError},
};

struct Style7 {
    _arc: Arc<'static>,
    _style: Style,
}

impl View for Style7 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut builder = StyleBuilder::new();
        builder.arc_color(palette_main(Palette::Red)).arc_width(4);
        let style = builder.build();

        let arc = Arc::new(&screen)?;
        arc.add_style(&style, Selector::DEFAULT);
        arc.center();

        Ok(Self {
            _arc: arc,
            _style: style,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style7);
