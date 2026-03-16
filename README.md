[![CI](https://github.com/emobotics-dev/oxivgl/actions/workflows/ci.yml/badge.svg)](https://github.com/emobotics-dev/oxivgl/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://gist.githubusercontent.com/hsteinhaus/f92c7e4991559affa2788d3a66364bcc/raw/oxivgl-coverage.json)](https://github.com/emobotics-dev/oxivgl/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)
![LVGL v9.3](https://img.shields.io/badge/LVGL-v9.3-brightgreen)
![Rust nightly](https://img.shields.io/badge/Rust-nightly-orange)

# oxivgl

Safe Rust bindings for [LVGL](https://github.com/lvgl/lvgl) on embedded (`no_std`) and host (`std`/SDL2) targets. Wraps all unsafe LVGL calls behind type-safe APIs — user code never touches `unsafe` or `lvgl_rust_sys` directly.

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

| Setting | Value | Why                                          |
|---------|-------|----------------------------------------------|
| `LV_COLOR_DEPTH` | 16 (RGB565) | Matches SPI TFT panel and DMA buffer format  |
| `LV_DPI_DEF` | 130 | LVGL default; matches Montserrat font metrics |
| `LV_DEF_REFR_PERIOD` | 32 ms | ~30 fps, balances smoothness vs CPU on ESP32 |
| `LV_USE_FONT_MONTSERRAT_14` | 1 | Default font (LVGL default)                  |
| `LV_USE_SDL` | 1 (host) / 0 (ESP32) | SDL2 display driver for host development     |
| `LV_USE_SNAPSHOT` | 1 (host) | Screenshot capture for visual regression     |
| `LV_USE_OS` | `LV_OS_NONE` | Single-threaded; no RTOS mutex overhead      |

Only widgets actually used are enabled (`LV_USE_<WIDGET> 1`) to minimize binary size on embedded. Adding a new widget requires enabling it here first.

## Memory Safety Across the FFI Boundary

LVGL is a C library that stores raw pointers to styles, image descriptors, point arrays, and callback data — with no built-in ownership tracking. The [official Rust wrapper](https://github.com/lvgl/lv_binding_rust) (`lv_binding_rust`) has known soundness gaps — wrong lifetimes on widget constructors that allow dangling pointers ([#166](https://github.com/lvgl/lv_binding_rust/issues/166)), SIGSEGV on basic SDL init ([#180](https://github.com/lvgl/lv_binding_rust/issues/180)), and is stuck on LVGL v8 with no active maintenance ([#201](https://github.com/lvgl/lv_binding_rust/issues/201)).

oxivgl solves this with a **compile-time ownership model** documented in [`docs/spec-memory-lifetime.md`](docs/spec-memory-lifetime.md):

- **Two-phase style system** — `StyleBuilder` (mutable, stack-local) → `Style` (immutable, `Rc<StyleInner>`). Sub-descriptors (gradients, transitions, color filters) are moved into the style by value and freed in the correct order via `Drop`.
- **`'static` enforcement** — APIs where LVGL stores a raw pointer (`Image::set_src`, `Line::set_points`, `Dropdown::set_symbol`, `StyleBuilder::bg_image_src`, `TransitionDsc::new`) require `'static` references. Non-`'static` data is rejected at compile time.
- **Rc-based style sharing** — `Obj::add_style` clones the `Rc` *before* passing the pointer to LVGL. The `lv_style_t` remains valid as long as any widget or user code holds a clone. `Obj::drop` calls `lv_obj_delete` (which internally removes all styles) before Rust drops the `_styles` Vec.
- **Lifetime-tied animations** — `Anim<'w>` uses `PhantomData<&'w ()>` to tie the animation descriptor to the target widget's lifetime. After `start()`, LVGL owns a copy; the widget's deletion auto-cancels the animation.

These guarantees are verified by [integration tests](#testing) that exercise style-drop-before-widget, widget-drop-with-styles-applied, shared-style-across-widgets, and explicit remove-then-drop sequences.

## API Philosophy

- **Full LVGL wrapping** — every LVGL function used goes through a safe Rust wrapper in `widgets/`. User code never imports `lvgl_rust_sys` or writes `unsafe`.
- **Newtypes over raw constants** — `ObjFlag`, `ObjState`, `EventCode`, `Opa`, `Selector`, `ScaleMode` etc. are newtype wrappers with named constants. Composable via `BitOr` where appropriate.
- **Close to LVGL's API** — method names and parameter order mirror the C API (e.g. `obj.align(Align::Center, 0, 0)`, `slider.set_range(0, 100)`). Developers familiar with LVGL's C docs can transfer knowledge directly.
- **Builder patterns for complex constructors** — `ScaleBuilder`, `GridCell`, `TransitionDsc` avoid long positional argument lists while keeping the API surface small.
- **AI-friendly** — discoverable, well-documented, free of footguns. No `unsafe`, no lifetime puzzles, no implicit state. Every widget follows the same `new(parent) -> Result` pattern.

## Architecture

![Architecture](docs/architecture.svg)

## Key Types

```rust
pub trait View: Sized {
    fn create() -> Result<Self, WidgetError>;
    fn update(&mut self) -> Result<(), WidgetError>;
    fn on_event(&mut self, _event: &Event) {}       // safe event dispatch
    fn register_events(&mut self) { /* … */ }       // override for nested containers
}
```

## Example-Driven Development

Every new LVGL feature is implemented by porting the corresponding [LVGL docs example](https://docs.lvgl.io/9.3/examples.html) first. This ensures the wrapper API is ergonomic for real use cases, not just theoretically correct. Examples serve as both documentation and visual regression tests — screenshots are auto-captured and compared against the LVGL reference.

70+ examples covering getting started, styles, animations, events, layouts, scrolling, and individual widgets. Each is a self-contained `View` impl + `example_main!` macro — runs on host SDL2 or ESP32 with zero code changes.

See the full gallery with screenshots: **[`examples/doc/README.md`](examples/doc/README.md)**

```sh
# Interactive SDL2 window:
./run_host.sh getting_started1

# Headless screenshot (no window):
./run_host.sh -s getting_started1

# Screenshot all examples:
./run_host.sh -s

# Flash to ESP32:
./run_fire27.sh event_trickle
```

## Testing

175 automated tests across three tiers — all run on host without hardware:

| Tier | Count | What it covers |
|------|-------|----------------|
| **Unit** | 41 | Pure logic — enums, value mapping, style bitflags, grid helpers |
| **Integration** | 99 | Full LVGL instance — widget lifecycle, style add/remove/drop ordering, layout, events, every widget type |
| **Leak detection** | 25 | Global heap tracking via `mallinfo2()` — catches leaks in both Rust and LVGL's C code across the FFI boundary |
| **Visual** | 70+ | Screenshot capture + comparison against LVGL reference docs |

```sh
./run_tests.sh all          # unit + integration + leak (< 5 seconds)
./run_tests.sh unit         # unit + doctests
./run_tests.sh int          # integration (headless LVGL)
./run_tests.sh leak         # memory leak detection
./run_host.sh -s            # visual — screenshot all examples
```

Integration and leak tests run against a real LVGL instance with `SDL_VIDEODRIVER=dummy` (no display server needed). Sequential execution (`--test-threads=1`) because LVGL is single-threaded. CI runs both host tests and ESP32 firmware builds on every push.

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
