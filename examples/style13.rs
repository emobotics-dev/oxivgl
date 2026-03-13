#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 13 — Parts and states

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        GradDir, LV_STATE_PRESSED, Palette, Part, Screen, Slider, Style, WidgetError,
        palette_lighten, palette_main,
    },
};

struct Style13 {
    _slider: Slider<'static>,
    _style_indic: Box<Style>,
    _style_indic_pr: Box<Style>,
}

impl View for Style13 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style_indic = Box::new(Style::new());
        style_indic
            .bg_color(palette_lighten(Palette::Red, 3))
            .bg_grad_color(palette_main(Palette::Red))
            .bg_grad_dir(GradDir::Hor);

        let mut style_indic_pr = Box::new(Style::new());
        style_indic_pr
            .shadow_color(palette_main(Palette::Red))
            .shadow_width(10)
            .shadow_spread(3);

        let slider = Slider::new(&screen)?;
        slider.add_style(&style_indic, Part::Indicator as u32);
        slider.add_style(&style_indic_pr, Part::Indicator as u32 | LV_STATE_PRESSED);
        slider.set_value(70);
        slider.center();

        Ok(Self {
            _slider: slider,
            _style_indic: style_indic,
            _style_indic_pr: style_indic_pr,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style13);
