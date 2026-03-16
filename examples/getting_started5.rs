#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 5 — Simple Horizontal Gradient

use oxivgl::{
    view::View,
    widgets::{
        color_make, lv_pct, GradDsc, Obj, Screen, Selector, Style, StyleBuilder, WidgetError,
    },
};

struct GettingStarted5 {
    _obj: Obj<'static>,
    _style: Style,
}

impl View for GettingStarted5 {
    fn create() -> Result<Self, WidgetError> {
        let colors = [color_make(0xff, 0, 0), color_make(0, 0xff, 0)];
        let opas = [255u8, 0];
        let fracs = [(20u16 * 255 / 100) as u8, (80u16 * 255 / 100) as u8];

        let mut grad = GradDsc::new();
        grad.init_stops(&colors, &opas, &fracs).horizontal();

        let mut style = StyleBuilder::new();
        style
            .bg_opa(255)
            .bg_grad(grad)
            .border_width(2)
            .radius(12)
            .pad_all(0);
        let style = style.build();

        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let obj = Obj::new(&screen)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

        Ok(Self {
            _obj: obj,
            _style: style,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted5);
