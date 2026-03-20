#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 3 — List with floating add button
//!
//! A list with initial tracks and a floating "+" button in the bottom-right
//! corner. Clicking the button adds a new track and scrolls it into view.

use oxivgl::{
    enums::{EventCode, ObjFlag},
    event::Event,
    style::Selector,
    symbols,
    view::View,
    widgets::{Align, Button, List, Part, Screen, WidgetError, RADIUS_MAX},
};

struct Scroll3 {
    list: List<'static>,
    float_btn: Button<'static>,
    btn_cnt: u32,
}

impl View for Scroll3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let list = List::new(&screen)?;
        list.size(280, 220).center();

        // Add initial tracks
        for i in 1..=2u32 {
            let mut buf = heapless::String::<32>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Track {}", i));
            list.add_button(Some(&symbols::AUDIO), &buf);
        }

        // Floating add button
        let float_btn = Button::new(&list)?;
        float_btn.size(50, 50);
        float_btn.add_flag(ObjFlag::FLOATING);
        let pad = list.get_style_pad_right(Part::Main);
        float_btn.align(Align::BottomRight, 0, -pad);
        float_btn.bubble_events();
        float_btn.radius(RADIUS_MAX, Selector::DEFAULT);
        float_btn.style_bg_image_src_symbol(&symbols::PLUS, Selector::DEFAULT);

        Ok(Self {
            list,
            float_btn,
            btn_cnt: 2,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.float_btn, EventCode::CLICKED) {
            self.btn_cnt += 1;
            let mut buf = heapless::String::<32>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Track {}", self.btn_cnt));
            let new_btn = self.list.add_button(Some(&symbols::AUDIO), &buf);

            self.float_btn.move_foreground();
            new_btn.scroll_to_view(true);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll3);
