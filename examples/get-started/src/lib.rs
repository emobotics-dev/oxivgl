// SPDX-License-Identifier: MIT OR Apache-2.0
/// Run `setup_fn` once, then loop driving `lv_timer_handler` every 5ms.
/// For host (SDL2) only — does not use async.
pub fn run_host<F: FnOnce()>(setup_fn: F) {
    use lvgl_rust_sys::lv_timer_handler;

    setup_fn();

    loop {
        // SAFETY: lv_init() was called inside LvglDriver::init() in setup_fn.
        unsafe { lv_timer_handler() };
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
