# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## MANDATORY: Consult Specs Before Any Change

**Before designing, implementing, or reviewing ANY code change**, read and follow the relevant specifications in `docs/`. This is not optional — every design decision, API addition, wrapper method, example, and test MUST comply with these specs. Check compliance DURING implementation, not after.

| Spec | When to consult |
|------|----------------|
| `spec-api-vision.md` | Any public API change — naming, return types, safety model |
| `spec-memory-lifetime.md` | Any code touching pointers, lifetimes, Drop, styles, animations |
| `spec-widget-wrapper.md` | Any new widget/module — struct pattern, SAFETY comments, doc comments, tests |
| `spec-example-porting.md` | Any new/modified example — run_host.sh, README, screenshots, doc audit |

## Build & Test

Toolchain: `nightly` (set via `rust-toolchain.toml`); `esp` for Xtensa cross-builds.

`LIBCLANG_PATH` is set to `/usr/lib64` in `.cargo/config.toml` for host builds.
For Xtensa targets, `oxivgl-sys/build.rs` auto-detects the ESP clang.

```sh
# Check host (std, no ESP toolchain needed):
cargo check

# Check embedded (requires Xtensa toolchain):
cargo +esp -Zbuild-std=alloc,core check --target xtensa-esp32-none-elf --features esp-hal,esp32,log-04

# Run tests (preferred — handles SDL_VIDEODRIVER, test separation):
./run_tests.sh unit        # unit + doc tests
./run_tests.sh int         # integration tests (needs SDL_VIDEODRIVER=dummy)
./run_tests.sh leak        # memory leak detection tests
./run_tests.sh all         # all of the above

# Run single unit test directly:
cargo test to_lvgl_half

# Audit doc coverage (should report 0 missing):
RUSTDOCFLAGS="-W missing-docs" cargo doc --no-deps 2>&1 | grep "warning:"

# Run host example (interactive SDL window):
./run_host.sh getting_started1

# Capture screenshot (headless, no window):
./run_host.sh -s getting_started1

# Capture all screenshots:
./run_host.sh -s

# Flash ESP32 example (requires Xtensa toolchain + connected board):
./run_fire27.sh getting_started1
```

## Architecture

`no_std` (embedded) / `std` (host) library providing LVGL bindings for ESP32 UIs.

**Entry point**: `view::run_lvgl::<V: View>(w, h, bufs)` — async task that never returns.

**Layering** (top to bottom):
1. `view` — `View` trait (`create`/`update`) + render loop
2. `widgets` — type-safe LVGL widget wrappers (`Arc`, `Bar`, `Label`, `Scale`, `Led`, …)
3. `display` — `DisplayOutput` trait, DMA `LvglBuffers`, flush pipeline (`DRAW_OPERATION`/`FLUSH_OPERATION` channels)
4. `driver` — `LvglDriver::init()`, tick source, log bridge
5. `oxivgl-sys` — raw C bindings (workspace sys crate, downloads LVGL at build time)

**Flush pipeline** (ESP32 only, `feature = "esp-hal"`):
- LVGL calls `flush_callback` (ISR-safe) → sends `DrawOperation` to `DRAW_OPERATION` channel
- `flush_frame_buffer` task receives it → calls `DisplayOutput::show_raw_data` → signals `FLUSH_OPERATION`
- `wait_callback` (on LVGL task) loops with `waiti 0` until `FLUSH_OPERATION` received, then calls `lv_display_flush_ready`

**Host target**: `LvglDriver::init` calls `init_host_display` (SDL2 backend via `oxivgl-sys`). Unit tests run on host without display hardware.

## Build System

- **Defaults should just work.** `cargo check`, `cargo test`, and `cargo doc` should work without explicit env vars, toolchain overrides, or target flags. Host-specific config (`LIBCLANG_PATH`, nightly toolchain) is handled by `.cargo/config.toml` and `rust-toolchain.toml`. ESP32-specific config (ESP clang path) is auto-detected in `oxivgl-sys/build.rs`. When adding build infrastructure, prefer declarative config over manual flags.
- **All targets**: `oxivgl-sys/build.rs` (cc crate) downloads LVGL v9.5.0 and compiles it from source. For Xtensa, `source ~/.clco-env` puts the ESP toolchain in PATH.
- `lv_conf.h` is **owned by the application**, not the library. The examples ship their config in `examples/conf/lv_conf.h`. Applications using oxivgl as a dependency must supply their own `lv_conf.h` and set `DEP_LV_CONFIG_PATH` to a directory containing it. The workspace default (`examples/conf`) is set in `.cargo/config.toml` and is only authoritative for the examples and tests in this repo.
- **CRITICAL — single LVGL source**: LVGL must only be compiled once, by `oxivgl-sys`. Never add a second compilation (e.g. cmake) from a different LVGL source tree — struct layouts change between versions, causing silent memory corruption on ESP32 (see issue #55).

## Specifications

- **API vision**: `docs/spec-api-vision.md` — design principles.
- **Memory & lifetime safety**: `docs/spec-memory-lifetime.md` — governs all core library changes.
- **Widget wrappers**: `docs/spec-widget-wrapper.md` — how to wrap a new LVGL widget.
- **Example porting**: `docs/spec-example-porting.md` — how to translate LVGL C examples.

## Hard Rules for Examples

Examples are the public face of the library. Every example file (`examples/*.rs`)
MUST satisfy these invariants — **no exceptions**:

- **Zero `unsafe`** — not a single `unsafe` block or `unsafe fn`
- **Zero `use lvgl_rust_sys` / `use oxivgl_sys`** — no direct FFI imports
- **Zero working around missing wrappers** — if a wrapper is needed, add it to `src/`

If an LVGL concept cannot be demonstrated without violating these rules, **simplify
the example** (note what was simplified) or **defer it** (list as blocked in the
coverage table). See `spec-example-porting.md` §2 and §6.

CI enforces this with `grep` — a PR that introduces `unsafe` or `use lvgl_rust_sys` in
any example file will fail.

## Key Constraints

- LVGL must run on a **single task** — no concurrent calls from other tasks/interrupts.
- `LvglBuffers` must be `'static` (allocated as `static mut` by the caller).
- Physical values → LVGL integer range: `widgets::to_lvgl(v, max)` maps to `0..LVGL_SCALE` (1000).
- Logging: use `defmt` feature for embedded, `log-04` for host/demo.
- **`unsafe extern "C" fn` (Rust 2024)**: unsafe calls inside must use explicit `unsafe {}` blocks.
- **`lv_anim_enable_t`** is `bool` in bindings — use `false`/`true` (no named constant).
- **`Align` enum** covers all `lv_align_t` values 0–21 including `Out*` variants; prefer it over raw constants.
