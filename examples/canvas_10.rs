#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 10 — Wavy text animation

use oxivgl::{
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

struct Canvas10 {
    canvas: Canvas<'static>,
    counter: i32,
}

impl View for Canvas10 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(
            &screen,
            DrawBuf::create(300, 200, ColorFormat::ARGB8888).ok_or(WidgetError::LvglNullPointer)?,
        )?;
        canvas.fill_bg(color_make(0xff, 0xff, 0xff), 255);
        canvas.align(Align::Center, 0, 0);
        Ok(Self { canvas, counter: 0 })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        use oxivgl::draw::DrawLetterDsc;
        use oxivgl::math::{atan2, trigo_sin};
        use oxivgl::style::color_hsv;
        const TXT: &[u8] = b"Hello wavy world!";
        self.canvas.fill_bg(color_make(0xff, 0xff, 0xff), 255);
        {
            let mut layer = self.canvas.init_layer();
            let mut pre_x = 10_i32;
            let mut pre_y = 100_i32;
            for (i, &ch) in TXT.iter().enumerate() {
                let angle = i as i32 * 10;
                let x = angle * 7 + 10;
                let y = trigo_sin(((angle + self.counter / 2) * 5) as i32) * 40 / 32767 + 100;
                let mut dsc = DrawLetterDsc::new();
                dsc.unicode(ch as u32)
                    .color(color_hsv(((i as u16 * 15) % 360) as u16, 100, 100))
                    .rotation(atan2(y - pre_y, x - pre_x) as i32 * 10);
                layer.draw_letter(&dsc, x, y);
                pre_x = x;
                pre_y = y;
            }
        }
        self.counter += 1;
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas10);
