# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
