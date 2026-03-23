#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 6 — Fixed-width glyph override
//!
//! Clones Montserrat 20 and overrides its glyph descriptor callback to force
//! a fixed advance width, producing a monospaced appearance. Two labels show
//! the same text: proportional (top) vs fixed-width (bottom).

use oxivgl::{
    fonts::{FixedWidthFont, MONTSERRAT_20},
    view::View,
    widgets::{Label, Screen, WidgetError},
};

/// Static storage for the cloned fixed-width font. LVGL stores a pointer to
/// the font, so it must live for `'static`.
static MONO_FONT: FixedWidthFont = FixedWidthFont::new();

struct WidgetLabel6 {
    _label1: Label<'static>,
    _label2: Label<'static>,
}

impl View for WidgetLabel6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Label with normal proportional font
        let label1 = Label::new(&screen)?;
        label1.text_font(MONTSERRAT_20);
        label1.text("0123.Wabc");

        // Label with fixed-width glyph override
        let mono = MONO_FONT.init(MONTSERRAT_20, 20);
        let label2 = Label::new(&screen)?;
        label2.y(30);
        label2.text_font(mono);
        label2.text("0123.Wabc");

        Ok(Self {
            _label1: label1,
            _label2: label2,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetLabel6);
