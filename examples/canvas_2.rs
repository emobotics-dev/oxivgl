#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 2 — Transparent pixel bands

use oxivgl::{
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas2 {
    _canvas: Canvas<'static>,
}

impl View for Canvas2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let buf = DrawBuf::create(80, 60, ColorFormat::ARGB8888)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(&screen, buf)?;
        canvas.fill_bg(color_make(0, 0, 196), 255);
        for y in 10..20_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 128);
            }
        }
        for y in 20..30_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 51);
            }
        }
        for y in 30..40_i32 {
            for x in 0..80_i32 {
                canvas.set_px(x, y, color_make(255, 0, 0), 0);
            }
        }
        canvas.align(Align::Center, 0, 0);
        Ok(Self { _canvas: canvas })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas2);
