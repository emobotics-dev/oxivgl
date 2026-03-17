# Widget Wrapper Specification

How to wrap a new LVGL widget in oxivgl.

---

## 1. Prerequisites

Enable the widget in `conf/lv_conf.h`:

```c
#define LV_USE_MYWIDGET 1
```

Rebuild to regenerate bindings. Verify the `lv_mywidget_*` functions
appear in the bindings output.

---

## 2. File Structure

Create `src/widgets/mywidget.rs`. Register it in `src/widgets/mod.rs`:

```rust
mod mywidget;
pub use mywidget::MyWidget;
```

If the widget introduces new enums or modes, define them in the same
file (not in a shared enums module) unless they are used by multiple
widgets.

---

## 3. Struct Pattern

Every widget wrapper follows the same structure:

- A struct containing `Obj<'p>`.
- `AsLvHandle` impl delegating to the inner Obj.
- `Deref<Target = Obj<'p>>` so all Obj methods (styling, positioning,
  events, layout) are inherited.
- A `new(parent)` constructor returning `Result<Self, WidgetError>`.

The constructor must:
1. Assert parent is non-null.
2. Call the LVGL create function.
3. Null-check the returned handle.
4. Return `Err(WidgetError::LvglNullPointer)` on failure.

---

## 4. Method Conventions

- Setters return `&Self` for chaining.
- Use `&self` (not `&mut self`) — mutation happens on the LVGL C side.
  If a Rust field needs mutation, use `Cell<T>`.
- Method names mirror the LVGL C API translated to Rust conventions:
  `lv_slider_set_value` → `set_value`, `lv_slider_get_value` →
  `get_value`.
- Enum arguments use Rust enums (`#[repr(u32)]` for small exhaustive
  sets, newtype struct for open-ended sets).

---

## 5. Pointer Safety

If an LVGL function stores a raw pointer to caller-provided data, the
Rust wrapper must enforce `'static` at the type level. See
`docs/spec-memory-lifetime.md` §3 for the full list of patterns.

Never pass a stack-local or temporary allocation to an LVGL function
that stores the pointer.

---

## 6. SAFETY Comments

Every `unsafe` block must have a SAFETY comment explaining why the call
is sound. Reference specific LVGL source locations where relevant
(file:line). Cite the memory spec section when enforcing a lifetime
invariant.

---

## 7. Doc Comments

Every public type and method must have `///` doc comments. CI enforces
this via `-W missing-docs`. Include a brief description of what the
LVGL function does and any constraints (e.g. value ranges, required
config flags).

---

## 8. After Adding a Widget

1. Add exports to `src/widgets/mod.rs`.
2. Add to `src/prelude.rs` if commonly used.
3. Write integration tests (`tests/integration.rs`).
4. Write a leak test (`tests/leak_check.rs`).
5. Port at least one LVGL example using the widget.
6. Follow `docs/spec-example-porting.md` §7 checklist.
7. Follow `docs/spec-testing.md` §7 portability check.
