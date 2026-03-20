#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Menu 2 — Root back button with message box
//!
//! Like menu1 but with the root back button enabled. Clicking the back button
//! at root level shows a message box.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::View,
    widgets::{Child, Label, Menu, Msgbox, Obj, Screen, WidgetError},
};

struct WidgetMenu2 {
    menu: Menu<'static>,
    _labels: [Child<Label<'static>>; 4],
}

impl View for WidgetMenu2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let menu = Menu::new(&screen)?;
        menu.set_mode_root_back_button(true);
        menu.bubble_events();
        menu.size(320, 240).center();

        // Sub-page
        let sub_page = menu.page_create(None);
        let cont = Menu::cont_create(&sub_page);
        let l0 = Label::new(&cont)?;
        l0.text("Hello, I am hiding here");

        // Main page
        let main_page = menu.page_create(None);

        let cont1 = Menu::cont_create(&main_page);
        let l1 = Label::new(&cont1)?;
        l1.text("Item 1");

        let cont2 = Menu::cont_create(&main_page);
        let l2 = Label::new(&cont2)?;
        l2.text("Item 2");

        let cont3 = Menu::cont_create(&main_page);
        let l3 = Label::new(&cont3)?;
        l3.text("Item 3 (Click me!)");
        menu.set_load_page_event(&cont3, &sub_page);

        menu.set_page(&main_page);

        Ok(Self {
            menu,
            _labels: [
                Child::new(l0),
                Child::new(l1),
                Child::new(l2),
                Child::new(l3),
            ],
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == EventCode::CLICKED {
            if self.menu.back_button_is_root(&event.target()) {
                let mbox = Msgbox::new(None::<&Obj<'_>>);
                if let Ok(mbox) = mbox {
                    mbox.add_title("Hello");
                    mbox.add_text("Root back btn click.");
                    mbox.add_close_button();
                    core::mem::forget(mbox);
                }
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetMenu2);
