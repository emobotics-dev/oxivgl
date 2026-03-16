#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Slider 2 — Styled slider
//!
//! Slider with cyan custom styles: pill-shaped track, padded knob with border,
//! and a bg-color transition on press.

use oxivgl::{
    view::View,
    widgets::{
        anim_path_linear, color_make, palette_darken, palette_main, props, ObjState, Palette, Part,
        Screen, Selector, Slider, Style, StyleBuilder, TransitionDsc, WidgetError, RADIUS_MAX,
    },
};

struct WidgetSlider2 {
    _slider: Slider<'static>,
    _style_main: Style,
    _style_indic: Style,
    _style_knob: Style,
    _style_pressed: Style,
}

/// Transition property: background color.
static TRANS_PROPS: [props::lv_style_prop_t; 2] = [props::BG_COLOR, props::LAST];

impl View for WidgetSlider2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Main track
        let mut style_main = StyleBuilder::new();
        style_main
            .bg_opa(255)
            .bg_color(color_make(0xBB, 0xBB, 0xBB))
            .radius(RADIUS_MAX as i16)
            .pad_ver(-2);
        let style_main = style_main.build();

        // Indicator
        let mut style_indic = StyleBuilder::new();
        style_indic
            .bg_opa(255)
            .bg_color(palette_main(Palette::Cyan))
            .radius(RADIUS_MAX as i16)
            .transition(TransitionDsc::new(
                &TRANS_PROPS,
                Some(anim_path_linear),
                300,
                0,
            ));
        let style_indic = style_indic.build();

        // Knob
        let mut style_knob = StyleBuilder::new();
        style_knob
            .bg_opa(255)
            .bg_color(palette_main(Palette::Cyan))
            .border_color(palette_darken(Palette::Cyan, 3))
            .border_width(2)
            .radius(RADIUS_MAX as i16)
            .pad_all(6)
            .transition(TransitionDsc::new(
                &TRANS_PROPS,
                Some(anim_path_linear),
                300,
                0,
            ));
        let style_knob = style_knob.build();

        // Pressed state color
        let mut style_pressed = StyleBuilder::new();
        style_pressed.bg_color(palette_darken(Palette::Cyan, 2));
        let style_pressed = style_pressed.build();

        let slider = Slider::new(&screen)?;
        slider.remove_style_all();
        slider.add_style(&style_main, Selector::DEFAULT);
        slider.add_style(&style_indic, Part::Indicator);
        slider.add_style(&style_pressed, Part::Indicator | ObjState::PRESSED);
        slider.add_style(&style_knob, Part::Knob);
        slider.add_style(&style_pressed, Part::Knob | ObjState::PRESSED);
        slider.center();

        Ok(Self {
            _slider: slider,
            _style_main: style_main,
            _style_indic: style_indic,
            _style_knob: style_knob,
            _style_pressed: style_pressed,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetSlider2);
