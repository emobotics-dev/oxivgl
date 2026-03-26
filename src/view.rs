// SPDX-License-Identifier: MIT OR Apache-2.0
use core::ffi::c_void;

use embassy_time::{Duration, Timer};

use oxivgl_sys::*;

use crate::{
    display::{lvgl_disp_init, LvglBuffers, DISPLAY_READY},
    driver::LvglDriver,
    enums::EventCode,
    event::Event,
    widgets::WidgetError,
};

/// UI view trait. Implement this for each screen layout.
///
/// `run_lvgl` calls [`create`](View::create) once, then [`update`](View::update)
/// in a loop at `LV_DEF_REFR_PERIOD / 4` ms intervals.
///
/// Override [`on_event`](View::on_event) to handle LVGL widget events (clicks,
/// presses, etc.) without writing `unsafe extern "C"` callbacks. Widgets that
/// should deliver events to `on_event` must have `ObjFlag::EVENT_BUBBLE`
/// set so the event reaches the screen-level handler.
///
/// For nested widget trees (e.g. buttons inside a container), override
/// [`register_events`](View::register_events) to add event handlers on
/// intermediate objects via [`register_event_on`].
pub trait View: Sized {
    /// Create all LVGL widgets for this view. Called once after display init.
    fn create() -> Result<Self, WidgetError>;
    /// Refresh widget values from the latest application state. Called every render tick.
    fn update(&mut self) -> Result<(), WidgetError>;
    /// Handle a bubbled LVGL event. Default is a no-op.
    fn on_event(&mut self, _event: &Event) {}
    /// Register event handlers. Called once after [`create`](View::create).
    /// Default registers on the active screen only. Override to register on
    /// additional objects (e.g. containers that catch bubbled events).
    fn register_events(&mut self) {
        // SAFETY: lv_screen_active() is valid after lv_init().
        register_event_on(self, unsafe { lv_screen_active() });
    }
}

/// Register event handlers for the view. Calls [`View::register_events`],
/// which by default registers on the active screen. Override the trait method
/// to register on additional objects.
///
/// The `view` reference must remain at a stable address for the lifetime of
/// the LVGL display (guaranteed by `run_lvgl` and `host_main!`).
pub fn register_view_events<V: View>(view: &mut V) {
    view.register_events();
}

/// Register the view's event trampoline on a specific LVGL object.
/// Use this from [`View::register_events`] to catch events on containers
/// or other intermediate objects that don't bubble to the screen.
///
/// # Safety requirement (not enforced by the type system)
///
/// `view` must remain at a stable address for the LVGL display lifetime.
/// This is guaranteed by `run_lvgl` (async frame pin) and `host_main!`
/// (stack-local before infinite loop). Do not call on a local that may move.
pub fn register_event_on<V: View>(view: &mut V, obj: *mut lv_obj_t) {
    let view_ptr = view as *mut V as *mut c_void;
    // SAFETY: obj must be a valid LVGL object; view_ptr remains valid for the
    // LVGL display lifetime (guaranteed by run_lvgl / host_main!).
    unsafe {
        lv_obj_add_event_cb(
            obj,
            Some(view_event_trampoline::<V>),
            EventCode::ALL.0,
            view_ptr,
        );
    };
}

unsafe extern "C" fn view_event_trampoline<V: View>(e: *mut lv_event_t) {
    if e.is_null() {
        return;
    }
    unsafe {
        let view = lv_event_get_user_data(e) as *mut V;
        if !view.is_null() {
            let event = Event::from_raw(e);
            (*view).on_event(&event);
        }
    }
}

/// Run the LVGL render loop with a [`View`] of type `V`.
///
/// Initialises LVGL, waits for the display driver to be ready, creates the view,
/// then loops: calls `V::update` and drives `lv_timer_handler` every tick.
/// `w` and `h` are the display resolution in pixels. `bufs` must be a `'static`
/// caller-allocated [`LvglBuffers`] sized for the screen width. Never returns.
pub async fn run_lvgl<V: View, const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
) -> ! {
    info!("UI task started");
    let driver = LvglDriver::init(w, h);
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

    register_view_events(&mut view);

    const LVGL_TIMER_DELAY: u64 = LV_DEF_REFR_PERIOD as u64 / 4;

    loop {
        debug!("Rendering UI loop iteration");
        view.update()
            .unwrap_or_else(|e| warn!("Failed to update widgets: {:?}", e));

        // Drive lv_timer_handler 4× per update cycle (once per refresh period)
        // so LVGL animations stay smooth while update() is called at ~30fps.
        for _ in 0..4 {
            debug!("LVGL tick/timer handler");
            driver.timer_handler();
            Timer::after(Duration::from_millis(LVGL_TIMER_DELAY)).await;
        }
    }
}
