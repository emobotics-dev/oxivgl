# oxivgl v0.5.0

This release adds a **POINTER (touchscreen) input device** and **keypad
hold-to-repeat**, lets applications **drop unused built-in fonts** without
breaking the build, and removes a **silent text-truncation** footgun. It bundles
all work since 0.4.0.

## Highlights

### 👆 POINTER (touchscreen) input device + keypad hold-to-repeat

oxivgl already shipped a generic **KEYPAD** indev; this release rounds out the
input story for real touch UIs — and stays BSP- and MCU-agnostic, taking input
in oxivgl's own vocabulary (LVGL keys, raw `(x, y)` coordinates), never a
board/driver type.

- **`PointerIndev`** — a direct-touch device: a view can be navigated by tapping
  a widget at a coordinate. Fed by either a lock-free `PointerState` cell or a
  polling closure — `PointerIndev::new_with(|| ft6336u::read_touch())` is the
  one-liner a touch board needs.
- **`KeypadIndev::with_repeat(after, every)`** — hold-to-repeat while a key is
  held, for value/setpoint editing on 3-button boards. A thin pass-through to
  LVGL's `long_press_time` / `long_press_repeat_time`.
- Both `PointerIndev` / `PointerState` are re-exported from the prelude.
- New example **`touch_setpoint`** — hold `-`/`+` to repeat (keypad) and tap a
  preset to jump the value (touch).

```rust
static TOUCH: PointerState = PointerState::new();
let _pointer = PointerIndev::new(&TOUCH)?;          // or ::new_with(closure)
let keypad = KeypadIndev::new(&KEYPAD)?
    .with_repeat(Duration::from_millis(400), Duration::from_millis(80));
```

### 🔤 Drop unused built-in fonts without breaking the build

Built-in `Font` constants are now **cfg-gated to the faces actually enabled in
`lv_conf.h`**. Previously `src/fonts.rs` referenced every Montserrat (8–48), the
DejaVu Persian/Hebrew face, and both Source Han CJK faces unconditionally, so
disabling *any* of them broke oxivgl's own compile — forcing every app to ship
~400 KiB of fonts it may not use. `oxivgl-sys` (now **0.2.2**) reports which
faces the LVGL build exposed, and each `Font` const is gated on that. Apps can
now set `LV_FONT_* 0` to drop unused faces; referencing a disabled face is a
plain "cannot find value" error pointing at the app's own code.

> **Requires `oxivgl-sys >= 0.2.2`.** (The previous `fonts` module docs claiming
> a *linker* error and "no code-size cost" were wrong and have been corrected.)

### 🩹 No more silent 127-byte text truncation

Every `&str`-taking text setter — `Label::text`, `Checkbox::text`,
`Textarea::{set_text, add_text, set_placeholder_text}`, `List`, `Msgbox`,
`Table::set_cell_value`, `Win::add_title`, `Menu::page_create`,
`Label::set_translation_tag`, and the `Label::bind_text_map` updater —
previously copied through a fixed `[u8; 128]` stack buffer and dropped everything
past 127 bytes **with no error**. They now pass the full string through a
heap-backed NUL-terminated temporary (LVGL copies it internally), so text of any
length renders verbatim — and 15 per-call 128-byte stack temporaries are gone
from widget construction. `Label::text_long` is now redundant and **deprecated**
(call `text` directly).

## ⚠️ Upgrading

- **`oxivgl-sys >= 0.2.2` is required** (font-gating support). `cargo update`
  picks it up.
- This is a **minor** bump, so `oxivgl = "0.4"` users must move to `"0.5"`.
- Nothing breaks at runtime; `Label::text_long` is a **compile-time deprecation
  warning** only — replace it with `Label::text`.

## Full changes

### Added

- POINTER input device: `PointerIndev` (`new(&PointerState)` and
  `new_with(closure)` forms) + `PointerState`; both in the prelude.
- `KeypadIndev::with_repeat(after, every)` for keypad hold-to-repeat.
- Example `touch_setpoint`; ESP32 cross-build of it in CI.

### Changed

- Built-in font constants gated to the faces enabled in `lv_conf.h`; requires
  `oxivgl-sys >= 0.2.2`. Corrected the `fonts` module documentation.
- Internal: zero-warnings build policy (`-D warnings` via the Cargo `[lints]`
  table) across all workspace crates.

### Fixed

- Text setters no longer silently truncate at 127 bytes (`Label`, `Checkbox`,
  `Textarea`, `List`, `Msgbox`, `Table`, `Win`, `Menu`, translation tags, and
  the `bind_text_map` updater).

### Deprecated

- `Label::text_long` — `Label::text` is now itself uncapped; call `text`.

**Full diff:** https://github.com/emobotics-dev/oxivgl/compare/v0.4.0...v0.5.0
