#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 6 — Curved Scroll
//!
//! A circular clipped flex column where items are displaced horizontally based
//! on their distance from the container centre, creating a curved/perspective
//! effect. Items far from centre are also made more transparent.

use oxivgl::{
    enums::{EventCode, ScrollDir, ScrollSnap, ScrollbarMode},
    event::Event,
    layout::FlexFlow,
    math::map,
    style::{lv_pct, Selector},
    view::{register_event_on, View},
    widgets::{Button, Label, Obj, Screen, WidgetError, detach},
};

struct Scroll6 {
    cont: Obj<'static>,
}

impl View for Scroll6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let cont = Obj::new(&screen)?;
        cont.size(200, 200).center();
        cont.set_flex_flow(FlexFlow::Column);
        cont.style_clip_corner(true, Selector::DEFAULT);
        cont.radius(0x7FFF, Selector::DEFAULT);
        cont.set_scroll_dir(ScrollDir::VER);
        cont.set_scroll_snap_y(ScrollSnap::Center);
        cont.set_scrollbar_mode(ScrollbarMode::Off);

        for i in 0u32..20 {
            let btn = Button::new(&cont)?;
            btn.width(lv_pct(100));
            let lbl = Label::new(&btn)?;
            let mut s = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut s, format_args!("Button {}", i));
            lbl.text(&s);
            detach(lbl);
            detach(btn);
        }

        // Scroll first child to center (layout must be done first)
        cont.update_layout();
        if let Some(first) = cont.get_child(0) {
            first.scroll_to_view(false);
        }

        Ok(Self { cont })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.cont.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::SCROLL {
            return;
        }
        if event.current_target_handle() != self.cont.handle() {
            return;
        }
        let coords = self.cont.get_coords();
        let cont_y_center = coords.y1 + coords.height() / 2;
        let r = self.cont.get_height() * 7 / 10;

        for i in 0..self.cont.get_child_count() as i32 {
            let Some(child) = self.cont.get_child(i) else {
                continue;
            };
            let child_coords = child.get_coords();
            let child_y_center = child_coords.y1 + child_coords.height() / 2;
            let diff_y = (child_y_center - cont_y_center).abs();
            let x = if diff_y >= r {
                r
            } else {
                let x_sqr = (r * r - diff_y * diff_y) as u32;
                r - x_sqr.isqrt() as i32
            };
            child.style_translate_x(x, Selector::DEFAULT);
            let opa = map(x, 0, r, 0, 255) as u8;
            child.style_opa(255 - opa, Selector::DEFAULT);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll6);
