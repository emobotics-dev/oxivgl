#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 1 — Grid navigation with keyboard focus
//!
//! Two containers side-by-side. Left container has 10 checkable buttons in a
//! row-wrap flex layout (no rollover). Right container has a textarea,
//! checkbox, and two switches arranged manually (rollover enabled).
//! Arrow keys move focus between children; Tab switches between containers.

use oxivgl::{
    enums::{ObjFlag, ObjState},
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    layout::FlexFlow,
    style::{LV_SIZE_CONTENT, Palette, lv_pct, palette_lighten},
    view::View,
    widgets::{Align, Button, Checkbox, Label, Screen, Switch, Textarea, WidgetError},
};

struct Gridnav1 {
    _screen: Screen,
    _group: Group,
    _label1: Label<'static>,
    _buttons: heapless::Vec<Button<'static>, 10>,
    _btn_labels: heapless::Vec<Label<'static>, 10>,
    _label2: Label<'static>,
    _ta: Textarea<'static>,
    _cb: Checkbox<'static>,
    _sw1: Switch<'static>,
    _sw2: Switch<'static>,
    // containers kept alive
    _cont1: oxivgl::widgets::Obj<'static>,
    _cont2: oxivgl::widgets::Obj<'static>,
}

impl View for Gridnav1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // ── Container 1: buttons, no rollover ──────────────────────────────
        let cont1 = oxivgl::widgets::Obj::new(&screen)?;
        cont1
            .set_flex_flow(FlexFlow::RowWrap)
            .style_bg_color(
                palette_lighten(Palette::Blue, 5),
                ObjState::FOCUSED,
            )
            .size(lv_pct(50), lv_pct(100));

        gridnav_add(&cont1, GridnavCtrl::None);

        let label1 = Label::new(&cont1)?;
        label1.text("No rollover");

        let mut buttons: heapless::Vec<Button<'static>, 10> = heapless::Vec::new();
        let mut btn_labels: heapless::Vec<Label<'static>, 10> = heapless::Vec::new();
        for i in 0u32..10 {
            let btn = Button::new(&cont1)?;
            btn.size(70, LV_SIZE_CONTENT);
            btn.add_flag(ObjFlag::CHECKABLE);
            group_remove_obj(&btn);

            let lbl = Label::new(&btn)?;
            // Format button index as text; heapless label buffer fits small numbers.
            lbl.text(match i {
                0 => "0", 1 => "1", 2 => "2", 3 => "3", 4 => "4",
                5 => "5", 6 => "6", 7 => "7", 8 => "8", _ => "9",
            });
            lbl.center();

            // heapless::Vec::push returns Err if full; capacity is 10 == loop count.
            let _ = buttons.push(btn);
            let _ = btn_labels.push(lbl);
        }

        // ── Container 2: textarea/checkbox/switches, rollover ──────────────
        let cont2 = oxivgl::widgets::Obj::new(&screen)?;
        cont2
            .style_bg_color(
                palette_lighten(Palette::Blue, 5),
                ObjState::FOCUSED,
            )
            .size(lv_pct(50), lv_pct(100))
            .align(Align::RightMid, 0, 0);

        gridnav_add(&cont2, GridnavCtrl::Rollover);

        let label2 = Label::new(&cont2)?;
        label2.width(lv_pct(100));
        label2.text("Rollover\nUse tab to focus the other container");

        let ta = Textarea::new(&cont2)?;
        ta.size(lv_pct(100), 80).pos(0, 80);
        group_remove_obj(&ta);

        let cb = Checkbox::new(&cont2)?;
        cb.pos(0, 170);
        group_remove_obj(&cb);

        let sw1 = Switch::new(&cont2)?;
        sw1.pos(0, 200);
        group_remove_obj(&sw1);

        let sw2 = Switch::new(&cont2)?;
        sw2.pos(lv_pct(50), 200);
        group_remove_obj(&sw2);

        // ── Group: only containers ─────────────────────────────────────────
        let group = Group::new()?;
        group.set_default();
        group.add_obj(&cont1);
        group.add_obj(&cont2);
        group.assign_to_keyboard_indevs();

        Ok(Self {
            _screen: screen,
            _group: group,
            _label1: label1,
            _buttons: buttons,
            _btn_labels: btn_labels,
            _label2: label2,
            _ta: ta,
            _cb: cb,
            _sw1: sw1,
            _sw2: sw2,
            _cont1: cont1,
            _cont2: cont2,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

// ── Platform entry points ──────────────────────────────────────────────────

#[cfg(target_arch = "xtensa")]
oxivgl_examples_common::fire27_main!(Gridnav1);

#[cfg(not(target_arch = "xtensa"))]
fn main() {
    use oxivgl_examples_common::host::{H, W, capture, pump};
    use oxivgl::driver::LvglDriver;
    use oxivgl::view::{View, register_view_events};

    oxivgl_examples_common::env_logger::init();
    let screenshot_only = std::env::var("SCREENSHOT_ONLY").as_deref() == Ok("1");
    let driver = if screenshot_only {
        LvglDriver::init(W, H)
    } else {
        LvglDriver::sdl(W, H).title(c"oxivgl — gridnav").mouse(true).keyboard(true).build()
    };
    let mut _view = Gridnav1::create().expect("view create failed");
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
