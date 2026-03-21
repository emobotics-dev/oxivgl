# Plan: Child<W> as universal widget return type

## Problem

`Widget::new(&parent)` returns owning `Widget<'p>` (with Drop → `lv_obj_delete`).
LVGL also owns the widget via its parent tree. When Rust drops the widget, LVGL
removes it from the parent tree early (visual glitch). Requires fragile `detach()`
call to opt out of Rust ownership. Forgetting `detach()` is silently wrong.

## Solution

All widget constructors return `Child<Widget<'p>>` instead of `Widget<'p>`.
`Child<W>` uses `ManuallyDrop<W>` — dropping is always a no-op.
LVGL tree owns and cleans up all widgets.
Explicit deletion via new `Child::delete(this: Self)` when needed.

## Changes

### child.rs
- Add `Child::delete(this: Self) where W: AsLvHandle` — calls `lv_obj_delete` once
- Add `compile_fail` doctest showing old owning pattern no longer compiles

### src/widgets/*.rs (25 constructors)
- `new(&parent) -> Result<W, WidgetError>` → `Result<Child<W>, WidgetError>`
- Wrap return: `Ok(W { ... })` → `Ok(Child::new(W { ... }))`
- All sub-constructors that create lv_obj_t children (Obj::new, etc.)

### src/widgets/mod.rs
- `detach` function: mark deprecated (no longer needed)

### examples/*.rs (131 files)
- Struct fields: `Widget<'static>` → `Child<Widget<'static>>`
- Imports: add `Child` to widget import list
- Remove `detach(w)` calls (4 files: scroll6, scroll7, scroll8, widget_list2)

### tests/leak_check.rs
- `drop(w)` → `Child::delete(w)` for explicit deletion in `assert_no_leak` closures
- Add `Child` to imports

### tests/integration.rs
- No explicit drop changes needed (already no explicit drops in most tests)
- Add `Child` to imports

## Compile-fail test

```rust,compile_fail
// Old pattern: storing owning Widget — now type mismatch
fn bad(screen: &Screen) -> Label<'_> {
    Label::new(screen).unwrap()  // returns Child<Label>, not Label
}
```

## Key Properties After Change

- Dropping any widget: always no-op (safe)
- Explicit deletion: `Child::delete(w)` calls `lv_obj_delete` exactly once
- Old owning pattern: compile error (type mismatch)
- Method calls: unchanged (Child<W> derefs to W)
- View structs: hold `Child<W<'static>>` — LVGL/screen cleans up

## Unresolved Questions
- ScaleSection / ScaleLabels: are they lv_obj_t children? Check Drop behavior.
- ChartSeries: not lv_obj_t, stays as-is.
- AnimHandle / Anim: separate lifecycle, stays as-is.
