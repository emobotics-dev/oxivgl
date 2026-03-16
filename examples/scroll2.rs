#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 2 — Scroll snap with center alignment
//!
//! A horizontal row of buttons snaps to center. Panel 3 is marked non-snappable.
//! A switch toggles "scroll one" mode.

use oxivgl::{
    view::View,
    widgets::{
        Align, Button, Event, EventCode, FlexFlow, Label, Obj, ObjFlag, Screen, ScrollSnap, Switch,
        WidgetError,
    },
};

struct Scroll2 {
    panel: Obj<'static>,
    sw: Switch<'static>,
    _btns: heapless::Vec<Button<'static>, 10>,
    _labels: heapless::Vec<Label<'static>, 12>,
}

impl View for Scroll2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let panel = Obj::new(&screen)?;
        panel
            .size(280, 120)
            .set_scroll_snap_x(ScrollSnap::Center)
            .set_flex_flow(FlexFlow::Row)
            .align(Align::Center, 0, 20);

        let mut btns = heapless::Vec::<Button<'static>, 10>::new();
        let mut labels = heapless::Vec::<Label<'static>, 12>::new();

        for i in 0u32..10 {
            let btn = Button::new(&panel)?;
            btn.size(150, oxivgl::widgets::lv_pct(100));

            let label = Label::new(&btn)?;
            if i == 3 {
                let mut buf = heapless::String::<20>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Panel {}\nno snap", i));
                label.text(&buf).center();
                btn.remove_flag(ObjFlag::SNAPPABLE);
            } else {
                let mut buf = heapless::String::<10>::new();
                let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Panel {}", i));
                label.text(&buf).center();
            }

            let _ = btns.push(btn);
            let _ = labels.push(label);
        }

        panel.update_snap(true);

        // Switch to toggle "one scroll" mode
        let sw = Switch::new(&screen)?;
        sw.align(Align::TopRight, -20, 10);
        sw.bubble_events();

        let sw_label = Label::new(&screen)?;
        sw_label
            .text("One scroll")
            .align_to(&sw, Align::OutBottomMid, 0, 5);
        let _ = labels.push(sw_label);

        Ok(Self {
            panel,
            sw,
            _btns: btns,
            _labels: labels,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.sw, EventCode::VALUE_CHANGED) {
            if self.sw.has_state(oxivgl::widgets::ObjState::CHECKED) {
                self.panel.add_flag(ObjFlag::SCROLL_ONE);
            } else {
                self.panel.remove_flag(ObjFlag::SCROLL_ONE);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll2);
