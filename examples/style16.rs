#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 16 — Conical gradient (metallic knob)

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{GradDsc, GradExtend, Obj, Screen, Style, WidgetError, color_black, color_make, lv_pct},
};

struct Style16 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}

impl View for Style16 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let colors = [
            color_make(0xe8, 0xe8, 0xe8),
            color_make(0xff, 0xff, 0xff),
            color_make(0xfa, 0xfa, 0xfa),
            color_make(0x79, 0x79, 0x79),
            color_make(0x48, 0x48, 0x48),
            color_make(0x4b, 0x4b, 0x4b),
            color_make(0x70, 0x70, 0x70),
            color_make(0xe8, 0xe8, 0xe8),
        ];

        let mut grad = Box::new(GradDsc::new());
        grad.init_stops(&colors, &[], &[])
            .conical(lv_pct(50), lv_pct(50), 0, 120, GradExtend::Reflect);

        let mut style = Box::new(Style::new());
        style
            .radius(500)
            .bg_opa(255)
            .shadow_color(color_black())
            .shadow_width(50)
            .shadow_offset_x(20)
            .shadow_offset_y(20)
            .shadow_opa(127)
            .bg_grad(&grad);

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, 0);
        obj.size(200, 200);
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

oxivgl_examples_common::example_main!(Style16);
