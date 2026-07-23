# Navigation Specification

**Status**: Implemented
**LVGL version**: v9.5

How oxivgl manages screen transitions, navigation stacks, and modal
overlays. oxivgl provides a `View` + `Navigator` model for multi-screen
applications, built on LVGL's screen and layer primitives. This document
specifies the shipped design.

---

## 1. Motivation

The `View` trait was designed early in oxivgl's life as a minimal
single-screen abstraction. It serves well for simple examples but has
structural limitations:

| Limitation | Impact |
|---|---|
| Single-screen only | No push/pop navigation, no back stack |
| `create()` returns `Self` | Cannot rebuild widgets after navigation (create runs once) |
| No lifecycle hooks | No way to save widget state before teardown or restore it after |
| Raw pointer handles for dynamic widgets | Error-prone, no Rust-side invalidation |
| Poll-only `update()` | No framework-level dirty tracking |

The `View` trait is a navigation-aware lifecycle, and a `Navigator`
manages a stack of views with push/pop transitions and modal overlays,
built on LVGL's screen and layer primitives (`lv_screen_load_anim`,
`lv_layer_top`, `lv_obj_clean`).

---

## 2. Design Principles

These principles are additive to `spec-api-vision.md`. Where they
overlap, the API vision spec takes precedence.

1. **Views are values, not singletons.** A view is a Rust struct
   that owns its state. Multiple instances of the same view type can
   coexist on the navigation stack with different state.

2. **Widget building is repeatable.** A view's widget tree can be
   destroyed and rebuilt from preserved struct state. This enables
   push/pop navigation where the background view's widgets are torn
   down and later recreated.

3. **The navigator owns the stack.** All view instances live inside
   the navigator. The application never holds a direct reference to a
   view on the stack — it interacts through the navigator's API.

4. **Modals overlay, full-screen views replace.** Full-screen
   navigation destroys the current widget tree. Modal navigation
   preserves it and renders on `lv_layer_top()`.

5. **`alloc` is acceptable.** oxivgl already uses `Box`, `Vec`, `Rc`
   throughout the core library. The navigator uses `Vec<Box<dyn
   AnyView>>` for the stack. No new allocator dependency is
   introduced.

---

## 3. The `View` Trait

The `View` trait provides a repeatable `create`, lifecycle hooks, and
`NavAction` returns. Every screen of UI implements `View`.

```rust
/// A single view of UI (one screen or modal in a navigation stack).
///
/// The navigator calls lifecycle methods in this order:
///
/// 1. **Construction** — caller creates the struct (e.g. `Default::default()`)
/// 2. [`create`] — build widgets into `container`; may be called multiple times
/// 3. [`register_events_on`] — attach the event trampoline to `container`
/// 4. [`did_show`] — post-creation setup (optional)
/// 5. [`update`] / [`on_event`] — per-tick polling and LVGL event dispatch
/// 6. [`will_hide`] — save transient state before widget teardown (optional)
/// 7. Widget tree destroyed (navigator calls `lv_obj_clean`)
///
/// Steps 2–7 repeat on each push/pop cycle for views that remain on
/// the stack.
pub trait View: Sized + 'static {
    /// Build all LVGL widgets for this view into `container`.
    ///
    /// Called each time this view becomes the active (topmost) view —
    /// both on initial push and when a view above it is popped. The
    /// implementation should read its own struct fields to restore any
    /// state that was saved in [`will_hide`].
    ///
    /// `container` is the LVGL screen object (for full-screen navigation)
    /// or a layer object (for modals). Widgets created as children of
    /// `container` are automatically destroyed when the navigator
    /// transitions away.
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;

    /// Refresh widget values from application state. Called every render
    /// tick while this view is the active full-screen, or while a modal
    /// is open.
    ///
    /// This is the primary integration point for external events (see §5.2).
    /// Poll application event channels here and return a `NavAction` to
    /// trigger navigation. Default returns `NavAction::None`.
    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }

    /// Handle a bubbled LVGL event. Return a `NavAction` to trigger
    /// navigation, or `NavAction::None` to do nothing. Default returns
    /// `NavAction::None`.
    ///
    /// The navigator installs a trampoline on the container object after
    /// each [`create`] call. Widgets that need to trigger `on_event` must
    /// set `ObjFlag::EVENT_BUBBLE`.
    fn on_event(&mut self, _event: &Event) -> NavAction {
        NavAction::None
    }

    /// Called before this view's widget tree is destroyed (navigating
    /// away or being replaced). Save any transient widget state (scroll
    /// position, text input content, selection index) back to struct
    /// fields here.
    ///
    /// Default is a no-op. Not called when the view is permanently
    /// removed from the stack (e.g. `replace` or final `pop`).
    fn will_hide(&mut self) {}

    /// Called after this view becomes visible again (navigated back to
    /// via pop). Widgets have been rebuilt by [`create`] — use this for
    /// post-creation setup that depends on the view being active.
    ///
    /// Default is a no-op.
    fn did_show(&mut self) {}

    /// Register event handlers. Called once after `create`, with the
    /// same `container` that was passed to `create`. Default attaches
    /// the view's event trampoline to `container`, so bubbled events
    /// from any descendant reach `on_event`.
    ///
    /// Receiving `container` as an argument — rather than reading
    /// `lv_screen_active()` — makes the default correct for modals as
    /// well as full-screen views: the navigator passes the modal
    /// backdrop, the toast container, or the screen, whichever applies.
    fn register_events_on(&mut self, container: &Obj<'static>) {
        register_event_on(self, container.lv_handle());
    }

    /// Focus group containing this view's focusable widgets (modals
    /// only). When non-`None`, the navigator activates this group on
    /// modal open (sets it as default + binds it to all KEYPAD /
    /// ENCODER input devices) and restores the previous focus state on
    /// dismiss. Default `None`. See §4.2 for the full lifecycle.
    fn input_group(&self) -> Option<GroupRef> { None }
}
```

### 3.1 Single-View Usage

A single-screen example implements `View` directly; the harness pushes it
as the root:

```rust
#[derive(Default)]
struct MyExample { label: Option<Label<'static>> }
impl View for MyExample {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        self.label = Some(Label::new(container)?);
        Ok(())
    }
}
example_main!(MyExample::default());
```

Widget fields are `Option<W>` because the struct exists before `create`
is called, and `create` may run again after a pop. `#[derive(Default)]`
supplies the initial `None`s. `example_main!` takes an expression that
produces the initial view instance.

---

## 4. The `Navigator`

Manages a stack of views with push/pop/replace semantics and modal
overlays.

```rust
/// View navigation stack with modal overlay support.
///
/// The navigator owns all view instances. Views lower in the stack
/// have their widget trees destroyed but their struct state preserved.
/// Only the topmost view (and any active modal/toast) have live widgets.
pub struct Navigator {
    // Full-screen navigation stack. Index 0 is the root view.
    stack: Vec<ViewEntry>,
    // Currently active modal, if any. Rendered inside `modal_backdrop`
    // which itself is a child of lv_layer_top().
    modal: Option<Box<dyn AnyView>>,
    modal_backdrop: Option<Obj<'static>>,
    // Focus state captured on modal open (if input_group was Some);
    // restored on dismiss. See §4.2.
    saved_focus: Option<SavedFocus>,
    // Currently active global toast, if any. Rendered on the current
    // topmost surface (active screen, or modal backdrop while a modal is
    // open) and re-parented across nav/modal changes — not on
    // lv_layer_sys(). See §4.3.
    toast: Option<Box<dyn AnyView>>,
    toast_container: Option<Obj<'static>>,
    // Auto-dismiss deadline in wrap-aware tick milliseconds (`get_tick_ms`),
    // not `Instant`. See §4.3.
    toast_deadline_ms: Option<u32>,
    // FIFO of timed toasts waiting behind the active one (capacity 4).
    toast_queue: VecDeque<PendingToast>,
}
```

### 4.1 Full-Screen Navigation

```rust
impl Navigator {
    /// Push the root view. Called once at startup (by `run_app_nav`); the
    /// root gets its own created + loaded screen so pop-to-root and toast
    /// re-parenting are uniform.
    pub fn push_root(&mut self, view: impl View);

    /// Push a new view onto the stack. Each view owns its own
    /// `Obj<'static>` screen (via `Screen::create`), so the previous
    /// screen's tree is destroyed *after* the new one is built.
    ///
    /// 1. Calls `will_hide()` on the current top view.
    /// 2. Creates and loads a new screen, then calls `create(container)`
    ///    and `register_events_on(container)` on the new view.
    /// 3. Re-parents any active toast onto the new screen.
    /// 4. Destroys the previous view's widget tree (`lv_obj_clean`) — its
    ///    struct state stays on the stack.
    /// 5. Calls `did_show()` on the new view.
    ///
    /// `anim` controls the screen transition animation. Pass `None` for
    /// an instant switch.
    pub fn push(&mut self, view: impl View, anim: Option<ScreenAnim>);

    /// Pop the current view and return to the previous one. The restored
    /// view's screen is loaded and its widgets rebuilt via `create`; the
    /// popped view's screen is cleaned up afterward.
    ///
    /// 1. Loads the restored view's screen and rebuilds it (`create`,
    ///    `register_events_on`), re-parenting any active toast onto it.
    /// 2. Drops the popped view and destroys its widget tree.
    /// 3. Calls `did_show()` on the restored view.
    ///
    /// Returns `Err(NavigationError::StackEmpty)` if only one view
    /// remains (the root view cannot be popped).
    pub fn pop(&mut self, anim: Option<ScreenAnim>) -> Result<(), NavigationError>;

    /// Replace the current view without preserving it on the stack.
    ///
    /// The current view is dropped (not preserved). The new view
    /// takes its place at the same stack depth. Use this for
    /// non-reversible transitions (e.g. login → home).
    pub fn replace(&mut self, view: impl View, anim: Option<ScreenAnim>);

    /// Number of views on the stack.
    pub fn depth(&self) -> usize;
}
```

### 4.2 Modal Navigation

```rust
impl Navigator {
    /// Show a modal overlay on top of the current view.
    ///
    /// The current view's widget tree stays alive and visible
    /// underneath. The modal's widgets are created on `lv_layer_top()`.
    /// A full-size transparent click-absorbing backdrop is inserted
    /// automatically to prevent interaction with the view below.
    ///
    /// Only one modal can be active at a time. Calling `modal()` while
    /// a modal is already open replaces it.
    pub fn modal(&mut self, view: impl View);

    /// Dismiss the current modal overlay.
    ///
    /// Destroys the modal's widgets and the backdrop. The underlying
    /// view regains input focus. Returns `Err` if no modal is active.
    pub fn dismiss_modal(&mut self) -> Result<(), NavigationError>;

    /// Whether a modal is currently showing.
    pub fn has_modal(&self) -> bool;
}
```

**Backdrop implementation**: when `modal()` is called, the navigator
creates a full-size `Obj` on `lv_layer_top()` with
`ObjFlag::CLICKABLE` set. This absorbs all touch/click events,
preventing them from reaching the view below. The modal's widgets are
then created as children of this backdrop object. Dismissing the modal
drops the backdrop `Obj`, which cascades the deletion to every modal
widget — a single root, no `lv_obj_clean(layer_top)` needed.

Optional: the backdrop can be styled with a semi-transparent dark fill
(`opa(128)`, `bg_color(Color::black())`) on the modal's own widgets
for a standard dim effect. The backdrop itself is transparent.

**Event registration.** `View::register_events_on` defaults to attaching
the trampoline on the container the navigator passed — which for a
modal is the backdrop, **not** `lv_screen_active()`. This is what
makes default modal event handling correct: every bubbled event from a
modal widget reaches the modal's `on_event`, and no handlers are
attached to the background view's screen (where they would dangle on a
push/pop transition).

Modals that catch events on intermediate widgets (containers that don't
auto-bubble) can still override `register_events_on` and call
`register_event_on(self, intermediate.handle())` for those.

**Focus management (OSD-style modals).** A modal that needs keyboard or
encoder input — e.g. an on-screen menu the user navigates with hardware
buttons — owns its own [`Group`](../src/group.rs) and exposes it via
`View::input_group`. The navigator does the rest:

1. **On `modal()` open**, if `view.input_group()` returns `Some(group)`:
   - capture the current default group + the per-indev (KEYPAD /
     ENCODER) group bindings into a private `SavedFocus`;
   - call `group.set_default()`;
   - call `group.assign_to_keyboard_indevs()`.

2. **On `dismiss_modal()`**, the captured `SavedFocus` is restored:
   - previous default group reinstated via `lv_group_set_default`;
   - each previously-bound indev re-bound to its previous group via
     `lv_indev_set_group`.

The app never has to touch `lv_indev_*` / `lv_group_set_default`
directly, and key events route to the OSD while it is open and back to
the background view as soon as it is dismissed. A modal that does *not*
need key input simply returns `None` from `input_group` (the default)
and the navigator leaves focus state untouched.

### 4.3 Global Status Overlay (Toast)

The toast is a **passive, page-independent** status surface, distinct
from `modal`. It is used for messages that must appear regardless of
which view is active (e.g. "No SD card" raised at boot before any
particular page has focus) and that must not steal input from the view
beneath.

```rust
impl Navigator {
    /// Show a global passive toast on the active screen.
    ///
    /// Persists across push/replace/pop. Registers no input handlers.
    /// Every widget the view creates has `CLICKABLE` cleared, so
    /// touches pass through to the view beneath.
    ///
    /// `duration` is an optional auto-dismiss timeout, owned by the
    /// navigator. `None` means the caller must dismiss explicitly.
    /// Calling `show_toast` while one is active replaces it.
    pub fn show_toast(&mut self, view: impl View, duration: Option<Duration>);

    /// Dismiss the active toast.
    pub fn dismiss_toast(&mut self) -> Result<(), NavigationError>;

    /// Whether a toast is currently showing.
    pub fn has_toast(&self) -> bool;

    /// Called once per render-loop iteration by `run_app_nav`.
    /// Auto-dismisses on deadline; self-heals if the toast container
    /// was destroyed externally.
    pub fn tick_toast(&mut self);
}
```

**Surface choice.** Toasts render on the **active screen** (or the modal
backdrop while a modal is open), as an ordinary child of the normal widget
tree — **not** on `lv_layer_sys()`. In `LV_DISPLAY_RENDER_MODE_PARTIAL`
(ESP32) the system layer is not composited reliably onto passive redraws, so
a toast on an otherwise-static screen could silently fail to appear (worst on
the first cold boot); ordinary screen content, by contrast, composites every
frame. Rendering the toast through the normal screen path makes it as reliable
as any other widget. Each `show_toast` creates its own dedicated container on
the current topmost surface; on dismissal only that container is deleted.
`reattach_toast` (see **Persistence**) re-parents the container whenever the
topmost surface changes, so the toast lifecycle stays separate from modal and
page teardown — dismissing a modal or switching pages never deletes the toast.

**Passivity contract.** The navigator enforces input-transparency:
- `register_events()` is **never** called on the toast view —
  preventing the dangling-handler hazard that `Modal` has (where
  default `register_events` would register on `lv_screen_active()`,
  the *background* view's screen).
- After `create()`, `CLICKABLE` and `CLICK_FOCUSABLE` are cleared on
  the toast container and every descendant. Touches pass through.

A toast view's `update()` and `on_event()` are *not* polled by
`run_app_nav` — toasts are display-only.

**Persistence.** `reattach_toast` re-parents the toast container onto the
current topmost surface after every change to it — `push` / `replace` / `pop`
(→ the new active screen) and modal open / dismiss (→ the backdrop, so the
toast stays above the modal; back to the screen on dismiss). The re-parent
runs before the previous surface's widget tree is destroyed, and re-appends the
toast as the last (top) child, so the same toast instance survives page
switches and modal teardown with no recreation, always on top.

**Auto-dismiss.** When `duration` is `Some`, the navigator records
`Instant::now() + duration` and dismisses when `tick_toast` next runs
after the deadline. The render loop calls `tick_toast` every
iteration; the dismiss latency is therefore the loop period
(~4×`LVGL_TICK_MS` ≈ 33 ms).

**Self-heal.** If something else destroys the toast container (e.g.
a third-party clear of its surface), `tick_toast` detects the
stale handle via `lv_obj_is_valid` and drops the orphaned view so the
slot becomes reusable.

**Raising from any task.** Background async tasks that hold no
`Navigator` handle (BLE listener, sensor poller, app shell at boot
before any view is on screen) can still raise toasts via two free
functions:

```rust
pub fn post_toast<V: View + Send>(view: V, duration: Option<Duration>);
pub fn post_dismiss_toast();
```

Both enqueue into a library-owned static `Channel`
(capacity `TOAST_QUEUE_CAPACITY = 4`, `CriticalSectionRawMutex`).
`run_app_nav` calls `nav.drain_toast_requests()` once per render-loop
iteration before `tick_toast`. The drained request flows through the
same `show_toast_boxed` / `dismiss_toast` paths as `NavAction::ShowToast`
— so all the passivity / persistence / auto-dismiss guarantees apply
identically; only the initiator differs.

The `Send` bound on the View type rules out keeping live `Obj` wrappers
in toast struct fields (raw pointers are `!Send`). In practice this is
not a constraint — toast views are typically simple config structs
(text, color, icon source) that build their widgets inside
`View::create` from the supplied `&Obj<'static>` container; the widgets
then live as children of the toast container, owned by LVGL.

If the queue is full when `post_toast` runs, the new request is dropped
with a logged warning rather than blocking — toasts are notifications,
not data; a dropped duplicate is preferable to async deadlock.

This closes the "no draining view" gap: a "No SD card" warning raised
at boot from an SD-card task no longer needs the currently-active view
to opt into draining a status channel.

**Distinction from `Modal`.**

| Property               | `Modal`                              | Toast                          |
|------------------------|--------------------------------------|--------------------------------|
| Surface                | backdrop on `lv_layer_top()`         | active screen / modal backdrop (re-parented) |
| Input                  | Absorbs (backdrop)                   | Transparent                    |
| Persists across nav    | Yes (until `dismiss_modal`)          | Yes (re-parented each transition) |
| Lifetime tied to view  | No (lives until dismissed)           | No                             |
| Auto-dismiss           | App-managed                          | Navigator-managed (optional)   |
| Receives events        | Yes                                  | No (handlers never registered) |
| Intended for           | Page-specific dialogs / future OSD   | Global status notifications    |

### 4.4 Screen Animations

```rust
/// Screen transition animation, wrapping `lv_screen_load_anim`.
pub struct ScreenAnim {
    /// Animation type (slide, fade, move, etc.)
    pub anim_type: ScreenAnimType,
    /// Duration in milliseconds.
    pub duration_ms: u32,
    /// Delay before animation starts, in milliseconds.
    pub delay_ms: u32,
}

/// Mirrors `lv_screen_load_anim_t` values.
pub enum ScreenAnimType {
    None,
    OverLeft,
    OverRight,
    OverTop,
    OverBottom,
    MoveLeft,
    MoveRight,
    MoveTop,
    MoveBottom,
    FadeIn,
    FadeOut,
}
```

### 4.5 `NavAction` — Navigation Requests

Views cannot call Navigator methods directly (the navigator owns
the view — `&mut Navigator` and `&mut self` cannot coexist). Instead,
both `update` and `on_event` return `NavAction`. The render loop
processes the action after the method returns.

```rust
/// Navigation action requested by a view.
pub enum NavAction {
    /// No navigation requested.
    None,
    /// Push a new view onto the stack.
    Push(Box<dyn AnyView>, Option<ScreenAnim>),
    /// Pop the current view.
    Pop(Option<ScreenAnim>),
    /// Replace the current view.
    Replace(Box<dyn AnyView>, Option<ScreenAnim>),
    /// Show a modal overlay (page-specific, takes input).
    Modal(Box<dyn AnyView>),
    /// Dismiss the current modal.
    DismissModal,
    /// Show a global passive toast on the active screen (see §4.3).
    ShowToast(Box<dyn AnyView>, Option<Duration>),
    /// Dismiss the active global toast.
    DismissToast,
}
```

Convenience constructors reduce boilerplate: `NavAction::push(view)`,
`NavAction::replace(view)`, `NavAction::modal(view)`,
`NavAction::show_toast(view, duration)`, plus `NavAction::is_none()`.

Both `update` and `on_event` return `NavAction`:

```rust
fn update(&mut self) -> Result<NavAction, WidgetError>;
fn on_event(&mut self, event: &Event) -> NavAction;
```

Navigation can be triggered from two sources:
- **Widget events** (on-screen button click) — via `on_event`
- **External events** (hardware button, BLE command, sensor threshold)
  — via `update`, which polls the application event channel (see §5.2)

The render loop collects actions from both and dispatches to the
navigator. If both produce an action in the same tick, `on_event`
takes priority (it fires during `lv_timer_handler`, before `update`
on the next iteration).

---

## 5. Render Loop and Task Architecture

### 5.1 Embassy Task Model

`run_app` is an **embassy async task** — one of several concurrent
tasks in a real application. It does not own the application; it owns
the LVGL render loop and the navigator.

```
┌──────────────────┐  ┌──────────────┐  ┌──────────────┐
│ run_app task     │  │ BLE task     │  │ button task  │
│  ├ Navigator     │  │              │  │  (debounced) │
│  ├ update()      │  │              │  │              │
│  └ timer_handler │  │              │  │              │
└────────┬─────────┘  └──────┬───────┘  └──────┬───────┘
         │                   │                  │
         └──── channel ──────┴──────────────────┘
              (app-defined event type)
```

Other tasks (BLE, sensors, hardware buttons with proper debouncing,
timers, network) produce events and make them available to the LVGL
task. The delivery mechanism is an **application concern** — oxivgl
does not define or constrain it. Common patterns:

| Pattern | Mechanism | Producer side | Consumer side (`update()`) |
|---------|-----------|---------------|---------------------------|
| Channel | `embassy_sync::Channel` | `.send(evt).await` | `.try_recv()` |
| Shared state | `Mutex<AppState>` | `lock()`, write fields | `lock()`, read fields |
| Flags | `AtomicBool` / `Signal` | `.store(true)` / `.signal()` | `.load()` / `.try_take()` |

All patterns work with `View::update()`, which is a periodic poll
point agnostic to how data arrives. oxivgl provides `run_app`,
`View`, and `Navigator`; the application provides the event
infrastructure and the tasks that feed it.

To reduce latency when events arrive between render ticks, the
event-driven entry point `run_app_nav_keypad_events` accepts an **async
wake closure** (see §5.2) that is raced against the inter-tick timer,
short-circuiting the sleep when the closure resolves.

### 5.2 The `run_app` family

Each entry point initializes LVGL, builds the render loop, and never
returns. There is no single universal `run_app(...wake...)`; the shape
depends on whether the app navigates and how input arrives:

```rust
/// Single view, no navigation. Ignores NavAction (asserts it is None).
pub async fn run_app<V: View, const BYTES: usize>(
    w: i32, h: i32, bufs: &'static mut LvglBuffers<BYTES>, view: V) -> !;

/// Navigation entry point: creates the Navigator, pushes `initial` as
/// the root (`push_root`), and processes NavActions each tick.
pub async fn run_app_nav<const BYTES: usize>(
    w: i32, h: i32, bufs: &'static mut LvglBuffers<BYTES>, initial: impl View) -> !;

/// Navigation + keypad input (timer-paced).
pub async fn run_app_nav_keypad<const BYTES: usize>(
    w: i32, h: i32, bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View, keypad: KeypadIndev) -> !;

/// Navigation + keypad input, event-driven: `wake` is raced against the
/// inter-tick timer to run the loop sooner when input arrives.
pub async fn run_app_nav_keypad_events<const BYTES: usize, Fut: Future<Output = ()>>(
    w: i32, h: i32, bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View, keypad: KeypadIndev, wake: impl Fn() -> Fut) -> !;

/// Navigation + encoder input (three inputs: turn−, turn+, press), driving
/// both focus navigation and in-place edit. Always event-driven with an
/// integrated wake — the loop reads the instant a decoded press arrives, so
/// there is no separate `wake` closure to wire.
pub async fn run_app_nav_encoder<const BYTES: usize>(
    w: i32, h: i32, bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View, encoder: &'static EncoderState) -> !;
```

All navigation variants share one inner loop (`run_app_nav_inner`):

1. `update()` the active view (and the modal, if present), collecting
   their `NavAction`s.
2. Drive `lv_timer_handler` four times, each iteration racing the tick
   timer against `wake()` via `embassy_time::with_timeout`, so an early
   wake shortens the sleep. Widget events fire here, producing pending
   `on_event` actions.
3. `process_pending_event_action()` first — `on_event` actions win over
   `update` actions in the same tick — then the `update` / modal actions.
4. `drain_toast_requests()` (cross-task `post_toast`), then `tick_toast()`.

The `wake` closure is called fresh each tick and only needs
`core::future::Future` — no dependency on a specific async runtime.

### 5.3 Real Application Example

A typical multi-task application using channel events with wake signal:

```rust
// Wake signal — shared between producer tasks and run_app
static WAKE: Signal<CriticalSectionRawMutex, ()> = Signal::new();

// Event channel (one of potentially many data paths)
static EVENT_CHANNEL: Channel<CriticalSectionRawMutex, AppEvent, 8> =
    Channel::new();

// View that consumes events
struct DashboardView {
    voltage_label: Option<Label<'static>>,
}

impl View for DashboardView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        self.voltage_label = Some(Label::new(container)?);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        // Drain the channel — may have 0..N events since last tick
        while let Ok(evt) = EVENT_CHANNEL.try_recv() {
            match evt {
                AppEvent::ButtonBack => return Ok(NavAction::Pop(None)),
                AppEvent::SensorReading(v) => {
                    if let Some(lbl) = &self.voltage_label {
                        lbl.set_text_fmt(fmt!("{:.1}V", v));
                    }
                }
                AppEvent::BleDisconnected => {
                    return Ok(NavAction::modal(ReconnectDialog::new()));
                }
                _ => {}
            }
        }
        Ok(NavAction::None)
    }
}

// Button task — proper debouncing, signals wake after send
#[embassy_executor::task]
async fn button_task(back_pin: Input<'static>) {
    loop {
        back_pin.wait_for_falling_edge().await;
        Timer::after(Duration::from_millis(50)).await; // debounce
        if back_pin.is_low() {
            EVENT_CHANNEL.send(AppEvent::ButtonBack).await;
            WAKE.signal(()); // wake run_app immediately
        }
    }
}

// Entry point — async closure wakes on signal
#[embassy_executor::task]
async fn ui_task(bufs: &'static mut LvglBuffers<BUF_BYTES>, keypad: KeypadIndev) -> ! {
    // The event-driven entry point is where the wake closure lives.
    run_app_nav_keypad_events(320, 240, bufs, DashboardView::default(), keypad,
        async || WAKE.wait().await,
    ).await
}
```

An alternative using shared state instead of channels:

```rust
static APP_STATE: Mutex<CriticalSectionRawMutex, RefCell<AppState>> =
    Mutex::new(RefCell::new(AppState::default()));

impl View for DashboardView {
    fn update(&mut self) -> Result<NavAction, WidgetError> {
        let state = APP_STATE.lock(|cell| cell.borrow().clone());
        if let Some(lbl) = &self.voltage_label {
            lbl.set_text_fmt(fmt!("{:.1}V", state.voltage));
        }
        Ok(NavAction::None)
    }
}

// Sensor task — updates shared state, signals wake
#[embassy_executor::task]
async fn sensor_task(adc: Adc<'static>) {
    loop {
        let reading = adc.read().await;
        APP_STATE.lock(|cell| cell.borrow_mut().voltage = reading);
        WAKE.signal(());
    }
}
```

Both patterns work. The choice is application-level. The wake source
is the same either way — producer tasks signal after updating their
data, `run_app` wakes and calls `update()`, the view reads whatever
is available.

The async closure can wrap any awaitable source:

```rust
// Signal
async || WAKE.signal.wait().await

// Channel (wake when a message is available)
async || { let _ = CHANNEL.receive().await; }

// GPIO pin directly (simple cases without a separate button task)
async || BACK_PIN.wait_for_falling_edge().await

// Timer-only (no external wake — for examples)
async || core::future::pending().await
```

### 5.4 Harness macros

The example harness (`examples/common`) provides three macros, each
taking an expression that produces the initial view:

```rust
example_main!(MyExample::default());                    // single view → run_app
example_main_nav!(RootView::default());                // navigation → run_app_nav
example_main_psram!(MyExample::default(), 512 * 1024);  // + runtime PSRAM pool
```

Each expands to the matching `run_app*` call on the selected board
(`fire27` / `cores3`), or to the host SDL loop with equivalent logic.

---

## 6. Type Erasure (`AnyView`)

The navigator stores views as `Box<dyn AnyView>` for heterogeneous
stack entries. `AnyView` is an object-safe supertrait of `View`.

```rust
/// Object-safe trait for type-erased views. Not implemented directly
/// — the blanket impl covers all `View` types.
pub trait AnyView: 'static {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;
    fn update(&mut self) -> Result<NavAction, WidgetError>;
    fn on_event(&mut self, event: &Event) -> NavAction;
    fn register_events_on(&mut self, container: &Obj<'static>);
    fn will_hide(&mut self);
    fn did_show(&mut self);
    fn input_group(&self) -> Option<GroupRef>;
}

impl<T: View> AnyView for T {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        View::create(self, container)
    }
    // ... delegate all methods ...
}
```

`AnyView` is `pub` but documented as an implementation detail. Users
implement `View`, never `AnyView` directly.

---

## 7. Memory and Lifetime Model

### 7.1 View Ownership

The navigator owns all view instances via `Box<dyn AnyView>`. When
a view is pushed, the navigator takes ownership. When popped, the
view is dropped. The application never holds a reference to a view
on the stack.

### 7.2 Widget Lifecycle

Widgets are children of the LVGL container object. The navigator
destroys them by calling `lv_obj_clean(container)` during transitions.
View structs store `Option<Widget>` fields — these become `None`
after `lv_obj_clean` invalidates the underlying LVGL objects.

**Important**: after `lv_obj_clean`, any `Obj` handles pointing to
deleted widgets are invalid. The `lv_obj_is_valid` guard in `Obj::drop`
(§5.5 of `spec-memory-lifetime.md`) prevents double-free, but views
MUST NOT dereference widget fields after `will_hide` returns. The
`Option<W>` pattern ensures this: `will_hide` can set fields to `None`
after saving state, and `create` repopulates them.

### 7.3 Style Survival

`Style` objects (backed by `Rc<StyleInner>`) can be stored directly in
the view struct (not inside `Option`). They survive widget teardown
because `Rc` ref-counting keeps the allocation alive. When `create` is
called again, the same `Style` can be re-applied to new widgets.

### 7.4 Subject Survival

`Subject` objects stored in the view struct survive widget teardown.
LVGL's observer cleanup (triggered by `lv_obj_delete` / `lv_obj_clean`)
removes widget-side bindings but does not touch the subject itself.
When `create` rebuilds widgets and re-binds them to the same subject,
the current value is immediately reflected.

### 7.5 Drop Ordering

The navigator drops views in stack order (top first) during shutdown.
Within a view struct, the standard Rust field-drop-order applies.
The same conventions from `spec-memory-lifetime.md` §4.7 and §5.5
govern cleanup: subjects must be declared after widgets so they outlive
observer bindings.

---

## 8. LVGL Primitives Used

| oxivgl concept | LVGL primitive | oxivgl wrapper | Status |
|---|---|---|---|
| Active screen handle | `lv_screen_active()` | `Screen::active()` → `Screen` (non-owning) | Exists |
| Top layer | `lv_layer_top()` | `Screen::layer_top()` → `Child<Obj>` | Exists |
| Widget teardown | `lv_obj_clean()` | `Obj::clean()` | Exists |
| New screen object | `lv_obj_create(NULL)` | `Screen::create()` → `Obj<'static>` | Shipped |
| Full-screen transition | `lv_screen_load_anim()` | `Screen::load(scr, anim, auto_del)` | Shipped |
| Instant screen switch | `lv_screen_load()` | `Screen::load_instant(scr)` | Shipped |
| System layer | `lv_layer_sys()` | `Screen::layer_sys()` → `Child<Obj>` | Shipped |
| Screen anim types | `lv_screen_load_anim_t` | `ScreenAnimType` enum | Shipped |

`Screen` is the namespace for LVGL's screen concept. It has two roles:

- **`Screen::active()`** returns a non-owning `Screen` handle with
  intentional style leaking (the LVGL screen outlives the Rust handle).
  Used by simple examples that don't use navigation.

- **`Screen::create()`** returns an owned `Obj<'static>`. The Navigator
  uses this to create screens it manages. `Obj::drop` handles normal
  cleanup — no style leaking. Views receive this `Obj` as `container`
  in `View::create()` and use `Obj`'s methods (`bg_color`, `add_style`,
  etc.) directly.

No deprecated or experimental LVGL APIs are used.

---

## 9. Implementation Status

Fully shipped. Reference examples: `nav1.rs` (two-screen push/pop),
`toast_hil_demo.rs` (toasts), `menu_keypad.rs` (keypad-navigated menu),
`menu_encoder.rs` (encoder navigate ↔ edit with three inputs).

---

## 10. Error Types

```rust
/// Errors from navigation operations. Implements `Debug` and `Display`.
#[derive(Debug)]
pub enum NavigationError {
    /// Cannot pop the root screen.
    StackEmpty,
    /// No modal is currently active.
    NoActiveModal,
    /// No toast is currently active.
    NoActiveToast,
    /// View creation failed during a navigation transition.
    CreateFailed(WidgetError),
}
```

---

## 11. Open Questions

### 11.1 Nested Navigation

Should a view be able to host a child navigator (e.g. a tab bar where
each tab has its own back stack)? The current spec does not address it.
If needed, a
view could own a `Navigator` instance and delegate `create`/`update`
to it, but the render loop integration needs design work.

**Recommendation**: defer. No current example requires nested navigation.
Add it when a real use case arises.

### 11.2 Animated Modal Transitions

The current spec does not define animation for modal show/dismiss.
`lv_screen_load_anim` applies to full-screen transitions only. Modal
animations would need to use `lv_anim_*` on the backdrop/content
objects directly.

**Recommendation**: modal show/dismiss is instant today; animation
remains deferred. Full-screen transitions do animate via `ScreenAnim`.

### 11.3 Background View `update`

Only the active (topmost) full-screen view and the modal get `update`
calls. Views lower in the stack do not receive `update` while hidden.
If a background view needs to track time or poll external state, it must
do so when `create` is called again (in `did_show`). This matches
LVGL's own model where hidden screens have no live timers.

External events destined for a hidden view accumulate in the channel.
When the view becomes active again, its first `update` call drains
them. If this causes unbounded buffering, the application should size
its channel appropriately or use a latest-value pattern (overwriting
channel / `watch`) for high-frequency data like sensor readings.

---

## 12. Spec Compliance Cross-References

| Constraint | Source | How this spec satisfies it |
|---|---|---|
| No `unsafe` in examples | `spec-example-porting.md` §2 | Navigator handles all unsafe LVGL calls internally |
| No `oxivgl_sys` in examples | `spec-example-porting.md` §2 | `View` trait uses only safe wrapper types |
| `'static` for pointer data | `spec-memory-lifetime.md` §1 | `View: 'static` bound; `Box<dyn AnyView>` is `'static` |
| Styles survive transitions | `spec-memory-lifetime.md` §4 | `Style` (Rc) stored in struct; not tied to widget lifetime |
| Subjects outlive observers | `spec-memory-lifetime.md` §5 | Field drop order; `lv_obj_clean` removes bindings first |
| `Rc` not `Arc` | `spec-memory-lifetime.md` §4.3 | Navigator is single-task; no `Send`/`Sync` required |
| Thin wrappers | `spec-api-vision.md` §3 | Navigator maps directly to `lv_screen_load_anim` + `lv_layer_top` |
| `no_std` + `alloc` | `spec-api-vision.md` §4 | Uses `Vec`, `Box` from `alloc` (already a dependency) |
| Zero warnings | `CLAUDE.md` (zero-warnings policy) | All types fully documented; no `#[allow()]` |
| Hardware input via channels | §5.1, §5.3 | Button/sensor tasks send events through embassy channels; `update()` polls; no GPIO interrupts or LVGL indev required |
