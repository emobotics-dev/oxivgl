// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 1: Hello World label.
//!
//! Rust port of lv_example_get_started_1():
//!   - Dark blue screen background
//!   - Centered white "Hello world" label

use oxivgl::{
    lvgl::LvglDriver,
    widgets::{Align, Label, Screen},
};

const W: i32 = 320;
const H: i32 = 240;

fn main() {
    env_logger::init();
    let _driver = LvglDriver::init(W, H);

    let screen = Screen::active().expect("no active screen");
    screen.bg_color(0x003a57).bg_opa(255);
    screen.text_color(0xffffff);

    let _label = Label::new(&screen).expect("label create failed");
    _label.text("Hello world\0").expect("label text failed");
    _label.align(Align::Center, 0, 0);

    // Widgets stay alive for the entire loop since run_host_loop never returns.
    oxivgl_examples_get_started::run_host_loop();
}
