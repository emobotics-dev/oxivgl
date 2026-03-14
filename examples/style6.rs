#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 6 — Image style properties
//!
//! Cogwheel image rotated 30°, blue recolor tint (50% opacity), grey
//! background with blue border.

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{palette_lighten, palette_main, Image, Palette, Screen, Selector, Style, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

struct Style6 {
    _style: Box<Style>,
    _img: Image<'static>,
}

impl View for Style6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style = Box::new(Style::new());
        style
            .radius(5)
            .bg_opa(255)
            .bg_color(palette_lighten(Palette::Grey, 3))
            .border_width(2)
            .border_color(palette_main(Palette::Blue))
            .image_recolor(palette_main(Palette::Blue))
            .image_recolor_opa(128)
            .transform_rotation(300);

        let img = Image::new(&screen)?;
        img.add_style(&style, Selector::DEFAULT);
        // SAFETY: img_cogwheel_argb is a static C symbol compiled by oxivgl-build
        img.set_src(unsafe { &img_cogwheel_argb });
        img.center();

        Ok(Self {
            _style: style,
            _img: img,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style6);
