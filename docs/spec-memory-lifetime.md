# Memory and Lifetime Specification

**Status**: Draft
**LVGL version**: v9.5 (upgraded from v9.3; key invariants re-verified)

All safety invariants in this document are normative. The implementation SHALL
conform to this specification. Where LVGL behavior is cited, the exact source
location is given; these locations MUST be re-verified on any LVGL upgrade.

---

## 1. Governing Principle

> **If LVGL stores a raw pointer, the Rust wrapper MUST either own the data
> or require it to be `'static`.**

| Data kind            | LVGL stores pointer to... | Rust encoding |
|----------------------|---|---|
| Compile-time resource | image, point array, symbol string | `'static` reference |
| Runtime resource     | style, sub-descriptor | ownership / ref-count |

No wrapper method SHALL pass a stack-local or short-lived heap address to an
LVGL function that stores the pointer beyond the call's duration.

---

## 2. LVGL Type Lifetime Reference

| LVGL type                     | Allocated by | Freed by | Lifetime | Referred by in LVGL |
|-------------------------------|---|---|---|---|
| `lv_obj_t`, including widgets | User --- `lv_obj_create` | User --- `lv_obj_delete` | Until explicitly deleted; children cascade | Parent's child list; display's `scr_ll` |
| `lv_obj_t::matrix` (if `LV_DRAW_TRANSFORM_USE_MATRIX`) | LVGL --- `lv_obj_set_transform_matrix` | LVGL --- destructor | Until object deleted | `obj->spec_attr->matrix` |
| `lv_style_t`                  | User | User --- `lv_style_reset` | Must outlive all widgets using it | Pointer array in `lv_obj_t` style list |
| `lv_style_transition_dsc_t`   | User | User | Must outlive the style referencing it | Pointer inside `lv_style_t` property map |
| `lv_font_t`                   | Typically static (compile-time font data) | N/A for static fonts | Must outlive all styles using it | Pointer inside `lv_style_t` property map |
| `lv_image_dsc_t` / pixel buf  | User | User | Must outlive all widgets using it | Pointer inside `lv_image_t` widget struct |
| `lv_anim_t` (descriptor)      | User | LVGL --- after `lv_anim_start` | Discardable after `lv_anim_start` | Not retained --- copied immediately |
| `lv_anim_t` (internal copy)   | LVGL --- on `lv_anim_start` | LVGL --- on complete or `lv_anim_delete` | Until animation ends or is cancelled | Global animation list (`lv_ll`) |
| `lv_anim_timeline_t`          | LVGL --- `lv_anim_timeline_create` | User --- `lv_anim_timeline_delete` | Until explicitly deleted | User holds the only pointer |
| `lv_timer_t`                  | LVGL --- `lv_timer_create` | User --- `lv_timer_delete` | Until deleted; one-shot if `repeat_count=1` | Global timer list (`lv_ll`) |
| `lv_display_t`                | LVGL --- `lv_display_create` | User --- `lv_display_delete` | Application lifetime typically | Global display list |
| `lv_indev_t`                  | LVGL --- `lv_indev_create` | User --- `lv_indev_delete` | Application lifetime typically | Global indev list |
| `lv_group_t`                  | LVGL --- `lv_group_create` | User --- `lv_group_delete` | Until explicitly deleted | User holds the pointer |
| `lv_theme_t`                  | User or LVGL --- `lv_theme_create` (v9.5) | User --- `lv_theme_delete` | Must outlive all widgets styled by it | Display's theme chain |
| `lv_subject_t`                | User --- `lv_subject_init_*` | User --- `lv_subject_deinit` | Must outlive all observers and bound widgets | Observer linked list; widget bindings |
| `lv_observer_t`               | LVGL --- `lv_subject_add_observer` / `lv_*_bind_*` | LVGL --- on widget delete, or User --- `lv_observer_remove` | Until widget deleted or manually removed | Subject's observer list |
| `lv_draw_buf_t`               | User --- `lv_draw_buf_create` | User --- `lv_draw_buf_destroy` | Until explicitly destroyed | Canvas widget; snapshot callers |
| `lv_event_t`                  | LVGL (stack frame) | LVGL --- automatic | Duration of the event callback only | Stack frame in `lv_event_send` |
| `lv_draw_task_t`              | LVGL --- render pipeline | LVGL --- after draw completes | Duration of `DRAW_TASK_ADDED` callback only | Render task list; passed via `lv_event_get_draw_task` |
| `lv_draw_label_dsc_t` (in draw task) | LVGL --- render pipeline | LVGL --- after draw completes | Same scope as parent `lv_draw_task_t` | `draw_task->draw_dsc` pointer |
| `lv_draw_rect_dsc_t` etc. (user draw) | User (stack) | Automatic | Duration of the draw call | Not retained by LVGL |

---

## 3. Static Resources

Any LVGL API that stores a pointer to user-supplied data whose content is
expected to be a compile-time constant SHALL require `'static` in its Rust
signature.

### 3.1 Image descriptors

```rust
Image::set_src(src: &'static lv_image_dsc_t)
StyleBuilder::bg_image_src(src: &'static lv_image_dsc_t)
```

Images produced by `image_declare!` are inherently `'static`.

### 3.2 Line points

```rust
Line::set_points(points: &'static [lv_point_precise_t])
```

Point arrays MUST be declared `static` or produced via `Box::leak`. The doc
comment SHALL warn that leaked memory is never reclaimed; `Box::leak` is
acceptable only for data whose effective lifetime matches the display's.

### 3.3 Dropdown symbol

```rust
Dropdown::set_symbol(symbol: &'static CStr)
```

**Rationale**: LVGL stores the raw pointer (`dropdown->symbol = symbol`,
`lv_dropdown.c:373`). The symbol string MUST outlive the widget.

### 3.4 Transition property array

```rust
TransitionDsc::new(props: &'static [lv_style_prop_t], ...)
```

LVGL's `lv_style_transition_dsc_init` stores a raw pointer to `props` inside
the `lv_style_transition_dsc_t`. This `'static` bound is **load-bearing** and
MUST NOT be relaxed.

---

## 4. Style Ownership Model

### 4.1 Two-phase type system

Style handling uses two distinct types enforcing a build-then-share discipline
at compile time:

| Type | Phase | Mutability | Clonability |
|---|---|---|---|
| `StyleBuilder` | Build | Mutable (`&mut self` setters) | Not `Clone` |
| `Style` | Share | Immutable (no setters) | `Clone` (cheap Rc bump) |

`StyleBuilder::build(self) -> Style` consumes the builder and produces a
frozen handle. Attempting to call a setter on `Style` is a compile error.

```rust
let style = StyleBuilder::new()
    .radius(5)
    .bg_color(blue)
    .build();
widget.add_style(&style, DEFAULT);
```

### 4.2 Struct layout

```rust
pub struct StyleBuilder {
    inner: Box<StyleInner>,
}

#[derive(Clone)]
pub struct Style {
    inner: Rc<StyleInner>,
}

#[repr(C)]  // lv MUST be at offset 0
struct StyleInner {
    lv: lv_style_t,
    _grad: Option<Box<GradDsc>>,
    _transition: Option<Box<TransitionDsc>>,
    _color_filter: Option<Box<ColorFilter>>,
}

const _: () = assert!(core::mem::offset_of!(StyleInner, lv) == 0);
```

**Invariants**:

- `StyleInner` MUST be `#[repr(C)]` with `lv` as the first field.
- The `offset_of!` assertion MUST be present as a compile-time check.
- Casting `*const StyleInner` to `*const lv_style_t` is valid only because of
  the above.
- `Style` has no public constructor; it is obtainable only via
  `StyleBuilder::build`.
- `StyleBuilder::new` is `pub`.

### 4.3 Ref counting

`Style` uses `alloc::rc::Rc<StyleInner>`. `Arc` SHALL NOT be used.

**Rationale**: the project operates under a single-task LVGL constraint. `Rc`
avoids atomic overhead on Xtensa (no hardware atomics). `Rc` is inherently
`!Send + !Sync`, correctly preventing cross-thread use.

If multi-thread support is added in the future, `Rc` MUST be replaced with
`Arc` and all LVGL calls MUST acquire `lv_lock`/`lv_unlock`.

### 4.4 Sub-descriptor ownership

Sub-descriptors (`GradDsc`, `TransitionDsc`, `ColorFilter`) are taken by value
(move) into `StyleBuilder` and stored as `Option<Box<T>>` inside `StyleInner`.

```rust
StyleBuilder::bg_grad(grad: GradDsc)         // move, not borrow
StyleBuilder::transition(trans: TransitionDsc)
StyleBuilder::color_filter(filter: ColorFilter)
```

**Why `Box`, not bare fields**: `StyleBuilder::build()` converts
`Box<StyleInner>` to `Rc<StyleInner>`, which MAY relocate the `StyleInner`
allocation. Sub-descriptors are passed to LVGL during the build phase; their
addresses MUST remain stable across the Box-to-Rc conversion. Each `Box<T>`
provides an independent, stable heap address.

**Replacement safety**: calling a sub-descriptor setter a second time MUST:

1. Call the LVGL `lv_style_set_*` function with the new pointer first
2. Then replace `Option<Box<T>>`, dropping the old allocation

This ordering is safe because LVGL overwrites property map entries in-place
(`lv_style_set_prop`, `lv_style.c:344-346`). The old pointer is no longer
stored in LVGL when the old Box drops.

### 4.5 StyleBuilder does not require Pin

`lv_style_t` is not self-referential: `values_and_props` targets LVGL-managed
heap. Sub-descriptor pointers target their own Box allocations. `StyleInner`
can be freely relocated (Box → Rc) without invalidating any pointer.

### 4.6 Constructor visibility

`GradDsc::new()`, `TransitionDsc::new()`, and `ColorFilter::new()` are `pub`.
They are safe to construct because they are immediately moved into
`StyleBuilder`; no stack-allocated descriptor is ever handed to LVGL directly.

`Style::boxed()`, `Style::leaked()`, `GradDsc::boxed()` SHALL NOT exist.

### 4.7 Drop sequence

`StyleInner` implements `Drop`. The drop sequence when Rc refcount hits 0:

1. `Drop::drop` runs --- calls `lv_style_reset` (`lv_style.c:192-201`), which
   frees the `values_and_props` buffer and zeroes the `lv_style_t`. After this
   call, no LVGL data structure references any sub-descriptor address.
2. Rust drops `Option<Box<...>>` fields --- sub-descriptor heap memory freed.

This order is guaranteed by Rust: `Drop::drop` always runs before field
destructors.

```rust
impl Drop for StyleInner {
    fn drop(&mut self) {
        unsafe { lv_style_reset(&mut self.lv) };
    }
}
```

**`mem::forget` caveat**: if a `Style` handle is the last clone and is leaked
via `mem::forget`, `lv_style_reset` never runs and the LVGL property map is
leaked. This is standard Rust behavior (`mem::forget` is safe; resource leaks
are not UB).

---

## 5. Widget Style Tracking

### 5.1 Storage

Each widget wrapper (`Obj`, `Button`, etc.) SHALL contain:

```rust
_styles: RefCell<Vec<Style>>
```

`RefCell` (not `Mutex`) is correct: widgets are `!Send + !Sync` (`Obj` contains
`*mut lv_obj_t`). `alloc::vec::Vec<Style>` is used on all targets.

### 5.2 add_style

`Obj::add_style(&self, style: &Style, selector: impl Into<Selector>)` SHALL:

1. Push `style.clone()` to `_styles` **before** calling `lv_obj_add_style`.
2. Obtain the LVGL pointer via `Rc::as_ptr(&style.inner).cast::<lv_style_t>()`.

**Ordering rationale**: if the push panics (OOM), LVGL is not updated and both
sides remain consistent. The inverse order would leave a dangling LVGL pointer
if push fails.

`add_style` SHALL NOT accept `StyleBuilder` --- only `Style`.

### 5.3 remove_style

`Obj::remove_style` SHALL:

1. Call `lv_obj_remove_style` first.
2. Remove exactly one `_styles` entry matching by Rc pointer identity.

`retain` MUST NOT be used: when the same style is added multiple times (same
Rc allocation, different selectors), `retain` would drop all clones at once
while LVGL still holds all but the removed one.

When `style` is `None`, LVGL removes all styles for the given selector, but
the `_styles` Vec is **not** updated — the Rc clones remain alive until the
widget is dropped. This is a known limitation: safely identifying which Rc
entries correspond to a selector-only removal would require tracking selectors
per entry. The extra Rc clones are a memory leak (not use-after-free) bounded
by widget lifetime. Use `remove_style_all` for full cleanup.

### 5.4 remove_style_all

`Obj::remove_style_all` SHALL call `lv_obj_remove_style_all` then clear
`_styles`.

### 5.5 Drop order

`Obj::drop` calls `lv_obj_delete`, which internally calls
`lv_obj_remove_style_all` (`lv_obj.c:521`) and `lv_anim_delete(obj, NULL)`
(`lv_obj.c:525`) before returning. This removes all LVGL-side style and
animation references before Rust drops `_styles`.

**Invariant**: LVGL releases pointers before Rust frees allocations.

**Version pinning**: verified for LVGL v9.5.
Both call sites MUST be re-verified on any LVGL upgrade. LVGL also frees
`name` and `matrix` in the destructor, but these are LVGL-managed and
do not affect Rust-side cleanup. An integration test SHALL exercise the
add-style-then-drop-widget path as part of CI.

The `Obj::drop` implementation uses an `lv_obj_is_valid` guard to handle the
parent-cascade case safely. When a parent is deleted, LVGL cascade-deletes all
children first; if Rust drops the child wrapper afterwards, `lv_obj_is_valid`
returns `false` and `lv_obj_delete` is skipped (safe no-op). This makes widget
constructors return `W` directly (no `ManuallyDrop` wrapper) for widgets stored
in View structs.

**Tradeoff:** `lv_obj_is_valid` performs an O(N) walk of the full object tree
on every `Obj::drop`. For large UIs with many widgets being torn down, this is
observable overhead. It is accepted because: (a) teardown is infrequent,
(b) the safety guarantee is unconditional, and (c) the common-path delete still
calls `lv_obj_delete` exactly once.

**Local-variable widgets** (created inside `create()`/`on_event()` but not
stored in the View struct) still require `core::mem::forget` if they must
outlive their Rust scope. `lv_obj_is_valid` returns `true` for these (LVGL
still holds them via parent), so `Obj::drop` would call `lv_obj_delete` and
remove them from the UI prematurely. This is the correct and intended behavior
for examples like modal msgboxes and dynamically-added menu items.

The `Obj::drop` SAFETY comment MUST document this reliance with LVGL source
references:

```rust
impl<'p> Drop for Obj<'p> {
    fn drop(&mut self) {
        // SAFETY: lv_obj_is_valid returns false for already-deleted objects
        // (parent cascade), making this a safe no-op in that case.
        // lv_obj_delete (LVGL v9.5, lv_obj.c) calls lv_obj_remove_style_all
        // (lv_obj.c:521) and lv_anim_delete(obj, NULL) (lv_obj.c:525) internally,
        // so all style and animation back-references are cleared before Rust
        // drops _styles and any live Anim.
        // Re-verify these call sites when upgrading LVGL.
        if !self.handle.is_null() && unsafe { lv_obj_is_valid(self.handle) } {
            unsafe { lv_obj_delete(self.handle) };
        }
    }
}
```

---

## 6. Animation Lifetime

### 6.1 Two-layer safety model

Animation safety requires two complementary layers. Neither is sufficient
alone.

**Layer 1 --- compile time**: `Anim<'w>` carries a lifetime tied to the target
widget, preventing the widget from being dropped before `start()`.

```rust
pub struct Anim<'w> {
    inner: lv_anim_t,
    _widget: PhantomData<&'w ()>,
}

impl<'w> Anim<'w> {
    pub fn set_var<W: AsLvHandle>(&mut self, widget: &'w W) -> &mut Self;
}
```

**Layer 2 --- runtime**: `lv_obj_delete` (called in `Obj::drop`) calls
`lv_anim_delete(obj, NULL)` (`lv_obj.c:525`), cancelling every animation
targeting the object. The exec callback never fires on a freed `lv_obj_t`.

The `'w` bound ensures the widget is alive at `start()` time. LVGL ensures the
animation is cancelled if the widget is deleted after `start()`.

### 6.2 AnimHandle

`AnimHandle` wraps a raw `*mut lv_anim_t` pointing to LVGL's internal copy
(allocated by `lv_anim_start`). LVGL frees this copy when the animation
completes or the target widget is deleted. `AnimHandle` has **no lifetime
enforcement** — it is an intentional escape hatch for operations on running
animations (e.g. `pause_for`).

**Invariant**: `AnimHandle` methods are `unsafe`. The caller MUST ensure the
animation is still running. Storing an `AnimHandle` past animation completion
or widget deletion and then calling methods on it is undefined behaviour.

**Rationale**: compile-time enforcement is impractical because animation
completion is a runtime event. The `unsafe` + safety doc on each method is
the only protection.

### 6.3 v9.3 animation API changes

v9.3 renames `playback_*` fields/setters to `reverse_*` (e.g.
`lv_anim_set_playback_delay` → `lv_anim_set_reverse_delay`) and adds
`pause_time`/`pause_duration`/`is_paused` fields. These are naming/feature
changes only; copy semantics (`lv_memcpy` in `lv_anim_start`) and the
`lv_anim_delete` cleanup in `lv_obj_delete` are unchanged.

### 6.4 Obj::drop --- no additional animation cleanup needed

`lv_obj_delete` already cancels animations. Explicitly calling `lv_anim_delete`
in `Obj::drop` before `lv_obj_delete` would be redundant.

---

## 7. Theme

### 7.1 Style ownership

`Theme::extend_current` takes a `Style` by value (ownership transfer). The
`Theme` stores it internally. The LVGL `user_data` pointer is obtained via
`Rc::as_ptr(&style.inner).cast::<lv_style_t>()` (valid due to the `#[repr(C)]`
offset-0 guarantee).

### 7.2 Known limitation: style not tracked per-widget

The `apply_cb` trampoline calls `lv_obj_add_style` on the C side with a raw
`lv_obj_t*`. It has no access to the Rust `Obj` wrapper and therefore cannot
push a `Style` clone into the widget's `_styles` Vec.

**Consequence**: Theme-applied styles are kept alive only by the Theme's own
clone. If `Theme` is dropped while widgets styled by it are still alive, those
widgets hold dangling pointers.

**Mitigation**: `Theme` documents a lifetime contract: it MUST be kept alive
for as long as any widgets styled by it exist. `Theme::drop` restores the
parent theme.

**Future options** (out of scope):

- Make `Theme` non-droppable (`ManuallyDrop` or `'static` leak) where themes
  are effectively permanent.
- Register a widget-create hook that injects clones into a global registry
  keyed by `lv_obj_t*`.

---

## 8. Known Gaps

### 8.1 C-side widget deletion

If LVGL deletes a widget through C-side code (e.g. `lv_obj_clean` on a
parent), the Rust `Obj` wrapper is not notified. `_styles` keeps its Rc clones
alive longer than necessary (memory leak, not use-after-free, until the Rust
`Obj` goes out of scope). This is a pre-existing architectural gap with all
C-side widget manipulation.

### 8.2 `lv_style_reset` buffer semantics

`lv_style_reset` (`lv_style.c:192-201`) frees the `values_and_props` buffer
and zeroes the `lv_style_t` with `lv_memzero`. It does NOT individually null
out each property pointer. The correctness argument is: "LVGL frees the buffer
(which contained the pointers), then Rust frees the pointed-to allocations" ---
not "LVGL individually nullifies each pointer." SAFETY comments MUST use this
formulation.

---

## 9. Memory Leak Analysis

All claims verified against LVGL v9.5 source.

| Scenario | Leak? | Reason |
|---|---|---|
| Style dropped normally | No | `lv_style_reset` frees `values_and_props`; Rust Box drops free sub-descriptors |
| `bg_grad` called twice on same StyleBuilder | No | `lv_style_set_prop` overwrites map entry in-place (`lv_style.c:344-346`); Rust replaces `_grad`, old Box drops |
| Widget dropped with styles applied | No | `lv_obj_delete` -> `lv_obj_remove_style_all` frees per-widget style array (`lv_obj.c:521`) |
| Style dropped while no widget refs it | No | `lv_style_reset` frees map; no LVGL-side refs exist |
| `lv_style_set_*` property map realloc | No | `lv_style_set_prop` reallocs `values_and_props`; old buffer freed by LVGL |
| `mem::forget(style)` | Yes (map) | `mem::forget` is safe; resource leaks are not UB |
| Theme dropped while widgets live | UAF | See section 7.2 |
| C-side widget deletion (`lv_obj_clean`) | Yes (styles pinned) | See section 8.1 |

---

## 10. Layer Model

```
Widget (Obj<'p>)
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

Anim<'w>                               <-- build-phase borrow; runtime via lv_obj_delete

Theme
  +-- Style (clone)                    <-- Theme keeps style alive (section 7.2 limitation)
```

---

## 11. Public API Summary

| Item | Specification |
|---|---|
| `StyleBuilder` | `pub struct`; mutable build phase, owns `Box<StyleInner>` |
| `StyleBuilder::new()` | `pub`; initializes `lv_style_t` via `lv_style_init` |
| `StyleBuilder::build(self)` | Consumes builder, returns `Style` |
| `StyleBuilder::{bg_grad, transition, color_filter}` | Take sub-descriptor by value (move) |
| `Style` | `pub struct`; frozen, `Clone`; wraps `Rc<StyleInner>` |
| `Style` constructors | None public; only via `StyleBuilder::build` |
| `Style::clone` | Cheap Rc clone |
| `StyleInner` | `#[repr(C)]`; `lv` at offset 0; `Drop` calls `lv_style_reset` |
| `GradDsc::new`, `TransitionDsc::new`, `ColorFilter::new` | `pub`; constructed then moved into `StyleBuilder` |
| `Obj::add_style` | Takes `&Style`; widget stores internal Rc clone |
| `Obj::remove_style` | Removes exactly one entry by Rc pointer identity |
| `Obj::_styles` | `RefCell<Vec<Style>>`; `alloc::vec::Vec` on all targets |
| `Obj::drop` | Calls `lv_obj_delete`; SAFETY comment documents LVGL cleanup |
| `Theme::extend_current` | Takes `Style` by value (ownership stored internally) |
| `Image::set_src` | `&'static lv_image_dsc_t` |
| `StyleBuilder::bg_image_src` | `&'static lv_image_dsc_t` |
| `Line::set_points` | `&'static [lv_point_precise_t]` |
| `Dropdown::set_symbol` | `&'static CStr` |
| `Anim<'w>` | Lifetime `'w` tied to target widget via `PhantomData` |
| `Anim::set_var` | `widget: &'w W` where `W: AsLvHandle` |

---

## 12. LVGL v9.2 → v9.5 Upgrade Considerations

This section catalogues memory-management and lifetime changes across LVGL
v9.3, v9.4, and v9.5 that affect the Rust wrapper. The core ownership model
(styles = borrowed pointers, animations = shallow copy, destructors free only
LVGL-allocated data) is unchanged. The additions below are incremental.

### 12.1 `lv_ext_data_t` — new extension-data mechanism (v9.5)

```c
typedef struct {
    void * data;
    void (* free_cb)(void * data);
} lv_ext_data_t;
```

Conditional on `LV_USE_EXT_DATA`. Added as first field of `lv_obj_t`,
`lv_anim_t`, and `lv_theme_t`. Enables attaching arbitrary data with a
destructor callback.

- `lv_anim.c`: `remove_anim` now calls `ext_data.free_cb` before freeing
  the animation node.
- `lv_obj.c`: destructor calls `ext_data.free_cb` during object teardown.

**Wrapper action**: if `LV_USE_EXT_DATA` is disabled in `lv_conf.h`, no
action needed. If enabled, any Rust wrapper exposing ext_data MUST use the
`Box::into_raw` / `Box::from_raw` pattern for the `free_cb` to prevent
double-free or leak. The `free_cb` is an `unsafe extern "C" fn` and MUST be
documented as such.

**Struct layout impact**: `ext_data` is inserted at offset 0 of `lv_obj_t`
and `lv_anim_t`, shifting all other fields. Bindings MUST be regenerated.

### 12.2 `lv_obj_t` struct changes

New bitfields added across v9.3–v9.5:

| Field | Version | Purpose |
|---|---|---|
| `h_ignore_size : 1` | v9.5 | Layout flag |
| `w_ignore_size : 1` | v9.5 | Layout flag |
| `radio_button : 1` | v9.5 | Radio-button behavior |
| `rendered : 1` | v9.5 | Render tracking |

These are layout/rendering flags with no pointer-safety impact. However, the
struct size changes — bindings MUST be regenerated on upgrade.

The destructor sequence (`lv_obj_remove_style_all` → `lv_anim_delete` →
group/event/spec_attr cleanup) is **unchanged** through v9.5.

### 12.3 Animation API changes

**Field renames (v9.3)**:

| v9.2 | v9.3+ |
|---|---|
| `playback_delay` | `reverse_delay` |
| `playback_duration` | `reverse_duration` |
| `playback_now` | `reverse_play_in_progress` |

Corresponding setter macros renamed (`lv_anim_set_playback_*` →
`lv_anim_set_reverse_*`).

**New fields (v9.3+)**: `pause_time`, `pause_duration`, `is_paused` —
support `lv_anim_pause()`.

**Copy semantics unchanged**: `lv_anim_start` still does
`lv_memcpy(new_anim, a, sizeof(lv_anim_t))`. Caller descriptor is
discardable after the call.

**Safety fix (v9.3, #7926)**: deleting an animation from within its own
callback no longer crashes. No wrapper-side change needed.

**Wrapper action**: update field names in any `lv_anim_t` wrapper. Expose
`lv_anim_pause` / `lv_anim_resume`. Regenerate bindings (struct size changed).

### 12.4 `lv_obj_bind_style_prop` — new pointer hazard (v9.5)

```c
void lv_obj_bind_style_prop(lv_obj_t * obj, lv_style_prop_t prop,
                            lv_style_selector_t selector,
                            lv_subject_t * subject);
```

Stores a raw pointer to `lv_subject_t`. The subject MUST outlive the object.

**Wrapper action**: if exposed, require `'static` or a lifetime-tied
reference to the subject, similar to the `'w` pattern on `Anim`. If not
exposed, no action needed.

### 12.5 Dropdown text ownership change (v9.5)

v9.5 splits `static_txt` into `static_options` and `static_text`, and the
destructor now frees dynamically-allocated `text`:

```c
// v9.5 destructor
if(!dropdown->static_options) lv_free(dropdown->options);
if(!dropdown->static_text)    lv_free(dropdown->text);
```

v9.3 only frees `options`. Symbol storage is unchanged (raw pointer, no
free in destructor).

**Wrapper action**: if wrapping `lv_dropdown_set_text`, verify whether LVGL
copies or borrows the string. If it borrows (likely, matching `options`
pattern), the wrapper MUST require `'static` or use `CString` + leak.

### 12.6 Theme lifecycle API (v9.5)

New APIs:

```c
lv_theme_t * lv_theme_create(void);
void lv_theme_delete(lv_theme_t * theme);
void lv_theme_copy(lv_theme_t * dst, const lv_theme_t * src);
void lv_theme_set_apply_cb(lv_theme_t * theme, lv_theme_apply_cb_t apply_cb);
void lv_theme_set_external_data(lv_theme_t * theme, void * data, void (*free_cb)(void *));
```

Previously themes were essentially static/global. v9.5 enables RAII-style
theme management.

**Wrapper action**: `lv_theme_create`/`lv_theme_delete` enable a proper
`Drop` impl on `Theme`, resolving the limitation in section 7.2. The
`external_data` + `free_cb` pattern requires the same `Box::into_raw` /
`Box::from_raw` discipline as `lv_ext_data_t` (section 12.1).

### 12.7 `lv_lock` / `lv_unlock` now public (v9.4)

Previously internal. Now available for multi-threaded LVGL access.

**Wrapper action**: relevant if the project adds multi-thread support (see
section 4.3 on `Rc` → `Arc` migration). No action for single-task mode.

### 12.8 Event handling changes

- **v9.3**: events comprehensively cleaned up during object destruction
  (#7811). `LV_EVENT_DELETE` can be deferred (#6655).
- **v9.4**: event trickle to children added; won't trickle to deleted
  widgets.
- **v9.5**: `LV_EVENT_DELETE` sent to display objects.

**Wrapper action**: deferred deletion means event callbacks may fire on
objects scheduled for deletion. If the wrapper registers callbacks that
access Rust state via `user_data`, the callback MUST check validity.
Currently mitigated by single-task execution (no concurrent deletion), but
worth documenting.

### 12.9 Memory allocator additions (v9.3–v9.4)

- `lv_reallocf` (v9.3, #7780): frees original on realloc failure. Used
  internally; no wrapper impact.
- `lv_calloc` (v9.3, #6743): zeroed allocation. Internal use only.
- All `lv_malloc`'d memory now zeroed (v9.4): reduces UB from uninitialized
  reads in LVGL internals. No wrapper API change.

### 12.10 Style system stability

`lv_style_t` struct, `lv_style_reset`, `lv_style_set_prop`,
`lv_style_transition_dsc_init` are **unchanged** across v9.2–v9.5. The
property-map layout (values then props in a single `lv_realloc`'d buffer)
is identical.

New addition: `lv_style_merge(dst, src)` (v9.4) — deep-copies properties
from `src` into `dst`. Safe (no pointer aliasing). Can be exposed directly.

### 12.11 Upgrade priority summary

| Priority | Item | Section |
|---|---|---|
| **Must** | Regenerate bindings (struct layout changes) | 12.1, 12.2, 12.3 |
| **Must** | Update animation field names (`playback_*` → `reverse_*`) | 12.3 |
| **Should** | Decide on `LV_USE_EXT_DATA` in `lv_conf.h` | 12.1 |
| **Should** | Verify dropdown `text` ownership semantics | 12.5 |
| **Should** | Evaluate new theme RAII APIs for section 7.2 fix | 12.6 |
| **Could** | Expose `lv_anim_pause`, `lv_style_merge` | 12.3, 12.10 |
| **Could** | Wrap `lv_obj_bind_style_prop` with lifetime safety | 12.4 |
| **Won't** (single-task) | Expose `lv_lock`/`lv_unlock` | 12.7 |
