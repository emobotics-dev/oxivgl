# Navigation Specification

**Status**: Draft
**LVGL version**: v9.5

How oxivgl manages screen transitions, navigation stacks, and modal
overlays. This spec evolves the `View` trait into a richer `View` +
`Navigator` model that supports multi-screen applications.

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

This spec evolves the `View` trait into a navigation-aware lifecycle
and adds a `Navigator` for managing a stack of views with push/pop
transitions and modal overlays, built on LVGL's screen and layer
primitives (`lv_screen_load_anim`, `lv_layer_top`, `lv_obj_clean`).

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

## 3. The `View` Trait (evolved)

The `View` trait keeps its name but gains new capabilities: repeatable
`create`, lifecycle hooks, and `NavAction` returns. Every screen of UI
implements `View`.

```rust
/// A single view of UI (one screen or modal in a navigation stack).
///
/// The navigator calls lifecycle methods in this order:
///
/// 1. **Construction** — caller creates the struct (e.g. `Default::default()`)
/// 2. [`create`] — build widgets into `container`; may be called multiple times
/// 3. [`did_show`] — post-creation setup (optional)
/// 4. [`update`] — per-tick polling (runs in render loop)
/// 5. [`on_event`] — LVGL event dispatch
/// 6. [`will_hide`] — save transient state before widget teardown (optional)
/// 7. Widget tree destroyed (navigator calls `lv_obj_clean`)
///
/// Steps 2–7 repeat on each push/pop cycle for views that remain on
/// the stack.
pub trait View: 'static {
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
}
```

### 3.1 Key Changes from Current `View`

| Aspect | Current `View` | Evolved `View` |
|---|---|---|
| Construction | `create() -> Result<Self>` | Caller constructs struct; `create(&mut self, container)` builds widgets |
| Widget rebuilding | Not possible (create runs once) | `create` called on every push/pop return |
| Lifecycle hooks | None | `will_hide`, `did_show` |
| Bound | `Sized` | `'static` (must live on navigator stack) |
| Active screen access | `lv_screen_active()` in create | `container` parameter passed by navigator |
| `update` return | `Result<(), WidgetError>` | `Result<NavAction, WidgetError>` |

### 3.2 Migration for Single-View Examples

For the common case of a single-screen example, the migration is
mechanical:

```rust
// Current View:
struct MyExample { label: Label<'static> }
impl View for MyExample {
    fn create() -> Result<Self, WidgetError> {
        let scr = Screen::active().unwrap();
        let label = Label::new(&scr)?;
        Ok(Self { label })
    }
    fn update(&mut self) -> Result<(), WidgetError> { Ok(()) }
}
example_main!(MyExample);

// Evolved View:
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

Widget fields become `Option<W>` because the struct exists before
`create` is called. `#[derive(Default)]` eliminates manual `None`
initialization. The `example_main!` macro accepts an expression that
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
/// Only the topmost view (and any active modal) have live widgets.
pub struct Navigator {
    // Full-screen navigation stack. Index 0 is the root view.
    stack: Vec<ViewEntry>,
    // Currently active modal, if any. Rendered on lv_layer_top().
    modal: Option<Box<dyn AnyView>>,
}
```

### 4.1 Full-Screen Navigation

```rust
impl Navigator {
    /// Push a new view onto the stack.
    ///
    /// 1. Calls `will_hide()` on the current top view.
    /// 2. Destroys the current view's widget tree (`lv_obj_clean`).
    /// 3. Stores the current view (state preserved, widgets gone).
    /// 4. Calls `create(container)` on the new view.
    /// 5. Registers the event trampoline.
    /// 6. Calls `did_show()` on the new view.
    ///
    /// `anim` controls the screen transition animation. Pass `None` for
    /// an instant switch.
    pub fn push(&mut self, view: impl View, anim: Option<ScreenAnim>);

    /// Pop the current view and return to the previous one.
    ///
    /// 1. Calls `will_hide()` on the current top view.
    /// 2. Destroys the current view's widget tree.
    /// 3. Drops the current view entirely (removed from stack).
    /// 4. Calls `create(container)` on the now-top view (rebuild widgets).
    /// 5. Registers the event trampoline.
    /// 6. Calls `did_show()` on the restored view.
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
deletes the backdrop and all its children.

Optional: the backdrop can be styled with a semi-transparent dark fill
(`opa(128)`, `bg_color(Color::black())`) for a standard dim effect.
This is a `ScreenAnim` option, not automatic.

### 4.3 Screen Animations

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

### 4.4 `NavAction` — Navigation Requests

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
    /// Show a modal overlay.
    Modal(Box<dyn AnyView>),
    /// Dismiss the current modal.
    DismissModal,
}
```

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

To reduce latency when events arrive between render ticks, `run_app`
accepts an **async wake closure** (see §5.2) that is raced against the
inter-tick timer, short-circuiting the sleep when the closure resolves.

### 5.2 `run_app`

Replaces `run_lvgl`. Initializes LVGL, creates the navigator with an
initial screen, and runs the render loop.

```rust
/// Run the LVGL application with navigation support.
///
/// This is an embassy async task. Spawn it alongside your other
/// application tasks. It initializes LVGL, pushes `initial` as the
/// root screen, and loops forever: calling `update`, driving
/// `lv_timer_handler`, and processing navigation actions.
///
/// `wake` is an async closure called each render tick and raced against
/// the inter-tick timer. If `wake` resolves before the timer, `run_app`
/// breaks out of the timer loop early and runs `update()` immediately,
/// reducing worst-case event-to-screen latency from ~33ms to
/// near-instant. For timer-only operation, pass
/// `async || core::future::pending().await`.
///
/// Never returns.
pub async fn run_app<const BYTES: usize, Fut: Future<Output = ()>>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
    initial: impl View,
    wake: impl Fn() -> Fut,
) -> ! {
    let driver = LvglDriver::init(w, h);
    unsafe { lvgl_disp_init(w, h, bufs) };
    DISPLAY_READY.wait().await;

    let mut nav = Navigator::new();
    nav.push(initial, None);

    const LVGL_TIMER_DELAY: u64 = LV_DEF_REFR_PERIOD as u64 / 4;

    loop {
        // Update active view — polls app state, returns NavAction
        let action = nav.active_view_mut()
            .update()
            .unwrap_or_else(|e| { warn!("view update: {:?}", e); NavAction::None });

        // Update modal if present
        let modal_action = if let Some(modal) = nav.active_modal_mut() {
            modal.update()
                .unwrap_or_else(|e| { warn!("modal update: {:?}", e); NavAction::None })
        } else {
            NavAction::None
        };

        // Drive LVGL timers (processes widget events → on_event → NavAction)
        for _ in 0..4 {
            driver.timer_handler();

            // Race the tick timer against the wake closure. If wake
            // resolves first, break out early to run update() sooner.
            let timer = Timer::after(Duration::from_millis(LVGL_TIMER_DELAY));
            match select(timer, wake()).await {
                Either::First(()) => {},   // normal tick
                Either::Second(()) => break, // early wake → run update
            }
        }

        // Process navigation: on_event actions (collected during timer_handler)
        // take priority, then update actions.
        nav.process_pending_event_action();
        if !nav.has_pending() {
            nav.process_action(action);
        }
        if !nav.has_pending() {
            nav.process_action(modal_action);
        }
    }
}
```

The `wake` closure is called fresh each tick, producing a new future
to race. It only needs `core::future::Future` — no dependency on any
specific async runtime or synchronization primitive.

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
async fn ui_task(bufs: &'static mut LvglBuffers<BUF_BYTES>) -> ! {
    run_app(320, 240, bufs, DashboardView::default(),
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

### 5.4 `example_main!` Update

The macro signature changes to accept an initial screen expression:

```rust
// Old:
example_main!(MyView);

// New:
example_main!(MyExample::default());
```

The macro expands to
`run_app(W, H, bufs, <expr>, async || core::future::pending().await).await`
on embedded (timer-only, no external wake) and to the host SDL loop
with equivalent logic.

---

## 6. Type Erasure (`AnyView`)

The navigator stores views as `Box<dyn AnyView>` for heterogeneous
stack entries. `AnyView` is an object-safe supertrait of `View`.

```rust
/// Object-safe trait for type-erased views. Not implemented directly
/// — the blanket impl covers all `View` types.
pub trait AnyView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>;
    fn update(&mut self) -> Result<NavAction, WidgetError>;
    fn on_event(&mut self, event: &Event) -> NavAction;
    fn will_hide(&mut self);
    fn did_show(&mut self);
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
| New screen object | `lv_obj_create(NULL)` | `Screen::create()` → `Obj<'static>` | **Phase 1** |
| Full-screen transition | `lv_screen_load_anim()` | `Screen::load()` | **Phase 1** |
| Instant screen switch | `lv_screen_load()` | `Screen::load()` (with `ScreenAnimType::None`) | **Phase 1** |
| System layer | `lv_layer_sys()` | `Screen::layer_sys()` → `Child<Obj>` | **Phase 1** |
| Screen anim types | `lv_screen_load_anim_t` | `ScreenAnimType` enum | **Phase 1** |

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

## 9. Migration Plan

### Phase 1: Prerequisites (additive, build stays green)

Add screen lifecycle wrappers to `src/widgets/screen.rs`:
- `Screen::create() -> Obj<'static>` — wraps `lv_obj_create(NULL)`,
  returns an owned screen object for Navigator to manage
- `Screen::load(obj, anim)` — wraps `lv_screen_load_anim`, takes
  any `&impl AsLvHandle`
- `Screen::layer_sys()` — wraps `lv_layer_sys`, same pattern as
  existing `layer_top()`
- `ScreenAnimType` enum mirroring `lv_screen_load_anim_t`
- `ScreenAnim` struct (type + duration + delay)

### Phase 2: Core Implementation

1. Evolve `View` trait in `src/view.rs` with new method signatures.
2. Add `AnyView` trait with blanket impl.
3. Add `Navigator` struct with push/pop/replace/modal.
4. Add `NavAction` enum.
5. Replace `run_lvgl` with `run_app` in `src/view.rs`.
6. Update `example_main!` macro.

### Phase 3: Validation

Port one complex multi-state example to evolved `View` + `Navigator`:
- `observer5.rs` (firmware update state machine) — currently simulates
  multi-screen behavior with manual widget teardown and rebuilding.
  Natural fit for push/pop.

Port one simple single-screen example to verify the mechanical
migration path works cleanly.

### Phase 4: Full Migration

1. Port all remaining examples to evolved `View` signatures.
2. Remove `run_lvgl` and old `register_view_events`.
3. Update all documentation.

### Phase 5: Extend

- Write a multi-screen example (e.g. menu → settings → back).
- Write a modal example (e.g. confirmation dialog).
- Add integration tests for push/pop/modal lifecycle.
- Add a leak test for navigator teardown.

---

## 10. Error Types

```rust
/// Errors from navigation operations.
pub enum NavigationError {
    /// Cannot pop the root screen.
    StackEmpty,
    /// No modal is currently active.
    NoActiveModal,
    /// Screen creation failed.
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

**Recommendation**: defer. Implement instant modal show/dismiss first.
Add animation support when needed.

### 11.3 `NavAction` Ergonomics

Returning `NavAction` from `on_event` is explicit but slightly
inconvenient for views that never navigate. The default implementation
returns `NavAction::None`, so non-navigating views don't need to
override `on_event` at all. For navigating screens, a helper constructor
pattern reduces boilerplate:

```rust
fn on_event(&mut self, event: &Event) -> NavAction {
    if event.matches(&self.settings_btn, EventCode::CLICKED) {
        return NavAction::push(SettingsView::default());
    }
    NavAction::None
}
```

### 11.4 Background View `update`

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
| Zero warnings | `spec-rust-code.md` §2 | All types fully documented; no `#[allow()]` |
| Hardware input via channels | §5.1, §5.3 | Button/sensor tasks send events through embassy channels; `update()` polls; no GPIO interrupts or LVGL indev required |
