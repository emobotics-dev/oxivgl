#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 2 — Runtime image recoloring
//!
//! Cogwheel image with RGB + intensity sliders controlling recolor tint.

use oxivgl::{
    style::{color_make, palette_main, Palette, Selector},
    view::View,
    widgets::{Align, Image, Part, Screen, Slider, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

struct WidgetImage2 {
    slider_r: Slider<'static>,
    slider_g: Slider<'static>,
    slider_b: Slider<'static>,
    slider_i: Slider<'static>,
    img: Image<'static>,
}

impl View for WidgetImage2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Create image on the right side
        let img = Image::new(&screen)?;
        img.set_src(img_cogwheel_argb());
        img.align(Align::RightMid, -80, 0);

        // Helper to create a vertical slider
        let make_slider =
            |parent: &Screen, x: i32, initial: i32| -> Result<Slider<'static>, WidgetError> {
                let s = Slider::new(parent)?;
                s.set_range(0, 255);
                s.set_value(initial);
                s.size(10, 200);
                s.align(Align::LeftMid, x, 0);
                Ok(s)
            };

        let slider_r = make_slider(&screen, 20, 51)?;
        slider_r.style_bg_color(palette_main(Palette::Red), Part::Knob);

        let slider_g = make_slider(&screen, 50, 230)?;
        slider_g.style_bg_color(palette_main(Palette::Green), Part::Knob);

        let slider_b = make_slider(&screen, 80, 153)?;
        slider_b.style_bg_color(palette_main(Palette::Blue), Part::Knob);

        let slider_i = make_slider(&screen, 110, 128)?;

        Ok(Self {
            slider_r,
            slider_g,
            slider_b,
            slider_i,
            img,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let r = self.slider_r.get_value() as u8;
        let g = self.slider_g.get_value() as u8;
        let b = self.slider_b.get_value() as u8;
        let intense = self.slider_i.get_value() as u8;

        let color = color_make(r, g, b);
        self.img
            .style_image_recolor(color, Selector::DEFAULT)
            .style_image_recolor_opa(intense, Selector::DEFAULT);
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetImage2);
