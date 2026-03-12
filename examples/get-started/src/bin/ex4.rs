// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 4: Slider with live value label.
//!
//! Rust port of lv_example_get_started_4().

use core::ffi::c_void;

use lvgl_rust_sys::*;
use oxivgl::{
    lvgl::LvglDriver,
    widgets::{Align, AsLvHandle, Label, Screen, Slider},
};

const W: i32 = 320;
const H: i32 = 240;

/// Event callback: updates the label text above the slider.
/// `user_data` must be the `*mut lv_obj_t` of the label.
unsafe extern "C" fn slider_event_cb(e: *mut lv_event_t) {
    use core::fmt::Write as _;
    // SAFETY: e is a valid lv_event_t pointer passed by LVGL; user_data is the label ptr.
    let (slider, label, val) = unsafe {
        let slider = lv_event_get_target_obj(e);
        let label = lv_event_get_user_data(e) as *mut lv_obj_t;
        let val = lv_slider_get_value(slider);
        (slider, label, val)
    };
    let mut s = heapless::String::<12>::new();
    let _ = core::fmt::write(&mut s, format_args!("{}\0", val));
    unsafe {
        lv_label_set_text(label, s.as_ptr() as *const core::ffi::c_char);
        lv_obj_align_to(label, slider, Align::OutTopMid as u32, 0, -15);
    }
}

fn main() {
    env_logger::init();
    let _driver = LvglDriver::init(W, H);

    let screen = Screen::active().expect("no active screen");

    let _slider = Slider::new(&screen).expect("slider create failed");
    _slider.width(200).center();

    let _label = Label::new(&screen).expect("label create failed");
    _label.text("0\0").expect("label text failed");
    _label.align_to(&_slider, Align::OutTopMid, 0, -15);

    // Register event with label ptr as user_data.
    _slider.on_event(
        slider_event_cb,
        lv_event_code_t_LV_EVENT_VALUE_CHANGED,
        _label.lv_handle() as *mut c_void,
    );

    oxivgl_examples_get_started::run_host_loop();
}
