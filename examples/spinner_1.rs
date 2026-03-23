#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Spinner 1 — Centered loading spinner
//!
//! 100×100 spinner with a 10 s animation cycle and 200° arc.

use oxivgl::{
    view::View,
    widgets::{Spinner, WidgetError, Screen},
};

struct Spinner1 {
    _spinner: Spinner<'static>,
}

impl View for Spinner1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let spinner = Spinner::new(&screen)?;
        spinner.size(100, 100).center();
        spinner.set_anim_params(10000, 200);

        Ok(Self { _spinner: spinner })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Spinner1);
