# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## MANDATORY: Consult Specs Before Any Change

**Before designing, implementing, or reviewing ANY code change**, read and follow the relevant specifications in `docs/`. This is not optional — every design decision, API addition, wrapper method, example, and test MUST comply with these specs. Check compliance DURING implementation, not after.

| Spec | When to consult |
|------|----------------|
| `spec-api-vision.md` | Any public API change — naming, return types, safety model |
| `spec-memory-lifetime.md` | Any code touching pointers, lifetimes, Drop, styles, animations |
| `spec-widget-wrapper.md` | Any new widget/module — struct pattern, SAFETY comments, doc comments, tests |
| `spec-testing.md` | After ANY new public method — integration test required, leak test if allocating |
| `spec-example-porting.md` | Any new/modified example — run_host.sh, README, screenshots, doc audit |
| `spec-git-workflow.md` | Commits, branches, PRs |

## Build & Test

Toolchain: `esp` (nightly Xtensa). Set via `rust-toolchain.toml`.

```sh
# Check host (std, no ESP toolchain needed):
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu

# Check embedded (requires Xtensa toolchain):
cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04

# Run tests (preferred — handles SDL_VIDEODRIVER, test separation):
./run_tests.sh unit        # unit + doc tests
./run_tests.sh int         # integration tests (needs SDL_VIDEODRIVER=dummy)
./run_tests.sh leak        # memory leak detection tests
./run_tests.sh all         # all of the above

# Run single unit test directly:
LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu to_lvgl_half

# Audit doc coverage (should report 0 missing):
LIBCLANG_PATH=/usr/lib64 RUSTDOCFLAGS="-W missing-docs" cargo +nightly doc --target x86_64-unknown-linux-gnu --no-deps 2>&1 | grep "warning:"

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
5. `lvgl_rust_sys` — raw C bindings (external crate, git dep)

**Flush pipeline** (ESP32 only, `feature = "esp-hal"`):
- LVGL calls `flush_callback` (ISR-safe) → sends `DrawOperation` to `DRAW_OPERATION` channel
- `flush_frame_buffer` task receives it → calls `DisplayOutput::show_raw_data` → signals `FLUSH_OPERATION`
- `wait_callback` (on LVGL task) loops with `waiti 0` until `FLUSH_OPERATION` received, then calls `lv_display_flush_ready`

**Host target**: `LvglDriver::init` calls `init_host_display` (SDL2 backend via `lvgl_rust_sys`). Unit tests run on host without display hardware.

## Build System

- **All targets**: `lvgl_rust_sys`'s `build.rs` (cc crate) compiles LVGL from source. For Xtensa, `source ~/.clco-env` puts the ESP toolchain in PATH.
- `lv_conf.h` lives in `conf/`; pointed to by `DEP_LV_CONFIG_PATH` in `.cargo/config.toml`.
- **CRITICAL — single LVGL source**: LVGL must only be compiled once, by `lvgl_rust_sys`. Never add a second compilation (e.g. cmake) from a different LVGL source tree — struct layouts change between versions, causing silent memory corruption on ESP32 (see issue #55).

## Specifications

- **API vision**: `docs/spec-api-vision.md` — design principles.
- **Memory & lifetime safety**: `docs/spec-memory-lifetime.md` — governs all core library changes.
- **Widget wrappers**: `docs/spec-widget-wrapper.md` — how to wrap a new LVGL widget.
- **Testing**: `docs/spec-testing.md` — test tiers, when to write what, portability.
- **Example porting**: `docs/spec-example-porting.md` — how to translate LVGL C examples.
- **Git workflow**: `docs/spec-git-workflow.md` — branching, commits, PRs, CI.

## Hard Rules for Examples

Examples are the public face of the library. Every example file (`examples/*.rs`)
MUST satisfy these invariants — **no exceptions**:

- **Zero `unsafe`** — not a single `unsafe` block or `unsafe fn`
- **Zero `use lvgl_rust_sys`** — no direct FFI imports
- **Zero working around missing wrappers** — if a wrapper is needed, add it to `src/`

If an LVGL concept cannot be demonstrated without violating these rules, **simplify
the example** (note what was simplified) or **defer it** (list as blocked in the
coverage table). See `spec-example-porting.md` §2 and §6.

CI enforces this with `grep` — a PR that introduces `unsafe` or `lvgl_rust_sys` in
any example file will fail.

## Key Constraints

- LVGL must run on a **single task** — no concurrent calls from other tasks/interrupts.
- `LvglBuffers` must be `'static` (allocated as `static mut` by the caller).
- Physical values → LVGL integer range: `widgets::to_lvgl(v, max)` maps to `0..LVGL_SCALE` (1000).
- Logging: use `defmt` feature for embedded, `log-04` for host/demo.
- **`unsafe extern "C" fn` (Rust 2024)**: unsafe calls inside must use explicit `unsafe {}` blocks.
- **`lv_anim_enable_t`** is `bool` in bindings — use `false`/`true` (no named constant).
- **`Align` enum** covers all `lv_align_t` values 0–21 including `Out*` variants; prefer it over raw constants.
