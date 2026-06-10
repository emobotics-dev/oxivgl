# CR: navigator modal keeps the full background view tree resident (peak-heap OOM)

> **Status:** open — fix intended in **oxivgl navigator** (`src/navigator.rs`).
> **Severity:** high — peak-heap OOM when a modal opens over a heap-heavy view on
> a memory-constrained target (ESP32-S3, ~16 KB free LVGL heap shared with the BLE
> blob). Half-builds the modal, leaves a near-opaque scrim, never recovers →
> **solid black display**.
> **Origin:** alternator-regulator OSD target-factor modal; cores3 (ESP32-S3)
> cold-boot black screen, 2026-06-10. Three-way heap audit + HIL evidence below.

## Symptom

Opening `NavAction::Modal` over an active full-screen view OOMs on cores3: during
the modal's `create()`, an `lv_malloc` returns NULL → `LV_ASSERT_MALLOC` →
`LV_ASSERT_HANDLER` (the app panics loudly). The modal tree is half-built, the
backdrop's near-opaque scrim covers the screen, and the modal never dismisses →
**solid black, no recovery**. fire27 (more free heap, less BLE pressure) survives
the same modal.

## Root cause — background tree + modal tree fully resident simultaneously

`modal()` / `modal_boxed` (`navigator.rs:363-438`) builds the backdrop + the
modal's widget tree on `lv_layer_top()` **while the background view's entire
widget tree stays alive** — by design ("modals overlay, full-screen views
replace", `spec-navigation.md` §2.4). Peak resident heap therefore equals:

```
background-view tree  +  backdrop  +  modal tree  +  their style/draw allocs   (all at once)
```

### Measured (audit of generated views + fire27 `:free` / heap reports)

- Per-object cost ≈ 120–200 B (`lv_obj_t` + `spec_attr` + `styles[]`) + ~16 B per
  `add_style` slot. (`lv_obj.c:419`, `lv_obj_style.c:136/765`.)
- cores3 steady-state heap (BLE blob + GlanceView) = **41 KB used / 57 KB → ~16 KB free**.
- OSD modal ≈ 11 objects + ~10 inline styles ≈ 2–3 KB resident, **plus** a transient
  ~2 KB card-shadow spike at create.
- Over **GlanceView** (~22 widgets + 12 styles ≈ 4–6 KB) the margin is already thin.
  Over **MeterView** (67 widgets + 43 styles ≈ 12–16 KB resident) the background
  tree alone nearly exhausts the 16 KB *before the modal allocates anything* →
  guaranteed OOM.
- fire27 (57 KB heap, lighter BLE): heap peaks **33 KB / 57 KB** → survives.

The retained background tree is **invisible anyway**: the OSD scrim is
`bg_opa ≈ 150` (near-opaque black), so almost nothing of the background composites
through.

## Proposed fix

On modal open, tear down the background screen's **widgets** (not its `View`
struct) and rebuild them on dismiss — the same widget teardown/rebuild that
`push`/`pop` already use (`lv_obj_clean`, `navigator.rs`). The evolved `View`
trait already supports this: `will_hide()` → `lv_obj_clean` → (on dismiss)
`create()` → `did_show()`. Reclaims the full background-tree heap (~4–16 KB) for
the modal's lifetime.

| Option | Heap reclaimed | Trade-off |
|---|---|---|
| **A. Free background widgets on open, rebuild on dismiss** (recommended) | full background tree (~4–16 KB) | one rebuild (~ms) on dismiss; the background can't composite *live* under the modal — fine, the scrim is near-opaque |
| B. Hide background tree (`HIDDEN` flag) | **0 KB** (objects stay allocated) | only saves render cost, **does not fix the OOM** — reject |
| C. Render the modal as a full-screen `Replace`/`push` | full background tree | loses the dim-live-screen *semantics*; visually equivalent here (scrim hides the background) |

Recommend **A**. An optional `modal_preserving_background()` variant could keep
today's behavior for heap-rich targets or genuinely translucent overlays, with
A as the default.

## Non-fixes — ruled out by the audit (do not chase)

- **Semi-transparent scrim does NOT allocate a full-screen transparency layer
  buffer.** `bg_opa < 255` is a per-pixel blend, not a layer; `calculate_layer_type`
  (`lv_obj_style.c:1087-1097`) triggers a layer only on `opa_layered` / transform /
  `bitmap_mask` / non-normal `blend_mode` — none used by the OSD. **0 KB to save.**
- **Draw buffers are correctly in `.bss`** (`LVGL_BUFS`, `ui/mod.rs`), not heap.
- **No global LVGL heap pool** (`LV_USE_STDLIB_MALLOC = CLIB`); style/image/object
  caches already at 0 in `lv_conf.h`.
- **Shadows are transient** (alloc+free within the draw call) — but the ~2 KB
  card-shadow spike *coincides with the create-time OOM instant*; cheap app-side
  mitigation is to lower `shadow_width`.

## HIL evidence

fire27 (1 Mbaud UART, `reset_reason = ChipPowerOn`): clean boot, heap peaks
33 KB/57 KB, OSD opens + dismisses. cores3 (CDC): firmware stays alive (60 s heap
reports) at 41 KB/57 KB used while the display is solid black — i.e. the modal
OOM'd at create and left a broken overlay, not a crash-loop.
