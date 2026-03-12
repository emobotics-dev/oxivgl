// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 2: Button with label.
//!
//! Rust port of lv_example_get_started_2().

use oxivgl::{
    lvgl::LvglDriver,
    widgets::{Button, Label, Screen},
};

const W: i32 = 320;
const H: i32 = 240;

fn main() {
    env_logger::init();
    let _driver = LvglDriver::init(W, H);

    let screen = Screen::active().expect("no active screen");

    let _btn = Button::new(&screen).expect("button create failed");
    _btn.pos(10, 10).size(120, 50);

    let _label = Label::new(&_btn).expect("label create failed");
    _label.text("Button\0").expect("label text failed").center();

    oxivgl_examples_get_started::run_host_loop();
}
