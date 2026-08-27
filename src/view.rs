// SPDX-License-Identifier: MIT OR Apache-2.0
//! View trait and navigation primitives.
//!
//! The [`View`](crate::view::View) trait defines a single screen of UI with a repeatable
//! lifecycle: `create` → `update` → `on_event` → `will_hide`, cycling
//! on each navigation transition. See `docs/spec-navigation.md`.

use alloc::boxed::Box;
use core::cell::UnsafeCell;
use core::ffi::c_void;
use core::time::Duration;
use embassy_futures::block_on;
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
/// called [`Self::init`] — not an embassy task.
///
/// ```ignore
/// let ui = make_static!(Ui::init(W, H, Buffers::full(a, b, FRAME_BYTES)));
/// let view = make_static!(MyView::default());
/// ui.bind(view, &RenderConfig::default().with_target_fps(60)).unwrap();
/// loop { let _ = ui.timer_handler(); }
/// ```
#[derive(Debug)]
pub struct Ui {
    driver: LvglDriver,
    /// DIRECT: refresh already blocked on vblank; skip the extra park.
    present_blocks: bool,
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
        }
    }

    /// Wait until the display driver reports ready — on ESP32 that is the flush
    /// task starting, on host it is immediate.
    pub async fn wait_ready(&self) {
        DISPLAY_READY.wait().await;
    }

    /// One LVGL tick. Returns recommended delay until the next call (ms).
    /// Blocks on scan-out (vblank). Render thread only.
    pub fn timer_handler(&self) -> u32 {
        self.driver.timer_handler()
    }

    /// Create the view. [`DISPLAY_READY`] must already have been signalled
    /// (`Buffers::full` does that in [`init`](Self::init)).
    pub fn bind<V: View>(&self, view: &mut V, cfg: &RenderConfig) -> Result<(), ()> {
        self.bind_view(view, cfg)
    }

    /// Apply a [`RenderConfig`]'s redraw period, if it sets one.
    fn apply(&self, cfg: &RenderConfig) {
        if let Some(ms) = cfg.refresh_period_ms
            && !crate::display::set_refresh_period(ms)
        {
            warn!("could not set refresh period to {} ms", ms);
        }
    }

    /// [`bind`](Self::bind), then loop refresh. Render thread; never returns.
    /// DIRECT skips the post-refresh park (vblank was the wait).
    pub fn run<V: View>(self, mut view: V, cfg: RenderConfig) -> ! {
        block_on(self.wait_ready());
        info!("Display ready");
        if self.bind_view(&mut view, &cfg).is_err() {
            loop {
                block_on(Timer::after(embassy_time::Duration::from_secs(60)));
            }
        }

        let update_period = embassy_time::Duration::from_millis(cfg.update_period_ms);
        let mut next_update = embassy_time::Instant::now();
        loop {
            poll_update(&mut view, update_period, &mut next_update);
            self.idle(&cfg);
        }
    }

    /// [`Navigator`](crate::navigator::Navigator) loop. Render thread; never returns.
    pub fn run_nav(self, initial: impl View, cfg: RenderConfig) -> ! {
        run_app_nav_inner(self, cfg, initial, None, None, false, no_wake)
    }

    /// Create the view on the active screen. `Err` if widget create failed
    /// (caller parks).
    fn bind_view<V: View>(&self, view: &mut V, cfg: &RenderConfig) -> Result<(), ()> {
        self.apply(cfg);
        let screen_handle = unsafe { lv_screen_active() };
        assert!(
            !screen_handle.is_null(),
            "no active screen after display init"
        );
        let container = Obj::from_raw_non_owning(screen_handle);
        if let Err(e) = view.create(&container) {
            warn!("Could not create LVGL widgets: {:?}, disabling UI", e);
            return Err(());
        }
        register_view_events(view, &container);
        Ok(())
    }

    fn idle(&self, cfg: &RenderConfig) {
        let delay = (self.driver.timer_handler() as u64).clamp(1, cfg.max_idle_ms);
        if !self.present_blocks {
            block_on(Timer::after(embassy_time::Duration::from_millis(delay)));
        }
    }
}

fn poll_update<V: View>(
    view: &mut V,
    update_period: embassy_time::Duration,
    next_update: &mut embassy_time::Instant,
) {
    if embassy_time::Instant::now() >= *next_update {
        // Resync rather than chase a backlog if a slow frame overran.
        *next_update = embassy_time::Instant::now() + update_period;
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

/// [`Ui::init`] + [`Ui::run`] with default [`RenderConfig`]. Render thread.
pub fn run_app<V: View, const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    view: V,
) -> ! {
    info!("UI render thread started");
    Ui::init(w, h, Buffers::partial(bufs)).run(view, RenderConfig::default())
}

/// Run the LVGL render loop with navigation support.
///
/// Like [`run_app`], but creates a [`Navigator`](crate::navigator::Navigator)
/// that processes [`NavAction`] values from `update()` and `on_event()`.
/// Use this for multi-screen applications that need push/pop/replace/modal.
///
/// `initial` is the root view. Never returns.
pub fn run_app_nav<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
) -> ! {
    run_app_nav_inner(
        Ui::init(w, h, Buffers::partial(bufs)),
        RenderConfig::default(),
        initial,
        None,
        None,
        false,
        no_wake,
    )
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
pub fn run_app_nav_keypad<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    keypad: &'static crate::indev::KeypadState,
) -> ! {
    run_app_nav_inner(
        Ui::init(w, h, Buffers::partial(bufs)),
        RenderConfig::default(),
        initial,
        Some(keypad),
        None,
        false,
        no_wake,
    )
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
pub fn run_app_nav_keypad_events<const BYTES: usize, Fut>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    keypad: &'static crate::indev::KeypadState,
    wake: impl Fn() -> Fut,
) -> !
where
    Fut: core::future::Future<Output = ()>,
{
    run_app_nav_inner(
        Ui::init(w, h, Buffers::partial(bufs)),
        RenderConfig::default(),
        initial,
        Some(keypad),
        None,
        true,
        wake,
    )
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
pub fn run_app_nav_encoder<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    encoder: &'static crate::indev::EncoderState,
) -> ! {
    run_app_nav_inner(
        Ui::init(w, h, Buffers::partial(bufs)),
        RenderConfig::default(),
        initial,
        None,
        Some(encoder),
        true,
        || encoder.wait(),
    )
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
fn run_app_nav_inner<Fut>(
    ui: Ui,
    cfg: RenderConfig,
    initial: impl View,
    keypad: Option<&'static crate::indev::KeypadState>,
    encoder: Option<&'static crate::indev::EncoderState>,
    event_mode: bool,
    wake: impl Fn() -> Fut,
) -> !
where
    Fut: core::future::Future<Output = ()>,
{
    info!("UI render thread started (navigator)");
    block_on(ui.wait_ready());
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

        let delay = ui.timer_handler() as u64;
        if !ui.present_blocks {
            match block_on(with_timeout(
                embassy_time::Duration::from_millis(delay.clamp(1, cfg.max_idle_ms)),
                wake(),
            )) {
                Ok(()) => {
                    if let Some(kp) = &keypad_dev {
                        kp.read();
                    }
                    if let Some(enc) = &encoder_dev {
                        enc.read();
                    }
                    poll_now = true;
                }
                Err(_timeout) => {} // normal tick
            }
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
