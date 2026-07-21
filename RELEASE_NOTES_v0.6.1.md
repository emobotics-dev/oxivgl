# oxivgl v0.6.1

A single, targeted fix: **LVGL's transient per-frame render scratch no longer
leaks into a runtime PSRAM pool**. This is what made a heavy screen *slower* on
the original ESP32 after moving LVGL's heap to PSRAM — the exact regression the
0.6.0 pool support could trigger. With it, the render hot path stays in internal
DRAM while the persistent widget tree lives in PSRAM, so both boards keep their
best render rate *with* the heap in PSRAM.

## The fix — render scratch stays internal (#124)

`oxivgl::mem::reserve_pool` (0.6.0) already kept LVGL's **pixel draw buffers**
out of a PSRAM pool, via the callback hooks LVGL exposes for them
(`buf_malloc_cb`/`buf_free_cb`). But LVGL allocates a second, larger class of
short-lived render memory with **no such hook** — a plain `lv_malloc`/`lv_free`
per draw op, every frame:

- **draw-task descriptors** (`lv_draw_add_task`),
- **SW scanline masks** — arc, line, fill, border, triangle, rect-mask,
- **box-shadow** blur/mask buffers,
- **image** mask/transform buffers,
- the **radius/circle mask cache** read per-scanline by arc, rounded-rect,
  border and shadow draws.

Once a runtime pool is registered, LVGL's TLSF heap is one fungible arena, so all
of this becomes eligible to land in the pool. When that pool is PSRAM the cost is
real: the original ESP32 (LX6) reads its SPI PSRAM slowly with no effective
cache, so serving per-frame allocation churn — and per-scanline mask reads — from
PSRAM-resident memory **halved a heavy meter screen's render rate**.

`oxivgl-sys` now patches the SW draw sources at build time to route these
allocations through `oxivgl_render_scratch_{malloc,zalloc,free}`
(`oxivgl::render_scratch`):

- **While a pool is active** they serve from the Rust global allocator —
  internal, DMA-capable DRAM by construction (a BSP that hands out a private
  PSRAM region consumes the peripheral, so the same PSRAM cannot also back the
  global heap). The direct analogue of the existing draw-buffer guard, extended
  from pixel buffers to *all* transient render scratch.
- **With no pool registered, or under `LV_STDLIB_CLIB`**, they delegate straight
  to LVGL's allocator — behaviour is byte-for-byte unchanged.

The guard flips once during driver init, before the first frame, so every block
is allocated and freed under one allocator regime. Nine SW files are routed
*exhaustively* with **pinned site counts**: a future LVGL that adds a non-render
allocation to one of them fails the build rather than silently half-routing a
buffer (which would corrupt the heap). This is safe because every alloc/free pair
is lexically inside its own file — including the mask cache's, whose frees live
in `lv_draw_sw_mask_free_param`/`_cleanup` that arc and line call across function
boundaries. LVGL's gradient cache (`lv_draw_sw_grad.c`) is intentionally left on
LVGL's allocator: it is gradient-only and cross-*function-and-path*, so it needs
a separate, careful pass if a gradient-heavy UI ever calls for it.

Net: render cost tracks **dirty-region complexity, not total UI size** — the
point of putting a large widget tree in PSRAM.

## Measured on hardware

Meter screen (arc + tick scale + glow + labels), `perf_test` on the rig, FPS:

| target | before PSRAM | PSRAM (0.6.0) | **0.6.1** |
|---|---:|---:|---:|
| **ESP32 (Fire27)** meter | 16 | **7** (7–15) | **15** |
| ESP32-S3 (CoreS3) meter | 15 | 20 | **21** |

CoreS3 CPU on the meter dropped **38% → 28%** with the pool and stays there. The
remaining Fire27 15-vs-16 gap is the widget-tree *read* from PSRAM — inherent to
keeping the tree there, and marginal. Soak and full HIL suites pass on both
boards with no stability or perf regression from the changed allocation path.

## Upgrade notes

- **No API changes.** `reserve_pool` and everything else in `oxivgl::mem` are
  unchanged. If you register a PSRAM pool, you simply stop paying the render
  penalty — no code change required.
- **`oxivgl 0.6.1` requires `oxivgl-sys 0.2.4`** (the SW-draw source patches live
  in the sys crate). A plain `cargo update` picks both up.
- Applications on 0.6.0 that sized `LV_MEM_SIZE` large enough to keep the render
  working set in the internal primary heap as a workaround no longer need that
  band-aid.

## Fixed

- **Transient per-frame render scratch leaked into a registered PSRAM pool**
  (#124), halving a heavy screen's render rate on ESP32. Routed to internal DRAM;
  see above.
