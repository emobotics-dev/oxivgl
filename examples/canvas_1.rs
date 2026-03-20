#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 1 — Dual canvas + image rotation

use oxivgl::{
    draw::{Area, DrawImageDsc, DrawLabelDscOwned, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas1 {
    _canvas1: Canvas<'static>,
    _canvas2: Canvas<'static>,
}

impl View for Canvas1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Canvas 1: RGB565, gradient rect + label
        let buf1 = DrawBuf::create(200, 150, ColorFormat::RGB565)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas1 = Canvas::new(&screen, buf1)?;
        canvas1.fill_bg(color_make(0xcc, 0xcc, 0xcc), 255);
        canvas1.align(Align::TopMid, 0, 10);
        {
            let mut layer = canvas1.init_layer();
            let mut rdc = DrawRectDsc::new();
            rdc.bg_color(color_make(0xff, 0x00, 0x00))
                .border_width(2)
                .border_color(color_make(0, 0, 0))
                .radius(5);
            layer.draw_rect(&rdc, Area { x1: 10, y1: 10, x2: 190, y2: 140 });
            let mut ldc = DrawLabelDscOwned::default_font();
            ldc.set_color(color_make(0xff, 0xa5, 0x00));
            layer.draw_label(&ldc, Area { x1: 80, y1: 65, x2: 180, y2: 85 }, "Canvas 1");
        }

        // Canvas 2: ARGB8888, rotated snapshot of canvas1
        let buf2 = DrawBuf::create(200, 150, ColorFormat::ARGB8888)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas2 = Canvas::new(&screen, buf2)?;
        canvas2.fill_bg(color_make(0x80, 0x80, 0x80), 255);
        canvas2.align(Align::BottomMid, 0, -10);
        {
            let img = canvas1.draw_buf().image_dsc();
            let mut layer = canvas2.init_layer();
            let mut dsc = DrawImageDsc::from_image_dsc(&img);
            dsc.rotation(120).pivot(100, 75).opa(255);
            layer.draw_image(&dsc, Area { x1: 0, y1: 0, x2: 199, y2: 149 });
        }

        Ok(Self { _canvas1: canvas1, _canvas2: canvas2 })
    }

    fn register_events(&mut self) {}
    fn on_event(&mut self, _: &oxivgl::event::Event) {}

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas1);
