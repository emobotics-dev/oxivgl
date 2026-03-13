#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 17 — Radial gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{GradDsc, GradExtend, Obj, Screen, Style, WidgetError, color_make, lv_pct},
};

struct Style17 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}

impl View for Style17 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let colors = [
            color_make(0x9b, 0x18, 0x42),
            color_make(0x00, 0x00, 0x00),
        ];

        let mut grad = Box::new(GradDsc::new());
        grad.init_stops(&colors, &[], &[])
            .radial(lv_pct(50), lv_pct(50), lv_pct(100), lv_pct(100), GradExtend::Pad);

        let mut style = Box::new(Style::new());
        style.bg_grad(&grad);

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, 0);
        obj.size(320, 240);
        obj.center();

        Ok(Self {
            _obj: obj,
            _style: style,
            _grad: grad,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style17);
