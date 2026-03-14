#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Obj 3 — 3D matrix transform
//!
//! Centered object with animated scale + rotation via matrix transform.
//! Scale grows from 0.1 to 2.0, rotation follows `value * 360°`.
//! Resets and repeats at 2.0.

use oxivgl::{
    view::View,
    widgets::{Matrix, Obj, Screen, WidgetError},
};

struct WidgetObj3 {
    obj: Obj<'static>,
    value: f32,
}

impl View for WidgetObj3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let obj = Obj::new(&screen)?;
        obj.center();

        Ok(Self { obj, value: 0.1 })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        self.value += 0.01;

        if self.value > 2.0 {
            self.obj.reset_transform();
            self.value = 0.1;
        } else {
            let mut m = Matrix::identity();
            m.scale(self.value, 1.0).rotate(self.value * 360.0);
            self.obj.set_transform(&m);
        }

        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetObj3);
