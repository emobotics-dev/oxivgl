#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 2 — Grid navigation on two side-by-side lists
//!
//! Two scrollable lists: one with 15 "File N" entries (no rollover) and one
//! with 15 "Folder N" entries (rollover). Arrow keys navigate within a list;
//! Tab switches focus between the two list containers.

use oxivgl::{
    enums::ObjState,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    style::{Palette, lv_pct, palette_lighten},
    symbols,
    view::View,
    widgets::{Obj, Align, List, WidgetError},
};

#[derive(Default)]
struct Gridnav2 {
    _group: Option<Group>,
    _list1: Option<List<'static>>,
    _list2: Option<List<'static>>,
}

impl View for Gridnav2 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // ── List 1: File items, no rollover ───────────────────────────────
        let list1 = List::new(container)?;
        list1
            .size(lv_pct(45), lv_pct(80))
            .align(Align::LeftMid, 5, 0)
            .style_bg_color(palette_lighten(Palette::Blue, 5), ObjState::FOCUSED);

        gridnav_add(&list1, GridnavCtrl::NONE);

        for i in 1u32..=15 {
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("File {}", i));
            let item = list1.add_button(Some(&symbols::FILE), &buf);
            item.bg_opa_selector(0, ObjState::DEFAULT);
            group_remove_obj(&item);
        }

        // ── List 2: Folder items, rollover ────────────────────────────────
        let list2 = List::new(container)?;
        list2
            .size(lv_pct(45), lv_pct(80))
            .align(Align::RightMid, -5, 0)
            .style_bg_color(palette_lighten(Palette::Blue, 5), ObjState::FOCUSED);

        gridnav_add(&list2, GridnavCtrl::ROLLOVER);

        for i in 1u32..=15 {
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Folder {}", i));
            let item = list2.add_button(Some(&symbols::DIRECTORY), &buf);
            item.bg_opa_selector(0, ObjState::DEFAULT);
            group_remove_obj(&item);
        }

        // ── Group: only list containers ───────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&list1);
        group.add_obj(&list2);
        group.assign_to_keyboard_indevs();

        self._group = Some(group);
        self._list1 = Some(list1);
        self._list2 = Some(list2);
        Ok(())
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

// ── Platform entry points ──────────────────────────────────────────────────

#[cfg(target_arch = "xtensa")]
oxivgl_examples_common::fire27_main!(Gridnav2);

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
            .title(c"oxivgl — gridnav 2")
            .mouse(true)
            .keyboard(true)
            .build()
    };
    let mut _view = Gridnav2::default();
    // SAFETY: lv_screen_active() is valid after LvglDriver::init/sdl.
    let screen_handle = unsafe { oxivgl_sys::lv_screen_active() };
    assert!(!screen_handle.is_null(), "no active screen");
    let screen_container = oxivgl::widgets::Obj::from_raw(screen_handle);
    _view.create(&screen_container).expect("view create failed");
    core::mem::forget(screen_container);
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
