#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 17 — Radial gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{Obj, Screen, Style, WidgetError, color_make, lv_pct},
};

struct Style17 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<lvgl_rust_sys::lv_grad_dsc_t>,
}

impl View for Style17 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let colors: [lvgl_rust_sys::lv_color_t; 2] = [
            color_make(0x9b, 0x18, 0x42),
            color_make(0x00, 0x00, 0x00),
        ];

        let mut grad = Box::new(unsafe { core::mem::zeroed::<lvgl_rust_sys::lv_grad_dsc_t>() });
        unsafe {
            lvgl_rust_sys::lv_grad_init_stops(
                &mut *grad,
                colors.as_ptr(),
                core::ptr::null(),
                core::ptr::null(),
                2,
            );
            lvgl_rust_sys::lv_grad_radial_init(
                &mut *grad,
                lv_pct(50),
                lv_pct(50),
                lv_pct(100),
                lv_pct(100),
                lvgl_rust_sys::lv_grad_extend_t_LV_GRAD_EXTEND_PAD,
            );
        }

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
