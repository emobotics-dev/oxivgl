# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] — 2026-07-23

### Added

- **`EncoderIndev` — owning LVGL ENCODER input device** (#127), the three-input
  analogue of `KeypadIndev`. One interaction set (turn−, turn+, press) drives
  *both* focus navigation and in-place value editing, because LVGL owns the
  navigate ↔ edit toggle — the classic three-button embedded idiom (M5Stack
  Fire buttons, CoreS3 touch zones) or a rotary encoder. `EncoderState` is a
  lock-free cell an input producer writes:
  - `turn(steps)` — signed step delta; deltas accumulate, so a multi-tap event
    maps straight to its count (a double-tap `+` is `turn(2)`);
  - `click()` — one short click (enter edit / confirm / click a button);
  - `long_press()` — a **direct route** for a pre-decoded long press that
    toggles edit mode on the focused group (the only way to *leave* edit in a
    multi-object group). It exists because the M5Stack input stack (and
    `async-button`) hands the app finished `Long` events, never held edges, so
    LVGL cannot re-derive the long press from a hold.

  Every producer call fires an **integrated wake signal**, so the event-driven
  loop `run_app_nav_encoder` reads the instant the LVGL task is scheduled — no
  ~30 ms read-timer latency and no separate signal to wire. New example
  `menu_encoder.rs`, driven on hardware by the board's three buttons (M5Stack
  Fire physical buttons / CoreS3 touch-strip zones) through the shared harness;
  `EncoderIndev`/`EncoderState` are re-exported from the prelude.

  Follows LVGL's standard encoder edit model: a short press enters edit (or
  confirms / clicks a button), a long press leaves edit. This refines issue
  #127's prose, which described a short press leaving edit.

- **`Group::is_editing()`** — reports whether a focus group is in edit vs
  navigate mode (`lv_group_get_editing`). Lets a UI show the current encoder
  mode; used by `menu_encoder` for a live NAVIGATE/EDIT indicator.

## [0.6.1] — 2026-07-21

### Fixed

- **Transient per-frame render scratch no longer leaks into a PSRAM pool**
  (#124). The software renderer allocates a scratch buffer per draw op, every
  frame — draw-task descriptors (`lv_draw_add_task`), scanline masks (arc, line,
  fill, border, triangle, rect-mask), box-shadow blur/mask buffers, and image
  mask/transform buffers — through a plain `lv_malloc`/`lv_free` with no callback
  hook (unlike draw buffers). Once a runtime pool was registered this scratch
  became eligible to land in it; when the pool is PSRAM, serving that per-frame
  churn against PSRAM-resident TLSF metadata halved the meter-screen render on
  ESP32 (16 → 7 FPS, measured downstream). `oxivgl-sys` now patches the SW draw
  sources to route these allocations through
  `oxivgl_render_scratch_{malloc,zalloc,free}`, which keep them in internal DRAM
  while a pool is active — the direct analogue of the existing draw-buffer guard,
  extended from pixel buffers to all transient render scratch. So the whole
  render hot path stays internal while the persistent widget tree lives in PSRAM,
  making render cost track dirty-region complexity rather than total UI size.
  The radius/circle mask cache (`lv_draw_sw_mask.c`), read per-scanline every
  frame by arc, rounded-rect, border and shadow draws, is routed too — its
  alloc/free sites are all self-contained in that file, so it is as safe to route
  as the per-frame scratch. LVGL's gradient cache (`lv_draw_sw_grad.c`) is left on
  LVGL's allocator: gradient-only and cross-function with multi-path frees. With
  no pool registered (or under `LV_STDLIB_CLIB`) the calls delegate straight to
  LVGL's allocator, so behaviour is unchanged. Requires `oxivgl-sys 0.2.4`.

## [0.6.0] — 2026-07-20

### Added

- **ESP32-S3 (M5Stack CoreS3) support** (#119). New `esp32s3` library feature —
  `src/` is chip-agnostic (generic over `DisplayOutput`), so it is a one-liner.
  The example harness now bring-ups both boards through the
  [`m5stack-core`](https://github.com/emobotics-dev/m5stack-core) BSP and picks
  one with the `fire27` / `cores3` cargo feature; the board's input model
  (Fire27 keypad vs CoreS3 touch) is selected automatically. `run_cores3.sh`
  flashes the S3. HIL-validated on both boards.
- `oxivgl::mem` — register a memory region with LVGL's heap at run time
  (`reserve_pool`, `reserve_pool_raw`, `MemError`). Lets LVGL's heap live in
  PSRAM, whose base address is not known at compile time: on ESP32-S3 it moves
  with the binary's rodata size, so `LV_MEM_ADR` cannot name it. Available only
  under `LV_USE_STDLIB_MALLOC = LV_STDLIB_BUILTIN`; the module is not compiled
  otherwise, because the CLIB backend implements `lv_mem_add_pool` as a no-op
  returning NULL and would accept a pool while silently ignoring it.
- `examples/psram_pool.rs` plus the `example_main_psram!` harness macro —
  end-to-end demonstration, validated on Fire27 hardware.
- `run_tests.sh pool` — asserts a reserved pool actually reaches LVGL's heap and
  that draw buffers stay outside it.
- **Leak detection now covers LVGL's own C heap**, not just the Rust side
  (#118). Every leak test additionally asserts on
  `lv_mem_monitor().total_size - free_size`, so a wrapper whose `Drop` stopped
  calling `lv_obj_delete` now fails the suite — previously it passed, because
  the wrapper struct itself was freed and LVGL's heap is invisible to a Rust
  `#[global_allocator]`. Gated on `LV_STDLIB_BUILTIN`, under which
  `lv_mem_monitor` reports real figures; compiled out under CLIB rather than
  degrading to an assertion that can only pass. A negative control
  (`leak_negative_control_forgotten_obj`) leaks an `lv_obj` per iteration and
  requires the assertion to fire. Both heaps assert an exact zero with no noise
  floor, and the sensitivity is itself tested: a leak of one byte per
  iteration, and the smallest allocation LVGL can make, are each required to
  fail.

### Changed

- **The example harness was ported to the `m5stack-core` BSP** (#119),
  replacing ~400 lines of hand-rolled Fire27 SPI/GPIO/button bring-up with the
  BSP's `Board::split` + `board::display` + input/console/heap loops. The public
  `example_main!` / `_nav!` / `_psram!` macros are unchanged, so no example file
  changed. LVGL-in-PSRAM now routes through the BSP's `mem::psram_split`.
  Library-only users are unaffected — the BSP is an example dev-dependency.
- **Local, CI, and example builds now compile against the emobotics `esp-hal`
  fork** via a workspace `[patch.crates-io]` at a pinned rev (#119): the BSP's
  DMA display flush needs its SPI/DMA fixes, and stock ESP32 PDMA wedges after
  the first frame. This does **not** affect crates.io consumers of the library —
  `cargo publish` strips `[patch]`, and oxivgl's own dependency on `esp-hal`
  stays a plain crates.io version, so a downstream `cargo add oxivgl` resolves
  stock esp-hal with no fork and no git sources.
- **Bundled `lv_conf.h` files now select `LV_STDLIB_BUILTIN`** instead of
  `LV_STDLIB_CLIB`. This changes where LVGL allocates: from the Rust global
  allocator to LVGL's own static TLSF pool sized by `LV_MEM_SIZE`. Applications
  that copied `examples/conf/lv_conf.h` or `oxivgl-sys/default-conf/lv_conf.h`
  should size `LV_MEM_SIZE` for their workload — it is a real static array, and
  too small a value now fails allocations rather than falling back to the system
  heap.
- **`LV_ASSERT_HANDLER` now panics** (`oxivgl_lv_assert_handler`) instead of
  LVGL's default `while(1);`. Spinning turned every assertion — a failed
  allocation, a NULL object — into an indefinite hang with no message and no
  backtrace; downstream traced a recurring-freeze class to exactly this.
  Applications keeping their own handler are unaffected: the symbol is prefixed
  precisely so it cannot collide with an unprefixed `lv_assert_handler`.
- When a runtime pool is registered, LVGL's draw buffers are routed through the
  Rust global allocator so they cannot be served from that pool. TLSF pools are
  fungible, and a draw buffer in PSRAM is unreachable by the ESP32's DMA engine.

### Fixed

- `tests/leak_check.rs` did not compile: two hand-declared libc externs
  (`write`, `open`) diverged from their real signatures, which newer rustc
  rejects. Its module docs also claimed to track LVGL's C heap — a Rust
  `#[global_allocator]` never observes it under either allocator backend.
- `translation::add_static` now documents that it must be called once per pack.
  Each call allocates on LVGL's heap and LVGL exposes no per-pack removal, so
  repeated registration grows the heap without bound. Surfaced by the new
  C-heap coverage, which measured the old `leak_translation` body — registering
  inside its loop — leaking one `lv_translation_pack_t` (72 bytes) per
  iteration while the Rust-side balance stayed clean.

## [0.5.0] — 2026-06-14

### Added

- **POINTER (touchscreen) input device** (#111). `PointerIndev` is the
  direct-touch analogue of `KeypadIndev`: a view can be navigated by tapping a
  widget at a coordinate, not only via focus keys. It is fed in oxivgl's own
  vocabulary — raw `(x, y)` coordinates — via either a lock-free `PointerState`
  cell (`PointerIndev::new`) or a polling closure (`PointerIndev::new_with`,
  e.g. `PointerIndev::new_with(|| ft6336u::read_touch())`). No BSP/MCU/driver
  type is involved, so it stays portable across boards. Both `PointerIndev` and
  `PointerState` are re-exported from the prelude.
- **Keypad hold-to-repeat** (#111). `KeypadIndev::with_repeat(after, every)` —
  while a key is held (the *held* model), LVGL re-sends it to the focused group
  first after `after`, then every `every`, for value/setpoint editing. A thin
  pass-through to LVGL's `long_press_time` / `long_press_repeat_time`.
- **Example** `touch_setpoint` — a setpoint editor combining both: hold `-`/`+`
  to repeat (keypad) and tap a preset to jump the value (touch).

### Changed

- **Built-in font constants are now gated to the faces enabled in `lv_conf.h`**
  (#106). `src/fonts.rs` previously referenced every `lv_font_montserrat_*`
  (8–48), the DejaVu Persian/Hebrew face, and both Source Han CJK faces
  unconditionally, so disabling *any* of them in an application's `lv_conf.h`
  broke `oxivgl`'s own compile (`cannot find value …` deep in the bindings) —
  forcing every app to ship ~400 KiB of fonts it may not use. `oxivgl-sys`
  (now **0.2.2**) reports which faces the LVGL build exposed, and each `Font`
  const is gated behind a `font_*` cfg derived from that report. Apps can now
  set `LV_FONT_* 0` to drop unused faces; referencing a disabled face is a
  plain "cannot find value" error pointing at the app's own code. Requires
  `oxivgl-sys >= 0.2.2`. (The previous `fonts` module docs claiming a *linker*
  error and "no code-size cost" were wrong and have been corrected.)

### Fixed

- **Text setters no longer silently truncate at 127 bytes** (#105). Every
  `&str`-taking setter — `Textarea::{set_text, add_text, set_placeholder_text}`,
  `Label::text`, `Checkbox::text`, `List::{add_text, add_button}`,
  `Msgbox::{add_title, add_text, add_footer_button}`, `Table::set_cell_value`,
  `Win::add_title`, `Menu::page_create`, `Label::set_translation_tag`, and the
  `Label::bind_text_map` updater — previously copied through a fixed `[u8; 128]`
  stack buffer and dropped everything past 127 bytes with no error. They now
  pass the full string through a heap-backed NUL-terminated temporary (LVGL
  copies it internally), so text of any length is rendered verbatim. This also
  removes 15 per-call 128-byte stack temporaries from widget construction.

### Deprecated

- **`Label::text_long`** — `Label::text` is now itself uncapped, so the two are
  identical. `text_long` remains as an alias and will be removed in a future
  release; call `text` directly.

## [0.4.0] — 2026-06-10

### Added

- **Resource diagnostics (`diag` module).** `census()` walks a widget subtree
  and reports object count + nesting depth (with an estimated-heap-bytes
  coefficient); `Budget` / `assert_budget` enforce per-view ceilings;
  `ResourceProbe` (host default `NullProbe`) is a pluggable hook for live
  on-target heap/stack figures. See `docs/memory-tuning.md`.
- **`Style::new(|s| …)`** — one-call shared-style constructor collapsing
  `StyleBuilder::new()` + setters + `.build()`. `add_style` `Rc`-retains the
  style, so a built `Style` may be applied across many widgets and the handle
  dropped (build-and-forget).
- **`StyleBuilder` parity methods**, so every shareable style property has a
  shared-style path: `pad_hor`, `pad_bottom`, `pad_column`, `pad_row`, `size`,
  `clip_corner`, `text_align`, `base_dir`, `radial_offset`, `line_opa`,
  `arc_rounded`, `blur_radius`, `blur_backdrop`, `radius_circle`, and the
  `bg_image_recolor` / `bg_image_recolor_hex` / `bg_image_recolor_opa` family.
- **Background-image wrappers on `Obj`**: `style_bg_image_src` (image
  descriptor), `style_bg_image_recolor[_hex]`, `style_bg_image_recolor_opa`.
- **Guide** `docs/memory-tuning.md` — measuring and reducing heap/stack on
  widget-heavy UIs.
- **Examples** `shared_styles1` (a style guide built from shared styles) and
  `widget_bar8` (recolored background image).

### Deprecated

- **49 inline `style_*` setters on `Obj`.** Each inline setter allocates a
  per-object local style; build a shared `Style` (`Style::new`) and apply it
  with `add_style` to amortize to one property buffer at scale — a heap *and*
  style-refresh-compute win wherever a treatment repeats. Transforms and
  image-content setters are intentionally not deprecated (per-instance /
  dynamic). See `docs/memory-tuning.md`.

### Changed

- All bundled examples migrated off inline setters to shared `Style` +
  `add_style` (verified pixel-identical via before/after screenshots).

## [0.3.4] — 2026-06-04

### Fixed

- **Toasts intermittently invisible in PARTIAL render mode (ESP32).** A passive
  toast on an otherwise-static screen could silently fail to appear — worst on
  the first cold boot — because `lv_layer_sys()` is not composited reliably onto
  passive redraws in PARTIAL mode. Toasts now render on the **active screen**
  (or the modal backdrop while a modal is open), an ordinary child of the normal
  widget tree that composites as reliably as any other widget. `Navigator`
  re-parents the toast onto the current topmost surface across
  `push`/`replace`/`pop` and modal open/dismiss, so the public contract is
  unchanged: it persists across page switches, stays above any modal, and stays
  passive. Supersedes the earlier `lv_layer_sys()` approach and its
  repaint-window mitigation (which only reached ~91% on hardware). Public API is
  unchanged — only the rendering surface and reliability.

### Changed

- A toast now follows the active screen, so an *animated* screen transition
  slides the toast with the incoming page (instant loads — the default — are
  unaffected). Previously the toast sat on the system layer and stayed fixed
  during transitions.

## [0.3.3] — 2026-06-03

### Added

- **`navigator::post_toast_with`** — post a cross-task toast by handing the
  render loop a `Send` *builder closure* instead of a `View` value. The view is
  constructed render-side, so it need not be `Send` — which lets a toast store
  its widget wrappers (the leak-free pattern, where `Drop` frees the style
  `Rc`s) even though those wrappers are `!Send`. `post_toast` is now thin sugar
  over it for trivial `Send` views. Unblocks posting real styled toasts from
  background tasks and from `.on(event)` closures that cannot return a
  `NavAction`.

## [0.3.2] — 2026-06-03

### Changed

- **Toast timeout type is now `core::time::Duration`** instead of
  `embassy_time::Duration`. The value was always collapsed to a `u32`
  millisecond count internally, so the `embassy_time` type was pure ceremony
  that forced every consumer of a *timed* toast to add an `embassy-time`
  dependency. Affected public API: `Navigator::show_toast`,
  `navigator::post_toast`, and `NavAction::ShowToast` / `NavAction::show_toast`.
  Callers now write `Some(Duration::from_secs(3))` with the dependency-free,
  no_std `core` type. `embassy-time` remains an internal dependency (render-loop
  timers, tick source) but no longer appears in the public toast signature.

## [0.3.1] — 2026-06-03

### Fixed

- **Invisible toasts in PARTIAL render mode (ESP32).** A toast raised before the
  first navigation (e.g. a "No SD card" boot warning) stayed invisible until the
  user happened to switch pages. `Navigator::push_root` now builds the root view
  on its own loaded screen — which arms `lv_layer_sys()` compositing from boot —
  and `show_toast` invalidates the toast container so it is flushed even without
  an accompanying navigation event. Verified on hardware. (No effect on host /
  SDL, which uses FULL/DIRECT rendering and never exhibited the bug.)
- **Rapid toasts collapsing to only the last one.** Toasts requested in quick
  succession were created and destroyed within a single render iteration and
  never drawn. Timed toasts now queue and play back sequentially (bounded FIFO),
  each shown for its full duration; persistent (`None`) toasts supersede the
  active toast and clear the queue. `dismiss_toast` advances to the next queued
  toast. Public API is unchanged — only behavior.
- **`Navigator::pop` back to the root** now loads the root's own screen via the
  normal path; the default-screen fallback became an unreachable defensive guard.

### Added

- **Example** `toast_hil_demo` — minimal navigator app that raises a persistent
  toast before any navigation, used to verify the fix on real ESP32 hardware.

## [0.3.0] — 2026-06-02

### Added

- **Keypad input device.** `KeypadState` + `KeypadIndev` — a safe LVGL `KEYPAD`
  indev fed by the application, for focus-navigated menus driven by hardware
  buttons or on-screen / touch keys. Two producer models:
  - *Held* — `KeypadState::press(key)` / `release()`; LVGL derives
    long-press/repeat (raw momentary buttons).
  - *One-shot* — `KeypadState::send(key)`; each event is exactly one focus step
    with no LVGL-side repeat, for input drivers that already decode
    debounce/long-press/repeat. Backed by a lock-free single-producer ring.
- **Event-driven, poll-free input.** `KeypadIndev::new_event` (LVGL
  `LV_INDEV_MODE_EVENT`, no read timer) + `read()` to drain on demand;
  `KeypadState::has_pending`. The device is read only when the application
  signals an event — nothing is polled.
- **Navigator focus routing for full-screen views.** Each active view's
  `View::input_group()` is now bound on `push` / `pop` / `replace` (previously
  modal-only), so a whole page — not just a modal — can be keypad-navigated.
- **Run-loop keypad entry points.** `view::run_app_nav_keypad` (TIMER mode) and
  `view::run_app_nav_keypad_events` (EVENT mode + async `wake` closure that races
  the inter-tick sleep for near-instant, poll-free input).
- `EventCode::RELEASED` (LVGL touch-up edge).
- Prelude re-exports `KeypadState` and `KeypadIndev`.
- **Example** `menu_keypad` — a focus-navigated menu driven by on-screen keys
  (host SDL); the same code is driven by the front-panel buttons on ESP32.

## [0.2.1] — 2026-05-31

### Added

- **Default toast geometry and shadow.** `show_toast` now positions
  the container as a bottom-anchored floating card — full sys-layer
  width, symmetric `TOAST_MARGIN_PX` (2 px) inset on left / right /
  bottom, height hugging content, plus a soft symmetric shadow
  (`TOAST_SHADOW_WIDTH_PX = 12`, `TOAST_SHADOW_OPA = 80`) to reinforce
  the elevated look. Views may still override on the container in
  `create`. New public constants in `navigator`.

## [0.2.0] — 2026-05-31

### Added

- **Global passive toast overlay.** `Navigator::show_toast` / `dismiss_toast` /
  `tick_toast`; `NavAction::{ShowToast, DismissToast}`. Lives on `lv_layer_sys()`,
  persists across push/replace/pop, input-transparent by contract,
  navigator-owned auto-dismiss.
- **Post toasts from any task.** Free functions `post_toast<V: View + Send>` and
  `post_dismiss_toast` enqueue into a library-owned channel drained by
  `run_app_nav` — no draining view required.
- **OSD-style modal support.** Click-absorbing backdrop on `lv_layer_top()`
  (was specified, now implemented). `View::input_group() -> Option<GroupRef>`:
  when `Some`, the navigator swaps focus into the modal's group on open and
  restores the previous default group + per-indev bindings on dismiss.
- `Group::as_ref`, `GroupRef::set_default`, `GroupRef::assign_to_keyboard_indevs`.

### Changed

- **Breaking:** `View::register_events(&mut self)` renamed to
  `register_events_on(&mut self, container: &Obj<'static>)`. The default attaches
  the trampoline to `container` (the navigator-supplied target) rather than
  `lv_screen_active()`.

  *Migration:* rename the override and add the `_container` parameter; bodies
  typically don't need to change. Overrides that just called the old default can
  be deleted — the new default is correct for screens and modals alike.

### Fixed

- Modal `register_events` no longer attaches handlers to the *background view's*
  screen (where they dangled after the next push/pop). The new
  `register_events_on(container)` default routes correctly for every overlay.

## [0.1.2] — 2026-05-06

### Fixed

- docs.rs build: the v0.1.1 fix was incomplete — docs.rs has no network
  access at build time, so `oxivgl-sys` could not download the LVGL
  source. Now under `DOCS_RS=1`, `oxivgl-sys` uses a pre-generated
  `bindings_docsrs.rs` (committed in the crate) and skips the LVGL
  download + cc compilation entirely.

## [0.1.1] — 2026-05-06

### Fixed

- docs.rs documentation build: `oxivgl-sys` now falls back to a bundled
  `default-conf/lv_conf.h` under `DOCS_RS=1` (the workspace
  `.cargo/config.toml` is unavailable on docs.rs), and `oxivgl/build.rs`
  skips image asset compilation in the same environment.

## [0.1.0] — 2026-04-09

Initial public release.

### Added

- Safe `no_std` Rust bindings for LVGL v9.5 on ESP32 (Xtensa) and host (SDL2).
- 37 type-safe widget wrappers covering all major LVGL widget categories.
- `View` trait with `create`/`update`/`on_event` lifecycle for building screens.
- `Navigator` with `NavAction` for multi-screen push/pop/replace navigation.
- `StyleBuilder` two-phase pattern enforcing compile-time lifetime safety.
- Animation helpers (`Anim`, `AnimTimeline`) tied to widget lifetimes.
- `DisplayOutput` trait and DMA flush pipeline for ESP32 hardware displays.
- 171 ported LVGL examples (zero `unsafe` in user code).
- 624 automated tests: unit, doc, integration, leak detection, and visual regression.

### Known limitations

- Observer examples 4–6 deferred (need animation/timer callbacks in observer context).
- GIF and IME_PINYIN widgets not yet wrapped (v9.5 additions, no upstream examples).
- Lottie and ThorVG-based features (canvas vector drawing) intentionally out of scope
  (240 KB bloat, C++ runtime dependency).

### Breaking changes during pre-release development

- `View::create` changed from `fn create() -> Result<Self, WidgetError>` to
  `fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError>`.
- `View::update` now returns `Result<NavAction, WidgetError>` instead of
  `Result<(), WidgetError>`.
- All 188 examples migrated to the evolved `View` trait.
