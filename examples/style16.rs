#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 16 — Conical gradient (metallic knob)

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{Obj, Screen, Style, WidgetError, color_black, color_make, lv_pct},
};

struct Style16 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<lvgl_rust_sys::lv_grad_dsc_t>,
}

impl View for Style16 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let colors: [lvgl_rust_sys::lv_color_t; 8] = [
            color_make(0xe8, 0xe8, 0xe8),
            color_make(0xff, 0xff, 0xff),
            color_make(0xfa, 0xfa, 0xfa),
            color_make(0x79, 0x79, 0x79),
            color_make(0x48, 0x48, 0x48),
            color_make(0x4b, 0x4b, 0x4b),
            color_make(0x70, 0x70, 0x70),
            color_make(0xe8, 0xe8, 0xe8),
        ];

        let mut grad = Box::new(unsafe { core::mem::zeroed::<lvgl_rust_sys::lv_grad_dsc_t>() });
        unsafe {
            lvgl_rust_sys::lv_grad_init_stops(
                &mut *grad,
                colors.as_ptr(),
                core::ptr::null(),
                core::ptr::null(),
                8,
            );
            lvgl_rust_sys::lv_grad_conical_init(
                &mut *grad,
                lv_pct(50),
                lv_pct(50),
                0,
                120,
                lvgl_rust_sys::lv_grad_extend_t_LV_GRAD_EXTEND_REFLECT,
            );
        }

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
