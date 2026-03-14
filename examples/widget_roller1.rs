#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Roller 1 — Simple month roller
//!
//! Infinite roller displaying month names with 4 visible rows.

extern crate alloc;

use oxivgl::{
    view::View,
    widgets::{Roller, RollerMode, Screen, WidgetError},
};

struct WidgetRoller1 {
    _roller: Roller<'static>,
}

impl View for WidgetRoller1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let roller = Roller::new(&screen)?;
        roller.set_options(
            "January\n\
             February\n\
             March\n\
             April\n\
             May\n\
             June\n\
             July\n\
             August\n\
             September\n\
             October\n\
             November\n\
             December",
            RollerMode::Infinite,
        );
        roller.set_visible_row_count(4);
        roller.center();

        Ok(Self { _roller: roller })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetRoller1);
