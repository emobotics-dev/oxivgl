#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: GPL-3.0-only
//! Canvas 6 — Draw an image onto a canvas

use oxivgl::{
    draw::{Area, DrawImageDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::color_make,
    view::View,
    widgets::{Align, Canvas, Screen, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

struct Canvas6 {
    _canvas: Canvas<'static>,
}

impl View for Canvas6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Canvas large enough for the 100x100 cogwheel image
        let buf = DrawBuf::create(100, 100, ColorFormat::ARGB8888)
            .ok_or(WidgetError::LvglNullPointer)?;
        let canvas = Canvas::new(&screen, buf)?;
        canvas.fill_bg(color_make(0xcf, 0xcf, 0xcf), 255);
        canvas.align(Align::Center, 0, 0);

        {
            let mut layer = canvas.init_layer();
            let dsc = DrawImageDsc::from_static_dsc(img_cogwheel_argb());
            layer.draw_image(&dsc, Area { x1: 0, y1: 0, x2: 99, y2: 99 });
        }

        Ok(Self { _canvas: canvas })
    }

    fn register_events(&mut self) {}
    fn on_event(&mut self, _: &oxivgl::event::Event) {}

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Canvas6);
