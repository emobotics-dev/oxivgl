# Feature Request (upstream → oxiforge / oxivgl): navigator-level global status / toast overlay

> **Status:** to be filed upstream against the oxiforge/oxivgl repo. Kept here so
> the design isn't lost; the alternator-regulator firmware will reuse this a lot.
> **Target:** oxivgl runtime (`run_app_nav` / `run_lvgl` render loop + `Navigator`).
> **Origin:** alternator-regulator no-card warning popup.

## Problem

Applications need a status / notification popup ("toast") that can be raised from
**anywhere** (a background async task, any view, the app shell) and that appears
**regardless of which `View` is currently active**.

Today the only overlay mechanism is `NavAction::Modal`, returned from a view's
`update()` / `on_event()`. That forces every view which should be able to *show* a
global message to drain the app's status channel itself and return
`NavAction::modal(...)`. Problems observed in production firmware:

1. **Misses views that don't opt in.** Our root `GlanceView` did not drain the
   status channel, so a "No SD card" warning posted at boot never appeared — only
   `MeterView` showed it. A global condition must not depend on which page happens
   to be foreground.
2. **Per-view duplication.** Every view repeats the same drain-channel +
   `NavAction::modal` boilerplate.
3. **Lifetime / event footguns.** A modal created as a `View` runs the default
   `register_events`, which registers on `lv_screen_active()` — i.e. the
   *background* view's screen (see `Navigator::modal_boxed`). A later page switch
   (`lv_obj_clean` / screen-load) then leaves that registration dangling and the
   overlay can shadow the background view's input (observed: touch soft-keys
   inoperable; crash on next nav). The app must remember to override
   `register_events` to a no-op and `remove_clickable()` for a passive overlay.

## Request

A navigator-/loop-level status overlay independent of the active view. Either:

- **Loop hook:** `run_app_nav` / `run_lvgl` accept an app-supplied per-tick closure
  `mut global: impl FnMut() -> NavAction`, called once per iteration; the loop
  processes its returned action. The app drains its own status channel there and
  returns `NavAction::Modal(..)` / `DismissModal` / `None`.
- **or dedicated Navigator API:** `nav.show_toast(view)` / `nav.dismiss_toast()`,
  with the loop owning a single toast slot, separate from view-triggered modals.

Requirements for the overlay:

- created on `lv_layer_top()`, **input-transparent by default** (does not steal
  touches from the view beneath — critical for bottom touch soft-keys);
- **persists across page switches** — created once, not recreated per view;
- auto-dismiss timeout owned by the loop / navigator, not the view;
- registers **no** input handlers on the background screen.

## Acceptance criteria

- A status message raised from a background task appears over **any** active view.
- Exactly one overlay instance; survives `push` / `replace` / `pop` without
  recreation or crash.
- No dangling event registration on the background screen after a switch.
- Touches pass through to the view beneath (e.g. bottom soft-keys keep working).
- No per-view boilerplate to participate.

## Context

alternator-regulator worked around this app-side by (a) overriding
`register_events` to a no-op + `remove_clickable()` on the bar, and (b, pending)
lifting the status-channel drain out of `MeterView::update()` into the render loop.
A first-class navigator overlay removes both the workarounds and the per-view drain.
