# oxivgl — Examples

LVGL example screens ported from the [LVGL docs](https://docs.lvgl.io/9.3/examples.html).

Each example is a self-contained file with a `View` impl and a cfg-gated
runner (`example_main!` macro selects host SDL2 or ESP32 fire27 backend).

## Getting Started

### Example 1 — Hello World

Dark blue screen, centered white label.

![getting_started1](screenshots/getting_started1.png)

### Example 2 — Button

Default-styled button with a centered label.

![getting_started2](screenshots/getting_started2.png)

### Example 3 — Custom styles

Two buttons with hand-crafted gradient styles and a darken press filter.
Button 2 uses a fully-rounded (pill) radius.

![getting_started3](screenshots/getting_started3.png)

### Example 4 — Slider

Centered slider; label above shows current value, updated live on
`VALUE_CHANGED` events.

![getting_started4](screenshots/getting_started4.png)

## Styles

### Style 1 — Size, Position and Padding

Object with explicit width, content-based height, padding, and percentage position.

![style1](screenshots/style1.png)

### Style 2 — Background Gradient

Centered object with a two-stop vertical gradient (shifted towards bottom).

![style2](screenshots/style2.png)

### Style 3 — Border

Centered object with bottom+right blue border on light grey background.

![style3](screenshots/style3.png)

### Style 4 — Outline

Centered object with blue outline and 8px padding around it.

![style4](screenshots/style4.png)

### Style 5 — Shadow

Centered object with a large blue drop shadow.

![style5](screenshots/style5.png)

### Style 7 — Arc

Arc widget with red color and 4px width.

![style7](screenshots/style7.png)

### Style 8 — Text Styles

Text with blue color, letter spacing, line spacing, and underline decoration.

![style8](screenshots/style8.png)

### Style 9 — Line Styles

Grey rounded polyline with 6px width.

![style9](screenshots/style9.png)

### Style 10 — Transition

Object with animated transitions on press (bg color, border color/width).

![style10](screenshots/style10.png)

### Style 11 — Multiple Styles

Base (light blue) and Warning (yellow) objects sharing a common style with overrides.

![style11](screenshots/style11.png)

### Style 12 — Local Styles

Green-bordered object with local orange background override.

![style12](screenshots/style12.png)

### Style 13 — Parts and States

Slider with gradient indicator and red shadow on pressed state.

![style13](screenshots/style13.png)

### Style 15 — Opacity

Three buttons at different opacity levels (100%, 50%, 70%).
Transforms omitted — `lv_snapshot_take` requires `LV_USE_MATRIX` + `LV_USE_FLOAT`.

![style15](screenshots/style15.png)

### Style 16 — Conical Gradient

Metallic knob using 8-stop conical gradient with reflected extend and drop shadow.

![style16](screenshots/style16.png)

### Style 17 — Radial Gradient

Full-screen radial gradient from purple to black.

![style17](screenshots/style17.png)

### Style 18 — Gradient Buttons

Four buttons: simple horizontal, simple vertical, complex linear, complex radial gradients.

![style18](screenshots/style18.png)

### Skipped

- **Style 6** — Image recolor/rotation (needs C image asset `img_cogwheel_argb`)
- **Style 14** — Extend theme (needs `LV_USE_PRIVATE_API`)
- **Style 19** — Modal overlay (meta-example, benchmarking)

## Running

```sh
# On host (SDL2, interactive):
./run_host.sh getting_started1
./run_host.sh style1

# On fire27 (ESP32):
./run_fire27.sh getting_started1

# Capture all screenshots:
./run_screenshots.sh
```
