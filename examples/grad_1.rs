#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grad 1 — Horizontal gradient with stop fracs
//!
//! Simplified: interactive drag omitted — `lv_indev_get_point` and direct
//! gradient field mutation not yet wrapped. Bullets shown at initial positions.

use oxivgl::{
    style::{GradDsc, Selector, Style, StyleBuilder, color_make, lv_pct},
    view::View,
    widgets::{Align, Button, Obj, Screen, WidgetError},
};

struct Grad1 {
    _obj: Obj<'static>,
    _bullet1: Button<'static>,
    _bullet2: Button<'static>,
    _style: Style, // last — drop after widgets that reference it
}

impl View for Grad1 {
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
            .pad_all(0)
            .radius(12);
        let style = style.build();

        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let obj = Obj::new(&screen)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

        // Bullet 1: magenta at (20%, 50%)
        let bullet1 = Button::new(&obj)?;
        bullet1
            .size(15, 15)
            .align(Align::TopLeft, lv_pct(20), lv_pct(50));
        bullet1.bg_color(0xff00ff).bg_opa(255);

        // Bullet 2: yellow at (80%, 50%)
        let bullet2 = Button::new(&obj)?;
        bullet2
            .size(15, 15)
            .align(Align::TopLeft, lv_pct(80), lv_pct(50));
        bullet2.bg_color(0xffff00).bg_opa(255);

        Ok(Self {
            _style: style,
            _obj: obj,
            _bullet1: bullet1,
            _bullet2: bullet2,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Grad1);
