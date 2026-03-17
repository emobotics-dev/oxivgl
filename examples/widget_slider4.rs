#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Slider 4 — Reversed slider
//!
//! Slider with opposite direction (100→0) and a percentage label below.

use oxivgl::{
    view::View,
    widgets::{Align, Label, Screen, Slider, WidgetError},
};

struct WidgetSlider4 {
    slider: Slider<'static>,
    label: Label<'static>,
}

impl View for WidgetSlider4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let slider = Slider::new(&screen)?;
        slider.center();
        slider.set_range(100, 0);

        let label = Label::new(&screen)?;
        label.text("0%");
        label.align_to(&slider, Align::OutBottomMid, 0, 10);

        Ok(Self { slider, label })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        use core::fmt::Write;
        let val = self.slider.get_value();
        let mut buf = heapless::String::<8>::new();
        let _ = write!(buf, "{}%", val);
        self.label.text(&buf);
        self.label
            .align_to(&self.slider, Align::OutBottomMid, 0, 10);
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetSlider4);
