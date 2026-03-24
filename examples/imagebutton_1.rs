#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Imagebutton 1 — Image button with state switching
//!
//! Creates an image button and a label. Without image assets the button
//! is invisible, but the wrapper API is exercised: state is toggled via
//! `set_state` and a child label is added for visual feedback.

use oxivgl::{
    view::View,
    widgets::{Imagebutton, ImagebuttonState, Label, Screen, WidgetError},
};

struct Imagebutton1 {
    _btn: Imagebutton<'static>,
    _label: Label<'static>,
}

impl View for Imagebutton1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let btn = Imagebutton::new(&screen)?;
        btn.size(200, 50).center();
        btn.set_state(ImagebuttonState::Released);

        let label = Label::new(&btn)?;
        label.text("Imagebutton");
        label.center();

        Ok(Self {
            _btn: btn,
            _label: label,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Imagebutton1);
