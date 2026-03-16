#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Switch 1 — Toggle switches with states
//!
//! Four switches in a column: default, checked, disabled, checked+disabled.

use oxivgl::{
    view::View,
    widgets::{FlexAlign, FlexFlow, ObjState, Screen, Switch, WidgetError},
};

struct WidgetSwitch1 {
    _sw1: Switch<'static>,
    _sw2: Switch<'static>,
    _sw3: Switch<'static>,
    _sw4: Switch<'static>,
}

impl View for WidgetSwitch1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.set_flex_flow(FlexFlow::Column);
        screen.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);

        let sw1 = Switch::new(&screen)?;

        let sw2 = Switch::new(&screen)?;
        sw2.add_state(ObjState::CHECKED);

        let sw3 = Switch::new(&screen)?;
        sw3.add_state(ObjState::DISABLED);

        let sw4 = Switch::new(&screen)?;
        sw4.add_state(ObjState::CHECKED | ObjState::DISABLED);

        Ok(Self {
            _sw1: sw1,
            _sw2: sw2,
            _sw3: sw3,
            _sw4: sw4,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetSwitch1);
