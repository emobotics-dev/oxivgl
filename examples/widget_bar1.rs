#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 1 — Simple progress bar
//!
//! A 200×20 bar centered on screen, set to 70%.

use oxivgl::{
    view::View,
    widgets::{Bar, Screen, WidgetError},
};

struct WidgetBar1 {
    _bar: Bar<'static>,
}

impl View for WidgetBar1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let bar = Bar::new(&screen)?;
        bar.size(200, 20).center();
        bar.set_range_raw(0, 100);
        bar.set_value_raw(70, false);

        Ok(Self { _bar: bar })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetBar1);
