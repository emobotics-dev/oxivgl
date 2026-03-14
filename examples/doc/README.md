# oxivgl — Examples

LVGL example screens ported from the [LVGL docs](https://docs.lvgl.io/9.3/examples.html).

Each example is a self-contained file with a `View` impl and a cfg-gated
runner (`example_main!` macro selects host SDL2 or ESP32 fire27 backend).

## Contents

- [Getting Started](#getting-started)
- [Styles](#styles)
- [Animations](#animations)
- [Events](#events)
- [Layouts — Flex](#layouts--flex)
- [Layouts — Grid](#layouts--grid)
- [Scrolling](#scrolling)
- [Widgets — Base Object](#widgets--base-object)
- [Widgets — Animation Image](#widgets--animation-image)
- [Widgets — Arc](#widgets--arc)
- [Widgets — Image](#widgets--image)
- [Widgets — Bar](#widgets--bar)
- [Widgets — Button](#widgets--button)
- [Widgets — Checkbox](#widgets--checkbox)
- [Widgets — Dropdown](#widgets--dropdown)
- [Widgets — Label](#widgets--label)
- [Widgets — LED](#widgets--led)
- [Widgets — Roller](#widgets--roller)
- [Widgets — Slider](#widgets--slider)
- [Running](#running)

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

- **Style 6** — Image recolor/rotation (needs `Image` rotation/recolor wrapper methods)
- **Style 14** — Extend theme (needs `LV_USE_PRIVATE_API`)
- **Style 19** — Modal overlay (meta-example, benchmarking)

## Animations

### Anim 1 — Start Animation on Event

Switch toggles label X-position animation (overshoot/ease-in paths).

![anim1](screenshots/anim1.png)

### Anim 2 — Playback Animation

Red circle with repeat/reverse size + X animations (ease-in-out).

![anim2](screenshots/anim2.png)

### Anim Timeline 1 — Animation Timeline

Three objects animated via timeline, controlled by start/pause buttons and a progress slider.

![anim_timeline1](screenshots/anim_timeline1.png)

### Skipped

- **Anim 3** — Cubic bezier with Chart (needs `LV_USE_CHART`, Chart wrapper)

## Events

All event examples use the safe `View::on_event` dispatch — no `unsafe` or raw callbacks in user code.

> **Note:** Hardware target (fire27) has no touch screen yet — input events require a physical input device. The GUI is fully wired; only the physical input is missing.

### Event Click — Button Click Counter

Button increments a counter label on each click.

![event_click](screenshots/event_click.png)

### Event Button — Multiple Event Types

Button reports pressed, clicked, long-pressed, and long-pressed-repeat events to an info label.

![event_button](screenshots/event_button.png)

### Event Bubble — Event Bubbling

30-button grid in a flex container; clicking any button turns it red via bubbled events.

![event_bubble](screenshots/event_bubble.png)

### Event Trickle — Event Trickle-Down

9-cell grid with trickle-down: pressing the container applies a black style to all children.

![event_trickle](screenshots/event_trickle.png)

### Skipped

- **Event Draw** — needs `lv_timer_create` + draw task APIs
- **Event Streak** — needs `lv_indev_get_short_click_streak` (requires input device)

## Layouts — Flex

### Flex 1 — Row and Column

Row container (scrollable) and column container with 10 buttons each.

![flex1](screenshots/flex1.png)

### Flex 2 — Row Wrap with Even Spacing

Style-based flex config: row-wrap flow with `SPACE_EVENLY` alignment. Items are checkable.

![flex2](screenshots/flex2.png)

### Flex 3 — Flex Grow

Fixed-size items alongside items with `flex_grow` (1 and 2 portions of free space).

![flex3](screenshots/flex3.png)

### Flex 4 — Column Reverse

Items added 0–5 but displayed bottom-to-top via `ColumnReverse` flow.

![flex4](screenshots/flex4.png)

### Flex 5 — Row and Column Gap

Row-wrap layout with animated `pad_row` (500 ms) and `pad_column` (3000 ms) gap changes.

![flex5](screenshots/flex5.png)

### Flex 6 — RTL Direction

Right-to-left base direction reverses item order in a row-wrap container.

![flex6](screenshots/flex6.png)

## Layouts — Grid

### Grid 1 — Simple Grid

3×3 grid with fixed 70 px columns and 50 px rows, stretched button cells.

![grid1](screenshots/grid1.png)

### Grid 2 — Cell Placement and Span

Different cell alignments (START, CENTER, END) and multi-column/row spanning.

![grid2](screenshots/grid2.png)

### Grid 3 — Free Units (FR)

Column 1 fixed 60 px, column 2 gets 1 FR, column 3 gets 2 FR of remaining space.

![grid3](screenshots/grid3.png)

### Grid 4 — Track Placement

`SPACE_BETWEEN` columns, rows aligned to `END` (bottom).

![grid4](screenshots/grid4.png)

### Grid 5 — Column and Row Gap

3×3 grid with animated row gap (500 ms) and column gap (3000 ms).

![grid5](screenshots/grid5.png)

### Grid 6 — RTL Direction

Same 3×3 grid but with `RTL` base direction — cells fill right-to-left.

![grid6](screenshots/grid6.png)

## Scrolling

### Scroll 1 — Basic Scrolling with Save/Restore

Panel with children placed outside its bounds triggers automatic scrolling.
Two buttons save and restore the scroll position.

![scroll1](screenshots/scroll1.png)

### Scroll 2 — Scroll Snap

Horizontal row of buttons with center snap alignment. Panel 3 is non-snappable.
A switch toggles "scroll one" mode.

![scroll2](screenshots/scroll2.png)

### Scroll 4 — Scrollbar Styling

Custom blue rounded scrollbar that widens and becomes fully opaque when
actively scrolling, with animated transitions.

![scroll4](screenshots/scroll4.png)

### Skipped

- **Scroll 3** — Floating button in list (needs `LV_USE_LIST`, List wrapper)
- **Scroll 5** — RTL scrolling (needs `LV_FONT_DEJAVU_16_PERSIAN_HEBREW`)
- **Scroll 6** — Curved scroll (needs `lv_obj_get_coords`, `lv_sqrt`, `lv_map`)
- **Scroll 7** — Dynamic widget loading (needs `lv_obj_move_to_index`, Checkbox wrapper)
- **Scroll 8** — Circular list (needs `lv_obj_move_to_index`, content size APIs)

## Widgets — Base Object

### Widget Obj 1 — Base Objects with Custom Styles

Two base objects: a plain one and one with a blue shadow style.

![widget_obj1](screenshots/widget_obj1.png)

### Skipped

- **Widget Obj 2** — Draggable object (needs `lv_indev_active`, `lv_indev_get_vect` APIs)
- **Widget Obj 3** — 3D matrix transform (needs `LV_DRAW_TRANSFORM_USE_MATRIX`, `lv_matrix_*`, `lv_timer_create`)

## Widgets — Animation Image

### Skipped

- **Widget AnimImg 1** — Animated image frames (needs `AnimImg` wrapper)

## Widgets — Arc

### Widget Arc 1 — Arc with Value Label

Arc with VALUE_CHANGED event; a label follows the arc's knob angle via `align_obj_to_angle`.
Uses `align` instead of `rotate` because LVGL's SW renderer does not support
`rotate_obj_to_angle` ([lvgl#7706](https://github.com/lvgl/lvgl/issues/7706)).

![widget_arc1](screenshots/widget_arc1.png)

### Widget Arc 2 — Animated Arc Loader

Full-circle arc animating 0→100 in 1 s (infinite repeat, 500 ms delay). Knob hidden, not clickable.

![widget_arc2](screenshots/widget_arc2.png)

## Widgets — Image

### Widget Image 1 — Basic Image Display

Centered cogwheel image from compiled PNG asset.

![image1](screenshots/image1.png)

## Widgets — Bar

### Widget Bar 1 — Simple Bar

Simple 200×20 bar at 70%.

![widget_bar1](screenshots/widget_bar1.png)

### Widget Bar 2 — Styled Progress Bar

Blue-themed bar with custom bg/indicator styles, rounded corners, padding, and animated fill.

![widget_bar2](screenshots/widget_bar2.png)

### Widget Bar 3 — Temperature Meter

Vertical bar with red-to-blue gradient indicator, animated between -20 and 40 (3 s each direction).

![widget_bar3](screenshots/widget_bar3.png)

### Widget Bar 5 — LTR vs RTL Bars

Two bars: one left-to-right (default), one right-to-left, with labels.

![widget_bar5](screenshots/widget_bar5.png)

### Skipped

- **Widget Bar 4** — Stripe pattern (needs `lv_style_set_bg_image_src` wrapper)

## Widgets — Button

### Widget Button 1 — Click and Toggle

Standard button logging clicks and a checkable toggle button logging state changes.

![widget_button1](screenshots/widget_button1.png)

### Widget Button 2 — Styled Button from Scratch

Button with gradient, shadow, outline, and a transition that expands outline on press.

![widget_button2](screenshots/widget_button2.png)

## Widgets — Checkbox

### Widget Checkbox 1 — Simple Checkboxes

Four checkboxes: unchecked, checked, disabled, and checked+disabled.

![widget_checkbox1](screenshots/widget_checkbox1.png)

## Widgets — Dropdown

### Widget Dropdown 1 — Simple Drop-Down

Dropdown with ten fruit options at top center.

![widget_dropdown1](screenshots/widget_dropdown1.png)

### Widget Dropdown 2 — Four Directions

Four dropdowns opening in each cardinal direction (down, up, right, left).

![widget_dropdown2](screenshots/widget_dropdown2.png)

## Widgets — Label

### Widget Label 1 — Wrap and Scroll

Wrapped centered text and a circularly scrolling label.

![widget_label1](screenshots/widget_label1.png)

### Widget Label 2 — Text Shadow

Fake shadow via duplicate label offset by 2 px with reduced opacity.

![widget_label2](screenshots/widget_label2.png)

## Widgets — LED

### Widget LED 1 — Brightness and Color

Three LEDs: off (dark), dim red (brightness 150), and full on (blue).

![widget_led1](screenshots/widget_led1.png)

## Widgets — Roller

### Widget Roller 1 — Month Roller

Infinite roller with month names, 4 visible rows.

![widget_roller1](screenshots/widget_roller1.png)

## Widgets — Slider

### Widget Slider 2 — Styled Slider

Cyan slider with pill-shaped track, padded knob with border, bg-color transition on press.

![widget_slider2](screenshots/widget_slider2.png)

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
