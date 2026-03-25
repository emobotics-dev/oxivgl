#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 9 — Scroll Property Toggles
//!
//! A scrollable panel with colored child objects, plus 4 switches that
//! toggle scroll flags: SCROLLABLE, SCROLL_CHAIN, SCROLL_ELASTIC,
//! SCROLL_MOMENTUM.

use oxivgl::{
    enums::ObjFlag,
    view::View,
    widgets::{Align, Label, Obj, Screen, Switch, WidgetError},
};

/// Grid columns and rows for child placement.
const COLS: i32 = 5;
const CHILD_W: i32 = 60;
const CHILD_H: i32 = 40;
const GAP: i32 = 10;

/// Colors for child objects (cycled).
const COLORS: [u32; 5] = [0xe74c3c, 0x3498db, 0x2ecc71, 0xf39c12, 0x9b59b6];

struct Scroll9 {
    _panel: Obj<'static>,
    _switches: [Switch<'static>; 4],
    _labels: [Label<'static>; 4],
    _children: [Obj<'static>; 20],
}

impl View for Scroll9 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Scrollable panel
        let panel = Obj::new(&screen)?;
        panel.size(200, 120);
        panel.align(Align::TopMid, 0, 5);
        panel.bg_color(0xeeeeee);

        // 20 colored children in a grid that exceeds panel bounds
        let mut children: [core::mem::MaybeUninit<Obj<'static>>; 20] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };

        for i in 0..20usize {
            let child = Obj::new(&panel)?;
            child.size(CHILD_W, CHILD_H);
            let col = (i as i32) % COLS;
            let row = (i as i32) / COLS;
            let x = 10 + col * (CHILD_W + GAP);
            let y = 10 + row * (CHILD_H + GAP);
            child.pos(x, y);
            child.bg_color(COLORS[i % COLORS.len()]);
            child.bg_opa(255);
            child.remove_flag(ObjFlag::SCROLLABLE);
            children[i] = core::mem::MaybeUninit::new(child);
        }

        let children = unsafe { core::mem::transmute::<_, [Obj<'static>; 20]>(children) };

        // Switch labels
        let flag_names = ["Scrollable", "Chain", "Elastic", "Momentum"];
        let mut switches: [core::mem::MaybeUninit<Switch<'static>>; 4] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        let mut labels: [core::mem::MaybeUninit<Label<'static>>; 4] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };

        for (i, name) in flag_names.iter().enumerate() {
            let lbl = Label::new(&screen)?;
            lbl.text(name);
            lbl.align(Align::TopLeft, 10, 135 + (i as i32) * 26);

            let sw = Switch::new(&screen)?;
            sw.align(Align::TopLeft, 100, 132 + (i as i32) * 26);
            // All switches initially checked
            sw.add_state(oxivgl::enums::ObjState::CHECKED);

            switches[i] = core::mem::MaybeUninit::new(sw);
            labels[i] = core::mem::MaybeUninit::new(lbl);
        }

        let switches =
            unsafe { core::mem::transmute::<_, [Switch<'static>; 4]>(switches) };
        let labels =
            unsafe { core::mem::transmute::<_, [Label<'static>; 4]>(labels) };

        Ok(Self {
            _panel: panel,
            _switches: switches,
            _labels: labels,
            _children: children,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll9);
