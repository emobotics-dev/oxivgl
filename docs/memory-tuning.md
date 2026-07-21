# Memory tuning for widget-heavy UIs

Practical guidance for keeping heap and stack in budget on memory-constrained
targets (ESP32 family). Grounded in measurement — every claim here is backed by
the [`diag`](../src/diag.rs) tooling or a cited audit, not intuition.

> **Scope.** This is general guidance for the library and its users. For the
> specific peak-heap OOM when a modal opens over a heavy view, see
> [`cr-navigator-modal-background-tree-residency.md`](cr-navigator-modal-background-tree-residency.md).

## 1. Measure first

Use [`oxivgl::diag`](../src/diag.rs). The portable, deterministic signal is the
**widget census** — a count of real `lv_obj` instances and nesting depth. Every
object-reduction technique moves it directly, and it works the same on host and
target.

```rust
use oxivgl::diag::{census, Budget, assert_budget};

// After a view's create():
let c = census(&screen);          // e.g. "61 objects, depth 2, ~9760 B est"
log::info!("{c}");

// Fail loudly in debug if a view outgrows its envelope (no-op in release):
assert_budget(&screen, &Budget { max_objects: 80, max_depth: 6 });
```

For *live* heap/stack on target, implement [`ResourceProbe`](../src/diag.rs):
`free_heap_bytes` from `esp_alloc` stats, `stack_high_water_bytes` from the
FreeRTOS task high-water mark. There is no portable default — off-target these
are meaningless, so [`NullProbe`] returns `None`.

### Why not `lv_mem_monitor`?

It reports meaningfully only under `LV_USE_STDLIB_MALLOC = LV_STDLIB_BUILTIN`,
where LVGL owns a TLSF pool to introspect. Under `LV_STDLIB_CLIB` it delegates
to libc `malloc` and reports nothing useful. Since `lv_conf.h` belongs to the
application, a library-level probe cannot assume either — and on target the
figure that usually matters is the **system** heap (esp-hal) anyway, exposed via
a `ResourceProbe`.

A Rust `#[global_allocator]` counter sees only Rust-side bytes under *both*
backends, never LVGL's C-side object allocations: `BUILTIN` allocates from
LVGL's own static array and `CLIB` calls libc directly, so neither passes
through `GlobalAlloc`.

That is a constraint on a *library-level* probe, not on the test suite.
`tests/leak_check.rs` does assert on `lv_mem_monitor` in addition to the
allocator counter — it can, because the tests own their `lv_conf.h` and so know
the backend is `BUILTIN`, the assumption a library cannot make. The C-side
assertion is `#[cfg(lvgl_builtin_malloc)]` and compiles out under CLIB.

## 2. What is already lean — don't chase these

Measured, so nobody re-spends effort here:

- **Wrapper struct size** (`tests/integration/diag.rs::wrapper_struct_sizes`):
  `Obj`/`Label`/`Button` = 40 B, `Bar`/`Arc` = 48 B. An `Obj` is a raw handle
  (8 B) + an **empty, unallocated** `RefCell<Vec<Style>>` (32 B, struct size
  only) + a ZST. This is inline storage (lives in the view), not per-widget
  heap.
- **No per-widget heap** until `add_style` is called — `Vec::new()` does not
  allocate. A "lazy `_styles`" rewrite saves nothing: an empty `Vec` already
  costs no heap, and `Option<Vec<_>>` is the same size as `Vec<_>`.
- **Event handlers are zero-allocation** — `Obj::on` stores a bare `fn` pointer
  in LVGL user-data; the `View` trampoline stores a raw `*mut V`. No boxed
  closures, no `Rc`, no per-handler heap.

## 3. Levers that pay off without costing per-frame compute

Ordered by leverage. All of these reduce memory while leaving (or improving)
runtime compute. Techniques that trade compute for memory — e.g. painting
decorations in a draw callback instead of instantiating child objects — are
**deliberately excluded**: paying per-frame CPU for a few kB is the wrong trade
on these targets.

1. **Free off-screen widget trees.** A view/page/tab that isn't visible should
   not keep its widget tree resident. The `View` lifecycle already supports it:
   `will_hide()` → `lv_obj_clean` → (on return) `create()` → `did_show()` — the
   same teardown `push`/`pop` use. Cost is one rebuild on navigation
   (occasional, not per-frame). This is the single largest reclaimable chunk on
   a multi-screen UI.

2. **Share styles; avoid inline setters at scale.** A shared `Style` (build it
   once with [`Style::new`](../src/style/style.rs) and apply with `add_style`)
   amortizes to a single property buffer for all widgets that use it; per-widget
   inline `style_*(…)` setters instead allocate a per-object local style (a
   `styles[]` entry **plus** its own property buffer). For N identical widgets
   with P properties that is N buffers + N allocations versus 1 buffer — and
   sharing reduces style-refresh compute too, so it is a pure win wherever a
   treatment repeats. You do **not** need to keep the `Style` in scope: every
   `add_style` retains its own `Rc` clone, so build it, apply it across a
   screenful, and drop your handle. See `examples/shared_styles1.rs`.

3. **Flatten the tree.** Layout, style refresh, and draw all recurse over
   depth — deep nesting costs heap *per container* and stack *per level*. A
   shallow, wide tree wins on both. Nesting depth is exactly what `census`
   reports as `max_depth`.

4. **Keep objects at base size.** `lv_obj` only allocates its larger `spec_attr`
   block when an object gains events, groups, or certain flags. Don't attach
   event trampolines or add to groups indiscriminately — purely decorative
   objects then stay at base size.

5. **Tame draw-time spikes.** Several effects allocate a full-region scratch
   buffer *at draw time* — the transient peak that actually trips an OOM:
   - **Shadows**: buffer ∝ object area × `shadow_width`. Lower it, or
     precompose the card as one image.
   - **`opa_layered`, transforms, `clip_corner` (radius + overflow-hidden),
     bitmap masks, non-normal blend modes** each force a full layer buffer.
     Avoid them on large areas. (A plain `bg_opa < 255` does *not* — it is a
     per-pixel blend, not a layer.)
   - **Dithered / multi-stop gradients** allocate; a 2-stop undithered gradient
     does not.

   With a runtime PSRAM pool (`oxivgl::mem::reserve_pool`), this transient render
   scratch is deliberately kept in **internal** DRAM — so these spikes hit the
   internal heap even when the object tree lives in PSRAM. Budget for them there.

6. **Compile out what you don't use** (`lv_conf.h`, app-owned). Disable unused
   widgets (smaller `.text`, fewer registrations). In production builds, turn
   **off** the dev-only `LV_USE_PERF_MONITOR` / `LV_USE_SYSMON` (each adds a
   persistent overlay object *and* per-frame work) and lower `LV_LOG_LEVEL` /
   the `LV_LOG_TRACE_*` flags (log strings cost flash, emission costs cycles).

## 4. Stack

- **Right-size the LVGL task stack against the deepest path** (deepest nesting +
  the heaviest custom-draw callback + flush), then measure the high-water mark
  rather than guessing — over-provisioning steals RAM the heap could use.
- **No large buffers on the LVGL task stack.** Format into a bounded
  `heapless::String` or static scratch, not a big stack array in
  `create()`/`update()`.
- **Shallower trees** (lever 3) also cap layout/draw recursion depth.

## 5. Standing instrument

Wire `census` (and a `ResourceProbe` on target) into your existing heap reports,
and put a per-view `assert_budget` right after each `create()`. A view that
regresses past its envelope then fails in CI/HIL instead of OOMing a device in
the field.
