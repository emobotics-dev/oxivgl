// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host-side helpers for oxivgl examples: SDL2 viewer loop, LVGL timer
//! pumping, and PNG screenshot capture.

use std::path::PathBuf;

use oxivgl_sys::*;
use oxivgl::driver::LvglDriver;

pub const W: i32 = 320;
pub const H: i32 = 240;

/// Drive the LVGL timer loop. Call after creating all widgets. Never returns.
pub fn run_host_loop(driver: &LvglDriver) -> ! {
    loop {
        driver.timer_handler();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Pump the LVGL timer `n` times (5 ms each).
pub fn pump(driver: &LvglDriver, n: u32) {
    for _ in 0..n {
        driver.timer_handler();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Load a fresh blank screen, discarding the current one.
pub fn fresh_screen() {
    // SAFETY: LVGL is initialised; null parent creates a screen-level object.
    unsafe { lv_screen_load(lv_obj_create(core::ptr::null_mut())) };
}

/// Capture current screen as a PNG file under `<dir>/<name>.png`.
pub fn capture(driver: &LvglDriver, name: &str, dir: &str) {
    use oxivgl::snapshot::Snapshot;

    let snap = Snapshot::take(driver).expect("lv_snapshot_take returned NULL");
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.png"));
    snap.write_png(&path).expect("PNG write failed");
    println!("Screenshot: {}", path.display());
}

/// Generate a host `main` function for the given [`oxivgl::view::View`].
///
/// The generated main:
/// 1. Initialises `env_logger` and the LVGL SDL2 driver.
/// 2. Creates the view from the given expression.
/// 3. If `SCREENSHOT_ONLY=1`, captures a screenshot and exits.
/// 4. Otherwise runs the interactive host loop.
#[macro_export]
macro_rules! host_main {
    ($view_expr:expr) => {
        fn main() {
            use $crate::host::{H, W, capture, pump};
            use $crate::oxivgl::driver::LvglDriver;
            use $crate::oxivgl::view::View;
            $crate::env_logger::init();
            let screenshot_only =
                std::env::var("SCREENSHOT_ONLY").as_deref() == Ok("1");
            let driver = if screenshot_only {
                LvglDriver::init(W, H)
            } else {
                LvglDriver::sdl(W, H).title(c"oxivgl").mouse(true).keyboard(true).build()
            };

            let mut _view = $view_expr;

            // Wrap the active screen as a non-owning container.
            let screen_handle = unsafe { oxivgl_sys::lv_screen_active() };
            assert!(!screen_handle.is_null(), "no active screen");
            let container = $crate::oxivgl::widgets::Obj::from_raw(screen_handle);
            _view.create(&container).expect("view create failed");
            // Don't delete the LVGL screen when container drops.
            core::mem::forget(container);

            $crate::oxivgl::view::register_view_events(&mut _view);

            // Derive screenshot name from source file path.
            let src = file!();
            let name = std::path::Path::new(src)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("screenshot");

            let dir = format!(
                "{}/examples/doc/screenshots",
                env!("CARGO_MANIFEST_DIR")
            );

            // Always capture a screenshot (call update once first for animated views).
            _view.update().expect("update failed");
            pump(&driver, 10);
            capture(&driver, name, &dir);

            if screenshot_only {
                // Skip Rust destructors — LVGL's internal state (timers,
                // display refresh) can race with lv_obj_delete during
                // Drop, causing intermittent SIGSEGV on exit.
                std::process::exit(0);
            }

            // Interactive loop: drive update() at ~30 fps (every 4 × 8 ms).
            loop {
                _view.update().unwrap_or_else(|e| eprintln!("update: {e:?}"));
                for _ in 0..4 {
                    driver.timer_handler();
                    std::thread::sleep(std::time::Duration::from_millis(8));
                }
            }
        }
    };
}
