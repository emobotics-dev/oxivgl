#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 1 — Size, Position and Padding

extern crate alloc;

use oxivgl::{
    view::View,
    widgets::{
        lv_pct, Label, Obj, Screen, Selector, Style, StyleBuilder, WidgetError, LV_SIZE_CONTENT,
    },
};

struct Style1 {
    _label: Label<'static>,
    _obj: Obj<'static>,
    _style: Style,
}

impl View for Style1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut builder = StyleBuilder::new();
        builder
            .radius(5)
            .width(150)
            .height(LV_SIZE_CONTENT)
            .pad_ver(20)
            .pad_left(5)
            .x(lv_pct(50))
            .y(80);
        let style = builder.build();

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, Selector::DEFAULT);

        let label = Label::new(&obj)?;
        label.text("Hello");

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
