#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 9 — Line styles

use oxivgl::{
    view::View,
    widgets::{
        lv_point_precise_t, palette_main, Line, Palette, Screen, Selector, Style, StyleBuilder,
        WidgetError,
    },
};

static POINTS: [lv_point_precise_t; 3] = [
    lv_point_precise_t { x: 10.0, y: 30.0 },
    lv_point_precise_t { x: 30.0, y: 50.0 },
    lv_point_precise_t { x: 100.0, y: 0.0 },
];

struct Style9 {
    _line: Line<'static>,
    _style: Style,
}

impl View for Style9 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut builder = StyleBuilder::new();
        builder
            .line_color(palette_main(Palette::Grey))
            .line_width(6)
            .line_rounded(true);
        let style = builder.build();

        let line = Line::new(&screen)?;
        line.add_style(&style, Selector::DEFAULT);
        line.set_points(&POINTS);
        line.center();

        Ok(Self {
            _line: line,
            _style: style,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style9);
