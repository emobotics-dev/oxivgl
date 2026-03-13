# oxivgl

Safe Rust bindings for LVGL on embedded (`no_std`) and host (`std`/SDL2) targets. Wraps all unsafe LVGL calls behind type-safe APIs — user code never touches `unsafe` or `lvgl_rust_sys` directly.

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
| `view` | `View` trait (`create`/`update`/`on_event`) + `Event` wrapper + `run_lvgl::<V>()` |
| `widgets` | Type-safe wrappers: `Obj`, `Label`, `Button`, `Slider`, `Switch`, `Arc`, `Bar`, `Scale`, `Led`, `Line`, `Image`, `Style`, `Anim`, `AnimTimeline`, `GradDsc` |
| `lvgl` | `LvglDriver`: init tick source + log forwarding |
| `lvgl_buffers` | `DisplayOutput` trait; DMA frame buffer; `flush_frame_buffer` task |
| `fonts` | `Font` type + `ALTREG_RPM` custom font |

## Key Types

```rust
pub trait View: Sized {
    fn create() -> Result<Self, WidgetError>;
    fn update(&mut self) -> Result<(), WidgetError>;
    fn on_event(&mut self, _event: &Event) {}  // safe event dispatch
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

```sh
# Run example on host (SDL2 viewer):
./run_host.sh getting_started1

# Flash to ESP32:
./run_fire27.sh event_trickle

# Capture all screenshots:
./run_screenshots.sh
```

## Features

| Feature | Effect |
|---------|--------|
| `esp-hal` | ESP32 tick source (`esp_hal::time`) |
| `defmt` | `defmt` logging |
| `log-04` | `log` v0.4 logging |

## Build

```sh
# Host check:
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu

# Host tests:
LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu

# Embedded check (requires Xtensa toolchain):
cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04
```

`build.rs` compiles LVGL from source via cmake. Expects `DEP_LV_CONFIG_PATH` pointing to `lv_conf.h`. Adding a new LVGL widget requires enabling it in `conf/lv_conf.h` first (`LV_USE_<WIDGET> 1`).
