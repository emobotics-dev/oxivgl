#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 9 — Gradient triangle

use oxivgl::{
    draw::DrawTriangleDsc,
    draw_buf::{ColorFormat, DrawBuf},
    style::{color_make, GradDir},
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas9 {
    _canvas: Canvas<'static>,
}

impl View for Canvas9 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(
            &screen,
            DrawBuf::create(100, 100, ColorFormat::RGB565).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas.align(Align::Center, 0, 0);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawTriangleDsc::new();
            dsc.points([(7.0, 7.0), (67.0, 20.0), (33.0, 67.0)])
                .opa(128)
                .grad_stops_count(2)
                .grad_dir(GradDir::Ver)
                .grad_stop(0, color_make(0xff, 0x00, 0x00), 64, 255)
                .grad_stop(1, color_make(0x00, 0x00, 0xff), 192, 0);
            layer.draw_triangle(&dsc);
        }
        Ok(Self { _canvas: canvas })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas9);
