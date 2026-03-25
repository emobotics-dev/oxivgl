#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 5 — Right-to-left scrolling
//!
//! A container with RTL base direction containing a wide label with
//! Persian text. The text scrolls from right to left.

use oxivgl::{
    fonts::DEJAVU_16_PERSIAN_HEBREW,
    style::Selector,
    view::View,
    widgets::{BaseDir, Label, Obj, Screen, WidgetError},
};

struct Scroll5 {
    _screen: Screen,
    _cont: Obj<'static>,
    _label: Label<'static>,
}

impl View for Scroll5 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let cont = Obj::new(&screen)?;
        cont.style_base_dir(BaseDir::Rtl, Selector::DEFAULT);
        cont.size(200, 100);
        cont.center();

        let label = Label::new(&cont)?;
        label.text("به وسیله یک ماشین نوشته شده است");
        label.width(400);
        label.style_text_font(DEJAVU_16_PERSIAN_HEBREW, Selector::DEFAULT);

        Ok(Self { _screen: screen, _cont: cont, _label: label })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll5);
