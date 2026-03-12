// SPDX-License-Identifier: MIT OR Apache-2.0
/// Drive the LVGL timer loop. Call after creating all widgets. Never returns.
pub fn run_host_loop() -> ! {
    use lvgl_rust_sys::lv_timer_handler;
    loop {
        // SAFETY: lv_init() was called inside LvglDriver::init() before entering main's loop.
        unsafe { lv_timer_handler() };
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
