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

extern crate alloc;
use alloc::vec::Vec;

use oxivgl::{
    enums::ObjFlag,
    view::{NavAction, View},
    widgets::{Align, Label, Obj, Switch, WidgetError},
};

/// Grid columns and rows for child placement.
const COLS: i32 = 5;
const CHILD_W: i32 = 60;
const CHILD_H: i32 = 40;
const GAP: i32 = 10;

/// Colors for child objects (cycled).
const COLORS: [u32; 5] = [0xe74c3c, 0x3498db, 0x2ecc71, 0xf39c12, 0x9b59b6];

#[derive(Default)]
struct Scroll9 {
    _panel: Option<Obj<'static>>,
    _switches: Option<Vec<Switch<'static>>>,
    _labels: Option<Vec<Label<'static>>>,
    _children: Option<Vec<Obj<'static>>>,
}

impl View for Scroll9 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // Scrollable panel
        let panel = Obj::new(container)?;
        panel.size(200, 120);
        panel.align(Align::TopMid, 0, 5);
        panel.bg_color(0xeeeeee);

        // 20 colored children in a grid that exceeds panel bounds
        let mut children = Vec::with_capacity(20);
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
            children.push(child);
        }

        // Switch labels
        let flag_names = ["Scrollable", "Chain", "Elastic", "Momentum"];
        let mut switches = Vec::with_capacity(4);
        let mut labels = Vec::with_capacity(4);

        for (i, name) in flag_names.iter().enumerate() {
            let lbl = Label::new(container)?;
            lbl.text(name);
            lbl.align(Align::TopLeft, 10, 135 + (i as i32) * 26);

            let sw = Switch::new(container)?;
            sw.align(Align::TopLeft, 100, 132 + (i as i32) * 26);
            // All switches initially checked
            sw.add_state(oxivgl::enums::ObjState::CHECKED);

            switches.push(sw);
            labels.push(lbl);
        }

                self._panel = Some(panel);
        self._switches = Some(switches);
        self._labels = Some(labels);
        self._children = Some(children);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main!(Scroll9::default());
