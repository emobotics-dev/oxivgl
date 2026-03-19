#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Menu 3 — Custom back button text and titled pages
//!
//! Full-screen menu with a "Back" label on the header back button.
//! Three sub-pages with titles, each reachable from the main page.

use oxivgl::{
    view::View,
    widgets::{Child, Label, Menu, Screen, WidgetError},
};

struct WidgetMenu3 {
    _menu: Menu<'static>,
    _back_label: Child<Label<'static>>,
}

impl View for WidgetMenu3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let menu = Menu::new(&screen)?;
        menu.size(320, 240).center();

        // Modify the header back button — add "Back" text
        let back_btn = menu.get_main_header_back_button();
        let back_label = Label::new(&back_btn)?;
        back_label.text("Back");

        // Sub-pages with titles
        let sub1 = menu.page_create(Some("Page 1"));
        let cont = Menu::cont_create(&sub1);
        let lbl = Label::new(&cont)?;
        lbl.text("Hello, I am hiding here");
        core::mem::forget(lbl);

        let sub2 = menu.page_create(Some("Page 2"));
        let cont = Menu::cont_create(&sub2);
        let lbl = Label::new(&cont)?;
        lbl.text("Hello, I am hiding here");
        core::mem::forget(lbl);

        let sub3 = menu.page_create(Some("Page 3"));
        let cont = Menu::cont_create(&sub3);
        let lbl = Label::new(&cont)?;
        lbl.text("Hello, I am hiding here");
        core::mem::forget(lbl);

        // Main page (untitled)
        let main_page = menu.page_create(None);

        let cont1 = Menu::cont_create(&main_page);
        let lbl = Label::new(&cont1)?;
        lbl.text("Item 1 (Click me!)");
        menu.set_load_page_event(&cont1, &sub1);
        core::mem::forget(lbl);

        let cont2 = Menu::cont_create(&main_page);
        let lbl = Label::new(&cont2)?;
        lbl.text("Item 2 (Click me!)");
        menu.set_load_page_event(&cont2, &sub2);
        core::mem::forget(lbl);

        let cont3 = Menu::cont_create(&main_page);
        let lbl = Label::new(&cont3)?;
        lbl.text("Item 3 (Click me!)");
        menu.set_load_page_event(&cont3, &sub3);
        core::mem::forget(lbl);

        menu.set_page(&main_page);

        Ok(Self {
            _menu: menu,
            _back_label: Child::new(back_label),
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetMenu3);
