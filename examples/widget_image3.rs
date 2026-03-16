#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 3 — Image rotation
//!
//! Cogwheel image rotating continuously via `update()`.

use oxivgl::{
    view::View,
    widgets::{Image, Screen, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

struct WidgetImage3 {
    img: Image<'static>,
    angle: i32,
}

impl View for WidgetImage3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let img = Image::new(&screen)?;
        img.set_src(img_cogwheel_argb());
        img.center();
        img.set_pivot(50, 50);

        Ok(Self { img, angle: 0 })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.angle = (self.angle + 30) % 3600;
        self.img.set_rotation(self.angle);
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetImage3);
