#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 4 — List with section separators and click logging
//!
//! A single list with 5 sections of 4 "File N" buttons each, separated by
//! text headers. Clicking any item logs its label text. A standalone button
//! sits to the right. Arrow keys navigate the list; Tab moves to the button.

use oxivgl::{
    enums::EventCode,
    event::Event,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    style::lv_pct,
    symbols,
    view::{View, register_event_on},
    widgets::{Obj, Align, Button, Label, List, WidgetError},
};

#[derive(Default)]
struct Gridnav4 {
    _group: Option<Group>,
    list: Option<List<'static>>,
    _btn: Option<Button<'static>>,
    _btn_label: Option<Label<'static>>,
}

impl View for Gridnav4 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // ── List with section separators ──────────────────────────────────
        let list = List::new(container)?;
        list.size(lv_pct(60), lv_pct(90))
            .align(Align::LeftMid, 10, 0);

        gridnav_add(&list, GridnavCtrl::ROLLOVER);

        for i in 0u32..20 {
            if i % 5 == 0 {
                let mut sec = heapless::String::<16>::new();
                let _ = core::fmt::Write::write_fmt(
                    &mut sec,
                    format_args!("Section {}", i / 5 + 1),
                );
                list.add_text(&sec);
            }

            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("File {}", i + 1));
            let item = list.add_button(Some(&symbols::FILE), &buf);
            item.bubble_events();
            group_remove_obj(&item);
        }

        // ── Standalone button to the right ────────────────────────────────
        let btn = Button::new(container)?;
        btn.align(Align::RightMid, -10, 0);

        let btn_label = Label::new(&btn)?;
        btn_label.text("Button").center();

        // ── Group: only the list ──────────────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&list);
        group.add_obj(&btn);
        group.assign_to_keyboard_indevs();

        self._group = Some(group);
        self.list = Some(list);
        self._btn = Some(btn);
        self._btn_label = Some(btn_label);
        Ok(())
    }

    fn register_events(&mut self) {
        if let Some(ref list) = self.list {
            register_event_on(self, list.handle());
        }
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::CLICKED {
            return;
        }
        let target = event.target();
        if let Some(ref list) = self.list {
            if target.handle() == list.handle() {
                return;
            }
            if let Some(text) = list.get_button_text(&target) {
                oxivgl_examples_common::log::info!("Clicked: {}", text);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

// ── Platform entry points ──────────────────────────────────────────────────

#[cfg(target_arch = "xtensa")]
oxivgl_examples_common::fire27_main!(Gridnav4);

#[cfg(not(target_arch = "xtensa"))]
fn main() {
    use oxivgl::driver::LvglDriver;
    use oxivgl::view::{View, register_view_events};
    use oxivgl_examples_common::host::{H, W, capture, pump};

    oxivgl_examples_common::env_logger::init();
    let screenshot_only = std::env::var("SCREENSHOT_ONLY").as_deref() == Ok("1");
    let driver = if screenshot_only {
        LvglDriver::init(W, H)
    } else {
        LvglDriver::sdl(W, H)
            .title(c"oxivgl — gridnav 4")
            .mouse(true)
            .keyboard(true)
            .build()
    };

    let mut _view = Gridnav4::default();
    let screen_handle = unsafe { oxivgl_sys::lv_screen_active() };
    assert!(!screen_handle.is_null(), "no active screen");
    let container = oxivgl::widgets::Obj::from_raw(screen_handle);
    _view.create(&container).expect("view create failed");
    core::mem::forget(container);
    register_view_events(&mut _view);

    let src = file!();
    let name = std::path::Path::new(src)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("screenshot");
    let dir = format!("{}/examples/doc/screenshots", env!("CARGO_MANIFEST_DIR"));

    _view.update().expect("update failed");
    pump(&driver, 10);
    capture(&driver, name, &dir);

    if screenshot_only {
        std::process::exit(0);
    }

    loop {
        _view.update().unwrap_or_else(|e| eprintln!("update: {e:?}"));
        for _ in 0..4 {
            driver.timer_handler();
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
    }
}
