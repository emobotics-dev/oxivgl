#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 18 — Various gradient buttons

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        color_make, lv_pct, Align, Button, GradDir, GradDsc, GradExtend, Label, Screen, Selector,
        Style, WidgetError,
    },
};

struct Style18 {
    _label4: Label<'static>,
    _btn4: Button<'static>,
    _label3: Label<'static>,
    _btn3: Button<'static>,
    _label2: Label<'static>,
    _btn2: Button<'static>,
    _label1: Label<'static>,
    _btn1: Button<'static>,
    _style_radial: Box<Style>,
    _style_linear: Box<Style>,
    _grad_radial: Box<GradDsc>,
    _grad_linear: Box<GradDsc>,
}

impl View for Style18 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let c0 = color_make(0x26, 0xa0, 0xda);
        let c1 = color_make(0x31, 0x47, 0x55);
        let colors = [c0, c1];

        let mut grad_linear = Box::new(GradDsc::new());
        grad_linear.init_stops(&colors, &[], &[]).linear(
            lv_pct(0),
            lv_pct(0),
            lv_pct(20),
            lv_pct(100),
            GradExtend::Reflect,
        );
        let mut style_linear = Box::new(Style::new());
        style_linear.bg_grad(&grad_linear).bg_opa(255);

        let mut grad_radial = Box::new(GradDsc::new());
        grad_radial.init_stops(&colors, &[], &[]).radial(
            lv_pct(30),
            lv_pct(30),
            lv_pct(100),
            lv_pct(100),
            GradExtend::Reflect,
        );
        let mut style_radial = Box::new(Style::new());
        style_radial.bg_grad(&grad_radial).bg_opa(255);

        let btn1 = Button::new(&screen)?;
        btn1.size(150, 50).align(Align::Center, 0, -100);
        btn1.style_bg_color(c0, Selector::DEFAULT);
        btn1.style_bg_grad_color(c1, Selector::DEFAULT);
        btn1.style_bg_grad_dir(GradDir::Hor, Selector::DEFAULT);
        let label1 = Label::new(&btn1)?;
        label1.text("Horizontal").center();

        let btn2 = Button::new(&screen)?;
        btn2.size(150, 50).align(Align::Center, 0, -40);
        btn2.style_bg_color(c0, Selector::DEFAULT);
        btn2.style_bg_grad_color(c1, Selector::DEFAULT);
        btn2.style_bg_grad_dir(GradDir::Ver, Selector::DEFAULT);
        let label2 = Label::new(&btn2)?;
        label2.text("Vertical").center();

        let btn3 = Button::new(&screen)?;
        btn3.size(150, 50).align(Align::Center, 0, 20);
        btn3.add_style(&style_linear, Selector::DEFAULT);
        let label3 = Label::new(&btn3)?;
        label3.text("Linear").center();

        let btn4 = Button::new(&screen)?;
        btn4.size(150, 50).align(Align::Center, 0, 80);
        btn4.add_style(&style_radial, Selector::DEFAULT);
        let label4 = Label::new(&btn4)?;
        label4.text("Radial").center();

        Ok(Self {
            _label4: label4,
            _btn4: btn4,
            _label3: label3,
            _btn3: btn3,
            _label2: label2,
            _btn2: btn2,
            _label1: label1,
            _btn1: btn1,
            _style_radial: style_radial,
            _style_linear: style_linear,
            _grad_radial: grad_radial,
            _grad_linear: grad_linear,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style18);
