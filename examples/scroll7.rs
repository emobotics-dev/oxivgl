#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 7 — Dynamic Widget Loading
//!
//! A scrollable column that dynamically loads and unloads items as the user
//! scrolls. Labels on the left show the current range of loaded item numbers.
//! A checkbox toggles scrollbar visibility.

use oxivgl::{
    enums::{EventCode, ObjState},
    event::Event,
    layout::FlexFlow,
    style::lv_pct,
    view::{register_event_on, View},
    widgets::{Align, Checkbox, Label, Obj, Part, Screen, WidgetError, detach},
};

struct Scroll7 {
    screen: Screen,
    cont: Obj<'static>,
    high_label: Label<'static>,
    low_label: Label<'static>,
    checkbox: Checkbox<'static>,
    top_num: i32,
    bottom_num: i32,
    running: bool,
}

impl Scroll7 {
    fn load_item_back(&self, num: i32) {
        if let Ok(item) = Obj::new(&self.cont) {
            item.size(lv_pct(100), 40);
            if let Ok(lbl) = Label::new(&item) {
                let mut s = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(&mut s, format_args!("{}", num));
                lbl.text(&s);
                detach(lbl);
            }
            detach(item);
        }
    }

    fn load_item_front(&self, num: i32) {
        if let Ok(item) = Obj::new(&self.cont) {
            item.size(lv_pct(100), 40);
            if let Ok(lbl) = Label::new(&item) {
                let mut s = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(&mut s, format_args!("{}", num));
                lbl.text(&s);
                detach(lbl);
            }
            item.move_to_index(0);
            detach(item);
        }
    }

    fn update_scroll(&mut self) {
        if self.running {
            return;
        }
        self.running = true;

        // Load items near the bottom
        while self.bottom_num > -30 && self.cont.get_scroll_bottom() < 200 {
            self.bottom_num -= 1;
            self.load_item_back(self.bottom_num);
            self.cont.update_layout();
        }

        // Load items near the top, compensating scroll position
        while self.top_num < 30 && self.cont.get_scroll_top() < 200 {
            self.top_num += 1;
            let bot_before = self.cont.get_scroll_bottom();
            self.load_item_front(self.top_num);
            self.cont.update_layout();
            let bot_after = self.cont.get_scroll_bottom();
            self.cont.scroll_by(0, bot_before - bot_after, false);
        }

        // Delete far-bottom items
        while self.cont.get_scroll_bottom() > 600 {
            self.bottom_num += 1;
            self.cont.delete_child(-1);
            self.cont.update_layout();
        }

        // Delete far-top items, compensating scroll position
        while self.cont.get_scroll_top() > 600 {
            self.top_num -= 1;
            let bot_before = self.cont.get_scroll_bottom();
            self.cont.delete_child(0);
            self.cont.update_layout();
            let bot_after = self.cont.get_scroll_bottom();
            self.cont.scroll_by(0, bot_before - bot_after, false);
        }

        let mut s = heapless::String::<48>::new();
        let _ = core::fmt::Write::write_fmt(
            &mut s,
            format_args!("current largest\nloaded value:\n{}", self.top_num),
        );
        self.high_label.text(&s);

        let mut s = heapless::String::<48>::new();
        let _ = core::fmt::Write::write_fmt(
            &mut s,
            format_args!("current smallest\nloaded value:\n{}", self.bottom_num),
        );
        self.low_label.text(&s);

        self.running = false;
    }
}

impl View for Scroll7 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let high_label = Label::new(&screen)?;
        high_label.text("current largest\nloaded value:\n3");
        high_label.align(Align::TopLeft, 10, 10);

        let low_label = Label::new(&screen)?;
        low_label.text("current smallest\nloaded value:\n3");
        low_label.align(Align::BottomLeft, 10, -10);

        let checkbox = Checkbox::new(&screen)?;
        checkbox.text("show\nscrollbar");
        checkbox.align(Align::LeftMid, 10, 0);
        checkbox.bubble_events();

        let cont = Obj::new(&screen)?;
        cont.size(160, 220);
        cont.align(Align::RightMid, -10, 0);
        cont.set_flex_flow(FlexFlow::Column);
        cont.style_opa(0, Part::Scrollbar);

        // Load initial item
        let item = Obj::new(&cont)?;
        item.size(lv_pct(100), 40);
        let lbl = Label::new(&item)?;
        lbl.text("3");
        detach(lbl);
        detach(item);

        let mut view = Self {
            screen,
            cont,
            high_label,
            low_label,
            checkbox,
            top_num: 3,
            bottom_num: 3,
            running: false,
        };
        view.cont.update_layout();
        view.update_scroll();
        Ok(view)
    }

    fn register_events(&mut self) {
        register_event_on(self, self.cont.handle());
        register_event_on(self, self.screen.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == EventCode::SCROLL
            && event.current_target_handle() == self.cont.handle()
        {
            self.update_scroll();
        } else if event.matches(&self.checkbox, EventCode::VALUE_CHANGED) {
            let checked = self.checkbox.has_state(ObjState::CHECKED);
            let opa = if checked { 255u8 } else { 0u8 };
            self.cont.style_opa(opa, Part::Scrollbar);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll7);
