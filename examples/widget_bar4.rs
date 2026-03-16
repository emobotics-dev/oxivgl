#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Bar 4 — Stripe pattern with range value
//!
//! Bar with tiled stripe background image on the indicator at 30% opacity,
//! using range mode (start=20, end=90).

use oxivgl::{
    style::{Style, StyleBuilder},
    view::View,
    widgets::{Bar, BarMode, Part, Screen, WidgetError},
};

oxivgl::image_declare!(img_skew_strip);

struct WidgetBar4 {
    _style: Style,
    _bar: Bar<'static>,
}

impl View for WidgetBar4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style = StyleBuilder::new();
        style
            .bg_image_src(img_skew_strip())
            .bg_image_tiled(true)
            .bg_image_opa(77); // LV_OPA_30
        let style = style.build();

        let bar = Bar::new(&screen)?;
        bar.add_style(&style, Part::Indicator);
        bar.size(260, 20).center();
        bar.set_range_raw(0, 100);
        bar.set_mode(BarMode::Range);
        bar.set_value_raw(90, false);
        bar.set_start_value_raw(20, false);

        Ok(Self {
            _style: style,
            _bar: bar,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetBar4);
