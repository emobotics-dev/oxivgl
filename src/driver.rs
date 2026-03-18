// SPDX-License-Identifier: MIT OR Apache-2.0
// Formerly `lvgl` — renamed to avoid confusion with the lvgl_rust_sys crate.
use core::ffi::c_char;

use lvgl_rust_sys::*;

/// Zero-sized init token. Proves `lv_init()` was called.
#[derive(Debug)]
pub struct LvglDriver;

/// Shared LVGL init sequence: lv_init + log + tick callbacks.
/// Called by all public constructors.
fn init_common() {
    // SAFETY: lv_init is called exactly once (LvglDriver is non-Clone,
    // and all constructors go through this path).
    unsafe {
        lv_init();
        lv_log_register_print_cb(Some(lvgl_log_print));
        lv_tick_set_cb(Some(get_tick_ms));
    }
}

impl LvglDriver {
    /// Initialise LVGL with a headless software display (for tests,
    /// screenshots, and embedded targets).
    pub fn init(w: i32, h: i32) -> Self {
        init_common();
        #[cfg(not(target_os = "none"))]
        // SAFETY: lv_init() was called in init_common() above.
        unsafe {
            init_host_display(w, h)
        };
        let _ = (w, h); // params unused on embedded target
        Self
    }

    /// Drive LVGL timers. Returns recommended delay in ms until next call.
    ///
    /// Safe to call because `LvglDriver` existence proves `lv_init()` was
    /// called. Caller is responsible for the single-task constraint: no
    /// other code may call LVGL concurrently while this is running.
    pub fn timer_handler(&self) -> u32 {
        // SAFETY: LvglDriver is the init token — lv_init() was called.
        unsafe { lv_timer_handler() }
    }

    /// Initialise LVGL with an SDL2 window display (interactive host demos).
    #[cfg(not(target_os = "none"))]
    pub fn init_sdl(w: i32, h: i32) -> Self {
        init_common();
        // SAFETY: lv_init() was called in init_common() above.
        unsafe { init_sdl_display(w, h) };
        Self
    }
}

// ── Host-only display setup
// ───────────────────────────────────────────────────

#[cfg(not(target_os = "none"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(not(target_os = "none"))]
unsafe extern "C" fn flush_cb(drv: *mut lv_display_t, _area: *const lv_area_t, _px_map: *mut u8) {
    // SAFETY: drv is a valid display pointer provided by LVGL.
    unsafe { lv_display_flush_ready(drv) };
}

/// Create a headless software display (for tests and screenshots).
#[cfg(not(target_os = "none"))]
unsafe fn init_host_display(w: i32, h: i32) {
    // Full-height buffer: rotated/scaled objects need sub-layers that can
    // span the entire screen height. A small band buffer (e.g. 40 lines)
    // causes SIGSEGV when the transformed bounding box exceeds the band.
    // This is heap-allocated so it doesn't affect embedded memory.
    let buf_size = w as usize * h as usize * 2; // RGB565
    // Intentionally leak: LVGL owns this buffer for the process lifetime.
    let cbuf = Box::into_raw(vec![0u8; buf_size].into_boxed_slice()) as *mut std::ffi::c_void;
    // SAFETY: lv_init() has been called by LvglDriver::init() before this function.
    let disp = unsafe { lv_display_create(w, h) };
    assert!(!disp.is_null(), "lv_display_create returned NULL");
    // SAFETY: disp is a valid non-null display pointer returned by
    // lv_display_create.
    unsafe { lv_display_set_color_format(disp, lv_color_format_t_LV_COLOR_FORMAT_RGB565) };
    // SAFETY: cbuf is heap-allocated with buf_size bytes and lives for the program
    // lifetime.
    unsafe {
        lv_display_set_buffers(
            disp,
            cbuf,
            std::ptr::null_mut(),
            buf_size as u32,
            lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
        )
    };
    // SAFETY: flush_cb is a valid extern "C" fn with the correct LVGL flush
    // callback signature.
    unsafe { lv_display_set_flush_cb(disp, Some(flush_cb)) };
}

/// Create an SDL2 window display (for interactive host demos).
/// Falls back to headless if `SDL_VIDEODRIVER=dummy`.
#[cfg(not(target_os = "none"))]
unsafe fn init_sdl_display(w: i32, h: i32) {
    // SAFETY: lv_init() has been called; lv_sdl_window_create initialises
    // SDL2 and creates a visible window with its own render loop.
    let disp = unsafe { lv_sdl_window_create(w, h) };
    assert!(!disp.is_null(), "lv_sdl_window_create returned NULL");
}

// ── Log callback
// ──────────────────────────────────────────────────────────────

/// LVGL log callback for host targets. Prints to stderr, trimming the trailing
/// newline LVGL adds.
#[cfg(not(target_os = "none"))]
pub unsafe extern "C" fn lvgl_log_print(_level: i8, c_str: *const c_char) {
    if !c_str.is_null() {
        let text = unsafe { std::ffi::CStr::from_ptr(c_str) };
        eprintln!("LVGL: {}", text.to_str().unwrap_or("<invalid utf8>").trim());
    }
}

/// LVGL log callback for embedded targets. Forwards log messages via defmt/log
/// debug macro.
#[cfg(target_os = "none")]
#[cfg_attr(feature = "esp-hal", esp_hal::ram)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lvgl_log_print(_level: i8, c_str: *const c_char) {
    if c_str.is_null() {
        return;
    }
    let text = unsafe { core::ffi::CStr::from_ptr(c_str) };
    debug!("LVGL: {}", text.to_str().unwrap_or("").trim());
}

// ── Tick source
// ───────────────────────────────────────────────────────────────

/// LVGL tick source for host targets. Returns milliseconds since UNIX epoch
/// (wraps at u32::MAX ≈ 49 days).
#[cfg(not(target_os = "none"))]
#[unsafe(no_mangle)]
pub extern "C" fn get_tick_ms() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32
}

/// LVGL tick source for embedded targets. Returns embassy-time milliseconds
/// since boot.
#[cfg(target_os = "none")]
#[cfg_attr(feature = "esp-hal", esp_hal::ram)]
#[unsafe(no_mangle)]
pub extern "C" fn get_tick_ms() -> u32 {
    embassy_time::Instant::now().as_millis() as u32
}
