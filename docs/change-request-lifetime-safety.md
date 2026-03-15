# Change Request: Prevent Style and Widget Lifetime Footguns

## Problem

LVGL's C ownership model clashes with Rust's RAII Drop semantics in two ways:

1. **Style use-after-free**: `add_style(&style, ...)` takes a Rust borrow, but LVGL
   stores a raw `*lv_style_t` indefinitely. When the `Style` drops, `lv_style_reset()`
   invalidates the pointer — widgets silently lose their styling.

2. **Widget deletion on function return**: `Obj::drop()` calls `lv_obj_delete()`.
   Widgets created as local variables in helper functions or generated code get deleted
   when the function returns, even though LVGL's parent-child tree still expects them.

Both bugs produce silent failures (empty screen, missing styles) with no panics or
warnings, making them very hard to diagnose.

**Discovered in**: oxiforge code generator — generated `create_*()` functions created
widgets and styles as locals, which were all destroyed on return.

## Proposed Changes

### 1. `Obj::leak(self)` (non-breaking, minimal)

Consume the Rust wrapper without calling `lv_obj_delete`. LVGL retains ownership
via the parent-child tree.

```rust
/// Relinquish Rust ownership of this LVGL object. The object continues to
/// live in LVGL's widget tree (owned by its parent). After this call the
/// Rust wrapper is consumed and `Drop` will NOT delete the object.
pub fn leak(mut self) {
    self.handle = core::ptr::null_mut();
    // Drop runs but sees null handle — no lv_obj_delete.
}
```

Applies to all widget types that deref to `Obj` (Label, Arc, Button, ...).

### 2. `Style::leak(self) -> &'static mut Style` (non-breaking, minimal)

Convenience for heap-allocating and leaking a style so LVGL's raw pointer
remains valid forever.

```rust
/// Leak this style onto the heap, returning a `&'static mut` reference.
/// Use this when passing styles to `add_style()` in contexts where the
/// style would otherwise be dropped (helper functions, generated code).
pub fn leak(self) -> &'static mut Style {
    alloc::boxed::Box::leak(alloc::boxed::Box::new(self))
}
```

### 3. Tighten `add_style` to `&'static Style` (breaking, strongest guarantee)

Change signature from `&Style` to `&'static Style`. This makes it a compile
error to pass a temporary or local style.

```rust
pub fn add_style(&self, style: &'static Style, selector: impl Into<Selector>) -> &Self
```

**Trade-off**: View-struct pattern needs `Box::leak` or a `Style` stored in a
`&'static` location. More verbose, but eliminates the footgun at compile time.

### 4. Alternative: `SharedStyle` wrapper (non-breaking, ergonomic)

A reference-counted wrapper that is always `'static`-safe:

```rust
pub struct SharedStyle {
    inner: &'static Style,
}

impl SharedStyle {
    pub fn new(style: Style) -> Self {
        Self { inner: Style::leak(style) }
    }
}
```

`add_style` gains an overload (or `SharedStyle` implements a trait) so it can
accept either `&'static Style` or `&SharedStyle`.

## Recommendation

- **Immediate** (non-breaking): Ship options 1 + 2. They solve the problem for
  generated code and helper functions with zero impact on existing code.
- **Follow-up** (breaking): Consider option 3 for the next major version. It
  catches the bug at compile time, which is the Rust way.
