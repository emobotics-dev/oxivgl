#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 2 — Styled progress bar
//!
//! Custom blue-themed bar with rounded corners, padding, and animated fill.

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        palette_main, Bar, Palette, Part, Screen, Selector, Style, WidgetError,
    },
};

struct WidgetBar2 {
    _bar: Bar<'static>,
    _style_bg: Box<Style>,
    _style_indic: Box<Style>,
}

impl View for WidgetBar2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style_bg = Box::new(Style::new());
        style_bg
            .border_color(palette_main(Palette::Blue))
            .border_width(2)
            .pad_all(6)
            .radius(6)
            .anim_duration(1000);

        let mut style_indic = Box::new(Style::new());
        style_indic
            .bg_opa(255)
            .bg_color(palette_main(Palette::Blue))
            .radius(3);

        let bar = Bar::new(&screen)?;
        bar.remove_style_all();
        bar.add_style(&style_bg, Selector::DEFAULT);
        bar.add_style(&style_indic, Part::Indicator);
        bar.size(200, 20).center();
        bar.set_range_raw(0, 100);
        bar.set_value_raw(100, true);

        Ok(Self {
            _bar: bar,
            _style_bg: style_bg,
            _style_indic: style_indic,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetBar2);
