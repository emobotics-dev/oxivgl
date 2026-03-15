// SPDX-License-Identifier: MIT OR Apache-2.0
//! Host-side helpers for oxivgl examples: SDL2 viewer loop, LVGL timer
//! pumping, and PNG screenshot capture.

use std::path::PathBuf;

use lvgl_rust_sys::*;

pub const W: i32 = 320;
pub const H: i32 = 240;

/// Drive the LVGL timer loop. Call after creating all widgets. Never returns.
pub fn run_host_loop() -> ! {
    loop {
        // SAFETY: lv_init() was called inside LvglDriver::init() before entering main's loop.
        unsafe { lv_timer_handler() };
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Pump the LVGL timer `n` times (5 ms each).
pub fn pump(n: u32) {
    for _ in 0..n {
        // SAFETY: LVGL is initialised and we are on the single LVGL task.
        unsafe { lv_timer_handler() };
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Load a fresh blank screen, discarding the current one.
pub fn fresh_screen() {
    // SAFETY: LVGL is initialised; null parent creates a screen-level object.
    unsafe { lv_screen_load(lv_obj_create(core::ptr::null_mut())) };
}

/// Capture current screen as a PNG file under `<dir>/<name>.png`.
pub fn capture(name: &str, dir: &str) {
    // SAFETY: LVGL is initialised; lv_snapshot_take allocates a draw buffer.
    let draw_buf = unsafe {
        lv_snapshot_take(lv_screen_active(), lv_color_format_t_LV_COLOR_FORMAT_RGB565)
    };
    assert!(!draw_buf.is_null(), "lv_snapshot_take returned NULL");

    // SAFETY: draw_buf is non-null and points to a valid lv_draw_buf_t.
    let buf = unsafe { &*draw_buf };
    let w = buf.header.w();
    let h = buf.header.h();
    // SAFETY: buf.data is valid for buf.data_size bytes (just allocated by LVGL).
    let data = unsafe { std::slice::from_raw_parts(buf.data, buf.data_size as usize) };

    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.png"));
    write_png(&path, w, h, data).expect("PNG write failed");
    println!("Screenshot: {}", path.display());

    // SAFETY: draw_buf was allocated by lv_snapshot_take above.
    unsafe { lv_draw_buf_destroy(draw_buf) };
}

/// Convert RGB565 pixel data to RGB8 and write as PNG.
fn write_png(path: &std::path::Path, w: u32, h: u32, data: &[u8]) -> std::io::Result<()> {
    let stride = w as usize * 2;
    let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
    for row in 0..h as usize {
        for col in 0..w as usize {
            let off = row * stride + col * 2;
            let p = u16::from_le_bytes([data[off], data[off + 1]]);
            let r = ((p >> 11) & 0x1F) as u8;
            let g = ((p >> 5) & 0x3F) as u8;
            let b = (p & 0x1F) as u8;
            rgb.push((r << 3) | (r >> 2));
            rgb.push((g << 2) | (g >> 4));
            rgb.push((b << 3) | (b >> 2));
        }
    }

    let file = std::fs::File::create(path)?;
    let buf = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(buf, w, h);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| std::io::Error::other(e))?;
    writer.write_image_data(&rgb).map_err(|e| std::io::Error::other(e))?;
    Ok(())
}

/// Generate a host `main` function for the given [`oxivgl::view::View`] type.
///
/// The generated main:
/// 1. Initialises `env_logger` and the LVGL SDL2 driver.
/// 2. Creates the view.
/// 3. If `SCREENSHOT_ONLY=1`, captures a screenshot and exits.
/// 4. Otherwise runs the interactive host loop.
#[macro_export]
macro_rules! host_main {
    ($View:ty) => {
        fn main() {
            use $crate::host::{H, W, capture, pump, run_host_loop};
            use $crate::oxivgl::lvgl::LvglDriver;
            use $crate::oxivgl::view::View;

            $crate::env_logger::init();
            let _driver = LvglDriver::init(W, H);
            let mut _view = <$View>::create().expect("view create failed");
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

            // Always capture a screenshot.
            pump(10);
            capture(name, &dir);

            if std::env::var("SCREENSHOT_ONLY").as_deref() == Ok("1") {
                // Skip Rust destructors — LVGL's internal state (timers,
                // display refresh) can race with lv_obj_delete during
                // Drop, causing intermittent SIGSEGV on exit.
                std::process::exit(0);
            }

            run_host_loop();
        }
    };
}
