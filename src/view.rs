// SPDX-License-Identifier: MIT OR Apache-2.0
//! View trait and navigation primitives.
//!
//! The [`View`](crate::view::View) trait defines a single screen of UI with a repeatable
//! lifecycle: `create` → `update` → `on_event` → `will_hide`, cycling
//! on each navigation transition. See `docs/spec-navigation.md`.

use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::future::Future;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::task::Poll;
use core::time::Duration;
use embassy_time::{Timer, with_timeout};

use oxivgl_sys::*;

use crate::{
    display::{Buffers, DISPLAY_READY, LvglBuffers, lvgl_disp_init},
    driver::LvglDriver,
    enums::EventCode,
    event::Event,
    widgets::{AsLvHandle, Obj, ScreenAnim, WidgetError},
};

/// LVGL timer tick interval (ms). `LV_DEF_REFR_PERIOD / 4` yields ~4 ticks
/// per refresh cycle, keeping animations smooth at ~30 fps.
const LVGL_TICK_MS: u64 = LV_DEF_REFR_PERIOD as u64 / 4;

// ---------------------------------------------------------------------------
// RenderConfig
// ---------------------------------------------------------------------------

/// Cadence of the render loop.
///
/// Default redraw period is `lv_conf.h`'s `LV_DEF_REFR_PERIOD` (32 ms, ~31 fps
/// ceiling). [`View::update`] is polled once per that period, independently of
/// the draw rate.
#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    /// LVGL redraw period (ms). `None` keeps `LV_DEF_REFR_PERIOD`.
    pub refresh_period_ms: Option<u32>,
    /// How often [`View::update`] is polled (ms). Cost is widgets touched, not pixels.
    pub update_period_ms: u64,
    /// Cap on the delay returned after each refresh (ms).
    pub max_idle_ms: u64,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            // Keep lv_conf.h's period — changing the default frame rate of
            // every existing application would be a surprise, not a fix.
            refresh_period_ms: None,
            update_period_ms: LV_DEF_REFR_PERIOD as u64,
            max_idle_ms: LVGL_TICK_MS,
        }
    }
}

impl RenderConfig {
    /// Target `fps`. Period `1000/fps`; idle cap a quarter of that — refresh
    /// runs *before* the sleep, so a cap equal to the period would make a
    /// cycle `period + draw`.
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        let period = (1000 / fps.max(1)).max(1);
        self.refresh_period_ms = Some(period);
        self.max_idle_ms = (period / 4).max(1) as u64;
        self
    }

    /// Poll [`View::update`] every `ms` milliseconds.
    pub fn with_update_period_ms(mut self, ms: u64) -> Self {
        self.update_period_ms = ms;
        self
    }
}

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
/// [`register_events_on`](View::register_events_on) to add event handlers
/// on intermediate objects via [`register_event_on`].
pub trait View: Sized + 'static {
    /// Build all LVGL widgets for this view into `container`.
    ///
    /// Called each time this view becomes the active (topmost) view —
    /// both on initial display and when a view above it is popped.
    /// `container` is the LVGL screen object to build into.
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;

    /// Refresh widget values from application state. Called every render tick.
    ///
    /// Return [`NavAction::None`] to stay on this view, or a navigation
    /// action to trigger a transition. This is the primary integration
    /// point for external events — poll channels/shared state here.
    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }

    /// Handle a bubbled LVGL event. Return [`NavAction::None`] to stay on
    /// this view, or a navigation action to trigger a transition.
    fn on_event(&mut self, _event: &Event) -> NavAction {
        NavAction::None
    }

    /// Register event handlers. Called once after [`create`](View::create),
    /// with the same `container` that was passed to `create`.
    ///
    /// Default registers the view's event trampoline on `container`, so
    /// bubbled events from any descendant reach [`on_event`](View::on_event).
    /// Override to register on additional objects (e.g. intermediate
    /// containers that catch bubbled events for sub-trees).
    ///
    /// Receiving the container as an argument — rather than reading
    /// `lv_screen_active()` — is what makes the default impl correct for
    /// modals: when the navigator builds a modal, `container` is
    /// `lv_layer_top()`, not the background view's screen.
    fn register_events_on(&mut self, container: &Obj<'static>) {
        register_event_on(self, container.lv_handle());
    }

    /// Called before this view's widget tree is destroyed (navigating away).
    /// Save any transient widget state here. Default is a no-op.
    fn will_hide(&mut self) {}

    /// Called after this view becomes visible again (navigated back to).
    /// Default is a no-op.
    fn did_show(&mut self) {}

    /// Focus group containing this view's focusable widgets.
    ///
    /// When non-`None`, the navigator activates this group on
    /// modal open (sets it as default + binds it to all keyboard /
    /// encoder input devices) and restores the previously active focus
    /// state on dismiss. The view owns the [`Group`](crate::group::Group)
    /// internally; this method just borrows a non-owning handle.
    ///
    /// Default `None` — only OSD-style modals that need key input
    /// usually return `Some`.
    fn input_group(&self) -> Option<crate::group::GroupRef> {
        None
    }
}

// ---------------------------------------------------------------------------
// NavAction
// ---------------------------------------------------------------------------

/// Navigation action requested by a view.
///
/// Returned from [`View::update`] and [`View::on_event`]. The render loop
/// (or [`Navigator`](crate::navigator::Navigator)) processes the action
/// after the method returns.
pub enum NavAction {
    /// No navigation requested.
    None,
    /// Push a new view onto the stack.
    Push(Box<dyn AnyView>, Option<ScreenAnim>),
    /// Pop the current view (return to previous).
    Pop(Option<ScreenAnim>),
    /// Replace the current view (non-reversible transition).
    Replace(Box<dyn AnyView>, Option<ScreenAnim>),
    /// Show a modal overlay on top of the current view.
    Modal(Box<dyn AnyView>),
    /// Dismiss the current modal overlay.
    DismissModal,
    /// Show a global passive status overlay (toast) on the system layer.
    ///
    /// Unlike [`Modal`](Self::Modal), the toast persists across page
    /// switches and registers no input handlers. If the `Duration` is
    /// `Some`, the navigator auto-dismisses on expiry. See
    /// [`Navigator::show_toast`](crate::navigator::Navigator::show_toast).
    ShowToast(Box<dyn AnyView>, Option<Duration>),
    /// Dismiss the active global toast overlay.
    DismissToast,
}

impl NavAction {
    /// Convenience: push a view with an optional animation.
    pub fn push(view: impl View, anim: Option<ScreenAnim>) -> Self {
        Self::Push(Box::new(view), anim)
    }

    /// Convenience: replace the current view.
    pub fn replace(view: impl View, anim: Option<ScreenAnim>) -> Self {
        Self::Replace(Box::new(view), anim)
    }

    /// Convenience: show a modal overlay.
    pub fn modal(view: impl View) -> Self {
        Self::Modal(Box::new(view))
    }

    /// Convenience: show a passive global toast.
    ///
    /// `duration` is an optional auto-dismiss timeout owned by the navigator.
    /// `None` means the toast stays until [`NavAction::DismissToast`] (or
    /// [`Navigator::dismiss_toast`](crate::navigator::Navigator::dismiss_toast)).
    pub fn show_toast(view: impl View, duration: Option<Duration>) -> Self {
        Self::ShowToast(Box::new(view), duration)
    }

    /// Returns `true` if this is [`NavAction::None`].
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

// ---------------------------------------------------------------------------
// AnyView — object-safe trait for type-erased views
// ---------------------------------------------------------------------------

/// Object-safe trait for type-erased views stored in a
/// [`Navigator`](crate::navigator::Navigator) stack.
///
/// Implemented automatically for all [`View`] types via blanket impl.
/// Users should implement [`View`], never `AnyView` directly.
pub trait AnyView: 'static {
    /// Build widgets into `container`. See [`View::create`].
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;
    /// Per-tick update. See [`View::update`].
    fn update(&mut self) -> Result<NavAction, WidgetError>;
    /// Handle a bubbled LVGL event. See [`View::on_event`].
    fn on_event(&mut self, event: &Event) -> NavAction;
    /// Register event handlers. See [`View::register_events_on`].
    fn register_events_on(&mut self, container: &Obj<'static>);
    /// Called before widget teardown. See [`View::will_hide`].
    fn will_hide(&mut self);
    /// Called after view becomes visible again. See [`View::did_show`].
    fn did_show(&mut self);
    /// Focus group for this view (modals only). See [`View::input_group`].
    fn input_group(&self) -> Option<crate::group::GroupRef>;
}

impl<T: View> AnyView for T {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        View::create(self, container)
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        View::update(self)
    }

    fn on_event(&mut self, event: &Event) -> NavAction {
        View::on_event(self, event)
    }

    fn register_events_on(&mut self, container: &Obj<'static>) {
        View::register_events_on(self, container)
    }

    fn will_hide(&mut self) {
        View::will_hide(self)
    }

    fn did_show(&mut self) {
        View::did_show(self)
    }

    fn input_group(&self) -> Option<crate::group::GroupRef> {
        View::input_group(self)
    }
}

// ---------------------------------------------------------------------------
// NavigationError
// ---------------------------------------------------------------------------

/// Errors from navigation operations.
#[derive(Debug)]
pub enum NavigationError {
    /// Cannot pop the root view — the stack has only one entry.
    StackEmpty,
    /// No modal is currently active.
    NoActiveModal,
    /// No toast overlay is currently active.
    NoActiveToast,
    /// View creation failed during a navigation transition.
    CreateFailed(WidgetError),
}

impl core::fmt::Display for NavigationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::StackEmpty => write!(f, "cannot pop the root view"),
            Self::NoActiveModal => write!(f, "no active modal to dismiss"),
            Self::NoActiveToast => write!(f, "no active toast to dismiss"),
            Self::CreateFailed(e) => write!(f, "view creation failed: {:?}", e),
        }
    }
}

// ---------------------------------------------------------------------------
// Pending event action (single-threaded stash for trampoline → navigator)
// ---------------------------------------------------------------------------

/// SAFETY: LVGL is single-threaded. The event trampoline writes this cell
/// during refresh; the render loop reads it on the same thread. `NavAction`
/// is `!Send`; the cell never crosses threads.
struct SyncCell(UnsafeCell<Option<NavAction>>);
unsafe impl Sync for SyncCell {}

static PENDING_EVENT_ACTION: SyncCell = SyncCell(UnsafeCell::new(None));

/// Take the pending event action stashed by the trampoline, if any.
pub(crate) fn take_pending_event_action() -> Option<NavAction> {
    // SAFETY: single-threaded access — see SyncCell doc.
    unsafe { (*PENDING_EVENT_ACTION.0.get()).take() }
}

/// Register event handlers for the view by delegating to
/// [`View::register_events_on`] with `container` as the target.
///
/// The `view` reference must remain at a stable address for the lifetime of
/// the LVGL display (guaranteed by `run_app` and `host_main!`).
pub fn register_view_events<V: View>(view: &mut V, container: &Obj<'static>) {
    view.register_events_on(container);
}

/// Register the view's event trampoline on a specific LVGL object.
/// Use this from [`View::register_events_on`] to catch events on containers
/// or other intermediate objects that don't bubble to the screen.
///
/// # Address stability (not enforced by the type system)
///
/// `view` must remain at a stable address for the LVGL display lifetime:
/// render-thread stack (`run_app`), `host_main!` stack, or `Box` in
/// [`crate::navigator::Navigator`]. Do not move it after registration.
pub fn register_event_on<V: View>(view: &mut V, obj: *mut lv_obj_t) {
    assert!(!obj.is_null(), "register_event_on: obj must not be null");
    let view_ptr = view as *mut V as *mut c_void;
    // SAFETY: `obj` non-null; `view` lives for the display lifetime (see above).
    unsafe {
        lv_obj_add_event_cb(
            obj,
            Some(view_event_trampoline::<V>),
            EventCode::ALL.0,
            view_ptr,
        );
    };
}

/// SAFETY: `user_data` is a `*mut V` set by `register_event_on`. The pointer
/// remains valid because the view lives behind Box indirection (navigator) or
/// in a pinned async frame (run_app), so address stability is guaranteed even
/// if the navigator's Vec reallocates. See `register_event_on` doc comment.
unsafe extern "C" fn view_event_trampoline<V: View>(e: *mut lv_event_t) {
    if e.is_null() {
        return;
    }
    unsafe {
        let view = lv_event_get_user_data(e) as *mut V;
        if !view.is_null() {
            let event = Event::from_raw(e);
            let action = (*view).on_event(&event);
            if !action.is_none() {
                // Stash the action for the render loop to process.
                // First action per tick wins (subsequent are dropped).
                let slot = &mut *PENDING_EVENT_ACTION.0.get();
                if slot.is_none() {
                    *slot = Some(action);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Ui — display setup, separated from the render loop
// ---------------------------------------------------------------------------

/// Initialised LVGL display. Every LVGL call belongs on the thread that
/// called [`Self::init`] — the async loops below must be driven by that same
/// thread's executor, never spawned onto another one.
///
/// The type supports two loop shapes:
///
/// * **Combined** — [`run`](Self::run) / [`run_nav`](Self::run_nav) (and the
///   free `run_app*` functions) do the redraw *and* the view polling in one
///   async task. Correct wherever the flush wait does not block the thread:
///   the host SDL backend, or a non-blocking `FlushSync` (not linked:
///   `flush_pipeline` exists only under the `esp-hal` / `rtos-sem` features, so
///   the link would not resolve on host).
/// * **Split** — [`run_events`](Self::run_events) and friends only poll views
///   and sleep; the blocking redraw is [`refresh`](Self::refresh), which the
///   render thread calls from its executor's idle hook. Use this whenever the
///   redraw blocks the thread — a blocking `FlushSync`, `waiti`-parking, or a
///   scan-out panel whose frame wait is a vblank — because a blocking step
///   inside an async task stalls every other task on that executor.
///
/// ```ignore
/// // Split loop: the executor's idle hook owns the blocking redraw.
/// let ui = make_static!(Ui::init(W, H, Buffers::full(a, b, FRAME_BYTES)));
/// let exec = make_static!(Executor::new());
/// exec.run_with_callbacks(
///     |s| s.must_spawn(render_task(ui)),   // awaits `ui.run_events(view, cfg)`
///     UiHooks(ui),                         // `on_idle` calls `ui.refresh()`
/// )
/// ```
#[derive(Debug)]
pub struct Ui {
    driver: LvglDriver,
    /// DIRECT scan-out: [`Ui::refresh`] waits for the frame to reach the panel,
    /// so that wait — not a timer — paces the combined loop.
    present_blocks: bool,
    /// Delay (ms) LVGL asked for after the last redraw. Written by
    /// [`Ui::timer_handler`] on the render thread, read by the async loops.
    /// An atomic rather than a `Cell` because the executor's idle hook and the
    /// async task hold separate borrows of the same `&Ui`.
    next_delay_ms: AtomicU32,
    /// Set once [`Ui::wait_ready`] has observed the display driver coming up.
    /// [`Ui::refresh`] is inert until then, so an idle hook installed before
    /// the flush path exists cannot drive LVGL into it.
    ready: AtomicBool,
}

impl Ui {
    /// Initialise LVGL, create the display, and register flush callbacks —
    /// without running a render loop.
    ///
    /// `bufs` is [`Buffers::partial`] (SPI stripes) or [`Buffers::full`]
    /// (scan-out frames). Call at most once — LVGL panics on a second `lv_init`.
    pub fn init(w: i32, h: i32, bufs: Buffers) -> Self {
        let driver = LvglDriver::init(w, h);
        let present_blocks = bufs.present_blocks();
        // SAFETY: `lv_init()` ran above; call-once; caller keeps `bufs` alive.
        unsafe { lvgl_disp_init(w, h, bufs) };
        Self {
            driver,
            present_blocks,
            // Before the first redraw there is no LVGL-supplied pace; 1 ms
            // makes the first async sleep short so the UI comes up promptly.
            next_delay_ms: AtomicU32::new(1),
            ready: AtomicBool::new(false),
        }
    }

    /// Wait until the display driver reports ready — on ESP32 that is the flush
    /// task starting, on host it is immediate.
    ///
    /// Arms [`refresh`](Self::refresh): before this resolves, `refresh` does
    /// nothing, so an idle hook cannot run LVGL before the flush path exists.
    pub async fn wait_ready(&self) {
        DISPLAY_READY.wait().await;
        self.ready.store(true, Ordering::Release);
    }

    /// One LVGL tick, without the scan-out frame wait. Returns the delay (ms)
    /// LVGL recommends before the next call and publishes it for the async
    /// loops. Render thread only.
    ///
    /// [`refresh`](Self::refresh) is what a render loop should call — it adds
    /// the scan-out wait and honours the display-ready gate.
    pub fn timer_handler(&self) -> u32 {
        let delay = self.driver.timer_handler();
        self.next_delay_ms.store(delay, Ordering::Relaxed);
        delay
    }

    /// The blocking LVGL refresh step: one [`timer_handler`](Self::timer_handler)
    /// tick plus, on a scan-out display, the wait for that frame to reach the
    /// panel. Returns the delay (ms) LVGL recommends before the next call.
    ///
    /// **Render thread only, and never from an async task** — it blocks for the
    /// whole draw and flush. In a split loop this belongs in the executor's
    /// idle hook, which runs exactly when the executor would otherwise sleep;
    /// see the [type documentation](Self).
    ///
    /// Does nothing and reports the previous delay until
    /// [`wait_ready`](Self::wait_ready) has resolved.
    pub fn refresh(&self) -> u32 {
        if !self.ready.load(Ordering::Acquire) {
            return self.next_delay_ms.load(Ordering::Relaxed);
        }
        let delay = self.timer_handler();
        if self.present_blocks {
            crate::scanout::wait_presented();
        }
        delay
    }

    /// Apply `cfg`'s redraw period and create `view` on the active screen.
    ///
    /// [`DISPLAY_READY`] must already have been signalled — await
    /// [`wait_ready`](Self::wait_ready) first (`Buffers::full` signals it in
    /// [`init`](Self::init)). Returns the error from [`View::create`] verbatim
    /// so the caller can report it.
    pub fn bind<V: View>(&self, view: &mut V, cfg: &RenderConfig) -> Result<(), WidgetError> {
        self.apply(cfg);
        let screen_handle = unsafe { lv_screen_active() };
        assert!(
            !screen_handle.is_null(),
            "no active screen after display init"
        );
        let container = Obj::from_raw_non_owning(screen_handle);
        view.create(&container)?;
        register_view_events(view, &container);
        Ok(())
    }

    /// Apply a [`RenderConfig`]'s redraw period, if it sets one.
    fn apply(&self, cfg: &RenderConfig) {
        if let Some(ms) = cfg.refresh_period_ms
            && !crate::display::set_refresh_period(ms)
        {
            warn!("could not set refresh period to {} ms", ms);
        }
    }

    /// Combined loop: [`bind`](Self::bind), then redraw and poll `view` in one
    /// async task. Never returns.
    ///
    /// Drives [`refresh`](Self::refresh) itself, so use it only where that does
    /// not block the thread — the host backend or a non-blocking flush. On a
    /// blocking flush or a scan-out panel use [`run_events`](Self::run_events)
    /// plus an idle-hook `refresh` instead.
    pub async fn run<V: View>(self, mut view: V, cfg: RenderConfig) -> ! {
        self.wait_ready().await;
        info!("Display ready");
        if self.bind_failed(&mut view, &cfg) {
            park().await
        }

        let update_period = embassy_time::Duration::from_millis(cfg.update_period_ms);
        let mut next_update = embassy_time::Instant::now();
        loop {
            poll_update(
                &mut view,
                embassy_time::Instant::now(),
                update_period,
                &mut next_update,
            );
            let delay = self.refresh();
            self.pace(delay, &cfg).await;
        }
    }

    /// Combined [`Navigator`](crate::navigator::Navigator) loop — [`run`](Self::run)
    /// with push/pop/replace/modal transitions. Never returns.
    pub async fn run_nav(self, initial: impl View, cfg: RenderConfig) -> ! {
        run_app_nav_inner(&self, cfg, initial, None, None, false, true, no_wake).await
    }

    /// Split loop: [`bind`](Self::bind), then poll `view` and sleep. Never
    /// returns and never draws — the render thread's executor must call
    /// [`refresh`](Self::refresh) from its idle hook.
    ///
    /// The sleep is LVGL's own recommended delay (published by `refresh`),
    /// capped by [`RenderConfig::max_idle_ms`], so LVGL keeps setting the pace.
    /// Sleeping is what makes the executor idle, which is what runs the hook —
    /// so an implementation that never awaits here would stop the display.
    pub async fn run_events<V: View>(&'static self, mut view: V, cfg: RenderConfig) -> ! {
        self.wait_ready().await;
        info!("Display ready");
        if self.bind_failed(&mut view, &cfg) {
            park().await
        }

        let update_period = embassy_time::Duration::from_millis(cfg.update_period_ms);
        let mut next_update = embassy_time::Instant::now();
        loop {
            poll_update(
                &mut view,
                embassy_time::Instant::now(),
                update_period,
                &mut next_update,
            );
            Timer::after(embassy_time::Duration::from_millis(self.idle_ms(&cfg))).await;
        }
    }

    /// Split [`Navigator`](crate::navigator::Navigator) loop — [`run_events`](Self::run_events)
    /// with push/pop/replace/modal transitions. Never returns and never draws;
    /// pair it with an idle-hook [`refresh`](Self::refresh).
    pub async fn run_events_nav(&'static self, initial: impl View, cfg: RenderConfig) -> ! {
        run_app_nav_inner(self, cfg, initial, None, None, false, false, no_wake).await
    }

    /// [`run_events_nav`](Self::run_events_nav) with an **encoder** input
    /// device, event-driven and poll-free.
    ///
    /// The device is created in EVENT mode and each sleep is raced against
    /// [`EncoderState::wait`](crate::indev::EncoderState::wait), so a decoded
    /// turn/click reaches LVGL as soon as the render thread is scheduled — no
    /// read-timer latency. `encoder` must be `'static`. Never returns and never
    /// draws; pair it with an idle-hook [`refresh`](Self::refresh).
    pub async fn run_events_nav_encoder(
        &'static self,
        initial: impl View,
        encoder: &'static crate::indev::EncoderState,
        cfg: RenderConfig,
    ) -> ! {
        run_app_nav_inner(
            self,
            cfg,
            initial,
            None,
            Some(encoder),
            true,
            false,
            || encoder.wait(),
        )
        .await
    }

    /// [`bind`](Self::bind), reporting failure as a `bool` and disarming
    /// [`refresh`](Self::refresh) so a half-built screen is not drawn forever.
    fn bind_failed<V: View>(&self, view: &mut V, cfg: &RenderConfig) -> bool {
        match self.bind(view, cfg) {
            Ok(()) => false,
            Err(e) => {
                warn!("Could not create LVGL widgets: {:?}, disabling UI", e);
                self.ready.store(false, Ordering::Release);
                true
            }
        }
    }

    /// Sleep for the split loop: LVGL's published delay, capped by
    /// [`RenderConfig::max_idle_ms`] and never zero (a zero sleep would keep
    /// the executor busy, so the idle hook — the redraw — would never run).
    fn idle_ms(&self, cfg: &RenderConfig) -> u64 {
        (self.next_delay_ms.load(Ordering::Relaxed) as u64)
            .min(cfg.max_idle_ms)
            .max(1)
    }

    /// Inter-frame wait for the combined loop.
    ///
    /// On a scan-out display [`refresh`](Self::refresh) already blocked until
    /// the frame reached the panel, so the frame is paced and a sleep on top
    /// would only cost frame rate; yield instead, so co-tasks still run.
    async fn pace(&self, delay_ms: u32, cfg: &RenderConfig) {
        if self.present_blocks {
            yield_now().await;
        } else {
            let ms = (delay_ms as u64).min(cfg.max_idle_ms).max(1);
            Timer::after(embassy_time::Duration::from_millis(ms)).await;
        }
    }
}

/// Stop driving the UI without returning: the caller's contract is `-> !`, and
/// a tight loop would starve every other task on the executor.
async fn park() -> ! {
    loop {
        Timer::after(embassy_time::Duration::from_secs(60)).await;
    }
}

/// Hand the executor one turn, then continue. Used where the frame is already
/// paced by something other than a timer, so the loop must add no delay but
/// still must not monopolise the executor.
async fn yield_now() {
    let mut yielded = false;
    core::future::poll_fn(|cx| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await
}

/// Wait up to `budget_ms` for `wake`, reporting whether it fired.
///
/// `budget_ms == 0` polls `wake` exactly once and yields — the scan-out case,
/// where the frame is already paced by the panel and any sleep would cost
/// frame rate, but input must still be picked up.
async fn wait_wake(budget_ms: u64, wake: impl Future<Output = ()>) -> bool {
    if budget_ms == 0 {
        let mut wake = core::pin::pin!(wake);
        let fired =
            core::future::poll_fn(|cx| Poll::Ready(wake.as_mut().poll(cx).is_ready())).await;
        yield_now().await;
        fired
    } else {
        with_timeout(embassy_time::Duration::from_millis(budget_ms), wake)
            .await
            .is_ok()
    }
}

/// Poll `view` if `now` has reached `next_update`, and rearm `next_update`.
///
/// `now` is passed in rather than read here so the cadence logic is a pure
/// function of the clock — that is what the unit tests exercise.
fn poll_update<V: View>(
    view: &mut V,
    now: embassy_time::Instant,
    update_period: embassy_time::Duration,
    next_update: &mut embassy_time::Instant,
) {
    if now >= *next_update {
        // Resync rather than chase a backlog if a slow frame overran: the next
        // deadline is one period from *now*, not from the deadline just missed.
        *next_update = now + update_period;
        let action = view.update().unwrap_or_else(|e| {
            warn!("Failed to update widgets: {:?}", e);
            NavAction::None
        });
        debug_assert!(
            action.is_none(),
            "NavAction ignored in run_app — use run_app_nav for navigation"
        );
        // Drain any pending event action (stashed by on_event trampoline).
        // NavAction processing is the Navigator's job; see run_app_nav.
        let _event_action = take_pending_event_action();
    }
}

/// [`Ui::init`] + [`Ui::run`] with the default [`RenderConfig`] — the combined
/// loop, redraw and view polling in one async task. Never returns.
///
/// Await this on the render thread's own executor. It drives
/// [`Ui::refresh`] inline, so it is right where the flush wait does not block
/// the thread (host, non-blocking pipelines) and wrong where it does: on a
/// blocking `FlushSync` (not linked: `flush_pipeline` exists only under the
/// `esp-hal` / `rtos-sem` features, so the link would not resolve on host) or a
/// scan-out panel, use [`Ui::run_events`] and call [`Ui::refresh`] from the
/// executor's idle hook instead.
pub async fn run_app<V: View, const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    view: V,
) -> ! {
    info!("UI render loop started");
    Ui::init(w, h, Buffers::partial(bufs))
        .run(view, RenderConfig::default())
        .await
}

/// Run the LVGL render loop with navigation support.
///
/// Like [`run_app`], but creates a [`Navigator`](crate::navigator::Navigator)
/// that processes [`NavAction`] values from `update()` and `on_event()`.
/// Use this for multi-screen applications that need push/pop/replace/modal.
///
/// `initial` is the root view. Never returns.
///
/// Combined loop, like [`run_app`] — see there for when to use [`Ui::run_events_nav`]
/// instead.
pub async fn run_app_nav<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
) -> ! {
    let ui = Ui::init(w, h, Buffers::partial(bufs));
    run_app_nav_inner(
        &ui,
        RenderConfig::default(),
        initial,
        None,
        None,
        false,
        true,
        no_wake,
    )
    .await
}

/// Like [`run_app_nav`], but also registers a **TIMER-mode** keypad input
/// device driven by `keypad`.
///
/// The navigator routes each active view's
/// [`input_group`](View::input_group) to the keypad, so focusable widgets can
/// be navigated with discrete keys — from a GPIO button task or, on a
/// touchscreen, from on-screen buttons that call
/// [`KeypadState::press`](crate::indev::KeypadState::press) /
/// [`release`](crate::indev::KeypadState::release). LVGL polls the device on its
/// own read timer.
///
/// For an interrupt-driven, poll-free input path (a driver that already decodes
/// long-press/repeat and uses [`KeypadState::send`](crate::indev::KeypadState::send)),
/// use [`run_app_nav_keypad_events`] instead.
///
/// `keypad` must be `'static` (typically a `static KeypadState`). Never returns.
///
/// Combined loop, like [`run_app`] — see there for when to use [`Ui::run_events_nav`]
/// instead.
pub async fn run_app_nav_keypad<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    keypad: &'static crate::indev::KeypadState,
) -> ! {
    let ui = Ui::init(w, h, Buffers::partial(bufs));
    run_app_nav_inner(
        &ui,
        RenderConfig::default(),
        initial,
        Some(keypad),
        None,
        false,
        true,
        no_wake,
    )
    .await
}

/// Like [`run_app_nav_keypad`], but **event-driven and poll-free**.
///
/// Creates the keypad in EVENT mode (no LVGL read timer) and races the
/// inter-tick sleep against `wake`. When `wake` resolves — e.g. your
/// interrupt-driven input task signalled after `KEYPAD.send(key)` — the loop
/// reads the device immediately, so a decoded key reaches the screen with no
/// periodic polling of either the button or the indev.
///
/// `wake` is called fresh each tick to produce a future to race; supply your
/// input signal, e.g. `|| async { WAKE.wait().await }`. Never returns.
///
/// Combined loop, like [`run_app`] — see there for when to use [`Ui::run_events_nav`]
/// instead.
pub async fn run_app_nav_keypad_events<const BYTES: usize, Fut>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    keypad: &'static crate::indev::KeypadState,
    wake: impl Fn() -> Fut,
) -> !
where
    Fut: Future<Output = ()>,
{
    let ui = Ui::init(w, h, Buffers::partial(bufs));
    run_app_nav_inner(
        &ui,
        RenderConfig::default(),
        initial,
        Some(keypad),
        None,
        true,
        true,
        wake,
    )
    .await
}

/// Like [`run_app_nav`], but also registers an encoder input device driven by
/// `encoder` — **event-driven and poll-free by default**.
///
/// The navigator routes each active view's
/// [`input_group`](View::input_group) to the encoder, so focusable widgets can
/// be navigated *and edited in place* with three inputs (turn−, turn+, press) —
/// from a rotary encoder or three buttons that call
/// [`EncoderState::turn`](crate::indev::EncoderState::turn) /
/// [`click`](crate::indev::EncoderState::click) /
/// [`long_press`](crate::indev::EncoderState::long_press). LVGL owns the
/// navigate ↔ edit toggle.
///
/// The device is created in EVENT mode and the loop awaits the encoder's
/// **integrated wake** ([`EncoderState::wait`](crate::indev::EncoderState::wait)):
/// a decoded press from the producer task is read the instant the render
/// thread is scheduled, with no ~30 ms read-timer latency and no separate
/// signal to wire.
///
/// `encoder` must be `'static` (typically a `static EncoderState`). Never returns.
///
/// Combined loop, like [`run_app`] — see there for when to use
/// [`Ui::run_events_nav_encoder`] instead.
pub async fn run_app_nav_encoder<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    encoder: &'static crate::indev::EncoderState,
) -> ! {
    let ui = Ui::init(w, h, Buffers::partial(bufs));
    run_app_nav_inner(
        &ui,
        RenderConfig::default(),
        initial,
        None,
        Some(encoder),
        true,
        true,
        || encoder.wait(),
    )
    .await
}

/// No-wake closure for the timer-only loops: a future that never resolves, so
/// the inter-tick race always falls through to the normal tick.
fn no_wake() -> core::future::Pending<()> {
    core::future::pending()
}

/// Shared implementation of the navigation render loop.
///
/// `event_mode` selects EVENT mode for the keypad (read only on `wake`) vs
/// TIMER mode (LVGL polls). `wake` is raced against each inter-tick sleep; when
/// it resolves the loop reads the keypad and runs `update()` immediately.
///
/// `drive_refresh` picks the loop shape: `true` is the combined loop, which
/// calls [`Ui::refresh`] itself; `false` is the split loop, which only reads
/// the delay `refresh` published from the render thread's idle hook.
async fn run_app_nav_inner<Fut>(
    ui: &Ui,
    cfg: RenderConfig,
    initial: impl View,
    keypad: Option<&'static crate::indev::KeypadState>,
    encoder: Option<&'static crate::indev::EncoderState>,
    event_mode: bool,
    drive_refresh: bool,
    wake: impl Fn() -> Fut,
) -> !
where
    Fut: Future<Output = ()>,
{
    info!("UI render loop started (navigator)");
    ui.wait_ready().await;
    info!("Display ready");
    ui.apply(&cfg);

    // Register the keypad/encoder device (if any) BEFORE push_root, so the root
    // view's input_group binds to it. Held for the loop's lifetime; since the
    // loop never returns, its Drop never runs. A single loop uses at most one of
    // the two — the public entry points pass exactly one.
    let keypad_dev = keypad.and_then(|state| {
        let res = if event_mode {
            crate::indev::KeypadIndev::new_event(state)
        } else {
            crate::indev::KeypadIndev::new(state)
        };
        match res {
            Ok(kp) => Some(kp),
            Err(e) => {
                warn!("keypad indev create failed: {:?}", e);
                None
            }
        }
    });
    let encoder_dev = encoder.and_then(|state| {
        let res = if event_mode {
            crate::indev::EncoderIndev::new_event(state)
        } else {
            crate::indev::EncoderIndev::new(state)
        };
        match res {
            Ok(enc) => Some(enc),
            Err(e) => {
                warn!("encoder indev create failed: {:?}", e);
                None
            }
        }
    });

    let mut nav = crate::navigator::Navigator::new();
    nav.push_root(initial);

    let update_period = embassy_time::Duration::from_millis(cfg.update_period_ms);
    let mut next_update = embassy_time::Instant::now();
    loop {
        // Poll views on their own cadence, decoupled from the redraw rate.
        // `wake` (input arrived) also forces a poll, so a keypress is not held
        // for up to a full update period before the view sees it.
        let mut poll_now = embassy_time::Instant::now() >= next_update;

        // Frame pacing. The combined loop redraws here; the split loop leaves
        // that to the idle hook and only reads the delay LVGL asked for.
        let (delay, paced_by_present) = if drive_refresh {
            (ui.refresh() as u64, ui.present_blocks)
        } else {
            (ui.next_delay_ms.load(Ordering::Relaxed) as u64, false)
        };
        // A scan-out redraw already waited for the panel, so a sleep on top
        // would only cost frame rate — poll input without one (budget 0).
        let budget = if paced_by_present {
            0
        } else {
            delay.min(cfg.max_idle_ms).max(1)
        };

        // Input is read on every buffer kind: whether the present blocks is a
        // property of the *frame wait*, and must not decide whether a keypress
        // is ever seen.
        if wait_wake(budget, wake()).await {
            if let Some(kp) = &keypad_dev {
                kp.read();
            }
            if let Some(enc) = &encoder_dev {
                enc.read();
            }
            poll_now = true;
        }

        if !poll_now {
            continue;
        }
        // Resync rather than chase a backlog if a slow frame overran.
        next_update = embassy_time::Instant::now() + update_period;

        let action = nav
            .active_view_mut()
            .map(|v| v.update())
            .unwrap_or(Ok(NavAction::None))
            .unwrap_or_else(|e| {
                warn!("view update: {:?}", e);
                NavAction::None
            });

        let modal_action = nav
            .active_modal_mut()
            .map(|m| m.update())
            .unwrap_or(Ok(NavAction::None))
            .unwrap_or_else(|e| {
                warn!("modal update: {:?}", e);
                NavAction::None
            });

        // Event actions (from on_event trampoline) take priority.
        // Only process update/modal actions if no event action fired.
        let event_handled = nav.process_pending_event_action();
        if !event_handled {
            if !action.is_none() {
                nav.process_action(action);
            }
            if !modal_action.is_none() {
                nav.process_action(modal_action);
            }
        }

        // Drain toast requests posted from background tasks via
        // navigator::post_toast / post_dismiss_toast.
        nav.drain_toast_requests();

        // Auto-dismiss expired toasts; self-heal if the slot was orphaned.
        nav.tick_toast();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_time::{Duration as TimeDuration, Instant};

    /// Counts `update()` calls so the cadence logic can be observed directly.
    /// `create` is never reached — these tests drive `poll_update`, not LVGL.
    struct CountingView {
        updates: u32,
    }

    impl View for CountingView {
        fn create(&mut self, _container: &Obj<'static>) -> Result<(), WidgetError> {
            Ok(())
        }

        fn update(&mut self) -> Result<NavAction, WidgetError> {
            self.updates += 1;
            Ok(NavAction::None)
        }
    }

    // -- RenderConfig::with_target_fps -------------------------------------

    #[test]
    fn target_fps_sets_period_and_quarter_cap() {
        let cfg = RenderConfig::default().with_target_fps(60);
        assert_eq!(cfg.refresh_period_ms, Some(16));
        assert_eq!(cfg.max_idle_ms, 4);

        let cfg = RenderConfig::default().with_target_fps(30);
        assert_eq!(cfg.refresh_period_ms, Some(33));
        assert_eq!(cfg.max_idle_ms, 8);
    }

    #[test]
    fn target_fps_leaves_the_update_period_alone() {
        // The view poll rate is deliberately independent of the draw rate.
        let default_update = RenderConfig::default().update_period_ms;
        let cfg = RenderConfig::default().with_target_fps(120);
        assert_eq!(cfg.update_period_ms, default_update);
    }

    #[test]
    fn target_fps_zero_does_not_divide_by_zero() {
        let cfg = RenderConfig::default().with_target_fps(0);
        // fps is floored at 1, so this is the one-frame-per-second period.
        assert_eq!(cfg.refresh_period_ms, Some(1000));
        assert_eq!(cfg.max_idle_ms, 250);
    }

    #[test]
    fn target_fps_above_1000_clamps_period_and_cap_to_one() {
        // 1000/2000 truncates to 0; a zero period would make LVGL's refresh
        // timer fire without bound and a zero cap would busy-spin the loop.
        let cfg = RenderConfig::default().with_target_fps(2000);
        assert_eq!(cfg.refresh_period_ms, Some(1));
        assert_eq!(cfg.max_idle_ms, 1);
    }

    #[test]
    fn target_fps_cap_never_exceeds_the_period() {
        // The cap bounds a sleep that happens *after* the refresh, so a cap
        // larger than the period would stretch every cycle past its budget.
        for fps in [1u32, 15, 24, 30, 50, 60, 90, 120, 144, 240, 1000, 5000] {
            let cfg = RenderConfig::default().with_target_fps(fps);
            let period = cfg
                .refresh_period_ms
                .expect("with_target_fps always sets a period");
            assert!(period >= 1, "fps {fps}: period {period} must be positive");
            assert!(
                cfg.max_idle_ms >= 1,
                "fps {fps}: cap {} must be positive",
                cfg.max_idle_ms
            );
            assert!(
                cfg.max_idle_ms <= period as u64,
                "fps {fps}: cap {} exceeds period {period}",
                cfg.max_idle_ms
            );
        }
    }

    // -- poll_update -------------------------------------------------------

    #[test]
    fn poll_update_skips_before_the_deadline() {
        let mut view = CountingView { updates: 0 };
        let period = TimeDuration::from_millis(10);
        let start = Instant::from_millis(1_000);
        let mut next = start + period;

        poll_update(&mut view, start, period, &mut next);

        assert_eq!(view.updates, 0);
        assert_eq!(next, start + period, "deadline must not move early");
    }

    #[test]
    fn poll_update_runs_on_the_deadline_and_rearms() {
        let mut view = CountingView { updates: 0 };
        let period = TimeDuration::from_millis(10);
        let due = Instant::from_millis(1_000);
        let mut next = due;

        poll_update(&mut view, due, period, &mut next);

        assert_eq!(view.updates, 1);
        assert_eq!(next, due + period);
    }

    #[test]
    fn poll_update_resyncs_instead_of_chasing_a_backlog() {
        let mut view = CountingView { updates: 0 };
        let period = TimeDuration::from_millis(10);
        let due = Instant::from_millis(1_000);
        let mut next = due;

        // A slow frame overran by ten periods.
        let late = due + TimeDuration::from_millis(100);
        poll_update(&mut view, late, period, &mut next);

        // One update, not one per missed period, and the next deadline is a
        // period from *now* — `next += period` would have left it in the past.
        assert_eq!(view.updates, 1);
        assert_eq!(next, late + period);

        // Proof that no backlog was queued: a poll half a period later is a
        // no-op. With a chasing deadline it would fire again immediately.
        let soon = late + TimeDuration::from_millis(5);
        poll_update(&mut view, soon, period, &mut next);
        assert_eq!(view.updates, 1);

        // ...and the poll after the new deadline does fire.
        poll_update(&mut view, late + period, period, &mut next);
        assert_eq!(view.updates, 2);
    }
}
