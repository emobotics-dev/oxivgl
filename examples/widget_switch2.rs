#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Switch 2 — Horizontal and vertical switches
//!
//! Two switches: horizontal (default) and vertical, with the vertical one
//! pre-checked.

use oxivgl::{
    view::View,
    enums::ObjState,
    layout::{FlexAlign, FlexFlow},
    widgets::{Screen, Switch, SwitchOrientation, WidgetError},
};

struct WidgetSwitch2 {
    _sw1: Switch<'static>,
    _sw2: Switch<'static>,
}

impl View for WidgetSwitch2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.set_flex_flow(FlexFlow::Column);
        screen.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);

        let sw1 = Switch::new(&screen)?;

        let sw2 = Switch::new(&screen)?;
        sw2.set_orientation(SwitchOrientation::Vertical);
        sw2.size(50, 120);
        sw2.add_state(ObjState::CHECKED);

        Ok(Self {
            _sw1: sw1,
            _sw2: sw2,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetSwitch2);
