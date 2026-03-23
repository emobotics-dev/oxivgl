#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Spinbox 1 — Numeric input with +/− buttons
//!
//! Spinbox with range −1000..25000, 5 digits and 2 decimal places.
//! Plus and minus buttons on either side increment/decrement the value.

use oxivgl::{
    enums::EventCode,
    event::Event,
    style::Selector,
    symbols,
    view::View,
    widgets::{Align, Button, Screen, Spinbox, WidgetError},
};

struct Spinbox1 {
    spinbox: Spinbox<'static>,
    btn_plus: Button<'static>,
    btn_minus: Button<'static>,
}

impl View for Spinbox1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let spinbox = Spinbox::new(&screen)?;
        spinbox
            .set_range(-1000, 25000)
            .set_digit_format(5, 2)
            .step_prev();
        spinbox.width(100).center();

        let h = spinbox.get_height();

        let btn_plus = Button::new(&screen)?;
        btn_plus.size(h, h).align_to(&spinbox, Align::OutRightMid, 5, 0);
        btn_plus.style_bg_image_src_symbol(&symbols::PLUS, Selector::DEFAULT);
        btn_plus.bubble_events();

        let btn_minus = Button::new(&screen)?;
        btn_minus.size(h, h).align_to(&spinbox, Align::OutLeftMid, -5, 0);
        btn_minus.style_bg_image_src_symbol(&symbols::MINUS, Selector::DEFAULT);
        btn_minus.bubble_events();

        Ok(Self { spinbox, btn_plus, btn_minus })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.btn_plus, EventCode::SHORT_CLICKED) {
            self.spinbox.increment();
        }
        if event.matches(&self.btn_minus, EventCode::SHORT_CLICKED) {
            self.spinbox.decrement();
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Spinbox1);
