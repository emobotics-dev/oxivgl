// SPDX-License-Identifier: MIT OR Apache-2.0
//! View trait and navigation primitives.
//!
//! The [`View`](crate::view::View) trait defines a single screen of UI with a repeatable
//! lifecycle: `create` → `update` → `on_event` → `will_hide`, cycling
//! on each navigation transition. See `docs/spec-navigation.md`.

use core::ffi::c_void;

use embassy_time::{Duration, Timer};

use oxivgl_sys::*;

use crate::{
    display::{lvgl_disp_init, LvglBuffers, DISPLAY_READY},
    driver::LvglDriver,
    enums::EventCode,
    event::Event,
    widgets::{Obj, WidgetError},
};

/// A single view of UI (one screen or modal in a navigation stack).
///
/// The lifecycle is:
///
/// 1. **Construction** — caller creates the struct (e.g. `Default::default()`)
/// 2. [`create`](View::create) — build widgets into `container`; may be called
///    multiple times across push/pop cycles
/// 3. [`did_show`](View::did_show) — post-creation setup (optional)
/// 4. [`update`](View::update) — per-tick polling (runs in render loop)
/// 5. [`on_event`](View::on_event) — LVGL event dispatch
/// 6. [`will_hide`](View::will_hide) — save transient state before teardown
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
    /// Build all LVGL widgets for this view into `container`.
    ///
    /// Called each time this view becomes the active (topmost) view —
    /// both on initial display and when a view above it is popped.
    /// `container` is the LVGL screen object to build into.
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;

    /// Refresh widget values from application state. Called every render tick.
    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }

    /// Handle a bubbled LVGL event. Default is a no-op.
    fn on_event(&mut self, _event: &Event) {}

    /// Register event handlers. Called once after [`create`](View::create).
    /// Default registers on the active screen only. Override to register on
    /// additional objects (e.g. containers that catch bubbled events).
    fn register_events(&mut self) {
        // SAFETY: lv_screen_active() is valid after lv_init().
        register_event_on(self, unsafe { lv_screen_active() });
    }

    /// Called before this view's widget tree is destroyed (navigating away).
    /// Save any transient widget state here. Default is a no-op.
    fn will_hide(&mut self) {}

    /// Called after this view becomes visible again (navigated back to).
    /// Default is a no-op.
    fn did_show(&mut self) {}
}

/// Register event handlers for the view. Calls [`View::register_events`],
/// which by default registers on the active screen. Override the trait method
/// to register on additional objects.
///
/// The `view` reference must remain at a stable address for the lifetime of
/// the LVGL display (guaranteed by `run_app` and `host_main!`).
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
/// This is guaranteed by `run_app` (async frame pin) and `host_main!`
/// (stack-local before infinite loop). Do not call on a local that may move.
pub fn register_event_on<V: View>(view: &mut V, obj: *mut lv_obj_t) {
    let view_ptr = view as *mut V as *mut c_void;
    // SAFETY: obj must be a valid LVGL object; view_ptr remains valid for the
    // LVGL display lifetime (guaranteed by run_app / host_main!).
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

/// Run the LVGL render loop with a [`View`].
///
/// This is an embassy async task. Spawn it alongside your other application
/// tasks. It initialises LVGL, creates the view, then loops: calls
/// `V::update` and drives `lv_timer_handler` every tick.
///
/// `w` and `h` are the display resolution in pixels. `bufs` must be a
/// `'static` caller-allocated [`LvglBuffers`] sized for the screen width.
///
/// `view` is the initial view instance. Its `create` method is called once
/// the display is ready.
///
/// Never returns.
pub async fn run_app<V: View, const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    mut view: V,
) -> ! {
    info!("UI task started");
    let driver = LvglDriver::init(w, h);
    // SAFETY: lv_init() has been called inside LvglDriver::init() above.
    unsafe { lvgl_disp_init(w, h, bufs) };

    DISPLAY_READY.wait().await;
    info!("Display ready");

    // Wrap the active screen in a non-owning Obj for the container parameter.
    let screen_handle = unsafe { lv_screen_active() };
    assert!(!screen_handle.is_null(), "no active screen after display init");
    let container = Obj::from_raw(screen_handle);

    if let Err(e) = view.create(&container) {
        warn!("Could not create LVGL widgets: {:?}, disabling UI", e);
        // Forget container so we don't delete the LVGL screen.
        core::mem::forget(container);
        loop {
            Timer::after(Duration::from_secs(60)).await;
        }
    }
    // Forget container — the LVGL screen must not be deleted.
    core::mem::forget(container);

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
