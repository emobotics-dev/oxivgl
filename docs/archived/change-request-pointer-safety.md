# Pointer Safety: Holistic Concept

All safety gaps share the same root cause: LVGL stores raw pointers to user data
with no way to express the required lifetime in C. This document establishes a
unified Rust model that encodes every lifetime constraint in the type system.

---

## Unifying principle

> **If LVGL stores a raw pointer, Rust must either own the data or require it
> to be `'static`.**

| Data kind | LVGL stores pointer to... | Rust encoding |
|---|---|---|
| Compile-time resource | image, point array, symbol string | `'static` reference |
| Runtime resource | style, sub-descriptor | ownership / ref-count |

---

## Important LVGL types

| LVGL type | Allocated by | Freed by | Lifetime | Referred by in LVGL |
|---|---|---|---|---|
| `lv_obj_t` (widget/screen) | User — `lv_obj_create()` | User — `lv_obj_delete()` | Until explicitly deleted; children cascade | Parent's child linked list; display's `scr_ll` for screens |
| `lv_style_t` | User | User — `lv_style_reset()` | Must outlive all widgets using it | Pointer array in `lv_obj_t` style list |
| `lv_style_transition_dsc_t` | User | User | Must outlive the style referencing it | Pointer inside `lv_style_t` |
| `lv_font_t` (runtime) | User | User — font-specific destroy fn | Must outlive all styles using it | Pointer inside `lv_style_t` |
| `lv_image_dsc_t` / pixel buf | User | User | Must outlive all widgets using it | Pointer inside `lv_image_t` widget struct |
| `lv_anim_t` (descriptor) | User (stack/static/heap) | LVGL — after `lv_anim_start()` | Can be discarded after `lv_anim_start()` | Not retained — copied immediately |
| `lv_anim_t` (internal copy) | LVGL — on `lv_anim_start()` | LVGL — on complete or `lv_anim_delete()` | Until animation ends or is cancelled | Global animation linked list (`lv_ll`) |
| `lv_anim_timeline_t` | LVGL — `lv_anim_timeline_create()` | User — `lv_anim_timeline_delete()` | Until explicitly deleted | User hold the only pointer |
| `lv_timer_t` | LVGL — `lv_timer_create()` | User — `lv_timer_delete()` | Until deleted; one-shot if `repeat_count=1` | Global timer linked list (`lv_ll`) |
| `lv_display_t` | LVGL — `lv_display_create()` | User — `lv_display_delete()` | Application lifetime typically | Global display linked list |
| `lv_indev_t` | LVGL — `lv_indev_create()` | User — `lv_indev_delete()` | Application lifetime typically | Global indev linked list |
| `lv_group_t` | LVGL — `lv_group_create()` | User — `lv_group_delete()` | Until explicitly deleted | User hold the pointer; optionally set as default group on indev |
| `lv_event_t` | LVGL (stack frame) | LVGL — automatic | Duration of the event callback only | Stack frame in `lv_event_send()`; pointer passed to callback |








## Layer model

```
Widget
  +-- _styles: RefCell<Vec<Style>>    <-- Rc clones; prevent dealloc while LVGL holds pointers
  +-- Style (clone)                   <-- cheaply clonable handle (Rc hidden inside)
        +-- StyleInner
              +-- lv_style_t
              +-- Box<GradDsc>       \
              +-- Box<TransitionDsc>  |- owned by StyleInner, stable heap address
              +-- Box<ColorFilter>   /

StyleBuilder                           <-- mutable build phase; owns Box<StyleInner>
  .build() -> Style                    <-- freezes into clonable Rc handle

Image / points / symbol               <-- 'static references, no heap involvement

Anim<'w>                               <-- build-phase borrow; runtime safety via lv_obj_delete

Theme
  +-- Style (clone)                    <-- Theme keeps style alive; see "Known limitation" below
```

---

## Current state

`Style` already has a `Drop` impl calling `lv_style_reset`, so the LVGL
property-map leak that existed earlier is fixed.

`Style::boxed()` and `GradDsc::boxed()` exist as interim scaffolding. They are
superseded by this design and removed when Levels 2 and 3 are implemented.

---

## Level 1 --- Static resources

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

**Constraint**: `'static` means point arrays must be declared `static` or
produced via `Box::leak`. Heap-allocated `Vec` of points is not accepted.
`Box::leak` is the intended escape hatch when points are computed at runtime;
the doc comment must warn that leaked memory is **never reclaimed** --- use only
for data whose effective lifetime matches the display's.

### 1c. Dropdown::set_symbol --- active use-after-free

The current wrapper passes a temporary stack buffer address to
`lv_dropdown_set_symbol`, which stores the raw pointer (`dropdown->symbol = symbol`,
verified in `lv_dropdown.c:370`). The pointer is dangling on return.

```rust
// after
pub fn set_symbol(&self, symbol: &'static CStr) -> &Self
```

---

## Level 2 --- Style as a two-phase type (StyleBuilder → Style)

`StyleBuilder` is the mutable build phase: it owns a `Box<StyleInner>` and
exposes all setter methods. Calling `.build()` freezes it into a `Style` ---
a cheaply-clonable handle wrapping `Rc<StyleInner>`. The Rc is an
implementation detail; users never see it.

`StyleInner` owns the `lv_style_t` and its sub-descriptors (`GradDsc`,
`TransitionDsc`, `ColorFilter`) via `Box`.

### Why two types, not a single type with runtime check

The build-then-share pattern is enforced at compile time. Attempting to call a
setter on a `Style` is a type error, not a runtime panic. On embedded targets
where `panic = "abort"`, a runtime check would halt the device; a compile error
costs nothing.

```rust
// User code --- build, then freeze, then share
let style = StyleBuilder::new()
    .radius(5)
    .bg_color(blue)
    .build();
obj1.add_style(&style, DEFAULT);
obj2.add_style(&style, DEFAULT);
```

### Struct layout

```rust
/// Mutable style builder. All setter methods live here.
///
/// Call [`.build()`](StyleBuilder::build) to freeze into a clonable [`Style`].
pub struct StyleBuilder {
    inner: Box<StyleInner>,
}

/// Frozen, cheaply-clonable LVGL style handle.
///
/// Obtained from [`StyleBuilder::build`]. Pass to widgets via
/// [`Obj::add_style`]. Cloning shares the underlying style data
/// (ref-counted internally).
#[derive(Clone)]
pub struct Style {
    inner: Rc<StyleInner>,
}

/// Actual LVGL style data. Not public.
#[repr(C)]  // lv must be at offset 0; required for *const StyleInner -> *const lv_style_t cast
struct StyleInner {
    lv: lv_style_t,
    _grad: Option<Box<GradDsc>>,
    _transition: Option<Box<TransitionDsc>>,
    _color_filter: Option<Box<ColorFilter>>,
}

// Compile-time proof that the cast is valid.
const _: () = assert!(core::mem::offset_of!(StyleInner, lv) == 0);
```

### Builder methods have direct &mut access

`StyleBuilder` owns a `Box<StyleInner>` with no ref-counting. Setters access
`&mut StyleInner` directly --- no `get_mut`, no panic path:

```rust
impl StyleBuilder {
    pub fn new() -> Self {
        let mut lv = MaybeUninit::<lv_style_t>::uninit();
        unsafe { lv_style_init(lv.as_mut_ptr()) };
        Self {
            inner: Box::new(StyleInner {
                lv: unsafe { lv.assume_init() },
                _grad: None,
                _transition: None,
                _color_filter: None,
            }),
        }
    }

    pub fn radius(&mut self, r: i16) -> &mut Self {
        unsafe { lv_style_set_radius(&mut self.inner.lv, r as lv_coord_t) };
        self
    }

    /// Freeze into a clonable Style handle. Consumes the builder.
    pub fn build(self) -> Style {
        Style { inner: Rc::from(self.inner) }
    }
}
```

`Rc::from(Box<T>)` reuses the Box allocation when possible (Rc 1.78+),
avoiding a redundant copy. The `StyleInner` address may change during this
conversion, but no LVGL pointer references it yet --- sub-descriptor pointers
target their own `Box` heap allocations, not `StyleInner` fields (see "Why Box
for sub-descriptors" below). The `lv_style_t` property map
(`values_and_props`) is LVGL-managed heap, also independent of `StyleInner`'s
address.

**No setter methods on `Style`**: `Style` exposes only `clone()` and is passed
to `add_style`. Attempting to mutate after `.build()` is a compile error.

### Why Box for sub-descriptors, not bare fields

Sub-descriptor pointers must remain stable. `StyleBuilder::build()` converts
`Box<StyleInner>` to `Rc<StyleInner>`, which may relocate the `StyleInner`
allocation. A bare field inside `StyleInner` would move with it, invalidating
any pointer LVGL stored during the build phase. A `Box` keeps each
sub-descriptor at its own fixed heap address, independent of `StyleInner`'s
location.

Note: even after the `Rc` allocation is created (post-`.build()`), the
`StyleInner` address is stable (Rc heap-pins it). But since LVGL receives
sub-descriptor pointers during the *build* phase (before `.build()` is called),
the Box indirection is required to survive the Box→Rc conversion.

### Why no Pin needed

`lv_style_t` is not self-referential: its `values_and_props` pointer targets
LVGL-managed heap, not anything inside `StyleInner`. Sub-descriptor pointers
target their own Box heap allocations, unaffected by `StyleInner` moving.

### API change: borrow -> move for sub-descriptors

```rust
// before (borrow --- caller must keep grad alive separately)
pub fn bg_grad(&mut self, grad: &GradDsc) -> &mut Self

// after (move --- StyleBuilder owns grad)
pub fn bg_grad(&mut self, grad: GradDsc) -> &mut Self
```

Internally (on `StyleBuilder`):

```rust
pub fn bg_grad(&mut self, grad: GradDsc) -> &mut Self {
    let inner = &mut *self.inner;
    let boxed = Box::new(grad);
    // SAFETY: boxed.inner is heap-allocated; address stable for Box lifetime.
    unsafe { lv_style_set_bg_grad(&mut inner.lv, &boxed.inner) };
    inner._grad = Some(boxed);
    self
}
```

**Calling `bg_grad` a second time**: `inner._grad = Some(boxed)` replaces the
Option, dropping the previous `Box<GradDsc>`. This is safe because
`lv_style_set_bg_grad` has already overwritten the map entry with the new
pointer (LVGL overwrites property values in-place via `lv_style_set_prop`,
verified in `lv_style.c:346`). The old pointer is no longer stored anywhere
in LVGL when the old Box is dropped.

### TransitionDsc props lifetime --- load-bearing `'static` bound

`TransitionDsc::new` takes `props: &'static [lv_style_prop_t]`. LVGL's
`lv_style_transition_dsc_init` stores a raw pointer to the `props` array
*inside* the `lv_style_transition_dsc_t`. Moving `TransitionDsc` into
`Box<TransitionDsc>` stabilizes the *descriptor's* address, but the *props*
pointer inside it is independent --- it still points at the original slice.

The `'static` bound on `props` is therefore **load-bearing**: it guarantees
the props array outlives any descriptor that references it. This bound must
not be relaxed when refactoring the `transition` method signature from
`&TransitionDsc` to `TransitionDsc` (move).

### Drop impl (load-bearing)

`StyleInner` implements `Drop` calling `lv_style_reset`. The drop sequence:

1. Rc refcount hits 0 -> `StyleInner` is dropped.
2. `Drop::drop` runs -> `lv_style_reset` frees the `values_and_props` buffer
   (the property map that held all raw pointers to sub-descriptors) and zeroes
   the `lv_style_t` struct (`lv_style.c:196`). After this call, no LVGL data
   structure references any sub-descriptor address.
3. Rust drops the `Option<Box<...>>` fields -> sub-descriptor heap memory freed.

This order is guaranteed by Rust: `Drop::drop` always runs before field
destructors, regardless of field declaration order.

```rust
impl Drop for StyleInner {
    fn drop(&mut self) {
        // SAFETY: lv is valid; lv_style_reset (lv_style.c:192-201) frees the
        // values_and_props buffer (which contained raw pointers to sub-descriptors)
        // and zeroes the lv_style_t. After this, no LVGL data structure holds
        // references to Box<GradDsc> / Box<TransitionDsc> / Box<ColorFilter>.
        // Rust then drops those Box fields safely.
        unsafe { lv_style_reset(&mut self.lv) };
    }
}
```

**`mem::forget` caveat**: if a `Style` clone is the last handle and is leaked
via `mem::forget`, the `lv_style_reset` call never runs and LVGL's property
map is leaked. This is standard Rust behavior (`mem::forget` is safe; resource
leaks are not UB). Document this in the `Style` type-level docs.

### Constructor visibility

`GradDsc::new()`, `TransitionDsc::new()`, and `ColorFilter::new()` remain
**public**. They are safe to construct because they are immediately moved into
`StyleBuilder` --- no stack-allocated descriptor is ever handed to LVGL
directly. The only removed methods are `boxed()` and `leaked()`, which
encouraged the old (unsafe) borrow-and-keep-alive pattern.

### Threading --- not applicable

`Rc` is `!Send + !Sync`, matching the single-task LVGL constraint. `Style`,
`StyleBuilder`, and `Obj` all contain raw pointers and are `!Send + !Sync` by
default. No thread-safety work is needed. If multi-thread support is added
later, `Rc` must be replaced with `Arc` and all LVGL calls must acquire
`lv_lock`/`lv_unlock`.

---

## Level 3 --- Styles owned by widgets

`lv_obj_add_style` stores a raw `*const lv_style_t` (verified in
`lv_obj_style.c:135`). The widget must keep the style alive for as long as it
is applied.

### Widget storage

Each widget stores `Style` clones (cheap Rc bumps) for all styles applied:

```rust
// inside the Obj / Button / ... wrapper
_styles: RefCell<Vec<Style>>
```

`RefCell` (not `Mutex`) is correct here: widgets are `!Send + !Sync` (Obj
contains `*mut lv_obj_t`), so no Rust-level concurrent access to the Vec is
possible. `alloc::vec::Vec<Style>` is used on all targets.

### Drop order correctness

`lv_obj_delete` (called in `Obj::drop`) internally calls `lv_obj_remove_style_all`
(`lv_obj.c:521`) and `lv_anim_delete(obj, NULL)` (`lv_obj.c:525`) before
returning. This removes all style references from the LVGL object before Rust
drops `_styles` and decrements the Rc counts inside each `Style` handle. The
invariant "LVGL releases pointer before Rust frees the allocation" is thus
upheld by LVGL itself. No additional ordering work is needed in `Obj::drop`.

**Version pinning**: this invariant is specific to LVGL v9.2. If LVGL is
upgraded, re-verify both call sites. An integration test that calls `add_style`
then drops the widget exercises the critical path and should be part of CI.

### Atomicity of add_style

The `Style` clone must be pushed to the Vec **before** the LVGL call. If the
push panics (OOM), LVGL is not updated and both
sides remain consistent. The inverse order (LVGL first, then push) would leave
a dangling LVGL pointer if push fails.

```rust
pub fn add_style(&self, style: &Style, selector: impl Into<Selector>) -> &Self {
    let selector = selector.into();
    // Push first: if this panics, LVGL is not yet updated.
    self._styles.borrow_mut().push(style.clone());
    // SAFETY: StyleInner is #[repr(C)] with `lv: lv_style_t` at offset 0,
    // so casting *const StyleInner to *const lv_style_t gives the address of `lv`.
    // Rc heap-pins StyleInner; address is stable for the Rc lifetime, which
    // is now at least as long as self (clone stored in self._styles above).
    let style_ptr: *const lv_style_t = Rc::as_ptr(&style.inner).cast();
    unsafe { lv_obj_add_style(self.handle(), style_ptr, selector.raw()) };
    self
}
```

### remove_style / remove_style_all

```rust
pub fn remove_style(&self, style: &Style, selector: impl Into<Selector>) -> &Self {
    let selector = selector.into();
    // SAFETY: StyleInner is #[repr(C)]; cast valid (see add_style). Rc still alive.
    let style_ptr: *const lv_style_t = Rc::as_ptr(&style.inner).cast();
    unsafe { lv_obj_remove_style(self.handle(), style_ptr, selector.raw()) };
    // Remove exactly one Vec entry whose allocation matches. Using retain here
    // would be wrong when the same style was added multiple times (different
    // clones of the same allocation), because it would drop all copies at
    // once while LVGL still holds all but the one just removed.
    let mut styles = self._styles.borrow_mut();
    if let Some(pos) = styles.iter().position(|s| Rc::as_ptr(&s.inner) == Rc::as_ptr(&style.inner)) {
        styles.swap_remove(pos);
    }
    self
}

pub fn remove_style_all(&self) -> &Self {
    // SAFETY: handle non-null.
    unsafe { lv_obj_remove_style_all(self.handle()) };
    self._styles.borrow_mut().clear();
    self
}
```

### Usage

```rust
let style = StyleBuilder::new()
    .radius(5)
    .bg_color(palette_main(Palette::Blue))
    .bg_grad(GradDsc::new()                   // GradDsc moved into StyleBuilder
        .set_dir(GradDir::Ver)
        .set_stops_count(2)
        ...)
    .build();                                 // freeze into clonable Style

obj1.add_style(&style, Selector::DEFAULT);
obj2.add_style(&style, Selector::DEFAULT);
// No Rc in user code. Widgets hold internal clones that keep the style alive.
// When both widgets are dropped, the last clone drops and StyleInner is freed.
```

### Visibility

`StyleBuilder::new()` is `pub`. `Style` has no public constructor --- it is
obtained only from `StyleBuilder::build()`. `Style::boxed()` and
`Style::leaked()` are removed.

### Theme --- known limitation: style not tracked per-widget

`Theme::extend_current` takes `Style` (clone). Internally `Theme` stores
`_style: Style` and sets
`user_data = Rc::as_ptr(&style.inner).cast::<lv_style_t>() as *mut c_void`
(valid because `StyleInner` is `#[repr(C)]` with `lv` at offset 0).

**Known limitation**: the `apply_cb` trampoline calls `lv_obj_add_style`
directly on the C side with a raw `lv_obj_t*`. It cannot push a `Style` clone
into the Rust widget's `_styles` Vec because the trampoline has no access to
the Rust `Obj` wrapper. This means:

- Theme-applied styles are kept alive **only** by the Theme's own `Style`
  clone, not by individual widgets.
- If Theme is dropped while widgets styled by it are still alive, those widgets
  hold dangling LVGL-internal pointers to the freed style.

**Mitigation**: Theme already documents a lifetime contract ("must be kept alive
for as long as any widgets styled by it exist") and `Theme::drop` restores the
parent theme. This is a pre-existing constraint, not introduced by the handle
model. The handle model does not make it worse, but it also does not fix it.

Possible future improvements (out of scope for this change):
- Make Theme non-droppable (`ManuallyDrop` or `'static` leak) on embedded where
  themes are effectively permanent.
- Register a widget-create hook that injects clones into a global registry
  keyed by `lv_obj_t*` --- heavyweight but would close the gap.

For now, the lifetime contract on Theme is accepted as-is.

---

## Level 4 --- Animation target widget lifetime

`lv_anim_set_var` stores a raw `*mut c_void` to the target widget.
`lv_anim_start` copies the full descriptor (including that pointer) into LVGL's
internal animation list. After `start()` the Rust `Anim` struct may be dropped,
but LVGL holds the pointer independently.

### Two-layer safety model

**Layer 1 --- compile time (`Anim<'w>`)**: prevents calling `set_var` with a
widget that has already been dropped or moved out of scope at the call site.

```rust
pub struct Anim<'w> {
    inner: lv_anim_t,
    _widget: PhantomData<&'w ()>,
}

impl<'w> Anim<'w> {
    pub fn set_var<W: AsLvHandle>(&mut self, widget: &'w W) -> &mut Self {
        // SAFETY: widget is valid for 'w; pointer stored for builder phase only.
        unsafe { lv_anim_set_var(&mut self.inner, widget.lv_handle() as *mut c_void) };
        self
    }
}
```

The `'w` bound means the compiler rejects programs that drop the widget before
`start()` is called. It does **not** constrain the widget lifetime after
`start()`.

**Layer 2 --- runtime (LVGL)**: `lv_obj_delete` (called in `Obj::drop`) internally
calls `lv_anim_delete(obj, NULL)` (`lv_obj.c:525`), cancelling every animation
that targets the object. The exec callback will never fire on a freed `lv_obj_t`.

Together: the compiler catches the build-phase misuse; LVGL catches the
post-start misuse. Neither layer alone is complete; both are required and both
are already in place.

### No changes to Obj::drop needed

`lv_obj_delete` already provides the runtime guarantee. Explicitly calling
`lv_anim_delete` in `Obj::drop` before `lv_obj_delete` would be redundant.
The SAFETY comment in `Obj::drop` should document this reliance:

```rust
impl<'p> Drop for Obj<'p> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: handle non-null; Obj is non-Clone so this is the unique owner.
            // lv_obj_delete (LVGL v9.2, lv_obj.c) calls lv_obj_remove_style_all
            // (lv_obj.c:521) and lv_anim_delete(obj, NULL) (lv_obj.c:525) internally,
            // so all style and animation back-references are cleared before Rust
            // drops _styles and any live Anim.
            // Re-verify these call sites when upgrading LVGL.
            unsafe { lv_obj_delete(self.handle) };
        }
    }
}
```

---

## Known gaps and edge cases

### C-side widget deletion

If LVGL deletes a widget through C-side code (e.g. `lv_obj_clean` on a parent),
the Rust `Obj` wrapper is not dropped and `_styles` keeps its Rc clones alive.
The styles are pinned in memory longer than needed (not a use-after-free, but a
leak until the Rust `Obj` is dropped or goes out of scope). This is a
pre-existing architectural issue with all C-side widget manipulation, not
introduced by this change.

### `lv_style_reset` does not individually clear sub-descriptor pointers

`lv_style_reset` (`lv_style.c:192-201`) frees the `values_and_props` buffer
and zeroes the entire `lv_style_t` with `lv_memzero`. The effect is that all
pointers to sub-descriptors (gradient, transition, color filter) cease to exist
because the buffer that contained them is freed. However, LVGL does not iterate
over individual properties or null them out. This distinction matters for SAFETY
comments: the correctness argument is "LVGL frees the buffer, then Rust frees
the pointed-to allocations" --- not "LVGL individually nullifies each pointer."

---

## LVGL-internal memory leak analysis

All claims below verified against LVGL v9.2 source.

| Scenario | Leak? | Reason |
|---|---|---|
| Style dropped normally | No | `lv_style_reset` frees `values_and_props` buffer; Rust Box drops free sub-descriptors |
| `bg_grad` called twice on same StyleBuilder | No | `lv_style_set_prop` overwrites map entry in-place (`lv_style.c:346`); Rust replaces `_grad`, old Box drops |
| Widget dropped with styles applied | No | `lv_obj_delete` -> `lv_obj_remove_style_all` frees per-widget style array (`lv_obj.c:521`) |
| Style dropped while no widget refs it | No | `lv_style_reset` frees map; no LVGL-side refs exist |
| `lv_style_set_*` property map realloc | No | `lv_style_set_prop` reallocs `values_and_props`; old buffer freed by LVGL |
| `mem::forget(style)` | Yes (map) | Standard Rust: `mem::forget` is safe; resource leaks are not UB |
| Theme dropped while widgets live | Use-after-free | See Theme known limitation above |
| C-side widget deletion (lv_obj_clean) | Yes (styles pinned) | See "C-side widget deletion" gap above |

---

## Summary of API changes

| Item | Before | After |
|---|---|---|
| `Style` struct | bare `lv_style_t` wrapper | Frozen clonable handle wrapping `Rc<StyleInner>` |
| `StyleBuilder` struct | n/a | New; mutable build phase, owns `Box<StyleInner>` |
| `StyleBuilder::build()` | n/a | Consumes builder, returns `Style` |
| `StyleInner` | n/a | `#[repr(C)]` with `lv` at offset 0 + `offset_of!` assertion |
| `Style::new()` | `pub` | removed; use `StyleBuilder::new().build()` |
| `Style::clone()` | not available | cheap Rc clone (impl `Clone`) |
| `Style::boxed()`, `Style::leaked()` | pub | removed |
| `StyleInner` `Drop` impl | calls `lv_style_reset` | unchanged (already correct) |
| `GradDsc::new()`, `TransitionDsc::new()`, `ColorFilter::new()` | pub | pub (unchanged) |
| `GradDsc::boxed()` etc. | pub | removed |
| `StyleBuilder::bg_grad` | `&GradDsc` | `GradDsc` (move); compile error after `.build()` |
| `StyleBuilder::transition` | `&TransitionDsc` | `TransitionDsc` (move); compile error after `.build()` |
| `StyleBuilder::color_filter` | `&ColorFilter` | `ColorFilter` (move); compile error after `.build()` |
| `Obj::add_style` | `&Style` | `&Style` (widget stores internal clone) |
| `Obj::_styles` field | absent | `RefCell<Vec<Style>>` |
| `Obj::drop` SAFETY comment | incomplete | documents lv_obj_delete cleanup contract with source refs |
| `Theme::extend_current` | `Style` (owned) | `Style` (clone stored internally) |
| `Image::set_src` | `&lv_image_dsc_t` | `&'static lv_image_dsc_t` |
| `StyleBuilder::bg_image_src` | `&lv_image_dsc_t` | `&'static lv_image_dsc_t` |
| `Line::set_points` | `&[lv_point_precise_t]` | `&'static [lv_point_precise_t]` |
| `Dropdown::set_symbol` | `&str` (dangling) | `&'static CStr` |
| `Anim` | no lifetime | `Anim<'w>` (build-phase only; runtime via lv_obj_delete) |
