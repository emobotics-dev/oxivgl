#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Arc 1 — Arc with value-changed event
//!
//! Draggable arc (150×150, 270° sweep) with a label that shows the current
//! percentage and follows the arc's edge.

use oxivgl::{
    view::View,
    enums::EventCode,
    event::Event,
    widgets::{Arc, Label, Screen, WidgetError},
};

struct WidgetArc1 {
    arc: Arc<'static>,
    label: Label<'static>,
}

impl View for WidgetArc1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label = Label::new(&screen)?;

        let arc = Arc::new(&screen)?;
        arc.size(150, 150);
        arc.set_rotation(135);
        arc.set_bg_angles(0, 270);
        arc.set_value_raw(10);
        arc.center();

        // Initial label update
        let v = arc.get_value_raw();
        let mut buf = heapless::String::<8>::new();
        let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}%", v));
        label.text(&buf);
        arc.rotate_obj_to_angle(&label, 25);

        Ok(Self { arc, label })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.arc, EventCode::VALUE_CHANGED) {
            let v = self.arc.get_value_raw();
            let mut buf = heapless::String::<8>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}%", v));
            self.label.text(&buf);
            self.arc.rotate_obj_to_angle(&self.label, 25);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetArc1);
