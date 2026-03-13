[![CI](https://github.com/emobotics-dev/oxivgl/actions/workflows/ci.yml/badge.svg)](https://github.com/emobotics-dev/oxivgl/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
![LVGL v9.3](https://img.shields.io/badge/LVGL-v9.3-brightgreen)
![Rust nightly](https://img.shields.io/badge/Rust-nightly-orange)

# oxivgl

Safe Rust bindings for [LVGL](https://lvgl.io/) on embedded (`no_std`) and host (`std`/SDL2) targets. Wraps all unsafe LVGL calls behind type-safe APIs — user code never touches `unsafe` or `lvgl_rust_sys` directly.

Built for AI-generated UIs: Rust's type system and borrow checker catch mistakes at compile time that would silently corrupt memory in C. When an AI agent generates widget code, the compiler enforces correct parent-child relationships, valid enum values, and proper resource lifetimes — turning runtime crashes into compile errors. Our vision is that AI agents become the primary users of this crate, generating embedded GUIs from high-level descriptions.

## Supported Platforms

| Platform | Target | Display | Use |
|----------|--------|---------|-----|
| **ESP32** (Xtensa) | `xtensa-esp32-none-elf` | SPI TFT via DMA flush pipeline | Production firmware |
| **Linux / macOS** (x86_64, aarch64) | `x86_64-unknown-linux-gnu` | SDL2 window or `SDL_VIDEODRIVER=dummy` (headless) | Development, testing, screenshots |

### Why test on host?

LVGL's widget tree, layout engine, and style system are pure C — platform-independent. Running tests on the host (x86_64) gives sub-second feedback without flashing hardware. The headless SDL2 dummy driver (`SDL_VIDEODRIVER=dummy`) enables CI without a display server. Only the flush pipeline and DMA buffer handling are ESP32-specific.

### LVGL configuration (`conf/lv_conf.h`)

LVGL is compiled from source with a project-specific `lv_conf.h`. Key settings:

| Setting | Value | Why |
|---------|-------|-----|
| `LV_COLOR_DEPTH` | 16 (RGB565) | Matches SPI TFT panel and DMA buffer format |
| `LV_DPI_DEF` | 130 | LVGL default; matches Montserrat font metrics |
| `LV_DEF_REFR_PERIOD` | 32 ms | ~30 fps, balances smoothness vs CPU on ESP32 |
| `LV_USE_FONT_MONTSERRAT_14` | 1 | Default font (LVGL default) |
| `LV_USE_SDL` | 1 (host) / 0 (ESP32) | SDL2 display driver for host development |
| `LV_USE_SNAPSHOT` | 1 (host) | Screenshot capture for visual regression |
| `LV_USE_OS` | `LV_OS_NONE` | Single-threaded; no RTOS mutex overhead |

Only widgets actually used are enabled (`LV_USE_<WIDGET> 1`) to minimize binary size on embedded. Adding a new widget requires enabling it here first.

## API Philosophy

- **Full LVGL wrapping** — every LVGL function used goes through a safe Rust wrapper in `widgets/`. User code never imports `lvgl_rust_sys` or writes `unsafe`.
- **Newtypes over raw constants** — `ObjFlag`, `ObjState`, `EventCode`, `Opa`, `Selector`, `ScaleMode` etc. are newtype wrappers with named constants. Composable via `BitOr` where appropriate.
- **Close to LVGL's API** — method names and parameter order mirror the C API (e.g. `obj.align(Align::Center, 0, 0)`, `slider.set_range(0, 100)`). Developers familiar with LVGL's C docs can transfer knowledge directly.
- **Builder patterns for complex constructors** — `ScaleBuilder`, `GridCell`, `TransitionDsc` avoid long positional argument lists while keeping the API surface small.
- **AI-friendly** — discoverable, well-documented, free of footguns. No `unsafe`, no lifetime puzzles, no implicit state. Every widget follows the same `new(parent) -> Result` pattern.

## Architecture

```
  examples/*.rs / consumer crate
  (implements View trait)
          │
          ▼
  oxivgl::view::run_lvgl::<V: View>()
  ├── LvglDriver::init()          — tick source, log bridge
  ├── lvgl_disp_init()            — display + DMA buffers, flush_cb
  ├── V::create()                 — build widget tree
  ├── register_view_events()      — safe event dispatch via on_event()
  └── loop: V::update() + lv_timer_handler()
```

## Modules

| Module | Purpose |
|--------|---------|
| `view` | `View` trait (`create`/`update`/`on_event`/`register_events`) + `run_lvgl::<V>()` |
| `widgets` | Type-safe wrappers: `Obj`, `Screen`, `Label`, `Button`, `Slider`, `Switch`, `Arc`, `Bar`, `Scale`, `Led`, `Line`, `Image`, `ValueLabel`, `Style`, `Anim`, `AnimTimeline`, `Event`, `GradDsc`, `ScaleBuilder`, `GridCell`, `Selector` |
| `lvgl` | `LvglDriver`: init tick source + log forwarding |
| `lvgl_buffers` | `DisplayOutput` trait; DMA frame buffer; `flush_frame_buffer` task |
| `fonts` | `Font` type + built-in Montserrat fonts |

## Key Types

```rust
pub trait View: Sized {
    fn create() -> Result<Self, WidgetError>;
    fn update(&mut self) -> Result<(), WidgetError>;
    fn on_event(&mut self, _event: &Event) {}       // safe event dispatch
    fn register_events(&mut self) { /* … */ }       // override for nested containers
}
```

## Examples

Self-contained examples in `examples/*.rs` — each implements `View` + uses the `example_main!` macro. Run on host via SDL2 or flash to ESP32.

| Chapter | Examples |
|---------|----------|
| Getting Started | `getting_started{1-4}` — label, button, slider event, style intro |
| Styles | `style{1-18}` — backgrounds, borders, shadows, transforms, transitions, gradients |
| Animations | `anim1` (switch event), `anim2` (playback), `anim_timeline1` (timeline) |
| Events | `event_click`, `event_button`, `event_bubble`, `event_trickle` |
| Layouts | `flex{1-6}` (flexbox), `grid{1-6}` (grid) |

```sh
# Run example on host (SDL2 viewer):
./run_host.sh getting_started1

# Flash to ESP32:
./run_fire27.sh event_trickle

# Capture all screenshots:
./run_screenshots.sh
```

## Testing

```sh
# Unit tests (pure logic — enums, value mapping, style bitflags):
LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu

# Integration tests (headless LVGL — widget creation, state, layout):
SDL_VIDEODRIVER=dummy LIBCLANG_PATH=/usr/lib64 cargo +nightly test \
  --test integration --target x86_64-unknown-linux-gnu -- --test-threads=1

# Visual regression (screenshot comparison):
./run_screenshots.sh
```

Integration tests run against a real LVGL instance with the SDL2 dummy driver (no display needed). Tests are sequential (`--test-threads=1`) because LVGL is single-threaded.

## Features

| Feature | Effect |
|---------|--------|
| `esp-hal` | ESP32 tick source (`esp_hal::time`) + DMA flush pipeline |
| `defmt` | `defmt` logging (embedded) |
| `log-04` | `log` v0.4 logging (host) |

## Build

```sh
# Host check:
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu

# Host tests:
LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu

# Embedded check (requires Xtensa toolchain):
cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04
```

`build.rs` compiles LVGL from source via cmake. Expects `DEP_LV_CONFIG_PATH` pointing to `lv_conf.h`.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
