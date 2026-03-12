# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

Toolchain: `esp` (nightly Xtensa). Set via `rust-toolchain.toml`.

```sh
# Check host (std, no ESP toolchain needed):
cargo check --target x86_64-unknown-linux-gnu

# Check embedded (requires Xtensa toolchain):
cargo check --target xtensa-esp32-none-elf --features esp-hal,log-04

# Run host unit tests (widgets::tests only; no display hardware needed):
cargo test --target x86_64-unknown-linux-gnu

# Run single test:
cargo test --target x86_64-unknown-linux-gnu to_lvgl_half

# Build fire27 demo (requires ESP32 toolchain + connected board):
cd examples/fire27 && cargo build --release

# Flash + monitor fire27 (must run from examples/fire27/, not workspace root):
cd examples/fire27 && ESPFLASH_PORT=/dev/ttyACM1 cargo run --bin <ex>
```

## Architecture

`no_std` (embedded) / `std` (host) library providing LVGL bindings for ESP32 UIs.

**Entry point**: `view::run_lvgl::<V: View>(w, h, bufs)` — async task that never returns.

**Layering** (top to bottom):
1. `view` — `View` trait (`create`/`update`) + render loop
2. `widgets` — type-safe LVGL widget wrappers (`Arc`, `Bar`, `Label`, `Scale`, `Led`, …)
3. `lvgl_buffers` — `DisplayOutput` trait, DMA `LvglBuffers`, flush pipeline (`DRAW_OPERATION`/`FLUSH_OPERATION` channels)
4. `lvgl` — `LvglDriver::init()`, tick source, log bridge
5. `lvgl_rust_sys` — raw C bindings (external crate, git dep)

**Flush pipeline** (ESP32 only, `feature = "esp-hal"`):
- LVGL calls `flush_callback` (ISR-safe) → sends `DrawOperation` to `DRAW_OPERATION` channel
- `flush_frame_buffer` task receives it → calls `DisplayOutput::show_raw_data` → signals `FLUSH_OPERATION`
- `wait_callback` (on LVGL task) loops with `waiti 0` until `FLUSH_OPERATION` received, then calls `lv_display_flush_ready`

**Host target**: `LvglDriver::init` calls `init_host_display` (SDL2 backend via `lvgl_rust_sys`). Unit tests run on host without display hardware.

## Build System

- **Xtensa targets**: `build.rs` compiles LVGL from source via cmake (`thirdparty/lvgl_rust_sys/lvgl`). Requires `DEP_LV_CONFIG_PATH` env var pointing to the dir containing `lv_conf.h` (provided by the consumer crate via `links` metadata).
- **Host targets**: `lvgl_rust_sys`'s own `build.rs` compiles LVGL; `build.rs` in this crate does nothing.
- cmake toolchain files: `src/toolchain-esp32.cmake` / `src/toolchain-esp32s3.cmake`
- `lv_conf.h` lives in `conf/`; cmake `target_include_directories` takes priority over `-I` cflags — don't duplicate the header in the cmake source tree.

## Key Constraints

- LVGL must run on a **single task** — no concurrent calls from other tasks/interrupts.
- `LvglBuffers` must be `'static` (allocated as `static mut` by the caller).
- Physical values → LVGL integer range: `widgets::to_lvgl(v, max)` maps to `0..LVGL_SCALE` (1000).
- Logging: use `defmt` feature for embedded, `log-04` for host/demo.
- **Adding a new LVGL widget**: enable it in `conf/lv_conf.h` first (`LV_USE_<WIDGET> 1`) or the functions won't appear in the generated bindings.
- **`unsafe extern "C" fn` (Rust 2024)**: unsafe calls inside must use explicit `unsafe {}` blocks.
- **`lv_anim_enable_t`** is `bool` in bindings — use `false`/`true` (no named constant).
- **`Align` enum** covers all `lv_align_t` values 0–21 including `Out*` variants; prefer it over raw constants.
