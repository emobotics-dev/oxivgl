#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tabview 1 — Simple 3-tab view with default top bar.
//!
//! Tab 1 has long content that becomes scrollable. Tab 3 is scrolled into
//! view on creation via `scroll_to_view_recursive`.

use oxivgl::{
    view::View,
    widgets::{Label, LabelLongMode, Screen, Tabview, WidgetError},
};

struct Tabview1 {
    _tv: Tabview<'static>,
}

impl View for Tabview1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let tv = Tabview::new(&screen)?;

        let tab1 = tv.add_tab("Tab 1");
        let tab2 = tv.add_tab("Tab 2");
        let tab3 = tv.add_tab("Tab 3");

        let label1 = Label::new(&*tab1)?;
        label1
            .set_long_mode(LabelLongMode::Wrap)
            .text("This the first tab\n\nIf the content\nof a tab\nbecomes too\nlonger\nthan the\ncontainer\nthen it\nautomatically\nbecomes\nscrollable.\n\n\n\nCan you see it?");

        let label2 = Label::new(&*tab2)?;
        label2.text("Second tab");

        let label3 = Label::new(&*tab3)?;
        label3.text("Third tab");

        // Scroll the last label into view (as in the LVGL example).
        label3.scroll_to_view_recursive(true);

        Ok(Self { _tv: tv })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Tabview1);
