#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Line 1 — Simple line with styled points
//!
//! A line drawn through 5 points with rounded ends and blue color.

use oxivgl::{
    style::{palette_main, Palette, Selector, Style, StyleBuilder},
    view::View,
    widgets::{lv_point_precise_t, Line, Screen, WidgetError},
};

static LINE_POINTS: [lv_point_precise_t; 5] = [
    lv_point_precise_t { x: 5.0, y: 5.0 },
    lv_point_precise_t {
        x: 70.0,
        y: 70.0,
    },
    lv_point_precise_t {
        x: 120.0,
        y: 10.0,
    },
    lv_point_precise_t {
        x: 180.0,
        y: 60.0,
    },
    lv_point_precise_t {
        x: 240.0,
        y: 10.0,
    },
];

struct WidgetLine1 {
    _line: Line<'static>,
    _style: Style,
}

impl View for WidgetLine1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut sb = StyleBuilder::new();
        sb.line_width(8)
            .line_color(palette_main(Palette::Blue))
            .line_rounded(true);
        let style = sb.build();

        let line = Line::new(&screen)?;
        line.set_points(&LINE_POINTS);
        line.add_style(&style, Selector::DEFAULT);
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

oxivgl_examples_common::example_main!(WidgetLine1);
