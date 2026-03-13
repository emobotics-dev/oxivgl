#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 2 — Playback animation

use core::ffi::c_void;
use lvgl_rust_sys::*;
use oxivgl::{
    view::View,
    widgets::{
        anim_path_ease_in_out, palette_main, Align, Anim, Obj, Palette, Screen, WidgetError,
        ANIM_REPEAT_INFINITE,
    },
};

struct Anim2 {
    _obj: Obj<'static>,
}

unsafe extern "C" fn anim_x_cb(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_x(var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn anim_size_cb(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_size(var as *mut lv_obj_t, v, v) };
}

impl View for Anim2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let obj = Obj::new(&screen)?;
        obj.remove_scrollable();
        obj.style_bg_color(palette_main(Palette::Red), 0);
        obj.radius(0x7fff, 0);
        obj.align(Align::LeftMid, 10, 0);

        let mut a = Anim::new();
        a.set_var(&obj)
            .set_values(10, 50)
            .set_duration(1000)
            .set_reverse_delay(100)
            .set_reverse_duration(300)
            .set_repeat_delay(500)
            .set_repeat_count(ANIM_REPEAT_INFINITE)
            .set_path_cb(Some(anim_path_ease_in_out));

        a.set_exec_cb(Some(anim_size_cb));
        a.start();

        a.set_exec_cb(Some(anim_x_cb));
        a.set_values(10, 240);
        a.start();

        Ok(Self { _obj: obj })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Anim2);
