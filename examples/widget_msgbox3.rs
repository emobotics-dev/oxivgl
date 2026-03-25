#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Msgbox 3 — Message box with backdrop blur
//!
//! Background content (labels) with a modal message box on the top layer.
//! The layer_top backdrop uses `style_blur_radius` and `style_blur_backdrop`
//! to blur the content behind the dialog, plus `bg_opa` for dimming.
//!
//! Note: blur requires a GPU-accelerated backend to produce visible results.
//! On the SDL host the blur may be a no-op; the backdrop dimming (bg_opa)
//! is always visible.

use oxivgl::{
    style::Selector,
    view::View,
    widgets::{Align, Label, Msgbox, Screen, WidgetError},
};

struct WidgetMsgbox3 {
    _screen: Screen,
    _label1: Label<'static>,
    _label2: Label<'static>,
    _label3: Label<'static>,
}

impl View for WidgetMsgbox3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Background content — visible through the blurred/dimmed backdrop
        let label1 = Label::new(&screen)?;
        label1.text("Background text line 1");
        label1.align(Align::TopMid, 0, 30);

        let label2 = Label::new(&screen)?;
        label2.text("Background text line 2");
        label2.align(Align::Center, 0, 0);

        let label3 = Label::new(&screen)?;
        label3.text("Background text line 3");
        label3.align(Align::BottomMid, 0, -30);

        // Style layer_top for blur + dimming (applied to the msgbox backdrop)
        let layer = Screen::layer_top();
        layer.style_blur_radius(20, Selector::DEFAULT);
        layer.style_blur_backdrop(true, Selector::DEFAULT);
        // Prevent layer_top from being deleted on drop — LVGL owns it
        core::mem::forget(layer);

        // Modal message box (parent = None → LVGL creates a full-screen
        // backdrop child on layer_top and centers the msgbox on it)
        let mbox = Msgbox::new(None::<&oxivgl::widgets::Obj<'_>>)?;
        mbox.add_title("Notice");
        mbox.add_text("This dialog has a blurred backdrop.\nClose to dismiss.");
        mbox.add_close_button();
        mbox.add_footer_button("OK");
        // LVGL owns the msgbox — forget the Rust handle
        core::mem::forget(mbox);

        Ok(Self {
            _screen: screen,
            _label1: label1,
            _label2: label2,
            _label3: label3,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetMsgbox3);
