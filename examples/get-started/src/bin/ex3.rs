// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 3: Custom button styles.
//!
//! Rust port of lv_example_get_started_3().

use oxivgl::{
    lvgl::LvglDriver,
    widgets::{
        AsLvHandle, Button, ColorFilter, GradDir, Label, Palette, Screen, Style,
        darken_filter_cb, palette_lighten, palette_main,
    },
};

const W: i32 = 320;
const H: i32 = 240;

const LV_OPA_COVER: u8 = 255;
const LV_OPA_20: u8 = 51;

fn main() {
    env_logger::init();
    let _driver = LvglDriver::init(W, H);

    let _color_filter = ColorFilter::new(darken_filter_cb);

    let mut style_btn = Style::new();
    style_btn
        .radius(10)
        .bg_opa(LV_OPA_COVER)
        .bg_color(palette_lighten(Palette::Grey, 3))
        .bg_grad_color(palette_main(Palette::Grey))
        .bg_grad_dir(GradDir::Ver)
        .border_color_hex(0x000000)
        .border_opa(LV_OPA_20)
        .border_width(2)
        .text_color_hex(0x000000);

    let mut style_pressed = Style::new();
    style_pressed.color_filter(&_color_filter, LV_OPA_20);

    let mut style_red = Style::new();
    style_red
        .bg_color(palette_main(Palette::Red))
        .bg_grad_color(palette_lighten(Palette::Red, 3));

    let screen = Screen::active().expect("no active screen");

    let _btn1 = Button::new(&screen).expect("btn1 create failed");
    _btn1.remove_style_all().pos(10, 10).size(120, 50);
    _btn1.add_style(&style_btn, 0);
    _btn1.add_style(&style_pressed, 0x0020); // LV_STATE_PRESSED

    let _lbl1 = Label::new(&_btn1).expect("lbl1 create failed");
    _lbl1.text("Button\0").expect("lbl1 text failed").center();

    let _btn2 = Button::new(&screen).expect("btn2 create failed");
    _btn2.remove_style_all().pos(10, 80).size(120, 50);
    _btn2.add_style(&style_btn, 0);
    _btn2.add_style(&style_red, 0);
    _btn2.add_style(&style_pressed, 0x0020); // LV_STATE_PRESSED
    unsafe {
        lvgl_rust_sys::lv_obj_set_style_radius(_btn2.lv_handle(), 0x7fff, 0);
    }

    let _lbl2 = Label::new(&_btn2).expect("lbl2 create failed");
    _lbl2.text("Button 2\0").expect("lbl2 text failed").center();

    // All widgets and styles stay alive for the entire loop.
    oxivgl_examples_get_started::run_host_loop();
}
