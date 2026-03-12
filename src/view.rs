// SPDX-License-Identifier: GPL-3.0-only
use embassy_time::{Duration, Timer};

use lvgl_rust_sys::{lv_timer_handler, LV_DEF_REFR_PERIOD};

use crate::{
    lvgl::LvglDriver,
    lvgl_buffers::{lvgl_disp_init, LvglBuffers, DISPLAY_READY},
    widgets::WidgetError,
};

/// UI view trait. Implement this for each screen layout.
///
/// `run_lvgl` calls [`create`](View::create) once, then [`update`](View::update)
/// in a loop at `LV_DEF_REFR_PERIOD / 4` ms intervals.
pub trait View: Sized {
    /// Create all LVGL widgets for this view. Called once after display init.
    fn create() -> Result<Self, WidgetError>;
    /// Refresh widget values from the latest application state. Called every render tick.
    fn update(&mut self) -> Result<(), WidgetError>;
}

/// Run the LVGL render loop with a [`View`] of type `V`.
///
/// Initialises LVGL, waits for the display driver to be ready, creates the view,
/// then loops: calls `V::update` and drives `lv_timer_handler` every tick.
/// `w` and `h` are the display resolution in pixels. `bufs` must be a `'static`
/// caller-allocated [`LvglBuffers`] sized for the screen width. Never returns.
pub async fn run_lvgl<V: View, const BYTES: usize>(w: i32, h: i32, bufs: &'static mut LvglBuffers<BYTES>) -> ! {
    info!("UI task started");
    let _driver = LvglDriver::init(w, h);
    // SAFETY: lv_init() has been called inside LvglDriver::init() above.
    unsafe { lvgl_disp_init(w, h, bufs) };

    DISPLAY_READY.wait().await;
    info!("Display ready");

    let Ok(mut view) = V::create() else {
        warn!("Could not create LVGL widgets, disabling UI");
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    };

    const LVGL_TIMER_DELAY: u64 = LV_DEF_REFR_PERIOD as u64 / 4;

    loop {
        debug!("Rendering UI loop iteration");
        view.update().unwrap_or_else(|e| warn!("Failed to update widgets: {:?}", e));

        // Drive lv_timer_handler 16× per update cycle (4 per refresh period × 4 periods)
        // so LVGL animations stay smooth while update() is called once per cycle.
        for _ in 0..16 {
            debug!("LVGL tick/timer handler");
            // SAFETY: lv_init() was called inside LvglDriver::init(); no other task
            // calls LVGL concurrently (single-task constraint).
            unsafe { lv_timer_handler() };
            Timer::after(Duration::from_millis(LVGL_TIMER_DELAY)).await;
        }
    }
}
