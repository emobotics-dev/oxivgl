#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 2 — Background gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        palette_lighten, palette_main, GradDir, GradDsc, Obj, Palette, Screen, Selector, Style,
        WidgetError,
    },
};

struct Style2 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}

impl View for Style2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut grad = Box::new(GradDsc::new());
        grad.set_dir(GradDir::Ver)
            .set_stops_count(2)
            .set_stop(0, palette_lighten(Palette::Grey, 1), 255, 128)
            .set_stop(1, palette_main(Palette::Blue), 255, 192);

        let mut style = Box::new(Style::new());
        style.radius(5).bg_opa(255).bg_grad(&grad);

        let obj = Obj::new(&screen)?;
        obj.add_style(&style, Selector::DEFAULT);
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
