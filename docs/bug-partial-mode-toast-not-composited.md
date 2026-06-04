# BUG: toast on `lv_layer_sys()` is not deterministically composited in PARTIAL render mode (ESP32)

> **Status:** **superseded** — the repaint-window mitigation (which only
> reached ~91 % and left a deterministic cold-boot miss) has been **replaced by
> an architectural fix**: toasts no longer use `lv_layer_sys()` at all; they are
> rendered on the **active screen** (re-parented across navigation/modals),
> which the panel composites as reliably as any other widget. This removes the
> dependency on the unreliable system-layer compositing entirely rather than
> trying to out-run it. See **Fix v2** below. The original repaint-window
> analysis and HIL numbers are kept for the record.
> **Severity:** high — a passive toast (e.g. a **No-SD safety warning**) shown
> without an accompanying navigation **silently fails to appear ~10–40 % of boots**.
> A user can miss a real warning.
> **Affects:** oxivgl `152da3a` (v0.3.3), ESP32 (fire27 / M5Stack Fire), LVGL 9.5,
> `LV_DISPLAY_RENDER_MODE_PARTIAL`. **Does NOT reproduce on host** (FULL/DIRECT
> compositing) — so the existing host integration tests cannot catch it.
> **Reported from:** alternator-regulator HIL, 2026-06-03 (camera + serial).

## Summary

The v0.3.1 fix (`fix: toast visibility and sequential playback`, `eddaec4`)
correctly addressed the *systematic* invisibility (root view built on the
never-loaded default screen → `lv_layer_sys` compositing never armed) by making
`push_root` create + `lv_screen_load` its own screen, plus a
`container.invalidate()` after creating the toast. On **host this is 100 %**.

On **fire27 (PARTIAL mode) a residual non-determinism remains**: a toast posted on
a static screen (no nav) composites onto the panel only *probabilistically*. The
`container.invalidate()` makes the toast's own stripe dirty and that stripe **is
flushed** — but the `lv_layer_sys` content is **sometimes not composited into it**.

## Quantified evidence (fire27, HIL, forced-no-SD boot → real No-SD toast on the boot page, no nav)

Each trial: `:nosd` (SW-reset → forced-no-SD boot) → the logger posts a Warning
toast (`[toast] Warning "No SD card - logging off"`) on the boot page ~5 s in,
with **no** navigation → camera snap inside the 10 s window → check the panel.

| Render trigger in `display_toast_now` | toast visible on boot page |
|---|---|
| `container.invalidate()` (v0.3.3, as shipped) | **~60 %** (3/5) |
| `lv_obj_update_layout(container)` + `lv_obj_invalidate(lv_screen_active())` + `lv_refr_now(NULL)` | **~89 %** (8/9) — improved, still misses |
| **any** subsequent nav / `lv_screen_load` (page switch) | **100 %** (always) |
| host (FULL/DIRECT) | **100 %** (always — never reproduces) |

The stronger full-screen invalidate + synchronous refresh **reduces but does not
eliminate** the miss. Only an actual `lv_screen_load` is deterministic.

## What is ruled out

- **Not the consuming app.** It only calls the documented `navigator::post_toast`
  (cross-task channel → `drain` → `display_toast_now`). It does not touch the
  render loop, the flush pipeline, or LVGL directly.
- **Not toast geometry.** Reproduced with the app's view honoring the new
  convention (no size/pos override — only `bg_color` + label, like
  `toast_hil_demo::ToastView`), and `lv_obj_update_layout` before invalidate does
  not close it.
- **Not the board blit (`DisplayOutput::show_raw_data`).** Per-stripe flush
  instrumentation shows the toast's `320×24 @ (0,216)` stripe **is** sent to the
  flush task on a miss, and the background view (meter) blits correctly every
  frame. The miss is upstream of the blit — in what `lv_refr` composites **into**
  the stripe.
- **Not the v0.3.1 `push_root`-loads-its-own-screen fix** — that is necessary and
  present; this is the *residual* on top of it.

## Localization

In PARTIAL mode, `lv_refr` redraws each invalidated area bottom-up
(screen → `lv_layer_top` → `lv_layer_sys`). When the **toast's own
invalidation** (or even a full-screen invalidation) drives the redraw, the
`lv_layer_sys` child is composited only *sometimes* — timing-dependent, and worse
in the early-boot window. When a **screen change** (`lv_screen_load`) drives the
redraw, it is composited every time. So the non-determinism is in how the
partial-mode refr composites the system layer onto a non-screen-load redraw — in
the oxivgl/LVGL render path, not the app and not the blit.

## Hypotheses for the maintainer

1. **`lv_layer_sys` refresh state vs a plain area invalidation.** A screen load
   appears to set/refresh layer state that a bare `lv_obj_invalidate` does not,
   so the sys layer is reliably included only after a load. Compare the display's
   `inv_areas` + layer `lv_obj`/refresh flags on a hit vs a miss frame.
2. **The custom render loop cadence.** `run_app_nav` drives `lv_timer_handler` 4×
   per cycle then drains toasts (creating the toast *after* that cycle's
   handler), so the toast first renders on the next cycle. An early-boot timing
   window between create-invalidate and the next refr may drop the sys-layer
   composite. Does invalidating/redrawing across the *first few* frames (not
   once) close it?
3. **Upstream LVGL 9.5 partial-mode + system-layer** edge: worth checking whether
   a minimal LVGL-only partial-mode repro (object on `lv_layer_sys`, invalidate,
   no screen load) reproduces independent of oxivgl.

## Suggested fix directions (none verified to 100 % yet)

- Force a `lv_screen_load(lv_screen_active())`-equivalent on toast show (the only
  trigger proven 100 % reliable), if a no-op reload can be made safe/cheap.
- Or re-invalidate the toast/screen for the first N render cycles after show
  (spray past the race) rather than a single invalidate.
- A full-screen invalidate + `lv_refr_now` alone is **not** sufficient (89 %).

## Fix v2 — render the toast on the active screen (current)

The repaint-window approach (below) treated the symptom: it gave the toast more
chances to land on the *unreliable* `lv_layer_sys()`. The HIL run proved that is
not enough — on the cold boot the system layer simply never composites the
toast, no matter how many repaints. So stop using the system layer.

The decisive observation is already in the HIL evidence: on a miss, **"the
background view (meter) blits correctly every frame."** The background view
lives on the **active screen**, and it composites reliably even on the cold boot
that drops the toast. Only `lv_layer_sys()` fails. The fix follows directly:

- The toast container is created on the **active screen** (or the modal backdrop
  while a modal is open), not on `lv_layer_sys()`. It is now an ordinary screen
  child, drawn through the exact path every working widget uses — there is no
  separate system layer whose compositing can be unreliable.
- `Navigator::reattach_toast()` re-parents the toast onto the current topmost
  surface whenever that surface changes — `push`/`pop`/`replace` (→ new active
  screen) and modal open/dismiss (→ backdrop / back to screen) — so the public
  contract is preserved: the toast persists across page switches and stays above
  any modal.
- Layout is settled synchronously (`lv_obj_update_layout`) before the single
  post-create `invalidate()`, so even that one redraw targets the final stripe
  deterministically — no race, nothing probabilistic.

Why this is reliable *under all circumstances*: it is not a higher-probability
mitigation, it is an architectural change. A toast on the active screen is
exactly as reliable as the meter, which the HIL data shows never misses —
including on the cold boot.

Trade-off: the toast rides the active screen, so an animated screen transition
slides it with the page (instant loads, the default, are unaffected). The
above-modal contract is preserved by re-parenting onto the modal backdrop.

Covered by host integration tests in `tests/integration/navigator_toast.rs`
(`show_then_dismiss_clears_surface`, `persists_across_push_replace_pop`,
`toast_rides_modal_backdrop_then_returns_to_screen`,
`toast_shown_during_modal_lands_on_backdrop`, `input_transparency_strips_clickable`),
which assert the toast is parented to the active screen / backdrop and **never**
to `lv_layer_sys()`.

### On-target measurement (fire27, board `5864000922`, ESP32 rev3.1, 2026-06-04)

Method: `toast_hil_demo` (persistent toast on first `update()`, no nav) in
PARTIAL mode, repeated SW-reset boots, panel photographed each boot via the rig
camera, hit/miss classified by the toast-bar color (every frame verified *live*
with a 2-frame sensor-noise check to exclude a frozen viewfinder).

| build | result |
|---|---|
| **Fix v2** (active-screen toast) | **12 / 12 HIT**, 0 miss |
| **v0.3.3 baseline** (sys-layer toast, the *shipped, buggy* code) | **16 / 16 HIT**, 0 miss |

**Honest reading: this rig run does NOT discriminate the fix.** The buggy
baseline showed the toast on every boot too, so `toast_hil_demo` **does not
reproduce the field failure** under this SW-reset process. The alternator-
regulator's `:nosd` miss needs conditions this example lacks — most likely its
early-boot contention (BLE/sensor init) and/or its *timed* toast posted ~5 s in
on a static page, versus the demo's persistent toast raised on the first tick.
Same HIL *process*, different *workload*.

So the rig confirms only that the active-screen toast **composites correctly and
the re-parenting / forced-layout path runs without fault on real PARTIAL
hardware** (12/12). It does **not** prove the cold-boot fix. That proof must come
from the **consumer's `:nosd` count built against this branch** — the only thing
that exercises the failing path. The fix rests on the architectural argument
(the toast now lives on the same surface as the background view, which the HIL
evidence shows composites on every frame including the failing boot), backed by
full host test coverage — not on a rig miss-rate delta, which this example
cannot produce.

## Original repaint-window analysis (superseded — kept for the record)

Reframing of the root cause after reading the LVGL 9.5 refr path
(`src/core/lv_refr.c`, `src/display/lv_display.c`):

- `refr_area_part` composites `lv_layer_top` **and** `lv_layer_sys`
  *unconditionally* onto **every** redrawn area — there is no per-layer "armed"
  state, and the sys layer is full-screen-sized from `lv_display_create`. So
  hypothesis 1 ("a screen load refreshes layer state a bare invalidate does
  not") does not hold for 9.5.
- What a screen load (page switch) actually does that a one-shot invalidate does
  not is **repaint over many frames**: it destroys/rebuilds a screen tree and
  re-lays-out across several refr cycles.
- The toast's default container is `LV_SIZE_CONTENT`-high and `BottomMid`
  aligned. `lv_obj_refr_size` grows it keeping the **top** fixed (briefly off the
  bottom of the screen) and `lv_obj_refr_pos` re-applies the bottom alignment;
  the first refresh after creation can race that settling.

The repaint window (`TOAST_REPAINT_FRAMES`) re-invalidated the toast for the
first few render-loop iterations. It lifted warm-boot reliability to ~91 % but
**could not** fix the cold boot — confirming the loss is in system-layer
compositing itself, not the redraw count. Removed in favour of **Fix v2**.

## Reproduction harness (HIL)

Build the consumer with a verb that posts a passive `lv_layer_sys` toast at boot
on the root page with no nav, snap the panel within the toast window across N
fresh boots, count visible. (Here: `:nosd` → No-SD boot toast; ~1 in 3–10 boots
the toast is absent though `[toast] …` is logged.) Camera, not just the serial
`[toast]` log — the log only proves the view was *created*, not *composited*.

## HIL verification of the repaint-window fix (fire27, 2026-06-03)

Method: `:nosd` (SW-reset → forced-no-SD boot → real No-SD Warning toast on the
GlanceView root page, **no nav**) repeated N× per session; camera snap ~1.5 s
after the `[toast] …` line; each photo classified hit/miss by eye. Two full
batches at two window sizes (`TOAST_REPAINT_FRAMES = 4`, the shipped value, and
`= 30`). Every photo read — no sampling.

| window | render rate | which boot missed |
|---|---|---|
| `=4` (shipped) | **10/11 (~91 %)** | only the **first** forced-no-SD boot of the session |
| `=30` | **10/11 (~91 %)** | only the **first** forced-no-SD boot of the session |
| pre-fix (`container.invalidate` only) | ~60 % (3/5) | (random) |

**Findings / insights:**

1. **Big improvement, not a fix.** Miss rate dropped from ~40 % to ~9 %. But it
   is **not 0** — a No-SD *safety* warning is still dropped, which fails the bar.
2. **The residual is NOT stochastic and NOT a window-length problem.** Across two
   independent batches the **warm** boots were **20/20** (trials 3–12 in each),
   and the **only** miss each time was the **first forced-no-SD boot of the
   session** (0/2). Growing the window 4 → 30 changed nothing (still 10/11, still
   the first boot). If the per-frame composite were probabilistically failing,
   7.5× more frames would have driven misses toward 0; it did not. So the loss is
   **correlated with the cold/first boot**, where the early-boot render sequence
   (display bring-up, first layout, BLE init contention) differs — exactly the
   case a real user hits on **first power-on with no card**.
3. **Window size has a real perf cost.** During the `=30` window the LCD ran
   ~22 FPS / 52 % CPU / ~41 ms vs the normal ~28 FPS / 40 % / ~17 ms — i.e. just
   cranking `TOAST_REPAINT_FRAMES` is not free even if it had worked.

**Recommended next directions (a frame-count window can't cover a cold-boot loss):**

- Gate the *first* toast of a boot until the display/refr pipeline is confirmed
  live (e.g. defer the show until after the first full flush has completed), then
  apply the repaint window — so the toast never lands during the cold-boot window
  where its repaints don't composite.
- Or close the loop: keep re-invalidating the toast's area until its composite is
  *confirmed* (e.g. a flush-completion / draw-callback signal for the toast
  region), rather than a fixed frame budget.
- Investigate why the **first** forced boot specifically loses every repaint
  while warm boots are 20/20 — capture the display `inv_areas` / refr timing on
  that first boot vs a warm one. That delta is the real residual.
