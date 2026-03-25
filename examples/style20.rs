#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 20 — Modal Overlay Dimming
//!
//! A screen with two buttons ("BG Dim" and "OPA Dim"). A full-screen dark
//! overlay is shown on top. For the screenshot the overlay is visible.

use oxivgl::{
    style::Selector,
    view::View,
    widgets::{Align, Button, Label, Obj, Screen, WidgetError},
};

struct Style20 {
    _btn1: Button<'static>,
    _btn2: Button<'static>,
    _lbl1: Label<'static>,
    _lbl2: Label<'static>,
    _overlay: Obj<'static>,
    _overlay_label: Label<'static>,
}

impl View for Style20 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.bg_color(0xffffff);
        screen.bg_opa(255);

        // Two buttons
        let btn1 = Button::new(&screen)?;
        btn1.size(120, 50);
        btn1.align(Align::Center, -70, 0);
        let lbl1 = Label::new(&btn1)?;
        lbl1.text("BG Dim");
        lbl1.center();

        let btn2 = Button::new(&screen)?;
        btn2.size(120, 50);
        btn2.align(Align::Center, 70, 0);
        let lbl2 = Label::new(&btn2)?;
        lbl2.text("OPA Dim");
        lbl2.center();

        // Full-screen overlay (visible for screenshot)
        let overlay = Obj::new(&screen)?;
        overlay.size(320, 240);
        overlay.align(Align::Center, 0, 0);
        overlay.bg_color(0x000000);
        overlay.bg_opa(180);
        overlay.border_width(0);
        overlay.radius(0, Selector::DEFAULT);

        let overlay_label = Label::new(&overlay)?;
        overlay_label.text("Modal overlay\nclick to dismiss");
        overlay_label.center();
        overlay_label.text_color(0xffffff);

        Ok(Self {
            _btn1: btn1,
            _btn2: btn2,
            _lbl1: lbl1,
            _lbl2: lbl2,
            _overlay: overlay,
            _overlay_label: overlay_label,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style20);
