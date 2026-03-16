#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 7 — Reversed vertical bar
//!
//! Vertical bar filling top-to-bottom via reversed range (100→0), at 70%.

use oxivgl::{
    view::View,
    widgets::{Align, Bar, Label, Screen, WidgetError},
};

struct WidgetBar7 {
    _bar: Bar<'static>,
    _label: Label<'static>,
}

impl View for WidgetBar7 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let bar = Bar::new(&screen)?;
        bar.size(20, 200);
        bar.set_range_raw(100, 0);
        bar.set_value_raw(70, false);
        bar.align(Align::Center, 0, -30);

        let label = Label::new(&screen)?;
        label.text("Top to bottom");
        label.align_to(&bar, Align::OutTopMid, 0, -5);

        Ok(Self {
            _bar: bar,
            _label: label,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetBar7);
