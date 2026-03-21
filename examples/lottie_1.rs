#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lottie 1 — Lottie animation from in-memory data
//!
//! Plays the bundled "approve" Lottie animation (64×64 px) centred on screen.
//! Requires `LV_USE_LOTTIE = 1` and `LV_USE_THORVG_INTERNAL = 1`.

use oxivgl::{
    view::View,
    widgets::{Align, Lottie, Screen, WidgetError},
};

const APPROVE_JSON: &[u8] = include_bytes!(
    "../thirdparty/lvgl_rust_sys/lvgl/examples/widgets/lottie/lv_example_lottie_approve.json"
);

struct Lottie1 {
    lottie: Lottie<'static>,
}

impl View for Lottie1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let lottie = Lottie::new(&screen)?;
        lottie.set_buffer(64, 64).set_src_data(APPROVE_JSON);
        lottie.align(Align::Center, 0, 0);

        Ok(Self { lottie })
    }

    fn on_event(&mut self, _event: &oxivgl::event::Event) {}

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Lottie1);
