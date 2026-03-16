# Gradient Examples Design

## Goal

Add static ports of the 4 LVGL gradient examples (`lv_example_grad_1–4`) as
`getting_started5–8.rs`. Omit interactive drag elements; show gradient on a
styled container only.

## Wrapper Additions — `src/widgets/grad.rs`

### `GradDsc::horizontal()`

```rust
/// Configure as a simple horizontal gradient (left-to-right).
pub fn horizontal(&mut self) -> &mut Self {
    unsafe { lv_grad_horizontal_init(&mut self.inner) };
    self
}
```

Not gated by `LV_USE_DRAW_SW_COMPLEX_GRADIENTS` (basic gradient feature).

### `GradDsc::radial_set_focal()`

```rust
/// Set the focal point of a radial gradient.
/// `fx`, `fy`: focal center coords; `fr`: focal radius.
pub fn radial_set_focal(&mut self, fx: i32, fy: i32, fr: i32) -> &mut Self {
    unsafe { lv_grad_radial_set_focal(&mut self.inner, fx, fy, fr) };
    self
}
```

Required by `getting_started7` to match C original.

Export both `GradDsc` and `GradExtend` from `src/widgets/mod.rs` if not already.

## Color Stops

Use `color_make(r, g, b)` (already exported — no `lvgl_rust_sys` in examples).

| Example | colors | opas | fracs |
|---------|--------|------|-------|
| 5 (horizontal) | `[color_make(0xff,0,0), color_make(0,0xff,0)]` | `[255, 0]` | `[20*255/100, 80*255/100]` |
| 6 (linear) | same colors | `[255, 0]` | `&[]` (NULL → even spacing) |
| 7 (radial) | same colors | `[255, 0]` | `&[]` |
| 8 (conical) | same colors | `[255, 0]` | `&[]` |

## Examples

Container style for all: 80%×80%, centered, `border_width(2)`, `radius(12)`,
`pad_all(0)`. Applied via `style.bg_grad(&grad)`.

| File | Gradient setup |
|------|----------------|
| `getting_started5.rs` | `init_stops(colors, opas, fracs).horizontal()` |
| `getting_started6.rs` | `init_stops(…).linear(100,100, 200,150, GradExtend::Pad)` |
| `getting_started7.rs` | `init_stops(…).radial(100,100, 200,100, GradExtend::Pad).radial_set_focal(50,50,10)` |
| `getting_started8.rs` | `init_stops(…).conical(lv_pct(50),lv_pct(50), 0,180, GradExtend::Pad)` |

Conical angles are in degrees (0–360), matching `lv_grad_conical_init` param docs.
`lv_pct(50)` is already exported from `widgets/style.rs` as `lv_pct(v: i32) -> i32`.

## Struct Layout (each example)

```rust
struct GettingStarted5 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}
```

`Style` and `GradDsc` must outlive `Obj` — all stored as struct fields satisfies this.

## Housekeeping

- Add `getting_started5`, `6`, `7`, `8` to `run_screenshots.sh`
- Add README entries + screenshots to `examples/doc/README.md`
- Verify build after each file: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`
