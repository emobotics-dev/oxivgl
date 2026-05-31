# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
