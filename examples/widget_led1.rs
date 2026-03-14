#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget LED 1 — LED brightness and color
//!
//! Three LEDs: off, dim red (brightness 150), and full on.

use oxivgl::{
    view::View,
    widgets::{palette_main, Align, Led, Palette, Screen, WidgetError},
};

struct WidgetLed1 {
    _led1: Led<'static>,
    _led2: Led<'static>,
    _led3: Led<'static>,
}

impl View for WidgetLed1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let led1 = Led::new(&screen)?;
        led1.align(Align::Center, -80, 0);
        led1.off();

        let led2 = Led::new(&screen)?;
        led2.align(Align::Center, 0, 0);
        led2.set_brightness(150);
        led2.set_color(palette_main(Palette::Red));

        let led3 = Led::new(&screen)?;
        led3.align(Align::Center, 80, 0);
        led3.on();

        Ok(Self {
            _led1: led1,
            _led2: led2,
            _led3: led3,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetLed1);
