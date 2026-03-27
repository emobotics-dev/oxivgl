#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gridnav 3 — Nested grid navigations
//!
//! A main container (ROLLOVER | SCROLL_FIRST) holds two buttons and two
//! sub-containers. `cont_sub1` contains a long scrollable text. `cont_sub2`
//! has its own ROLLOVER gridnav with two buttons; pressing ENTER focuses it,
//! ESC moves group focus to the next member.
//!
//! Only the containers are in the group; buttons are removed so gridnav
//! controls focus inside each container.

use oxivgl::{
    enums::{EventCode, Key, ObjState},
    event::Event,
    gridnav::{GridnavCtrl, gridnav_add},
    group::{Group, group_remove_obj},
    layout::FlexFlow,
    style::{LV_SIZE_CONTENT, Palette, lv_pct, palette_lighten},
    view::{View, register_event_on},
    widgets::{AsLvHandle, Button, Label, Obj, Screen, WidgetError},
};

struct Gridnav3 {
    _screen: Screen,
    _group: Group,
    // containers kept alive
    _cont_main: Obj<'static>,
    _cont_sub1: Obj<'static>,
    cont_sub2: Obj<'static>,
    // child widgets kept alive
    _btn1: Button<'static>,
    _btn1_lbl: Label<'static>,
    _btn2: Button<'static>,
    _btn2_lbl: Label<'static>,
    _sub1_lbl: Label<'static>,
    _sub2_hint_lbl: Label<'static>,
    _btn3: Button<'static>,
    _btn3_lbl: Label<'static>,
    _btn4: Button<'static>,
    _btn4_lbl: Label<'static>,
}

impl View for Gridnav3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // ── Main container ────────────────────────────────────────────────
        let cont_main = Obj::new(&screen)?;
        cont_main
            .set_flex_flow(FlexFlow::RowWrap)
            .style_bg_color(palette_lighten(Palette::Blue, 5), ObjState::FOCUSED)
            .size(lv_pct(80), LV_SIZE_CONTENT);

        gridnav_add(&cont_main, GridnavCtrl::ROLLOVER | GridnavCtrl::SCROLL_FIRST);

        let group = Group::new()?;
        group.set_default();
        group.add_obj(&cont_main);
        group.assign_to_keyboard_indevs();

        // ── Two plain buttons in main (not in group) ──────────────────────
        let btn1 = Button::new(&cont_main)?;
        group_remove_obj(&btn1);
        let btn1_lbl = Label::new(&btn1)?;
        btn1_lbl.text("Button 1");

        let btn2 = Button::new(&cont_main)?;
        group_remove_obj(&btn2);
        let btn2_lbl = Label::new(&btn2)?;
        btn2_lbl.text("Button 2");

        // ── Sub-container 1: long scrollable text ────────────────────────
        let cont_sub1 = Obj::new(&cont_main)?;
        cont_sub1
            .style_bg_color(palette_lighten(Palette::Red, 5), ObjState::FOCUSED)
            .size(lv_pct(100), 100);

        let sub1_lbl = Label::new(&cont_sub1)?;
        sub1_lbl.width(lv_pct(100));
        sub1_lbl.text(
            "I'm a very long text which is makes my container scrollable. \
             As LV_GRIDNAV_FLAG_SCROLL_FIRST is enabled arrow will scroll me first \
             and a new objects will be focused only when an edge is reached with the scrolling.\n\n\
             This is only some placeholder text to be sure the parent will be scrollable. \n\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!\n\
             Hello world!",
        );

        // ── Sub-container 2: nested gridnav, ENTER/ESC focus control ─────
        let cont_sub2 = Obj::new(&cont_main)?;
        cont_sub2
            .set_flex_flow(FlexFlow::RowWrap)
            .style_bg_color(palette_lighten(Palette::Red, 5), ObjState::FOCUSED)
            .size(lv_pct(100), LV_SIZE_CONTENT);

        gridnav_add(&cont_sub2, GridnavCtrl::ROLLOVER);
        group.add_obj(&cont_sub2);

        let sub2_hint_lbl = Label::new(&cont_sub2)?;
        sub2_hint_lbl.text("Use ENTER/ESC to focus/defocus this container");
        sub2_hint_lbl.width(lv_pct(100));

        let btn3 = Button::new(&cont_sub2)?;
        group_remove_obj(&btn3);
        let btn3_lbl = Label::new(&btn3)?;
        btn3_lbl.text("Button 3");

        let btn4 = Button::new(&cont_sub2)?;
        group_remove_obj(&btn4);
        let btn4_lbl = Label::new(&btn4)?;
        btn4_lbl.text("Button 4");

        Ok(Self {
            _screen: screen,
            _group: group,
            _cont_main: cont_main,
            _cont_sub1: cont_sub1,
            cont_sub2,
            _btn1: btn1,
            _btn1_lbl: btn1_lbl,
            _btn2: btn2,
            _btn2_lbl: btn2_lbl,
            _sub1_lbl: sub1_lbl,
            _sub2_hint_lbl: sub2_hint_lbl,
            _btn3: btn3,
            _btn3_lbl: btn3_lbl,
            _btn4: btn4,
            _btn4_lbl: btn4_lbl,
        })
    }

    fn register_events(&mut self) {
        // Receive KEY events from cont_sub2 (no bubble needed — direct registration).
        register_event_on(self, self.cont_sub2.lv_handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::KEY {
            return;
        }
        // Only handle events from cont_sub2.
        if event.target_handle() != self.cont_sub2.lv_handle() {
            return;
        }
        match event.key() {
            Some(Key::ENTER) => {
                self._group.focus_obj(&self.cont_sub2);
            }
            Some(Key::ESC) => {
                self._group.focus_next();
            }
            _ => {}
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

// ── Platform entry points ──────────────────────────────────────────────────

#[cfg(target_arch = "xtensa")]
oxivgl_examples_common::fire27_main!(Gridnav3);

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
            .title(c"oxivgl — gridnav 3")
            .mouse(true)
            .keyboard(true)
            .build()
    };
    let mut _view = Gridnav3::create().expect("view create failed");
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
