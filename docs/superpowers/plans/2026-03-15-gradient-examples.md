# Gradient Examples Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `GradDsc::horizontal()` and `GradDsc::radial_set_focal()` wrappers, then port LVGL gradient examples 1–4 as static `getting_started5–8.rs`.

**Architecture:** Two new methods on `GradDsc` in `src/widgets/grad.rs`. Four example files each store `Box<Style>` + `Box<GradDsc>` + `Obj<'static>` as struct fields to satisfy LVGL lifetime requirements. No interactive elements.

**Tech Stack:** Rust (nightly), LVGL 9.3 via `lvgl_rust_sys`, host target `x86_64-unknown-linux-gnu` for build checks.

**Spec:** `docs/superpowers/specs/2026-03-15-gradient-examples-design.md`

---

## Chunk 1: GradDsc wrapper methods

### Task 1: Add `GradDsc::horizontal()` and `GradDsc::radial_set_focal()`

**Files:**
- Modify: `src/widgets/grad.rs`

- [ ] **Add two methods** after the `conical()` method (line 110):

```rust
    /// Configure as a simple horizontal gradient (left-to-right).
    pub fn horizontal(&mut self) -> &mut Self {
        unsafe { lv_grad_horizontal_init(&mut self.inner) };
        self
    }

    /// Set the focal point of a radial gradient.
    ///
    /// Call after [`radial`](Self::radial). `fx`/`fy` are the focal center
    /// coords; `fr` is the focal circle radius.
    pub fn radial_set_focal(&mut self, fx: i32, fy: i32, fr: i32) -> &mut Self {
        unsafe { lv_grad_radial_set_focal(&mut self.inner, fx, fy, fr) };
        self
    }
```

- [ ] **Build check:**

```sh
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu
```

Expected: no errors.

- [ ] **Commit:**

```sh
git add src/widgets/grad.rs
git commit -m "feat: add GradDsc::horizontal() and radial_set_focal()"
```

---

## Chunk 2: Example files

All four examples share this pattern:
- Struct fields declared `_obj, _style, _grad` in that order (drop order: obj first, style second, grad last — correct since obj releases style ref before style resets before grad frees)
- `Box::new(Style::new())` and `Box::new(GradDsc::new())` — matches existing example convention
- Style always includes `.bg_opa(255)` — plain `Obj` defaults to `bg_opa = 0` (transparent); without this the gradient is invisible

### Task 2: `getting_started5.rs` — horizontal gradient

**Files:**
- Create: `examples/getting_started5.rs`

- [ ] **Create the file:**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 5 — Simple Horizontal Gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{color_make, lv_pct, GradDsc, Obj, Screen, Selector, Style, WidgetError},
};

struct GettingStarted5 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}

impl View for GettingStarted5 {
    fn create() -> Result<Self, WidgetError> {
        let colors = [color_make(0xff, 0, 0), color_make(0, 0xff, 0)];
        let opas = [255u8, 0];
        let fracs = [20u8 * 255 / 100, 80 * 255 / 100];

        let mut grad = Box::new(GradDsc::new());
        grad.init_stops(&colors, &opas, &fracs).horizontal();

        let mut style = Box::new(Style::new());
        style
            .bg_opa(255)
            .bg_grad(&grad)
            .border_width(2)
            .radius(12)
            .pad_all(0);

        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let obj = Obj::new(&screen)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

        Ok(Self {
            _obj: obj,
            _style: style,
            _grad: grad,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted5);
```

- [ ] **Build check:**

```sh
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --example getting_started5 --target x86_64-unknown-linux-gnu
```

Expected: no errors.

- [ ] **Commit:**

```sh
git add examples/getting_started5.rs
git commit -m "feat: add getting_started5 — horizontal gradient"
```

---

### Task 3: `getting_started6.rs` — linear (skew) gradient

**Files:**
- Create: `examples/getting_started6.rs`

- [ ] **Create the file:**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 6 — Linear (Skew) Gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        color_make, lv_pct, GradDsc, GradExtend, Obj, Screen, Selector, Style, WidgetError,
    },
};

struct GettingStarted6 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}

impl View for GettingStarted6 {
    fn create() -> Result<Self, WidgetError> {
        let colors = [color_make(0xff, 0, 0), color_make(0, 0xff, 0)];
        let opas = [255u8, 0];

        let mut grad = Box::new(GradDsc::new());
        grad.init_stops(&colors, &opas, &[])
            .linear(100, 100, 200, 150, GradExtend::Pad);

        let mut style = Box::new(Style::new());
        style
            .bg_opa(255)
            .bg_grad(&grad)
            .border_width(2)
            .radius(12)
            .pad_all(0);

        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let obj = Obj::new(&screen)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

        Ok(Self {
            _obj: obj,
            _style: style,
            _grad: grad,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted6);
```

- [ ] **Build check:**

```sh
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --example getting_started6 --target x86_64-unknown-linux-gnu
```

- [ ] **Commit:**

```sh
git add examples/getting_started6.rs
git commit -m "feat: add getting_started6 — linear skew gradient"
```

---

### Task 4: `getting_started7.rs` — radial gradient

**Files:**
- Create: `examples/getting_started7.rs`

- [ ] **Create the file:**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 7 — Radial Gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        color_make, lv_pct, GradDsc, GradExtend, Obj, Screen, Selector, Style, WidgetError,
    },
};

struct GettingStarted7 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}

impl View for GettingStarted7 {
    fn create() -> Result<Self, WidgetError> {
        let colors = [color_make(0xff, 0, 0), color_make(0, 0xff, 0)];
        let opas = [255u8, 0];

        let mut grad = Box::new(GradDsc::new());
        grad.init_stops(&colors, &opas, &[])
            .radial(100, 100, 200, 100, GradExtend::Pad)
            .radial_set_focal(50, 50, 10);

        let mut style = Box::new(Style::new());
        style
            .bg_opa(255)
            .bg_grad(&grad)
            .border_width(2)
            .radius(12)
            .pad_all(0);

        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let obj = Obj::new(&screen)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

        Ok(Self {
            _obj: obj,
            _style: style,
            _grad: grad,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted7);
```

- [ ] **Build check:**

```sh
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --example getting_started7 --target x86_64-unknown-linux-gnu
```

- [ ] **Commit:**

```sh
git add examples/getting_started7.rs
git commit -m "feat: add getting_started7 — radial gradient"
```

---

### Task 5: `getting_started8.rs` — conical gradient

**Files:**
- Create: `examples/getting_started8.rs`

- [ ] **Create the file:**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Getting Started 8 — Conical Gradient

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        color_make, lv_pct, GradDsc, GradExtend, Obj, Screen, Selector, Style, WidgetError,
    },
};

struct GettingStarted8 {
    _obj: Obj<'static>,
    _style: Box<Style>,
    _grad: Box<GradDsc>,
}

impl View for GettingStarted8 {
    fn create() -> Result<Self, WidgetError> {
        let colors = [color_make(0xff, 0, 0), color_make(0, 0xff, 0)];
        let opas = [255u8, 0];

        let mut grad = Box::new(GradDsc::new());
        grad.init_stops(&colors, &opas, &[])
            .conical(lv_pct(50), lv_pct(50), 0, 180, GradExtend::Pad);

        let mut style = Box::new(Style::new());
        style
            .bg_opa(255)
            .bg_grad(&grad)
            .border_width(2)
            .radius(12)
            .pad_all(0);

        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let obj = Obj::new(&screen)?;
        obj.size(lv_pct(80), lv_pct(80)).center();
        obj.add_style(&style, Selector::DEFAULT);

        Ok(Self {
            _obj: obj,
            _style: style,
            _grad: grad,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(GettingStarted8);
```

- [ ] **Build check:**

```sh
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --example getting_started8 --target x86_64-unknown-linux-gnu
```

- [ ] **Commit:**

```sh
git add examples/getting_started8.rs
git commit -m "feat: add getting_started8 — conical gradient"
```

---

## Chunk 3: Housekeeping

### Task 6: Screenshots and README

**Files:**
- Modify: `run_screenshots.sh`
- Modify: `examples/doc/README.md`

- [ ] **Update `run_screenshots.sh`** — change `getting_started{1,2,3,4}` to `getting_started{1,2,3,4,5,6,7,8}` on line 4.

- [ ] **Run screenshots:**

```sh
./run_screenshots.sh
```

Expected: PNGs appear in `examples/doc/screenshots/getting_started{5,6,7,8}.png`. Visually compare against LVGL docs gradient section to verify correct rendering.

- [ ] **Add entries to `examples/doc/README.md`** after the Example 4 block:

```markdown
### Example 5 — Simple Horizontal Gradient

Container with a horizontal red→green gradient (opacity 100%→0%), stops at 20% and 80%.

![getting_started5](screenshots/getting_started5.png)

### Example 6 — Linear (Skew) Gradient

Container with a skewed linear gradient from (100,100) to (200,150).

![getting_started6](screenshots/getting_started6.png)

### Example 7 — Radial Gradient

Container with a radial gradient centered at (100,100), focal point at (50,50).

![getting_started7](screenshots/getting_started7.png)

### Example 8 — Conical Gradient

Container with a conical gradient sweeping 0°–180° from center.

![getting_started8](screenshots/getting_started8.png)
```

- [ ] **Commit:**

```sh
git add run_screenshots.sh examples/doc/README.md examples/doc/screenshots/getting_started5.png examples/doc/screenshots/getting_started6.png examples/doc/screenshots/getting_started7.png examples/doc/screenshots/getting_started8.png
git commit -m "chore: add getting_started5-8 screenshots and README entries"
```
