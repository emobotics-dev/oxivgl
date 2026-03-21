#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget List 2 — Reorderable list
//!
//! Left panel: 15 items that can be selected (checked state). Right panel:
//! Top/Up/Center/Down/Bottom/Shuffle buttons to reorder the selected item.
//! Shuffle uses a deterministic permutation (no `lv_rand` in safe API).

use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    layout::FlexFlow,
    style::lv_pct,
    symbols,
    view::{register_event_on, View},
    widgets::{
        Align, AsLvHandle, Button, Child, List, Obj, Screen, WidgetError,
    },
};

struct WidgetList2 {
    list1: List<'static>,
    _list2: List<'static>,
    _item_btns: heapless::Vec<Button<'static>, 15>,
    btn_top: Child<Button<'static>>,
    btn_up: Child<Button<'static>>,
    btn_center: Child<Button<'static>>,
    btn_down: Child<Button<'static>>,
    btn_bottom: Child<Button<'static>>,
    btn_shuffle: Child<Button<'static>>,
    current: Option<*mut lvgl_rust_sys::lv_obj_t>,
}

impl View for WidgetList2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Left list: items
        let list1 = List::new(&screen)?;
        list1.size(lv_pct(60), lv_pct(100));
        list1.style_pad_row(5, oxivgl::style::Selector::DEFAULT);

        let mut item_btns = heapless::Vec::<Button<'static>, 15>::new();
        for i in 0..15 {
            let btn = Button::new(&list1)?;
            btn.width(lv_pct(50));
            btn.add_flag(ObjFlag::CHECKABLE);
            btn.bubble_events();
            let lab = oxivgl::widgets::Label::new(&btn)?;
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Item {}", i));
            lab.text(&buf).center();
            let _ = item_btns.push(btn);
        }

        // Select first button
        let first_ptr = item_btns.first().map(|b| b.lv_handle());
        if let Some(btn) = item_btns.first() {
            btn.add_state(ObjState::CHECKED);
        }

        // Right list: control buttons
        let list2 = List::new(&screen)?;
        list2.size(lv_pct(40), lv_pct(100));
        list2.align(Align::TopRight, 0, 0);
        list2.set_flex_flow(FlexFlow::Column);

        let btn_top = list2.add_button(None, "Top");
        btn_top.bubble_events();
        let btn_up = list2.add_button(Some(&symbols::UP), "Up");
        btn_up.bubble_events();
        let btn_center = list2.add_button(Some(&symbols::LEFT), "Center");
        btn_center.bubble_events();
        let btn_down = list2.add_button(Some(&symbols::DOWN), "Down");
        btn_down.bubble_events();
        let btn_bottom = list2.add_button(None, "Bottom");
        btn_bottom.bubble_events();
        let btn_shuffle = list2.add_button(Some(&symbols::SHUFFLE), "Shuffle");
        btn_shuffle.bubble_events();

        Ok(Self {
            list1,
            _list2: list2,
            _item_btns: item_btns,
            btn_top,
            btn_up,
            btn_center,
            btn_down,
            btn_bottom,
            btn_shuffle,
            current: first_ptr,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.list1.lv_handle());
        register_event_on(self, self._list2.lv_handle());
    }

    fn on_event(&mut self, event: &Event) {
        let code = event.code();
        let target = event.target_handle();

        // Item click in list1 — toggle selection
        if code == EventCode::CLICKED && event.current_target_handle() == self.list1.lv_handle() {
            if self.current == Some(target) {
                self.current = None;
            } else {
                self.current = Some(target);
            }
            // Update checked state on all item children
            let count = self.list1.get_child_count();
            for i in 0..count as i32 {
                if let Some(child) = self.list1.get_child(i) {
                    if Some(child.lv_handle()) == self.current {
                        child.add_state(ObjState::CHECKED);
                    } else {
                        child.remove_state(ObjState::CHECKED);
                    }
                }
            }
            return;
        }

        let Some(cur) = self.current else { return };
        let cur_obj = Obj::from_raw_non_owning(cur);

        // Control buttons check code for CLICKED or LONG_PRESSED_REPEAT
        if code != EventCode::CLICKED && code != EventCode::LONG_PRESSED_REPEAT {
            return;
        }

        if target == self.btn_top.lv_handle() && code == EventCode::CLICKED {
            cur_obj.move_background();
            cur_obj.scroll_to_view(true);
        } else if target == self.btn_up.lv_handle() {
            let idx = cur_obj.get_index();
            if idx > 0 {
                cur_obj.move_to_index(idx - 1);
                cur_obj.scroll_to_view(true);
            }
        } else if target == self.btn_center.lv_handle() {
            let count = self.list1.get_child_count();
            cur_obj.move_to_index(count as i32 / 2);
            cur_obj.scroll_to_view(true);
        } else if target == self.btn_down.lv_handle() {
            let idx = cur_obj.get_index();
            cur_obj.move_to_index(idx + 1);
            cur_obj.scroll_to_view(true);
        } else if target == self.btn_bottom.lv_handle() && code == EventCode::CLICKED {
            cur_obj.move_foreground();
            cur_obj.scroll_to_view(true);
        } else if target == self.btn_shuffle.lv_handle() {
            // Deterministic shuffle: rotate each child by 7 positions
            let count = self.list1.get_child_count();
            if count > 1 {
                let idx = cur_obj.get_index();
                let new_idx = (idx + 7) % count as i32;
                cur_obj.move_to_index(new_idx);
                cur_obj.scroll_to_view(true);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetList2);
