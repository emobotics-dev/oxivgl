#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 3 — Custom Styles

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        darken_filter_cb, palette_lighten, palette_main, Button, ColorFilter, GradDir, Label,
        ObjState, Opa, Palette, Screen, Style, WidgetError,
    },
};

struct GettingStarted3 {
    _lbl2: Label<'static>,
    _btn2: Button<'static>,
    _lbl1: Label<'static>,
    _btn1: Button<'static>,
    _style_red: Box<Style>,
    _style_pressed: Box<Style>,
    _style_btn: Box<Style>,
    _color_filter: Box<ColorFilter>,
}

impl View for GettingStarted3 {
    fn create() -> Result<Self, WidgetError> {
        let color_filter = Box::new(ColorFilter::new(darken_filter_cb));

        let mut style_btn = Box::new(Style::new());
        style_btn
            .radius(10)
            .bg_opa(Opa::COVER.0)
            .bg_color(palette_lighten(Palette::Grey, 3))
            .bg_grad_color(palette_main(Palette::Grey))
            .bg_grad_dir(GradDir::Ver)
            .border_color_hex(0x000000)
            .border_opa(Opa::OPA_20.0)
            .border_width(2)
            .text_color_hex(0x000000);

        let mut style_pressed = Box::new(Style::new());
        style_pressed.color_filter(&color_filter, Opa::OPA_20.0);

        let mut style_red = Box::new(Style::new());
        style_red
            .bg_color(palette_main(Palette::Red))
            .bg_grad_color(palette_lighten(Palette::Red, 3));

        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let btn1 = Button::new(&screen)?;
        btn1.remove_style_all().pos(10, 10).size(120, 50);
        btn1.add_style(&style_btn, 0);
        btn1.add_style(&style_pressed, ObjState::PRESSED.0);

        let lbl1 = Label::new(&btn1)?;
        lbl1.text("Button").center();

        let btn2 = Button::new(&screen)?;
        btn2.remove_style_all().pos(10, 80).size(120, 50);
        btn2.add_style(&style_btn, 0);
        btn2.add_style(&style_red, 0);
        btn2.add_style(&style_pressed, ObjState::PRESSED.0);
        btn2.radius(0x7fff, 0);

        let lbl2 = Label::new(&btn2)?;
        lbl2.text("Button 2").center();

        Ok(Self {
            _lbl2: lbl2,
            _btn2: btn2,
            _lbl1: lbl1,
            _btn1: btn1,
            _style_red: style_red,
            _style_pressed: style_pressed,
            _style_btn: style_btn,
            _color_filter: color_filter,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted3);
