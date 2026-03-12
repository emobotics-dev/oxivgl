// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 1: Hello World label.
//!
//! Rust port of lv_example_get_started_1():
//!   - Dark blue screen background
//!   - Centered white "Hello world" label

use oxivgl::{
    lvgl::LvglDriver,
    lvgl_buffers::{lvgl_disp_init, LvglBuffers},
    widgets::{Align, Label, Screen},
};
use static_cell::StaticCell;

const W: i32 = 320;
const H: i32 = 240;

static BUFS: StaticCell<LvglBuffers<{ W as usize * 40 * 2 }>> = StaticCell::new();

fn setup() {
    let _driver = LvglDriver::init(W, H);
    let bufs = BUFS.init(LvglBuffers::new());
    // SAFETY: lv_init() called above; bufs is 'static via StaticCell.
    unsafe { lvgl_disp_init(W, H, bufs) };

    let screen = Screen::active().expect("no active screen");
    screen.bg_color(0x003a57).bg_opa(255);
    screen.text_color(0xffffff);

    let label = Label::new(&screen).expect("label create failed");
    label.text("Hello world\0").expect("label text failed");
    label.align(Align::Center, 0, 0);
}

fn main() {
    env_logger::init();
    oxivgl_examples_get_started::run_host(setup);
}
