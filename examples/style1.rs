#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 1 — Size, Position and Padding

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{LV_SIZE_CONTENT, Label, Obj, Screen, Style, WidgetError, lv_pct},
};

struct Style1 {
    _label: Label<'static>,
    _obj: Obj<'static>,
    _style: Box<Style>,
}

impl View for Style1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style = Box::new(Style::new());
        style
            .radius(5)
            .width(150)
            .height(LV_SIZE_CONTENT)
            .pad_ver(20)
            .pad_left(5)
            .x(lv_pct(50))
            .y(80);

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, 0);

        let label = Label::new(&obj)?;
        label.text("Hello\0")?;

        Ok(Self {
            _label: label,
            _obj: obj,
            _style: style,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style1);
