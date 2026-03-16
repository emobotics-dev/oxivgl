# Style Lifetime Safety Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement memory/lifetime spec from `docs/spec-memory-lifetime.md` — two-phase style system, Rc ownership, widget style tracking, Anim lifetime.

**Architecture:** Split `Style` into `StyleBuilder` (mutable build) + `Style` (frozen `Rc<StyleInner>`). Sub-descriptors (GradDsc, TransitionDsc, ColorFilter) moved into `StyleInner` by value. Widgets track applied styles via `RefCell<Vec<Style>>`.

**Tech Stack:** `alloc::rc::Rc`, `core::cell::RefCell`, `alloc::vec::Vec`

---

## Chunk 1: Core Types

### Task 1: StyleInner + StyleBuilder + Style refactor

**Files:**
- Modify: `src/widgets/style.rs`

- [ ] 1.1 Add `StyleInner` struct (`#[repr(C)]`, `lv` at offset 0, sub-descriptor `Option<Box<T>>` fields)
- [ ] 1.2 Impl `Drop for StyleInner` — call `lv_style_reset`
- [ ] 1.3 Add `const _: () = assert!(core::mem::offset_of!(StyleInner, lv) == 0);`
- [ ] 1.4 Rename current `Style` to `StyleBuilder` — owns `Box<StyleInner>`, all setter methods move to it
- [ ] 1.5 Change setter signatures: `&mut self.inner` → `&mut self.inner.lv`
- [ ] 1.6 New `Style` struct: wraps `Rc<StyleInner>`, derives `Clone`, no setters
- [ ] 1.7 `StyleBuilder::build(self) -> Style` — converts `Box<StyleInner>` to `Rc<StyleInner>`
- [ ] 1.8 Remove `Style::boxed()` (spec: SHALL NOT exist)
- [ ] 1.9 Sub-descriptor setters on `StyleBuilder` take by value (move):
  - `bg_grad(self, grad: GradDsc) -> Self` — stores in `_grad`, calls `lv_style_set_bg_grad`
  - `transition(self, tr: TransitionDsc) -> Self` — stores in `_transition`, calls `lv_style_set_transition`
  - `color_filter(self, filter: ColorFilter, opa: u8) -> Self` — stores in `_color_filter`
- [ ] 1.10 `Style::lv_ptr(&self) -> *const lv_style_t` — `Rc::as_ptr(&self.inner) as *const _`
- [ ] 1.11 Remove old `Drop for Style`
- [ ] 1.12 Update existing tests

### Task 2: GradDsc cleanup

**Files:**
- Modify: `src/widgets/grad.rs`

- [ ] 2.1 Remove `GradDsc::boxed()`
- [ ] 2.2 Keep `GradDsc::new()` and builder methods unchanged

### Task 3: Obj style tracking

**Files:**
- Modify: `src/widgets/obj.rs`
- Modify: `src/widgets/obj_style.rs`

- [ ] 3.1 Add `_styles: RefCell<Vec<Style>>` field to `Obj<'p>`
- [ ] 3.2 Update `Obj::from_raw` and `Obj::new` to initialize `_styles`
- [ ] 3.3 Update `Obj::drop` — add SAFETY comment referencing LVGL v9.3 `lv_obj.c:521,525`
- [ ] 3.4 `add_style(&self, style: &Style, selector)` — push clone to `_styles` before LVGL call
- [ ] 3.5 `remove_style` — call LVGL first, then remove one entry by Rc pointer identity
- [ ] 3.6 `remove_style_all` — call LVGL, then clear `_styles`

### Task 4: Theme adaptation

**Files:**
- Modify: `src/widgets/theme.rs`

- [ ] 4.1 Change `_style: Box<Style>` → `_style: Style`
- [ ] 4.2 `extend_current(style: Style)` — use `style.lv_ptr()` for `user_data`
- [ ] 4.3 Update `apply_cb_trampoline` accordingly

### Task 5: Anim lifetime

**Files:**
- Modify: `src/widgets/anim.rs`

- [ ] 5.1 Add `'w` lifetime: `Anim<'w>` with `PhantomData<&'w ()>`
- [ ] 5.2 `set_var` takes `&'w impl AsLvHandle`
- [ ] 5.3 Update `Anim::new()` return type

### Task 6: Exports

**Files:**
- Modify: `src/widgets/mod.rs`

- [ ] 6.1 Export `StyleBuilder` alongside `Style`
- [ ] 6.2 Verify all public API items have doc comments

## Chunk 2: Example Migration

### Task 7: Migrate all examples

Pattern: `Box::new(Style::new())` + setters + `Box::new(GradDsc::new())` → `StyleBuilder::new()` + setters + `.bg_grad(grad)` + `.build()`.

View structs: `_style: Box<Style>` → `_style: Style`. Remove `_grad`, `_trans`, `_color_filter` fields (now owned by Style).

- [ ] 7.1–7.N: Migrate each example (≈30 files). Commit per logical group.

### Task 8: Build + test

- [ ] 8.1 `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`
- [ ] 8.2 `./run_tests.sh unit`
- [ ] 8.3 `./run_tests.sh int`
- [ ] 8.4 Audit doc coverage
- [ ] 8.5 `./run_screenshots.sh` — compare against existing PNGs
