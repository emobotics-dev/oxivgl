#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 14 — Extending the current theme

use oxivgl::{
    view::View,
    widgets::{
        palette_darken, palette_main, Align, Button, Label, Palette, Screen, StyleBuilder, Theme,
        WidgetError,
    },
};

struct Style14 {
    _theme: Theme,
    _label2: Label<'static>,
    _btn2: Button<'static>,
    _label1: Label<'static>,
    _btn1: Button<'static>,
}

impl View for Style14 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Button created before the theme extension — uses the default theme.
        let btn1 = Button::new(&screen)?;
        btn1.align(Align::TopMid, 0, 20);
        let label1 = Label::new(&btn1)?;
        label1.text("Original theme").center();

        // Install the theme extension: all buttons created from here on are green.
        let mut style = StyleBuilder::new();
        style
            .bg_color(palette_main(Palette::Green))
            .border_color(palette_darken(Palette::Green, 3))
            .border_width(3);
        let theme = Theme::extend_current(style.build())?;

        // Button created after the theme extension — receives the green style.
        let btn2 = Button::new(&screen)?;
        btn2.align(Align::BottomMid, 0, -20);
        let label2 = Label::new(&btn2)?;
        label2.text("New theme").center();

        Ok(Self {
            _theme: theme,
            _label2: label2,
            _btn2: btn2,
            _label1: label1,
            _btn1: btn1,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style14);
