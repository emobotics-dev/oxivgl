#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 4 — Text label

use oxivgl::{
    draw::{Area, DrawLabelDscOwned},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas4 {
    _canvas: Canvas<'static>,
}

impl View for Canvas4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(
            &screen,
            DrawBuf::create(80, 30, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas.align(Align::Center, 0, 0);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawLabelDscOwned::default_font();
            dsc.set_color(color_make(0xff, 0, 0));
            layer.draw_label(&dsc, Area { x1: 5, y1: 5, x2: 75, y2: 25 }, "Hello");
        }
        Ok(Self { _canvas: canvas })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas4);
