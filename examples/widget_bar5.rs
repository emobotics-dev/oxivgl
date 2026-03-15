#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 5 — LTR vs RTL bars
//!
//! Two bars side by side: one left-to-right (default), one right-to-left.

use oxivgl::{
    view::View,
    widgets::{Align, Bar, BaseDir, Label, Screen, Selector, WidgetError},
};

struct WidgetBar5 {
    _bar_ltr: Bar<'static>,
    _bar_rtl: Bar<'static>,
    _label_ltr: Label<'static>,
    _label_rtl: Label<'static>,
}

impl View for WidgetBar5 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // LTR bar
        let bar_ltr = Bar::new(&screen)?;
        bar_ltr.size(200, 20);
        bar_ltr.set_range_raw(0, 100);
        bar_ltr.set_value_raw(70, false);
        bar_ltr.align(Align::Center, 0, -30);

        let label_ltr = Label::new(&screen)?;
        label_ltr.text("Left to Right base direction");
        label_ltr.align_to(&bar_ltr, Align::OutTopMid, 0, -5);

        // RTL bar
        let bar_rtl = Bar::new(&screen)?;
        bar_rtl.size(200, 20);
        bar_rtl.set_range_raw(0, 100);
        bar_rtl.set_value_raw(70, false);
        bar_rtl.set_style_base_dir(BaseDir::Rtl, Selector::DEFAULT);
        bar_rtl.align(Align::Center, 0, 30);

        let label_rtl = Label::new(&screen)?;
        label_rtl.text("Right to Left base direction");
        label_rtl.align_to(&bar_rtl, Align::OutTopMid, 0, -5);

        Ok(Self {
            _bar_ltr: bar_ltr,
            _bar_rtl: bar_rtl,
            _label_ltr: label_ltr,
            _label_rtl: label_rtl,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetBar5);
