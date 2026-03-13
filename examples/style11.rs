#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 11 — Multiple styles

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        Align, LV_SIZE_CONTENT, Label, Obj, Palette, Screen, Style, WidgetError, color_white,
        palette_darken, palette_main,
    },
};

struct Style11 {
    _label_warn: Label<'static>,
    _obj_warn: Obj<'static>,
    _label_base: Label<'static>,
    _obj_base: Obj<'static>,
    _style_warning: Box<Style>,
    _style_base: Box<Style>,
}

impl View for Style11 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style_base = Box::new(Style::new());
        style_base
            .bg_color(palette_main(Palette::LightBlue))
            .border_color(palette_darken(Palette::LightBlue, 3))
            .border_width(2)
            .radius(10)
            .shadow_width(10)
            .shadow_offset_y(5)
            .shadow_opa(127)
            .text_color(color_white())
            .width(100)
            .height(LV_SIZE_CONTENT);

        let mut style_warning = Box::new(Style::new());
        style_warning
            .bg_color(palette_main(Palette::Yellow))
            .border_color(palette_darken(Palette::Yellow, 3))
            .text_color(palette_darken(Palette::Yellow, 4));

        let obj_base = Obj::new(&screen)?;
        obj_base.add_style(&style_base, 0);
        obj_base.align(Align::LeftMid, 20, 0);

        let label_base = Label::new(&obj_base)?;
        label_base.text("Base\0")?.center();

        let obj_warn = Obj::new(&screen)?;
        obj_warn.add_style(&style_base, 0);
        obj_warn.add_style(&style_warning, 0);
        obj_warn.align(Align::RightMid, -20, 0);

        let label_warn = Label::new(&obj_warn)?;
        label_warn.text("Warning\0")?.center();

        Ok(Self {
            _label_warn: label_warn,
            _obj_warn: obj_warn,
            _label_base: label_base,
            _obj_base: obj_base,
            _style_warning: style_warning,
            _style_base: style_base,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style11);
