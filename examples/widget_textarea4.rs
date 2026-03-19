#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Textarea 4 — Cursor styles
//!
//! Three one-line textareas, each with a unique cursor style:
//! simple red, underline blue, and block orange/yellow gradient.

use oxivgl::{
    enums::ObjState,
    style::{palette_main, BorderSide, GradDir, Palette, Style, StyleBuilder},
    view::View,
    widgets::{Align, Part, Screen, Textarea, WidgetError},
};

struct WidgetTextarea4 {
    _ta1: Textarea<'static>,
    _ta2: Textarea<'static>,
    _ta3: Textarea<'static>,
    _style_simple: Style,
    _style_underline: Style,
    _style_block: Style,
}

impl View for WidgetTextarea4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Thin left bar cursor (simple) — red border
        let style_simple = {
            let mut sb = StyleBuilder::new();
            sb.border_color(palette_main(Palette::Red));
            sb.build()
        };

        // Underline cursor — blue, bottom border only
        let style_underline = {
            let mut sb = StyleBuilder::new();
            sb.bg_opa(0)
                .border_color(palette_main(Palette::Blue))
                .border_side(BorderSide::BOTTOM)
                .pad_left(1)
                .pad_right(1)
                .border_width(3);
            sb.build()
        };

        // Full block cursor — orange/yellow gradient
        let style_block = {
            let mut sb = StyleBuilder::new();
            sb.bg_opa(255)
                .bg_color(palette_main(Palette::Orange))
                .bg_grad_color(palette_main(Palette::Yellow))
                .bg_grad_dir(GradDir::Ver)
                .border_color(palette_main(Palette::Red))
                .border_side(BorderSide::FULL)
                .border_width(1)
                .radius(4)
                .text_color_hex(0xFFFFFF)
                .pad_all(1);
            sb.build()
        };

        let cursor_focused = Part::Cursor | ObjState::FOCUSED;

        let ta1 = Textarea::new(&screen)?;
        ta1.set_text("This is a simple red cursor");
        ta1.width(280).align(Align::TopMid, 0, 10);
        ta1.set_one_line(true);
        ta1.add_state(ObjState::FOCUSED);
        ta1.add_style(&style_simple, cursor_focused);
        ta1.set_cursor_pos(0);

        let ta2 = Textarea::new(&screen)?;
        ta2.set_text("This is an underline blue cursor");
        ta2.width(280).align(Align::TopMid, 0, 110);
        ta2.set_one_line(true);
        ta2.add_state(ObjState::FOCUSED);
        ta2.add_style(&style_underline, cursor_focused);
        ta2.set_cursor_pos(0);

        let ta3 = Textarea::new(&screen)?;
        ta3.set_text("This is a complex block cursor");
        ta3.width(280).align(Align::TopMid, 0, 60);
        ta3.set_one_line(true);
        ta3.add_state(ObjState::FOCUSED);
        ta3.add_style(&style_block, cursor_focused);
        ta3.set_cursor_pos(0);

        Ok(Self {
            _ta1: ta1,
            _ta2: ta2,
            _ta3: ta3,
            _style_simple: style_simple,
            _style_underline: style_underline,
            _style_block: style_block,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetTextarea4);
