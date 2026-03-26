#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 1 — Slider bound to a temperature label via a Subject

use oxivgl::{
    view::View,
    widgets::{Align, Label, Screen, Slider, Subject, WidgetError},
};

struct Observer1 {
    _subject: Subject,
    _slider: Slider<'static>,
    _label: Label<'static>,
}

impl View for Observer1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let subject = Subject::new_int(28);

        let slider = Slider::new(&screen)?;
        slider.set_range(0, 100).align(Align::Center, 0, 0);
        slider.bind_value(&subject);

        let label = Label::new(&screen)?;
        label.align(Align::Center, 0, 30);
        label.bind_text(&subject, c"%d \u{00b0}C");

        Ok(Self {
            _subject: subject,
            _slider: slider,
            _label: label,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer1);
