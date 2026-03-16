#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 1 — Round gauge with tick marks
//!
//! A 270° round scale with labeled major ticks, built via `ScaleBuilder`.

use oxivgl::{
    view::View,
    widgets::{Scale, ScaleMode, Screen, WidgetError},
};

struct WidgetScale1 {
    _scale: Scale<'static>,
}

impl View for WidgetScale1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let scale = Scale::tick_ring(
            &screen,
            200,
            ScaleMode::RoundInner,
            135,
            270,
            100,
            21,
            5,
            true,
            15,
            8,
            0x333333,
            0x999999,
        )?;

        Ok(Self { _scale: scale })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetScale1);
