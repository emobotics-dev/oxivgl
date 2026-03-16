#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 1 — Basic scrolling with save/restore position
//!
//! A panel with children placed outside its bounds triggers automatic scrolling.
//! Two buttons save and restore the scroll position.

use oxivgl::{
    view::View,
    widgets::{Align, Button, Event, EventCode, Label, Obj, Screen, WidgetError},
};

struct Scroll1 {
    panel: Obj<'static>,
    save_btn: Button<'static>,
    restore_btn: Button<'static>,
    _inner_btn: Button<'static>,
    _children: heapless::Vec<Obj<'static>, 3>,
    _labels: heapless::Vec<Label<'static>, 4>,
    _btn_labels: [Label<'static>; 2],
    saved_x: i32,
    saved_y: i32,
}

impl View for Scroll1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let panel = Obj::new(&screen)?;
        panel.size(200, 200).align(Align::Center, 44, 0);

        // Child at top-left
        let child0 = Obj::new(&panel)?;
        child0.pos(0, 0).size(70, 70);
        let label0 = Label::new(&child0)?;
        label0.text("Zero").center();

        // Child at right — overflows panel, so scrolling kicks in
        let child1 = Obj::new(&panel)?;
        child1.pos(160, 80).size(80, 80);
        let btn_inner = Button::new(&child1)?;
        btn_inner.size(100, 50);
        let label1 = Label::new(&btn_inner)?;
        label1.text("Right").center();

        // Child at bottom
        let child2 = Obj::new(&panel)?;
        child2.pos(40, 160).size(100, 70);
        let label2 = Label::new(&child2)?;
        label2.text("Bottom").center();

        // Save button
        let save_btn = Button::new(&screen)?;
        save_btn.align_to(&panel, Align::OutLeftMid, -10, -20);
        save_btn.bubble_events();
        let save_lbl = Label::new(&save_btn)?;
        save_lbl.text("Save").center();

        // Restore button
        let restore_btn = Button::new(&screen)?;
        restore_btn.align_to(&panel, Align::OutLeftMid, -10, 20);
        restore_btn.bubble_events();
        let restore_lbl = Label::new(&restore_btn)?;
        restore_lbl.text("Restore").center();

        let mut children = heapless::Vec::new();
        let _ = children.push(child0);
        let _ = children.push(child1);
        let _ = children.push(child2);

        let mut labels = heapless::Vec::new();
        let _ = labels.push(label0);
        let _ = labels.push(label1);
        let _ = labels.push(label2);

        Ok(Self {
            panel,
            save_btn,
            restore_btn,
            _inner_btn: btn_inner,
            _children: children,
            _labels: labels,
            _btn_labels: [save_lbl, restore_lbl],
            saved_x: 0,
            saved_y: 0,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.save_btn, EventCode::CLICKED) {
            self.saved_x = self.panel.get_scroll_x();
            self.saved_y = self.panel.get_scroll_y();
        } else if event.matches(&self.restore_btn, EventCode::CLICKED) {
            self.panel.scroll_to(self.saved_x, self.saved_y, true);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll1);
