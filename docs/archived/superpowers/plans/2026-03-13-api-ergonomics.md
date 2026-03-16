# API Ergonomics Improvements

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate raw u32 selectors, magic numbers, and positional-argument footguns so both humans and AI agents make fewer mistakes.

**Architecture:** Six independent improvements, ordered by dependency. Selector type first (needed by later tasks), then standalone fixes in parallel. Each task produces a commit.

**Tech Stack:** Rust, LVGL FFI, `#[repr(u32)]` enums, newtype structs

**Verification after every task:**
```sh
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu
LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu --lib
```

---

## Task 1: `Selector` type — replace `selector: u32`

**Files:**
- Create: `src/widgets/selector.rs`
- Modify: `src/widgets/mod.rs` — add `mod selector; pub use selector::Selector;`
- Modify: `src/widgets/obj_style.rs` — change 10 methods
- Modify: `src/widgets/obj.rs` — change `add_style`
- Modify: `src/widgets/enums.rs` — add `BitOr<ObjState> for Part`
- Modify: all examples using `add_style(_, 0)` or `add_style(_, ObjState::PRESSED.0)`
- Test: `src/widgets/selector.rs` — inline `#[cfg(test)]` module

### Design

```rust
// src/widgets/selector.rs

/// Style selector = Part + ObjState bits.
/// Replaces raw `u32` in all style-related methods.
#[derive(Clone, Copy, Debug, Default)]
pub struct Selector(u32);

impl Selector {
    /// Default selector (main part, default state). Same as `Selector::default()`.
    pub const DEFAULT: Self = Self(0);

    /// Raw u32 value for passing to LVGL FFI.
    pub fn raw(self) -> u32 { self.0 }
}

impl From<super::obj::Part> for Selector {
    fn from(p: super::obj::Part) -> Self { Self(p as u32) }
}

impl From<super::ObjState> for Selector {
    fn from(s: super::ObjState) -> Self { Self(s.0) }
}
```

In `src/widgets/enums.rs` or `selector.rs`, add:
```rust
impl core::ops::BitOr<ObjState> for Part {
    type Output = Selector;
    fn bitor(self, rhs: ObjState) -> Selector {
        Selector(self as u32 | rhs.0)
    }
}
```

### Steps

- [ ] **Step 1:** Create `src/widgets/selector.rs` with `Selector` struct, `DEFAULT` const, `From<Part>`, `From<ObjState>`, `BitOr<ObjState> for Part`, and tests:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::widgets::obj::Part;
      use crate::widgets::ObjState;

      #[test]
      fn default_is_zero() {
          assert_eq!(Selector::DEFAULT.raw(), 0);
          assert_eq!(Selector::default().raw(), 0);
      }

      #[test]
      fn from_part() {
          let s: Selector = Part::Indicator.into();
          assert_eq!(s.raw(), Part::Indicator as u32);
      }

      #[test]
      fn from_state() {
          let s: Selector = ObjState::PRESSED.into();
          assert_eq!(s.raw(), ObjState::PRESSED.0);
      }

      #[test]
      fn part_bitor_state() {
          let s = Part::Indicator | ObjState::PRESSED;
          assert_eq!(s.raw(), Part::Indicator as u32 | ObjState::PRESSED.0);
      }
  }
  ```

- [ ] **Step 2:** Register module in `mod.rs`, export `Selector`.

- [ ] **Step 3:** Change all `selector: u32` params in `obj_style.rs` and `obj.rs` to `selector: impl Into<Selector>`. Inside each method body: `let selector = selector.into().raw();`. Methods affected (10 in obj_style + `add_style` in obj_style):
  - `add_style`, `radius`, `style_bg_color`, `style_bg_grad_color`, `style_bg_grad_dir`, `style_transform_rotation`, `style_transform_scale`, `style_transform_pivot_x`, `style_transform_pivot_y`, `set_style_base_dir`

- [ ] **Step 4:** Update all examples. Replace:
  - `add_style(&s, 0)` → `add_style(&s, Selector::DEFAULT)`
  - `add_style(&s, ObjState::PRESSED.0)` → `add_style(&s, ObjState::PRESSED)`
  - `style_bg_color(c, Part::Indicator as u32 | ObjState::PRESSED.0)` → `style_bg_color(c, Part::Indicator | ObjState::PRESSED)`
  - `radius(0x7fff, 0)` → `radius(RADIUS_MAX, Selector::DEFAULT)` (after Task 2)

- [ ] **Step 5:** Build, test, commit.

---

## Task 2: `RADIUS_MAX` constant — replace `0x7fff`

**Files:**
- Modify: `src/widgets/obj_style.rs` — add constant + doc
- Modify: examples using `0x7fff`

### Design

```rust
// src/widgets/obj_style.rs (or mod.rs)

/// Maximum corner radius — creates a pill/capsule shape.
/// Equivalent to LVGL's `LV_RADIUS_CIRCLE` (0x7FFF).
pub const RADIUS_MAX: i32 = 0x7FFF;
```

### Steps

- [ ] **Step 1:** Add `RADIUS_MAX` constant, export from `mod.rs`.
- [ ] **Step 2:** Replace all `0x7fff` / `0x7FFF` in examples with `RADIUS_MAX`.
- [ ] **Step 3:** Build, test, commit.

---

## Task 3: `style_bg_grad_dir` — accept `GradDir` not `u32`

**Files:**
- Modify: `src/widgets/obj_style.rs:147` — change `dir: u32` to `dir: GradDir`
- Modify: examples calling `style_bg_grad_dir(GradDir::Hor as u32, ...)` → `style_bg_grad_dir(GradDir::Hor, ...)`

### Steps

- [ ] **Step 1:** Change signature to `dir: super::palette::GradDir`. Inside: `dir as lv_grad_dir_t`.
- [ ] **Step 2:** Update examples (remove `as u32` casts).
- [ ] **Step 3:** Build, test, commit.

---

## Task 4: `GridCell` — replace 6-arg `set_grid_cell`

**Files:**
- Create: `src/widgets/grid.rs` — `GridCell` struct
- Modify: `src/widgets/obj_layout.rs` — new `set_grid_cell` signature
- Modify: `src/widgets/mod.rs` — add module, export
- Modify: grid examples (grid1–6)

### Design

```rust
// src/widgets/grid.rs

/// Grid cell placement (alignment + position + span).
#[derive(Clone, Copy, Debug)]
pub struct GridCell {
    pub align: super::obj::GridAlign,
    pub pos: i32,
    pub span: i32,
}

impl GridCell {
    pub fn new(align: super::obj::GridAlign, pos: i32, span: i32) -> Self {
        Self { align, pos, span }
    }

    /// Single-cell at given position with default (Start) alignment.
    pub fn at(pos: i32) -> Self {
        Self { align: super::obj::GridAlign::Start, pos, span: 1 }
    }
}
```

Change `set_grid_cell` to:
```rust
pub fn set_grid_cell(&self, col: GridCell, row: GridCell) -> &Self
```

Example migration:
```rust
// Before:
obj.set_grid_cell(GridAlign::Stretch, 0, 1, GridAlign::Center, 0, 1);
// After:
obj.set_grid_cell(
    GridCell::new(GridAlign::Stretch, 0, 1),
    GridCell::new(GridAlign::Center, 0, 1),
);
// Or simpler cases:
obj.set_grid_cell(GridCell::at(0), GridCell::at(0));
```

### Steps

- [ ] **Step 1:** Create `src/widgets/grid.rs` with `GridCell` struct, `new()`, `at()`, and tests:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::widgets::obj::GridAlign;

      #[test]
      fn at_defaults_to_start_span1() {
          let c = GridCell::at(2);
          assert_eq!(c.pos, 2);
          assert_eq!(c.span, 1);
      }
  }
  ```

- [ ] **Step 2:** Register module, export `GridCell` from `mod.rs`.

- [ ] **Step 3:** Change `set_grid_cell` in `obj_layout.rs` to accept `col: GridCell, row: GridCell`. Destructure inside.

- [ ] **Step 4:** Update grid examples (grid1–6). Each `set_grid_cell` call changes from 6 positional args to 2 `GridCell` values.

- [ ] **Step 5:** Build, test, commit.

---

## Task 5: `ScaleBuilder` — replace 13-arg `tick_ring`

**Files:**
- Modify: `src/widgets/scale.rs` — add `ScaleBuilder` struct, keep old method (deprecated)
- Modify: examples using `tick_ring` (likely scale examples if any, or just internal usage)

### Design

```rust
/// Builder for Scale::tick_ring configuration.
pub struct ScaleBuilder {
    pub size: i32,
    pub mode: ScaleMode,
    pub rotation: i32,
    pub sweep: i32,
    pub range_max: i32,
    pub total_ticks: u32,
    pub major_every: u32,
    pub show_labels: bool,
    pub major_len: i32,
    pub minor_len: i32,
    pub major_color: u32,
    pub minor_color: u32,
}

impl ScaleBuilder {
    /// Start with required fields, sensible defaults for the rest.
    pub fn new(size: i32, mode: ScaleMode) -> Self {
        Self {
            size, mode,
            rotation: 0,
            sweep: 360,
            range_max: 100,
            total_ticks: 11,
            major_every: 5,
            show_labels: true,
            major_len: 10,
            minor_len: 5,
            major_color: 0x000000,
            minor_color: 0x808080,
        }
    }

    pub fn rotation(mut self, v: i32) -> Self { self.rotation = v; self }
    pub fn sweep(mut self, v: i32) -> Self { self.sweep = v; self }
    pub fn range_max(mut self, v: i32) -> Self { self.range_max = v; self }
    pub fn total_ticks(mut self, v: u32) -> Self { self.total_ticks = v; self }
    pub fn major_every(mut self, v: u32) -> Self { self.major_every = v; self }
    pub fn show_labels(mut self, v: bool) -> Self { self.show_labels = v; self }
    pub fn major_len(mut self, v: i32) -> Self { self.major_len = v; self }
    pub fn minor_len(mut self, v: i32) -> Self { self.minor_len = v; self }
    pub fn major_color(mut self, v: u32) -> Self { self.major_color = v; self }
    pub fn minor_color(mut self, v: u32) -> Self { self.minor_color = v; self }

    /// Build the scale widget.
    pub fn build(self, parent: &impl AsLvHandle) -> Result<Scale, WidgetError> {
        Scale::tick_ring(
            parent, self.size, self.mode, self.rotation, self.sweep,
            self.range_max, self.total_ticks, self.major_every,
            self.show_labels, self.major_len, self.minor_len,
            self.major_color, self.minor_color,
        )
    }
}
```

Keep `tick_ring` as-is (internal/power-user). Export `ScaleBuilder` for the ergonomic path.

### Steps

- [ ] **Step 1:** Add `ScaleBuilder` struct + methods to `scale.rs`.
- [ ] **Step 2:** Export from `mod.rs`.
- [ ] **Step 3:** Update any examples using `tick_ring` directly to use builder instead.
- [ ] **Step 4:** Build, test, commit.

---

## Task 6: Complete the prelude

**Files:**
- Modify: `src/widgets/prelude.rs`
- Modify: `src/widgets/mod.rs` (ensure all types are exported)

### Design

Add commonly-used types that examples currently import individually:

```rust
pub use super::{
    // Widgets
    Align, Arc, Bar, Button, Label, Led, Obj, Scale, Screen, Slider, Switch, ValueLabel,
    // Styling
    Style, ColorFilter, GradDsc, GradDir, GradExtend, Palette, Opa,
    BorderSide, TextDecor,
    // Layout
    FlexAlign, FlexFlow, GridAlign, GridCell, Layout,
    // Events
    Event, EventCode,
    // Enums / flags
    ObjFlag, ObjState, Selector,
    // Traits
    AsLvHandle,
    // Constants & helpers
    LV_SIZE_CONTENT, RADIUS_MAX, GRID_TEMPLATE_LAST, grid_fr, lv_pct,
    // Animation
    Anim, ANIM_REPEAT_INFINITE,
    // Error
    WidgetError,
    // Palette helpers
    color_black, color_make, color_white, palette_darken, palette_lighten, palette_main,
};
```

### Steps

- [ ] **Step 1:** Update `prelude.rs` with all commonly-used exports (after Tasks 1–5 land).
- [ ] **Step 2:** Pick 2–3 examples, convert to `use oxivgl::widgets::prelude::*` to verify coverage.
- [ ] **Step 3:** Build, test, commit.

---

## Execution order

```
Task 1 (Selector)  ──┐
Task 2 (RADIUS_MAX)──┤
Task 3 (GradDir)   ──┼── all independent, can run in parallel
Task 4 (GridCell)  ──┤
Task 5 (ScaleBuilder)┘
                      │
                      v
              Task 6 (Prelude) ── depends on new types from 1–5
```

## Unresolved

- Should `Selector` support `From<u32>` for escape-hatch compatibility, or force explicit `Selector(raw)` construction? Recommend: no `From<u32>`, keep `Selector(raw)` as explicit escape hatch via pub field.
- Should `ScaleBuilder` colors use `lv_color_t` instead of `u32` hex? Current `tick_ring` uses hex — keep consistent for now, consider `Palette`/`lv_color_t` later.
- `major_color`/`minor_color` in Scale: should these be `lv_color_t`? The current impl takes `u32` and calls `lv_color_hex`. Keep `u32` for builder consistency with existing code.
