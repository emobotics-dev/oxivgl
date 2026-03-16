#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Checkbox 1 — Simple checkboxes
//!
//! Four checkboxes in a column: unchecked, checked, disabled, checked+disabled.

use oxivgl::{
    view::View,
    enums::ObjState,
    layout::{FlexAlign, FlexFlow},
    widgets::{Checkbox, Screen, WidgetError},
};

struct WidgetCheckbox1 {
    _cb1: Checkbox<'static>,
    _cb2: Checkbox<'static>,
    _cb3: Checkbox<'static>,
    _cb4: Checkbox<'static>,
}

impl View for WidgetCheckbox1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.set_flex_flow(FlexFlow::Column);
        screen.set_flex_align(FlexAlign::Center, FlexAlign::Start, FlexAlign::Center);

        let cb1 = Checkbox::new(&screen)?;
        cb1.text("Apple");

        let cb2 = Checkbox::new(&screen)?;
        cb2.text("Banana");
        cb2.add_state(ObjState::CHECKED);

        let cb3 = Checkbox::new(&screen)?;
        cb3.text("Lemon");
        cb3.add_state(ObjState::DISABLED);

        let cb4 = Checkbox::new(&screen)?;
        cb4.text("Melon");
        cb4.add_state(ObjState::CHECKED | ObjState::DISABLED);

        Ok(Self {
            _cb1: cb1,
            _cb2: cb2,
            _cb3: cb3,
            _cb4: cb4,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetCheckbox1);
