# oxivgl v0.4.0

This release moves oxivgl toward a **shared-style** model for memory-efficient
UIs, adds a **resource-diagnostics** toolkit for widget-heavy screens, and
rounds out the styling and background-image APIs. It bundles all work since
0.3.4.

## Highlights

### 🎨 Shared styles are the recommended path — inline setters deprecated

Per-object inline `style_*` setters each allocate a **per-object local style**
(a `styles[]` entry *plus* its own property buffer). A shared `Style` amortizes
to **one** buffer for every widget that uses it — a heap *and* style-refresh
compute win wherever a treatment repeats.

- New **`Style::new(|s| …)`** — a one-call constructor (no separate `.build()`).
- `add_style` `Rc`-retains the style, so you can **build once, apply to many
  widgets, and drop your handle** ("build-and-forget").
- **49 inline `style_*` setters are now `#[deprecated]`.** Migrate them to
  shared styles. Transforms and image-content setters are intentionally *not*
  deprecated (they're per-instance / dynamic).

```rust
// before — a per-object local style on each widget
label.bg_color(0x222244).bg_opa(255).radius(8);

// after — build once, share across widgets
let card = Style::new(|s| { s.bg_color_hex(0x222244).bg_opa(255).radius(8); });
label.add_style(&card, Selector::DEFAULT);
```

See **[docs/memory-tuning.md](https://github.com/emobotics-dev/oxivgl/blob/v0.4.0/docs/memory-tuning.md)**
for the rationale and a measurement guide. All bundled examples have been
migrated (verified pixel-identical via before/after screenshots).

### 📊 Resource diagnostics (`diag` module)

Measure what widget-heavy UIs actually cost:

- **`census(&obj)`** — object count + nesting depth for a subtree, with an
  estimated-heap-bytes coefficient.
- **`Budget` / `assert_budget`** — per-view ceilings that fail loudly in debug
  builds (no-op in release), so a regression trips CI/HIL instead of a device.
- **`ResourceProbe`** — a pluggable hook for live on-target heap/stack figures
  (host default `NullProbe`). `lv_mem_monitor` is intentionally avoided under
  `LV_STDLIB_CLIB`; the meaningful figure is the system (esp-hal) heap.

### 🧱 Fuller styling + background-image API

- New `StyleBuilder` methods for full parity with the inline setters:
  `pad_hor`, `pad_bottom`, `pad_column`, `pad_row`, `size`, `clip_corner`,
  `text_align`, `base_dir`, `radial_offset`, `line_opa`, `arc_rounded`,
  `blur_radius`, `blur_backdrop`, `radius_circle`.
- Background-image recolor wrappers on both `Obj` and `StyleBuilder`
  (`style_bg_image_src`, `bg_image_recolor[_hex]`, `bg_image_recolor_opa`).

## ⚠️ Upgrading

Nothing is removed and nothing breaks at runtime — the deprecations are
**compile-time warnings only**. This is a **minor** bump, so `oxivgl = "0.3"`
users must move to `"0.4"` to receive it (and to see the deprecation warnings).
Migrate inline styling to shared `Style` + `add_style` at your own pace; for
genuinely *dynamic* per-frame styling (animated opacity, live recolor), keep the
inline setter under a localized `#[allow(deprecated)]`.

## Full changes

### Added

- `diag` module: `census()`, `Census`, `Budget` / `assert_budget`,
  `ResourceProbe` / `NullProbe`.
- `Style::new(|s| …)` shared-style constructor.
- `StyleBuilder` parity methods (see Highlights) and the `bg_image_recolor`
  family on `Style`.
- Background-image wrappers on `Obj`: `style_bg_image_src`,
  `style_bg_image_recolor[_hex]`, `style_bg_image_recolor_opa`.
- `docs/memory-tuning.md` — measuring and reducing heap/stack on widget-heavy
  UIs.
- Examples `shared_styles1` (a style guide built from shared styles) and
  `widget_bar8` (recolored background image).

### Deprecated

- 49 inline `style_*` setters on `Obj` — build a shared `Style` (`Style::new`)
  and apply with `add_style`.

### Changed

- All bundled examples migrated off inline setters to shared `Style` +
  `add_style`.

**Full diff:** https://github.com/emobotics-dev/oxivgl/compare/v0.3.4...v0.4.0
