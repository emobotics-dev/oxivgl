#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 11 — Windstorm text animation

use oxivgl::{
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas11 {
    canvas: Canvas<'static>,
    counter: i32,
}

impl View for Canvas11 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(
            &screen,
            DrawBuf::create(160, 100, ColorFormat::RGB565).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0, 0, 0), 255);
        canvas.align(Align::Center, 0, 0);
        Ok(Self { canvas, counter: 0 })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        use oxivgl::draw::DrawLetterDsc;
        use oxivgl::math::{trigo_cos, trigo_sin};
        use oxivgl::style::color_hsv;
        const TXT: &[u8] = b"windstorm";
        const W: i32 = 160;
        const H: i32 = 100;
        self.canvas.fill_bg(color_make(0, 0, 0), 255);
        {
            let mut layer = self.canvas.init_layer();
            let n = TXT.len() as i32;
            for i in 0..n * 2 {
                let ch_idx = (i % n) as usize;
                let ch = TXT[ch_idx];
                let x = (i * 7 + self.counter / 3) % W;
                let y = trigo_sin(i * 7 + self.counter) * 25 / 32767
                    + H / 2
                    + trigo_cos(i * 3 + self.counter / 2) * 15 / 32767;
                let mut dsc = DrawLetterDsc::new();
                dsc.unicode(ch as u32)
                    .color(color_hsv(
                        ((i as u16 * 20 + self.counter as u16) % 360) as u16,
                        100,
                        80,
                    ))
                    .rotation(0);
                layer.draw_letter(&dsc, x, y);
            }
        }
        self.counter += 1;
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas11);
