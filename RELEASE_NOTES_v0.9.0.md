# oxivgl v0.9.0

LVGL's benchmark demo becomes something an application can call, and the two
defects that stood in the way turn out to be worth more than the feature.

## The benchmark as an in-application diagnostic (#11)

`lv_demo_benchmark` is the broadest end-to-end check LVGL ships — ~30 scenes
covering rectangles, borders, shadows, images, text, arcs, masking, blending and
scrolling, reporting averaged FPS, CPU, render and flush time per scene. It
exercises the whole pipeline: draw unit, buffer strategy, DMA flush, display
driver.

```rust
oxivgl::demo::benchmark(|summary| {
    // per-scene and total: fps, cpu_pct, render_ms, flush_ms, peak_heap_bytes
})?;
```

It could not simply be called. The demo does **not** create a screen of its own:
it builds every scene on the *active* screen and starts each one with
`lv_obj_clean(lv_screen_active())`, which frees the widgets a running `View`
still holds raw pointers to — a use-after-free on the next `View::update` and a
double free when the `View` drops. Saving and restoring `lv_screen_active()`
does not help, because the pointers are already dangling. The run gets a
throwaway screen, and the application's is restored before that screen is
dropped.

Preconditions are refused rather than allowed to produce plausible numbers.
Without `LV_USE_PERF_MONITOR` every scene records nothing and the run reports
`0 FPS over 0 scenes`. Without enough heap it would die mid-run.

## The demos' memory guards tested the wrong quantity

Both demos gate on `LV_MEM_SIZE` — `lv_demo_widgets.c` at 38 KB,
`lv_demo_benchmark.c` at 128 KB — and the benchmark `#error`s without the widgets
demo, so both gate the benchmark.

That was right when `LV_MEM_SIZE` was the whole heap. Since `lv_mem_add_pool` it
is only the *primary*, with `LV_MEM_POOL_EXPAND_SIZE` capping what may be
registered later. So a board keeping a small internal primary and spilling the
bulk into PSRAM — the shape `mem::reserve_pool` exists to create, and the only
one that fits beside ESP32's link-asserted main-stack floor — was refused while
having the memory.

Both guards are widened to `LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE`, which judges
single-pool builds exactly as before since that define is 0 there. A ceiling is
still only what *may* be registered, so `benchmark()` asks the heap that exists
via `lv_mem_monitor` and returns `InsufficientHeap` with both figures.

## A flush completion could be lost forever

`SemaphoreFlushSync::wait()` passed `None` to `take()` — wait forever — and the
comment there predicted the consequence. It happened: on an ESP32 under
sustained load the panel froze mid-frame for 100+ s while every other task kept
running. It is bounded at three times the transfer's own 300 ms ceiling now,
with the expiry logged, so a lost completion costs one frame and announces
itself.

This bounds the symptom and does not explain it. With the bound compiled in a
board wedged again without the bound ever firing, so the render thread was not
in that wait at all.

## `WaitiFlushSync` is deprecated

It parks the core with `waiti 0` for the whole transfer, and under a split
render loop that is the wrong shape entirely: the refresh blocks from the
executor's idle hook, so parking there halts the whole scheduler rather than
only the render thread. It also has a lost-wakeup window `SemaphoreFlushSync`
does not. Register `SemaphoreFlushSync` (feature `rtos-sem`); the stock board
harness now does.

## Breaking

- **`Ui::init` no longer knows PARTIAL vs DIRECT.** One constructor takes
  `display::Buffers`: `Buffers::partial` (SPI stripes) or `Buffers::full`
  (scan-out frames). `init_direct` and `lvgl_disp_init_direct` are gone.
- **The esp-hal 1.2 stack.** The requirement is `1.2`, resolving to stable
  **1.2.1** and unifying with a consumer's plain `^1.2`. Nothing names a
  pre-release, so no opt-in is needed. If you patch the fork, patch it at
  `local-1.2` or later: a caret that names no pre-release cannot accept a fork
  at `-rc.0`, and cargo drops the unmatched patch silently — leaving a stock
  build that looks healthy and has none of the fixes.

## Upgrading from 0.8.x

`oxivgl-sys` declares `links = "lv"`, so only one copy may exist in a build.
An **exact** pin (`oxivgl = "=0.8.0"`) cannot take 0.9.0 by `[patch]`: the
requirement is unsatisfiable by the patched crate, cargo keeps the registry copy
as well, and the failure is a `links` collision or a split of the
`oxivgl_render_scratch_*` symbols — neither of which names the version that
caused it. Use carets, not `=`.

Two separate things, easily conflated:

**Version requirements.** A caret on each. A consumer that only uses `oxivgl`
can omit `oxivgl-sys` entirely — `oxivgl`'s own requirement already selects a
compatible one. Declare it only if you call into it directly (`lv_mem_monitor`,
`lv_mem_add_pool`, `lv_sysmon_*` and the like are not wrapped).

**`[patch.crates-io]` entries.** These must list **both** crates at the **same
rev**. A patch redirects the *source*, and the `oxivgl_render_scratch_*` hooks
are emitted by the sys crate with their Rust half in `oxivgl`; resolve the two
from different places and they drift into a link failure. Patching only
`oxivgl` is not enough, whatever the requirements say.

## Enabling the demo

An `lv_conf.h` decision, not a cargo feature: the demo sources sit inside
`#if LV_USE_DEMO_BENCHMARK`, so a feature would compile them to nothing while
appearing to work. `oxivgl-sys` compiles LVGL's `demos/` tree when
`LV_BUILD_DEMOS` is set, so no per-application `#include` shims are needed.
`examples/conf-benchmark` is a ready-made configuration and `./run_benchmark.sh`
selects it.

It costs roughly 900 KB of flash, nearly all image assets. On a 4 MB part with
OTA partitions that may not fit at all — it belongs in a diagnostic build, never
a shipped one.
