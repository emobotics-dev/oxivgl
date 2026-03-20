#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 5 — Arc

use oxivgl::{
    draw::DrawArcDsc,
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas5 {
    _canvas: Canvas<'static>,
}

impl View for Canvas5 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(
            &screen,
            DrawBuf::create(50, 50, ColorFormat::RGB565).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas.align(Align::Center, 0, 0);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawArcDsc::new();
            dsc.center(25, 25)
                .radius(15)
                .angles(0.0, 220.0)
                .width(10)
                .color(color_make(0xff, 0, 0))
                .opa(255);
            layer.draw_arc(&dsc);
        }
        Ok(Self { _canvas: canvas })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas5);
