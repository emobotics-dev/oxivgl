#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 4 — Slider

use oxivgl::{
    view::View,
    widgets::{Align, Label, Screen, Slider, WidgetError},
};

struct GettingStarted4 {
    _slider: Slider<'static>,
    _label: Label<'static>,
}

impl View for GettingStarted4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let slider = Slider::new(&screen)?;
        slider.width(200).center();

        let label = Label::new(&screen)?;
        label.text("0");
        label.align_to(&slider, Align::OutTopMid, 0, -15);

        Ok(Self {
            _slider: slider,
            _label: label,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted4);
