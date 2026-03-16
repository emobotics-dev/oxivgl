#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 15 — Opacity and Transformations

use oxivgl::{
    style::Selector,
    view::View,
    widgets::{Align, Button, Label, Screen, WidgetError},
};

struct Style15 {
    _label3: Label<'static>,
    _btn3: Button<'static>,
    _label2: Label<'static>,
    _btn2: Button<'static>,
    _label1: Label<'static>,
    _btn1: Button<'static>,
}

impl View for Style15 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let btn1 = Button::new(&screen)?;
        btn1.size(100, 40).align(Align::Center, 0, -70);
        let label1 = Label::new(&btn1)?;
        label1.text("Normal").center();

        let btn2 = Button::new(&screen)?;
        btn2.size(100, 40).opa(128).align(Align::Center, 0, 0);
        let label2 = Label::new(&btn2)?;
        label2.text("Opa:50%").center();

        let btn3 = Button::new(&screen)?;
        btn3.size(100, 40).opa(128).align(Align::Center, 0, 70);
        btn3.style_transform_rotation(150, Selector::DEFAULT)
            .style_transform_scale(256 + 64, Selector::DEFAULT)
            .style_transform_pivot_x(50, Selector::DEFAULT)
            .style_transform_pivot_y(20, Selector::DEFAULT);
        let label3 = Label::new(&btn3)?;
        label3.text("Transf.").center();

        Ok(Self {
            _label3: label3,
            _btn3: btn3,
            _label2: label2,
            _btn2: btn2,
            _label1: label1,
            _btn1: btn1,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style15);
