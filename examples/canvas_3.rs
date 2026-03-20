#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 3 — Rectangle with border and outline

use oxivgl::{
    draw::{Area, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas3 {
    _canvas: Canvas<'static>,
}

impl View for Canvas3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(
            &screen,
            DrawBuf::create(70, 70, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas.align(Align::Center, 0, 0);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawRectDsc::new();
            dsc.bg_color(color_make(0xff, 0, 0))
                .border_color(color_make(0, 0, 0xff))
                .border_width(4)
                .outline_color(color_make(0, 0xff, 0))
                .outline_width(2)
                .outline_pad(4)
                .radius(5);
            layer.draw_rect(&dsc, Area { x1: 10, y1: 10, x2: 60, y2: 60 });
        }
        Ok(Self { _canvas: canvas })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas3);
