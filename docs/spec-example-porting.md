# Example Porting Specification

How to port an LVGL C example to an oxivgl Rust example.

---

## 1. Goal

Porting LVGL examples drives the core library forward and keeps it
consistent across upstream LVGL changes — a development method inspired
by test-driven development.

Each ported example reproduces the visual output and interactive behavior
of its LVGL C original — same widgets, same layout, same styling.

**Screenshot mode**: the host runner captures a PNG after one render
cycle. Set up a representative initial state so the screenshot is
meaningful (e.g. pre-check a checkbox, set a slider to a non-zero value,
position animated needles at a visible angle).

**Interactive/target mode**: animations, timers, and input-driven
behavior must match the C original. SDL2 host and ESP32 hardware should
show the same experience as the LVGL demo.

**Follow the C structure**: keep widget creation order, API call
sequence, and argument values as close to the C original as possible. If
the C example creates a button, sets its text, then applies a style — do
the same, in the same order, with the same values.

---

## 2. Hard Constraints

The core safety promise (`docs/spec-api-vision.md` §1) applies to all
examples: no `unsafe`, no direct FFI imports, no working around missing
wrappers. Pointer-stored data must be `'static`
(`docs/spec-memory-lifetime.md` §3).

---

## 3. File Skeleton

Every example file needs:

1. The `no_std` / `no_main` cfg-attr block for ESP32 compatibility.
2. The `SPDX-License-Identifier: MIT OR Apache-2.0` header.
3. A `//!` doc comment with a title and brief description.
4. A single struct implementing `View`.
5. `oxivgl_examples_common::example_main!(StructName);` at the end.

Look at any existing example for the exact boilerplate.

---

## 4. View Behavior

### create()

Builds the entire UI once. Get the active screen, create widgets, apply
styles, return the struct. All widgets and styles that LVGL references
must be stored in the struct to prevent drop.

### update()

Called every render tick. Use for polling widget values, timer-triggered
updates, or frame-counter animations. Return `Ok(())` for static
examples.

### on_event()

Override for input-driven behavior (clicks, value changes). Use
`event.matches(&widget, EventCode)` to dispatch. By default, events
bubble to the active screen.

### register_events()

Override when events should be caught on intermediate containers instead
of the screen.

### Extending the View trait

If the `View` trait does not cover an LVGL concept needed by an example,
extend the trait rather than working around it in the example.

---

## 5. Translating C Patterns

| C pattern | Rust equivalent |
|---|---|
| File-scope `static` variables | Fields on the View struct |
| `lv_obj_add_event_cb` + function pointer | `View::on_event` + `event.matches()` |
| `lv_timer_create` + callback | `Timer::new()` in `create()`, poll `triggered()` in `update()` |
| `lv_anim_*` with exec callback | `Anim` builder with predefined `anim_set_*` callbacks, or frame counter in `update()` |
| `lv_style_init` + `lv_style_set_*` | `StyleBuilder` chain → `.build()` → `add_style()` |
| `lv_pct()` | `lv_pct()` |
| `LV_SIZE_CONTENT` | `LV_SIZE_CONTENT` |
| `LV_IMAGE_DECLARE` + `&img` | `image_declare!` macro, then call as `img_name()` |

---

## 6. Simplification Policy

When a C example uses LVGL APIs not yet wrapped:

1. **Preferred**: add the wrapper to the core library, then use it.
2. **Acceptable**: simplify the example to demonstrate the same concept
   with available APIs. Note what was simplified in the doc comment.
3. **Defer**: if the missing API is major infrastructure (canvas, draw
   tasks, custom fonts), skip the example and list it as blocked in the
   coverage table.

---

## 7. After Creating an Example

1. Add the example name to `ALL_EXAMPLES` in `run_host.sh`.
2. Generate the screenshot: `./run_host.sh -s <name>`.
3. Visually compare against the LVGL docs.
4. Add an entry to `examples/doc/README.md` (title, description,
   screenshot link).
5. Update the Implementation Coverage table and TOC in
   `examples/doc/README.md`.
6. Check for regressions: `./run_tests.sh unit`.
7. Check doc coverage:
   ```
   RUSTDOCFLAGS="-W missing-docs" cargo doc --no-deps
   ```

---

## 8. After Extending the Core Library

1. Review against `docs/spec-memory-lifetime.md` — verify `'static`
   bounds, pointer ownership, drop ordering, and SAFETY comments.
2. Regenerate all screenshots (`./run_host.sh -s`) and inspect for
   visual regressions.
3. Update the Implementation Coverage table in `examples/doc/README.md`.
4. Assess test coverage impact — add integration tests for new wrapper
   methods, especially those that store pointers or manage lifetimes.