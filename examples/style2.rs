#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 2 — Background gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{Obj, Palette, Screen, Style, WidgetError, palette_lighten, palette_main},
};

struct Style2 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<lvgl_rust_sys::lv_grad_dsc_t>,
}

impl View for Style2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut grad = Box::new(unsafe { core::mem::zeroed::<lvgl_rust_sys::lv_grad_dsc_t>() });
        grad.set_dir(lvgl_rust_sys::lv_grad_dir_t_LV_GRAD_DIR_VER);
        grad.stops_count = 2;
        grad.stops[0].color = palette_lighten(Palette::Grey, 1);
        grad.stops[0].opa = 255;
        grad.stops[1].color = palette_main(Palette::Blue);
        grad.stops[1].opa = 255;
        grad.stops[0].frac = 128;
        grad.stops[1].frac = 192;

        let mut style = Box::new(Style::new());
        style.radius(5).bg_opa(255).bg_grad(&grad);

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, 0);
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

oxivgl_examples_common::example_main!(Style2);
