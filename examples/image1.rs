#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 1 — Basic image display
//!
//! Centered cogwheel image.

use oxivgl::{
    view::View,
    widgets::{Image, Screen, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

struct WidgetImage1 {
    _img: Image<'static>,
}

impl View for WidgetImage1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let img = Image::new(&screen)?;
        img.set_src(unsafe { &img_cogwheel_argb });
        img.center();

        Ok(Self { _img: img })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetImage1);
