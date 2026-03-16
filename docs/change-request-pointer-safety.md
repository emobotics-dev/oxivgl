# Pointer Safety: Holistic Concept

All safety gaps share the same root cause: LVGL stores raw pointers to user data
with no way to express the required lifetime in C. This document establishes a
unified Rust model that encodes every lifetime constraint in the type system.

---

## Unifying principle

> **If LVGL stores a raw pointer, Rust must either own the data or require it
> to be `'static`.**

Two cases arise:

| Data kind | LVGL stores pointer to… | Rust encoding |
|---|---|---|
| Compile-time resource | image, point array, symbol string | `'static` reference |
| Runtime resource | style, sub-descriptor | ownership / ref-count |

---

## Layer model

```
Widget
  └── Arc<Style>               ← ref-counted, widget keeps style alive
        └── Box<GradDsc>       ┐
        └── Box<TransitionDsc> ├─ owned by Style, stable heap address via Box
        └── Box<ColorFilter>   ┘

Image / points / symbol        ← 'static references, no heap involvement

Anim<'w>                       ← borrows widget for its lifetime
```

---

## Current state (interim)

The previous change introduced `Style::leaked()`, `GradDsc::leaked()` etc. as a
first step: making stack allocation of descriptors impossible while deferring the
full ownership model. This was deliberate scaffolding. **These methods are
superseded by this design and are removed when Levels 2 and 3 are implemented.**

---

## Level 1 — Static resources

Data that is always a compile-time constant (images in flash, point arrays,
symbol strings). The fix is a `'static` bound; no ownership transfer or heap
involvement required.

### 1a. Image::set_src / Style::bg_image_src

```rust
// before
pub fn set_src(&self, src: &lv_image_dsc_t) -> &Self
pub fn bg_image_src(&mut self, src: &lv_image_dsc_t) -> &mut Self

// after
pub fn set_src(&self, src: &'static lv_image_dsc_t) -> &Self
pub fn bg_image_src(&mut self, src: &'static lv_image_dsc_t) -> &mut Self
```

Images declared via `image_declare!` are already `static`. Non-breaking for
correct usage.

### 1b. Line::set_points

```rust
// before
pub fn set_points(&self, points: &[lv_point_precise_t]) -> &Self

// after
pub fn set_points(&self, points: &'static [lv_point_precise_t]) -> &Self
```

### 1c. Dropdown::set_symbol — active use-after-free

The current wrapper passes a temporary buffer address to `lv_dropdown_set_symbol`,
which stores the raw pointer. The pointer is dangling on return.

```rust
// after
pub fn set_symbol(&self, symbol: &'static CStr) -> &Self
```

---

## Level 2 — Style sub-descriptors owned by Style

`GradDsc`, `TransitionDsc`, and `ColorFilter` are referenced by raw pointer from
`lv_style_t`. Instead of requiring users to manage their lifetimes, `Style` owns
them internally via `Box`.

### Why Box, not a bare field

The LVGL pointer to the sub-descriptor must be set *before* the descriptor is
moved into `Style`. A bare field would be moved to a new address when `Style`
itself moves, invalidating the pointer. A `Box` keeps the descriptor at a stable
heap address independent of where `Style` lives.

### Why no Pin needed

`lv_style_t` is not self-referential: its `map` pointer targets LVGL-managed heap,
not anything inside `Style`. Sub-descriptor pointers target Box heap, which is
unaffected by `Style` moving. So `Style` can be freely moved (e.g. into `Arc::new`)
after sub-descriptors are attached, with no risk of pointer invalidation.

### Struct

```rust
pub struct Style {
    inner: lv_style_t,
    _grad: Option<Box<GradDsc>>,
    _transition: Option<Box<TransitionDsc>>,
    _color_filter: Option<Box<ColorFilter>>,
}
```

### API change: borrow → move

```rust
// before (borrow — caller must keep grad alive separately)
pub fn bg_grad(&mut self, grad: &GradDsc) -> &mut Self

// after (move — Style keeps grad alive)
pub fn bg_grad(&mut self, grad: GradDsc) -> &mut Self
```

Internally:

```rust
pub fn bg_grad(&mut self, grad: GradDsc) -> &mut Self {
    let boxed = Box::new(grad);
    // SAFETY: boxed.inner is heap-allocated; address stable for Box lifetime.
    unsafe { lv_style_set_bg_grad(&mut self.inner, &boxed.inner) };
    self._grad = Some(boxed);
    self
}
```

Calling `bg_grad` a second time drops the previous `Box<GradDsc>` — LVGL has
already been given the new pointer so this is safe.

### Drop order

`Style::drop` calls `lv_style_reset` before Rust drops the fields. LVGL
relinquishes the sub-descriptor pointers before the Boxes are freed. ✓

### Consequence

`GradDsc`, `TransitionDsc`, and `ColorFilter` constructors become private. Users
create them only to immediately move into `Style`; they are no longer standalone
types in the public API. Their `boxed()` and `leaked()` methods are removed.

---

## Level 3 — Styles owned by widgets via Arc

`lv_obj_add_style` stores a raw `*const lv_style_t`. The widget must keep the
style alive for as long as it is applied.

### Why Arc, not Rc

LVGL's `lv_lock`/`lv_unlock` mechanism enables multi-threaded access. `Rc` is
`!Send`, which would prevent styles from being constructed on one thread and
applied on the LVGL thread. `Arc` is required.

### Widget storage

Each widget stores the Arcs for all styles applied to it:

```rust
// inside the Obj / Button / … wrapper
styles: RefCell<Vec<Arc<Style>>>
```

`RefCell` (not `Mutex`) is correct here: widgets are `!Send + !Sync` in Rust
(LVGL's threading is managed at the C level via `lv_lock`), so no Rust-level
concurrent access to the Vec is possible.

For the embedded target a `heapless::Vec<Arc<Style>, N>` with a fixed capacity
may be preferable; the maximum number of styles per widget must then be chosen.

### API

```rust
pub fn add_style(&self, style: Arc<Style>, selector: impl Into<Selector>) -> &Self {
    // SAFETY: Arc heap-pins Style; lv_style_t address stable for Arc lifetime.
    unsafe { lv_obj_add_style(self.handle(), &style.inner, selector.into().0) };
    self.styles.borrow_mut().push(style);
    self
}

pub fn remove_style(&self, style: &Arc<Style>, selector: impl Into<Selector>) {
    // SAFETY: pointer derived from the Arc; still valid.
    unsafe { lv_obj_remove_style(self.handle(), &style.inner, selector.into().0) };
    let ptr = Arc::as_ptr(style);
    self.styles.borrow_mut().retain(|s| Arc::as_ptr(s) != ptr);
}

pub fn remove_style_all(&self) {
    unsafe { lv_obj_remove_style_all(self.handle()) };
    self.styles.borrow_mut().clear();
}
```

### Usage

```rust
let mut style = Style::new();
style
    .radius(5)
    .bg_color(palette_main(Palette::Blue))
    .bg_grad(GradDsc::new()                   // GradDsc moved into Style
        .set_dir(GradDir::Ver)
        .set_stops_count(2)
        ...);
let style = Arc::new(style);

obj1.add_style(Arc::clone(&style), Selector::DEFAULT);
obj2.add_style(Arc::clone(&style), Selector::DEFAULT);
// No field in the View struct — widgets keep the style alive.
// When both widgets are dropped, the Arc count hits zero and Style is freed.
```

### Style::new() visibility

`Style::new()` is currently private (interim state). It is restored to `pub` as
part of this implementation — it is the only constructor needed.
`Style::boxed()` and `Style::leaked()` are removed.

### Theme

`Theme::extend_current` takes `Arc<Style>` for consistency.

---

## Level 4 — Animation borrows widget lifetime

`lv_anim_set_var` stores a raw `*mut c_void` to the target widget. The animation
must not outlive the widget.

```rust
pub struct Anim<'w> {
    inner: lv_anim_t,
    _widget: PhantomData<&'w ()>,
}

impl<'w> Anim<'w> {
    pub fn set_var<W: AsLvHandle>(&mut self, widget: &'w W) -> &mut Self
}
```

`'w` is propagated through the builder chain. The compiler rejects any program
where the widget is dropped while `Anim<'w>` is still live.

---

## Summary of API changes

| Item | Before | After |
|---|---|---|
| `Style::new()` | private (interim) | `pub` |
| `Style::boxed()`, `Style::leaked()` | pub | removed |
| `GradDsc`, `TransitionDsc`, `ColorFilter` constructors | pub | private |
| `Style::bg_grad` | `&GradDsc` | `GradDsc` (move) |
| `Style::transition` | `&TransitionDsc` | `TransitionDsc` (move) |
| `Style::color_filter` | `&ColorFilter` | `ColorFilter` (move) |
| `Obj::add_style` | `&Style` | `Arc<Style>` |
| `Theme::extend_current` | `Box<Style>` | `Arc<Style>` |
| `Image::set_src` | `&lv_image_dsc_t` | `&'static lv_image_dsc_t` |
| `Style::bg_image_src` | `&lv_image_dsc_t` | `&'static lv_image_dsc_t` |
| `Line::set_points` | `&[lv_point_precise_t]` | `&'static [lv_point_precise_t]` |
| `Dropdown::set_symbol` | `&str` (dangling) | `&'static CStr` |
| `Anim` | no lifetime | `Anim<'w>` |
