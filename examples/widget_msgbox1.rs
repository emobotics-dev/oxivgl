#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Msgbox 1 — Standalone message box
//!
//! A modal message box with a title, body text, and a close button.
//! Clicking the button dismisses the message box.
//!
//! Ownership note: when `parent = None`, LVGL creates a full-screen backdrop
//! and becomes the owner of the msgbox object. The Rust handle is detached
//! with `mem::forget` to prevent a double-free on drop.

use oxivgl::{
    view::View,
    widgets::{Msgbox, Obj, Screen, WidgetError},
};

struct WidgetMsgbox1 {
    _screen: Screen,
}

impl View for WidgetMsgbox1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mbox = Msgbox::new(None::<&Obj<'_>>)?;
        mbox.add_title("Hello");
        mbox.add_text("This is a message box.\nClick Close to dismiss.");
        mbox.add_close_button();
        // LVGL owns the msgbox (its modal backdrop is the parent). Forget the
        // Rust handle so lv_obj_delete is not called when the function returns.
        core::mem::forget(mbox);

        Ok(Self { _screen: screen })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetMsgbox1);
