#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 9 — Line styles

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        lv_point_precise_t, palette_main, Line, Palette, Screen, Selector, Style, WidgetError,
    },
};

struct Style9 {
    _line: Line<'static>,
    _style: Box<Style>,
    _points: Box<[lv_point_precise_t; 3]>,
}

impl View for Style9 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style = Box::new(Style::new());
        style
            .line_color(palette_main(Palette::Grey))
            .line_width(6)
            .line_rounded(true);

        let points = Box::new([
            lv_point_precise_t { x: 10, y: 30 },
            lv_point_precise_t { x: 30, y: 50 },
            lv_point_precise_t { x: 100, y: 0 },
        ]);

        let line = Line::new(&screen)?;
        line.add_style(&style, Selector::DEFAULT);
        line.set_points(&*points);
        line.center();

        Ok(Self {
            _line: line,
            _style: style,
            _points: points,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style9);
