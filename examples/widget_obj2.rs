#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Obj 2 — Draggable object
//!
//! A base object that follows the pointer when pressed, using
//! `Indev::active().get_vect()` to read the movement delta.

use oxivgl::{
    enums::EventCode,
    event::Event,
    indev::Indev,
    view::View,
    widgets::{Label, Obj, Screen, WidgetError},
};

struct WidgetObj2 {
    obj: Obj<'static>,
    _label: Label<'static>,
}

impl View for WidgetObj2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let obj = Obj::new(&screen)?;
        obj.size(150, 100);

        let label = Label::new(&obj)?;
        label.text("Drag me").center();

        Ok(Self { obj, _label: label })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.obj, EventCode::PRESSING) {
            if let Some(indev) = Indev::active() {
                let vect = indev.get_vect();
                let x = self.obj.get_x() + vect.x;
                let y = self.obj.get_y() + vect.y;
                self.obj.pos(x, y);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetObj2);
