#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Roller 2 — Styled rollers with alignments
//!
//! Three rollers: left-aligned on green gradient, center-aligned (default),
//! right-aligned. All share a custom selected-row style with larger font.

extern crate alloc;

use oxivgl::{
    fonts::MONTSERRAT_20,
    style::{GradDir, Selector, Style, StyleBuilder},
    view::View,
    widgets::{Align, Part, Roller, RollerMode, Screen, TextAlign, WidgetError},
};

struct WidgetRoller2 {
    _r1: Roller<'static>,
    _r2: Roller<'static>,
    _r3: Roller<'static>,
    _style_sel: Style,
    _style_green: Style,
}

const OPTS: &str = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10";

impl View for WidgetRoller2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Selected-row style: larger font, pink bg, red border
        let mut sb = StyleBuilder::new();
        sb.text_font(MONTSERRAT_20)
            .bg_color_hex(0xFF8888)
            .border_width(2)
            .border_color_hex(0xFF0000);
        let style_sel = sb.build();

        // Green vertical gradient style for left roller
        let mut sb_green = StyleBuilder::new();
        sb_green
            .bg_color_hex(0x00FF00)
            .bg_grad_color_hex(0xAAFFAA)
            .bg_grad_dir(GradDir::Ver);
        let style_green = sb_green.build();

        // Left roller: left-aligned text, green gradient, 2 visible rows
        let r1 = Roller::new(&screen)?;
        r1.set_options(OPTS, RollerMode::Normal);
        r1.set_visible_row_count(2);
        r1.width(100);
        r1.add_style(&style_sel, Selector::from(Part::Selected));
        r1.add_style(&style_green, Selector::DEFAULT);
        r1.text_align(TextAlign::Left);
        r1.align(Align::LeftMid, 10, 0);
        r1.set_selected(2, false);

        // Center roller: default alignment, 3 visible rows
        let r2 = Roller::new(&screen)?;
        r2.set_options(OPTS, RollerMode::Normal);
        r2.set_visible_row_count(3);
        r2.add_style(&style_sel, Selector::from(Part::Selected));
        r2.align(Align::Center, 0, 0);
        r2.set_selected(5, false);

        // Right roller: right-aligned text, 4 visible rows
        let r3 = Roller::new(&screen)?;
        r3.set_options(OPTS, RollerMode::Normal);
        r3.set_visible_row_count(4);
        r3.width(80);
        r3.add_style(&style_sel, Selector::from(Part::Selected));
        r3.text_align(TextAlign::Right);
        r3.align(Align::RightMid, -10, 0);
        r3.set_selected(8, false);

        Ok(Self {
            _r1: r1,
            _r2: r2,
            _r3: r3,
            _style_sel: style_sel,
            _style_green: style_green,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetRoller2);
