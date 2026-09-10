# oxivgl v0.9.1

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
`0 FPS over 0 scenes`. Without enough heap it would die mid-run — 64 KiB free,
against a measured peak of 44,404 B on ESP32 and 48,292 B on ESP32-S3.

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

## The heap gate was very nearly the same defect again

The first cut of that gate asked for 128 KiB, taken from `lv_demo_benchmark.c`.
That number is a `#warning` — "It's recommended to have at least 128kB RAM" —
and it tests `LV_MEM_SIZE`, the configured primary pool. Used as a hard runtime
precondition on *free* heap it refused every ESP32-class board, including the
one this feature was written for: an ESP32 with 71,884 B free and an ESP32-S3
with 88,312 B, both of which complete every scene.

Which is the defect described immediately above, moved from compile time to run
time: a documented figure treated as a requirement, refusing a configuration
that demonstrably has the memory. It was caught on hardware before release.

## LVGL's log channel was silent on release builds

The embedded `lv_log_register_print_cb` bridge discarded LVGL's level and
emitted every message as `debug!`, which `release_max_level_info` deletes at
compile time — so `LV_LOG_ERROR` and `LV_LOG_WARN` left the image along with the
traces. It maps the level now.

LVGL reports some failures by warning and retrying rather than asserting, so
"Allocating layer buffer failed. Try later" was unobservable: a diagnosable
stall presenting as a silent hang.

## The heap gate could not see fragmentation

The gate compared `lv_mem_monitor`'s `free_size` against 64 KiB. That is a
*sum*, and the thing it was protecting is a *single* allocation — so a heap
holding plenty of memory in small pieces passed the check and the run then died
on one request. Measured on an ESP32: 39,172 B free, no 23,760 B block, and the
renderer retrying that allocation 73,679 times.

The bound was already available and belongs to the caller, not to us. LVGL
slices a layer into chunks of `LV_DRAW_LAYER_SIMPLE_BUF_SIZE` — `lv_refr.c`
derives the row count from it — so no single layer-buffer request can exceed it,
and it is per-board because every application sets it in its own `lv_conf.h`.

The check **asks the registered draw-buffer allocator for one such chunk** and
frees it again, rather than reading a free-block figure. That distinction is the
whole fix: which allocator serves layer buffers depends on configuration —
`mem::reserve_pool` routes them to the Rust heap — so a number taken from LVGL's
pool describes memory the renderer will not be using, and would have passed in
exactly the case that wedges.

`BenchmarkError::FragmentedHeap` reports that case separately, because the fix
differs: adding memory does not help a fragmented heap, while a smaller chunk
lets the renderer ask for slices it can still place. `InsufficientHeap` carries
the largest block too, so a refused caller can tell the two apart at a glance.

This is not confined to the benchmark. The same shortage was seen wedging
ordinary page navigation on a board whose free memory was split across two
regions, neither big enough for the configured chunk.

## A failed draw-buffer allocation stalled the render thread forever

`lv_draw_layer_alloc_buf` treats a failed allocation as transient: it logs and
returns NULL. The software draw unit then declines the task and
`draw_buf_flush` loops until it succeeds — *before* `disp->flushing` is set, so
no flush is ever issued. One transient refusal is a permanent stall, with no
assert and no panic, and until the log fix above, no output either.

It is not a shortage of memory but of *contiguity*. On an ESP32 running the
benchmark the Rust heap had 35,080 B free and no 23,760 B block, while LVGL's
own pool held a 33,568 B block that went unused for the entire stall — because
the draw-buffer guard routes every such allocation to the Rust allocator.

`mem::declare_pool_internal()` lets a failed allocation retry from `lv_malloc`.
It is opt-in because the guard exists for a real hazard: a runtime pool may be
PSRAM, which the ESP32 cannot DMA from. An application that knows its pools are
internal has no such hazard; one that declares it with an external pool
re-opens it.

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
An **exact** pin (`oxivgl = "=0.8.0"`) cannot take 0.9.1 by `[patch]`: the
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
