#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Slider 3 — Range slider
//!
//! Range-mode slider with two handles and a label showing min–max values.

use oxivgl::{
    view::View,
    widgets::{Align, Label, Screen, Slider, SliderMode, WidgetError},
};

struct WidgetSlider3 {
    slider: Slider<'static>,
    label: Label<'static>,
}

impl View for WidgetSlider3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let slider = Slider::new(&screen)?;
        slider
            .set_range(0, 100)
            .set_mode(SliderMode::Range)
            .set_value(80)
            .set_start_value(20);
        slider.center();

        let label = Label::new(&screen)?;
        label.text("20 \u{2013} 80");
        label.align_to(&slider, Align::OutBottomMid, 0, 10);

        Ok(Self { slider, label })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        use core::fmt::Write;
        let left = self.slider.get_left_value();
        let right = self.slider.get_value();
        let mut buf = heapless::String::<16>::new();
        let _ = write!(buf, "{} \u{2013} {}", left, right);
        self.label.text(&buf);
        self.label
            .align_to(&self.slider, Align::OutBottomMid, 0, 10);
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetSlider3);
