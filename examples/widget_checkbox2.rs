#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Checkbox 2 — Radio button groups
//!
//! Two independent groups of checkboxes acting as radio buttons via event
//! bubbling. Clicking one unchecks the rest in its group.

extern crate alloc;

use alloc::vec::Vec;
use oxivgl::{
    view::{register_event_on, View},
    enums::{EventCode, ObjState},
    event::Event,
    layout::{FlexAlign, FlexFlow},
    widgets::{Checkbox, Label, Obj, Screen, WidgetError},
};

struct WidgetCheckbox2 {
    group1: Obj<'static>,
    group2: Obj<'static>,
    _checkboxes: Vec<Checkbox<'static>>,
    _label: Label<'static>,
}

impl View for WidgetCheckbox2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.set_flex_flow(FlexFlow::Column);
        screen.set_flex_align(FlexAlign::Center, FlexAlign::Start, FlexAlign::Center);

        let label = Label::new(&screen)?;
        label.text("Selected: none");

        let mut checkboxes = Vec::new();

        // Group 1
        let group1 = Obj::new(&screen)?;
        group1.set_flex_flow(FlexFlow::Column);

        let titles1 = ["A1", "A2", "A3"];
        for (i, title) in titles1.iter().enumerate() {
            let cb = Checkbox::new(&group1)?;
            cb.text(title);
            cb.bubble_events();
            if i == 0 {
                cb.add_state(ObjState::CHECKED);
            }
            checkboxes.push(cb);
        }

        // Group 2
        let group2 = Obj::new(&screen)?;
        group2.set_flex_flow(FlexFlow::Column);

        let titles2 = ["B1", "B2", "B3"];
        for title in titles2.iter() {
            let cb = Checkbox::new(&group2)?;
            cb.text(title);
            cb.bubble_events();
            checkboxes.push(cb);
        }

        Ok(Self {
            group1,
            group2,
            _checkboxes: checkboxes,
            _label: label,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.group1.handle());
        register_event_on(self, self.group2.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::CLICKED {
            return;
        }
        let target = event.target_handle();
        let container = event.current_target_handle();

        // Ignore clicks on the container itself
        if target == container {
            return;
        }

        // Uncheck all children, check only the clicked one
        let cont_obj = if container == self.group1.handle() {
            &self.group1
        } else {
            &self.group2
        };

        let count = cont_obj.get_child_count();
        for i in 0..count as i32 {
            if let Some(child) = cont_obj.get_child(i) {
                child.remove_state(ObjState::CHECKED);
            }
        }

        // Check the clicked target
        let target_obj = event.target();
        target_obj.add_state(ObjState::CHECKED);
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetCheckbox2);
