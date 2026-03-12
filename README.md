# oxivgl

Generic LVGL Rust bindings for embedded (no_std) and host (std/SDL2) targets. Provides widget wrappers, the LVGL driver/buffer layer, a `View` trait with generic render loop, and a custom RPM font. Used by the alternator-regulator UI.

## Architecture

```
  altreg-fire27 / altreg-cores3
  (implements DisplayOutput)
          │
          ▼
  oxivgl::view::run_lvgl::<V: View>()
  ├── LvglDriver::init()          — tick source, log bridge
  ├── lvgl_disp_init()            — display + DMA buffers, flush_cb
  ├── V::create()                 — build widget tree
  └── loop: V::update() + lv_timer_handler()
```

## Modules

| Module | Purpose |
|--------|---------|
| `lvgl` | `LvglDriver`: init tick source + log forwarding |
| `lvgl_buffers` | `DisplayOutput` trait; DMA frame buffer; `flush_frame_buffer` task |
| `view` | `View` trait (`create`/`update`) + `run_lvgl::<V>()` |
| `widgets` | Type-safe wrappers: `Obj`, `Arc`, `Bar`, `Label`, `Scale`, `Meter`, `Led`, … |
| `fonts` | `Font` type + `ALTREG_RPM` custom font (generated C source) |

## Key Types

```rust
// Implement this to define your UI screen:
pub trait View: Sized {
    fn create() -> Result<Self, WidgetError>;
    fn update(&mut self) -> Result<(), WidgetError>;
}

// Supply this from the board crate:
pub trait DisplayOutput: Send {
    fn flush(&mut self, op: DrawOperation);
    fn wait_for_flush(&mut self);
}

// Entry point (async task):
pub async fn run_lvgl<V: View>() -> !;
```

## Features

| Feature | Effect |
|---------|--------|
| `esp-hal` | Enables ESP32 tick source (`get_tick_ms` via `esp_hal::time`) |
| `defmt` | `defmt` logging |
| `log-04` | `log` v0.4 logging |

## Build System

`build.rs` compiles LVGL from source via cmake + cc. It expects:
- `DEP_LV_CONFIG_PATH` env var → directory containing `lv_conf.h`
- cmake toolchain files in `oxivgl/src/` for cross-compilation

**cmake include priority**: `target_include_directories` takes precedence over `-I` cflags. Do not rely on `cflag()` in `build.rs` to override a file in the cmake source tree — remove the competing file instead.

**Absolute paths**: `build.rs` uses `std::env::var("CARGO_MANIFEST_DIR")` for all cmake paths because relative paths resolve from the cmake build dir, not the crate root.

## Host Testing / SDL2 Viewer

On host (x86_64-unknown-linux-gnu), `oxivgl` links against the host's SDL2 via `init_host_display`. The SDL2 viewer binary lives in `alternator-regulator/src/bin/sdl_viewer.rs`.

Run:
```sh
LIBCLANG_PATH=/usr/lib64 cargo run -p alternator-regulator --bin sdl_viewer \
  --features sdl --target x86_64-unknown-linux-gnu --config 'unstable.build-std=["std"]'
```
