#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 4 — Slider with live value label
//!
//! A centered slider with a label above showing the current value, updated
//! via VALUE_CHANGED event.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::{register_event_on, View},
    widgets::{Align, Label, Screen, Slider, WidgetError},
};

struct GettingStarted4 {
    slider: Slider<'static>,
    label: Label<'static>,
}

impl View for GettingStarted4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let slider = Slider::new(&screen)?;
        slider.width(200).center();
        slider.bubble_events();

        let label = Label::new(&screen)?;
        label.text("0");
        label.align_to(&slider, Align::OutTopMid, 0, -15);

        Ok(Self { slider, label })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.slider.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == EventCode::VALUE_CHANGED {
            let val = self.slider.get_value();
            let mut buf = heapless::String::<8>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", val));
            self.label.text(&buf);
            self.label
                .align_to(&self.slider, Align::OutTopMid, 0, -15);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted4);
