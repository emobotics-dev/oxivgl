# LVGL Getting Started Examples — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add missing widget API surface and implement LVGL "Getting Started" examples 1–3 as host-runnable binaries.

**Architecture:** Extend `src/widgets/` with `Style`, `ColorFilter`, `Palette`/`GradDir` types and add missing `Obj`/`Screen` methods; add a new `examples/get-started` workspace crate with three host binaries (`ex1`, `ex2`, `ex3`) that run on x86_64 via the SDL2 backend.

**Tech Stack:** Rust (esp toolchain), lvgl_rust_sys FFI, heapless, SDL2 (host only)

---

## Background

Three LVGL "Getting Started" examples, adapted from C to Rust:

**ex1 — Hello World:**
```c
lv_obj_set_style_bg_color(lv_screen_active(), lv_color_hex(0x003a57), LV_PART_MAIN);
lv_obj_t *label = lv_label_create(lv_screen_active());
lv_label_set_text(label, "Hello world");
lv_obj_set_style_text_color(lv_screen_active(), lv_color_hex(0xffffff), LV_PART_MAIN);
lv_obj_align(label, LV_ALIGN_CENTER, 0, 0);
```

**ex2 — Button + click counter:**
```c
// event callback increments counter, updates child label text
lv_obj_t *btn = lv_button_create(lv_screen_active());
lv_obj_set_pos(btn, 10, 10);
lv_obj_set_size(btn, 120, 50);
lv_obj_add_event_cb(btn, btn_event_cb, LV_EVENT_ALL, NULL);
lv_obj_t *label = lv_label_create(btn); lv_label_set_text(label, "Button");
lv_obj_center(label);
```

**ex3 — Custom button styles:**
```c
// lv_style_t with gradient, border, radius, color-filter-on-press, two buttons
```

---

## Task 1: Add `Obj::pos()`, `Obj::center()`, `Screen::text_color()`

**Files:**
- Modify: `src/widgets/obj.rs`

**Step 1: Add methods to `Obj<'p>` impl block** (after the `opa` method):

```rust
pub fn pos(&self, x: i32, y: i32) -> &Self {
    assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
    // SAFETY: handle non-null (asserted above).
    unsafe { lv_obj_set_pos(self.handle, x, y) };
    self
}

pub fn center(&self) -> &Self {
    assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
    // SAFETY: handle non-null (asserted above).
    unsafe { lv_obj_center(self.handle) };
    self
}
```

**Step 2: Add `text_color` to `Screen` impl block** (after `pad_right`):

```rust
pub fn text_color(&self, color: u32) -> &Self {
    // SAFETY: handle non-null (Screen::active() returns None for null).
    unsafe { lv_obj_set_style_text_color(self.handle, lv_color_hex(color), 0) };
    self
}
```

**Step 3: Build check**

```sh
cargo check --target x86_64-unknown-linux-gnu
```
Expected: no errors.

**Step 4: Commit**
```sh
git add src/widgets/obj.rs
git commit -m "feat(widgets): add Obj::pos, Obj::center, Screen::text_color"
```

---

## Task 2: Add `Obj::get_child()` and `Obj::on_event()`

**Files:**
- Modify: `src/widgets/obj.rs`

**Step 1: Add imports at top of `obj.rs`** (after existing `use` lines):

```rust
use core::ffi::c_void;
```

**Step 2: Add methods to `Obj<'p>` impl block:**

```rust
/// Get child widget by index (0-based). Returns `None` if index out of range.
/// The returned `Child` does NOT own the pointer — LVGL frees it when the parent is deleted.
pub fn get_child(&self, idx: i32) -> Option<super::Child<Obj<'_>>> {
    assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
    // SAFETY: handle non-null (asserted above); LVGL returns NULL for out-of-range idx.
    let child_ptr = unsafe { lv_obj_get_child(self.handle, idx) };
    if child_ptr.is_null() {
        None
    } else {
        Some(super::Child::new(Obj::from_raw(child_ptr)))
    }
}

/// Add an event callback. `cb` is an `extern "C"` function pointer.
/// `filter`: use `lv_event_code_t_LV_EVENT_ALL` to receive all events,
/// or a specific code like `lv_event_code_t_LV_EVENT_CLICKED`.
/// `user_data`: arbitrary pointer passed to the callback; pass `core::ptr::null_mut()` if unused.
pub fn on_event(
    &self,
    cb: unsafe extern "C" fn(*mut lv_event_t),
    filter: lv_event_code_t,
    user_data: *mut c_void,
) -> &Self {
    assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
    // SAFETY: handle non-null; cb is a valid extern "C" fn pointer.
    unsafe { lv_obj_add_event_cb(self.handle, Some(cb), filter, user_data) };
    self
}
```

**Step 3: Build check**
```sh
cargo check --target x86_64-unknown-linux-gnu
```

**Step 4: Commit**
```sh
git add src/widgets/obj.rs
git commit -m "feat(widgets): add Obj::get_child, Obj::on_event"
```

---

## Task 3: Add `Palette` and `GradDir` types

**Files:**
- Create: `src/widgets/palette.rs`
- Modify: `src/widgets/mod.rs`

**Step 1: Write unit tests first** (at bottom of new `palette.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::{GradDir, Palette};

    #[test]
    fn palette_discriminants() {
        assert_eq!(Palette::Red as u32, 0);
        assert_eq!(Palette::Grey as u32, 18);
    }

    #[test]
    fn grad_dir_discriminants() {
        assert_eq!(GradDir::None as u32, 0);
        assert_eq!(GradDir::Ver as u32, 1);
        assert_eq!(GradDir::Hor as u32, 2);
    }
}
```

**Step 2: Run tests — they fail (file doesn't exist yet)**
```sh
cargo test --target x86_64-unknown-linux-gnu 2>&1 | grep "error\|FAILED\|palette"
```

**Step 3: Create `src/widgets/palette.rs`:**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
use lvgl_rust_sys::*;

/// LVGL material design color palette (`lv_palette_t`).
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Palette {
    Red = 0,
    Pink = 1,
    Purple = 2,
    DeepPurple = 3,
    Indigo = 4,
    Blue = 5,
    LightBlue = 6,
    Cyan = 7,
    Teal = 8,
    Green = 9,
    LightGreen = 10,
    Lime = 11,
    Yellow = 12,
    Amber = 13,
    Orange = 14,
    DeepOrange = 15,
    Brown = 16,
    BlueGrey = 17,
    Grey = 18,
}

/// Gradient direction (`lv_grad_dir_t`).
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum GradDir {
    None = 0,
    Ver = 1,
    Hor = 2,
}

/// Returns the main (500-shade) color for a palette entry as a raw `lv_color_t`.
pub fn palette_main(p: Palette) -> lv_color_t {
    // SAFETY: lv_palette_main is a pure lookup function.
    unsafe { lv_palette_main(p as lv_palette_t) }
}

/// Returns a lightened shade of a palette color.
/// `level` is 1–5 (1 = lightest, 5 = darkest light variant).
pub fn palette_lighten(p: Palette, level: u8) -> lv_color_t {
    // SAFETY: pure lookup.
    unsafe { lv_palette_lighten(p as lv_palette_t, level as i16) }
}

#[cfg(test)]
mod tests {
    use super::{GradDir, Palette};

    #[test]
    fn palette_discriminants() {
        assert_eq!(Palette::Red as u32, 0);
        assert_eq!(Palette::Grey as u32, 18);
    }

    #[test]
    fn grad_dir_discriminants() {
        assert_eq!(GradDir::None as u32, 0);
        assert_eq!(GradDir::Ver as u32, 1);
        assert_eq!(GradDir::Hor as u32, 2);
    }
}
```

**Step 4: Add to `src/widgets/mod.rs`** (after `mod scale;`):

```rust
mod palette;
pub use palette::{palette_lighten, palette_main, GradDir, Palette};
```

**Step 5: Run tests**
```sh
cargo test --target x86_64-unknown-linux-gnu palette
```
Expected: 2 tests pass.

**Step 6: Commit**
```sh
git add src/widgets/palette.rs src/widgets/mod.rs
git commit -m "feat(widgets): add Palette, GradDir enums and palette_main/lighten helpers"
```

---

## Task 4: Add `Style` and `ColorFilter` structs

**Files:**
- Create: `src/widgets/style.rs`
- Modify: `src/widgets/mod.rs`

**Step 1: Create `src/widgets/style.rs`:**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
use lvgl_rust_sys::*;

use super::palette::{GradDir, palette_lighten, palette_main};

/// Owned LVGL style. Wraps `lv_style_t`.
///
/// # Lifetime contract
/// Once passed to [`Obj::add_style`], this struct MUST NOT be moved or dropped
/// while any widget holds a reference to it.  In practice: store styles as
/// fields in a `View` struct (which is itself kept alive for the LVGL lifetime).
pub struct Style {
    pub(crate) inner: lv_style_t,
}

impl Style {
    pub fn new() -> Self {
        // SAFETY: lv_style_t can be zero-initialized; lv_style_init sets it up correctly.
        let mut inner = unsafe { core::mem::zeroed::<lv_style_t>() };
        unsafe { lv_style_init(&mut inner) };
        Self { inner }
    }

    pub fn radius(&mut self, r: i32) -> &mut Self {
        unsafe { lv_style_set_radius(&mut self.inner, r) };
        self
    }

    pub fn bg_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_bg_opa(&mut self.inner, opa as lv_opa_t) };
        self
    }

    pub fn bg_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_color(&mut self.inner, color) };
        self
    }

    pub fn bg_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.bg_color(color)
    }

    pub fn bg_grad_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_grad_color(&mut self.inner, color) };
        self
    }

    pub fn bg_grad_dir(&mut self, dir: GradDir) -> &mut Self {
        unsafe { lv_style_set_bg_grad_dir(&mut self.inner, dir as lv_grad_dir_t) };
        self
    }

    pub fn border_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_border_color(&mut self.inner, color) };
        self
    }

    pub fn border_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.border_color(color)
    }

    pub fn border_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_border_opa(&mut self.inner, opa as lv_opa_t) };
        self
    }

    pub fn border_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_border_width(&mut self.inner, w) };
        self
    }

    pub fn text_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_text_color(&mut self.inner, color) };
        self
    }

    pub fn text_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.text_color(color)
    }

    pub fn color_filter(&mut self, filter: &ColorFilter, opa: u8) -> &mut Self {
        unsafe {
            lv_style_set_color_filter_dsc(&mut self.inner, &filter.inner);
            lv_style_set_color_filter_opa(&mut self.inner, opa as lv_opa_t);
        }
        self
    }
}

impl Drop for Style {
    fn drop(&mut self) {
        // SAFETY: inner was initialized by lv_style_init; safe to reset.
        unsafe { lv_style_reset(&mut self.inner) };
    }
}

/// Wraps `lv_color_filter_dsc_t` with a callback function pointer.
///
/// Common callback: `darken` — use `oxivgl::widgets::darken_filter_cb` for
/// the standard "darken on press" effect.
pub struct ColorFilter {
    pub(crate) inner: lv_color_filter_dsc_t,
}

impl ColorFilter {
    pub fn new(cb: unsafe extern "C" fn(*const lv_color_filter_dsc_t, lv_color_t, lv_opa_t) -> lv_color_t) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_color_filter_dsc_t>() };
        unsafe { lv_color_filter_dsc_init(&mut inner, Some(cb)) };
        Self { inner }
    }
}

/// Standard "darken" color filter callback — pass to [`ColorFilter::new`].
///
/// Darkens the widget color by `opa` when used as a pressed-state filter.
pub unsafe extern "C" fn darken_filter_cb(
    _dsc: *const lv_color_filter_dsc_t,
    color: lv_color_t,
    opa: lv_opa_t,
) -> lv_color_t {
    // SAFETY: lv_color_darken is a pure color computation.
    unsafe { lv_color_darken(color, opa) }
}
```

**Step 2: Add to `src/widgets/mod.rs`** (after `mod palette;`):

```rust
mod style;
pub use style::{darken_filter_cb, ColorFilter, Style};
```

**Step 3: Build check**
```sh
cargo check --target x86_64-unknown-linux-gnu
```
Expected: no errors. If `lv_style_reset` is not available (check bindings), replace with a comment and omit the Drop impl.

**Step 4: Commit**
```sh
git add src/widgets/style.rs src/widgets/mod.rs
git commit -m "feat(widgets): add Style, ColorFilter, darken_filter_cb"
```

---

## Task 5: Add `Obj::add_style()` and `Obj::remove_style_all()`; fix host `DISPLAY_READY`

**Files:**
- Modify: `src/widgets/obj.rs`
- Modify: `src/lvgl_buffers.rs`

**Step 1: Add to `Obj<'p>` impl block in `obj.rs`:**

```rust
/// Apply a style to this object for the given selector.
/// Pass `0` for default state, `lv_state_t_LV_STATE_PRESSED` (= 128) for pressed.
///
/// The `style` must outlive this object (see [`Style`](super::Style) docs).
pub fn add_style(&self, style: &super::Style, selector: u32) -> &Self {
    assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
    // SAFETY: handle non-null; style.inner pointer valid for style's lifetime.
    unsafe {
        lv_obj_add_style(
            self.handle,
            &style.inner as *const lv_style_t as *mut lv_style_t,
            selector,
        )
    };
    self
}

/// Remove all styles from this object.
pub fn remove_style_all(&self) -> &Self {
    assert_ne!(self.handle, null_mut(), "Obj handle cannot be null");
    // SAFETY: handle non-null.
    unsafe { lv_obj_remove_style_all(self.handle) };
    self
}
```

**Step 2: Fix `DISPLAY_READY` for host in `src/lvgl_buffers.rs`.**
At the end of `lvgl_disp_init`, add (after the `lv_display_set_flush_wait_cb` block):

```rust
    // On non-esp-hal targets the flush task never runs; signal ready immediately.
    #[cfg(not(feature = "esp-hal"))]
    DISPLAY_READY.signal(());
```

**Step 3: Build check**
```sh
cargo check --target x86_64-unknown-linux-gnu
```

**Step 4: Commit**
```sh
git add src/widgets/obj.rs src/lvgl_buffers.rs
git commit -m "feat(widgets): add Obj::add_style, remove_style_all; fix host DISPLAY_READY"
```

---

## Task 6: Create `examples/get-started` crate

**Files:**
- Create: `examples/get-started/Cargo.toml`
- Create: `examples/get-started/src/lib.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Add to workspace in root `Cargo.toml`:**
```toml
members = ["examples/fire27", "examples/get-started"]
```

**Step 2: Create `examples/get-started/Cargo.toml`:**

```toml
# SPDX-License-Identifier: MIT OR Apache-2.0
[package]
name = "oxivgl-examples-get-started"
version = "0.1.0"
edition = "2024"
publish = false

[[bin]]
name = "ex1"
path = "src/bin/ex1.rs"

[[bin]]
name = "ex2"
path = "src/bin/ex2.rs"

[[bin]]
name = "ex3"
path = "src/bin/ex3.rs"

[dependencies]
oxivgl = { path = "../..", features = ["log-04"] }
lvgl_rust_sys = { git = "https://github.com/emobotics-dev/lvgl_rust_sys.git", rev = "5cb45972e2ea0f5bcc009e64652f0734f8fd57a1", default-features = false }
embassy-executor = { version = "0.9.1", features = ["nightly", "thread"] }
embassy-time = "0.5.0"
log = "0.4"
env_logger = "0.11"
static_cell = { version = "2.1.1", features = ["nightly"] }
```

**Step 3: Create shared host runner in `examples/get-started/src/lib.rs`:**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
/// Run `setup_fn` once, then loop driving `lv_timer_handler` every 5ms.
/// For host (SDL2) only — does not use async or embassy.
pub fn run_host<F: FnOnce()>(setup_fn: F) {
    use lvgl_rust_sys::lv_timer_handler;

    setup_fn();

    loop {
        // SAFETY: lv_init() was called inside LvglDriver::init() in setup_fn.
        unsafe { lv_timer_handler() };
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
```

**Step 4: Build check (crate exists, no bins yet)**
```sh
cargo check -p oxivgl-examples-get-started --target x86_64-unknown-linux-gnu 2>&1 | grep "error" | head -10
```

**Step 5: Commit**
```sh
git add examples/get-started/ Cargo.toml
git commit -m "feat(examples): scaffold get-started example crate"
```

---

## Task 7: Implement Example 1 — Hello World

**Files:**
- Create: `examples/get-started/src/bin/ex1.rs`

**Step 1: Create `examples/get-started/src/bin/ex1.rs`:**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 1: Hello World label.
//!
//! Rust equivalent of lv_example_get_started_1():
//!   - Dark blue screen background
//!   - Centered white "Hello world" label

use oxivgl::{
    lvgl::LvglDriver,
    lvgl_buffers::{LvglBuffers, lvgl_disp_init},
    widgets::{Align, Label, Screen},
};
use static_cell::StaticCell;

// Display resolution for host SDL2 window.
const W: i32 = 320;
const H: i32 = 240;

static BUFS: StaticCell<LvglBuffers<{ (W as usize) * 40 * 2 }>> = StaticCell::new();

fn setup() {
    let _driver = LvglDriver::init(W, H);
    let bufs = BUFS.init(LvglBuffers::new());
    // SAFETY: lv_init() called above; bufs is 'static via StaticCell.
    unsafe { lvgl_disp_init(W, H, bufs) };

    let screen = Screen::active().expect("no active screen");
    screen.bg_color(0x003a57).bg_opa(255);
    screen.text_color(0xffffff);

    let label = Label::new(&screen).expect("label create failed");
    label.text("Hello world\0").expect("label text failed");
    label.align(Align::Center, 0, 0);
}

fn main() {
    env_logger::init();
    oxivgl_examples_get_started::run_host(setup);
}
```

**Step 2: Build**
```sh
cargo build -p oxivgl-examples-get-started --bin ex1 --target x86_64-unknown-linux-gnu
```
Expected: compiles cleanly.

**Step 3: Run (manual verification — SDL2 window with white "Hello world" on dark blue)**
```sh
LIBCLANG_PATH=/usr/lib64 cargo run -p oxivgl-examples-get-started --bin ex1 --target x86_64-unknown-linux-gnu
```

**Step 4: Commit**
```sh
git add examples/get-started/src/bin/ex1.rs
git commit -m "feat(examples): ex1 — Hello World label"
```

---

## Task 8: Implement Example 2 — Button + click counter

**Files:**
- Create: `examples/get-started/src/bin/ex2.rs`

**Step 1: Create `examples/get-started/src/bin/ex2.rs`:**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 2: Button with click counter.
//!
//! Rust equivalent of lv_example_get_started_2():
//!   - Button at (10,10), size 120×50
//!   - Click increments a counter displayed in the button label

use core::sync::atomic::{AtomicU8, Ordering};

use lvgl_rust_sys::{lv_event_code_t_LV_EVENT_ALL, lv_event_get_code, lv_event_get_target_obj, lv_event_t, lv_label_set_text, lv_obj_get_child, lv_event_code_t_LV_EVENT_CLICKED};
use oxivgl::{
    lvgl::LvglDriver,
    lvgl_buffers::{LvglBuffers, lvgl_disp_init},
    widgets::{Align, Button, Label, Screen},
};
use static_cell::StaticCell;

const W: i32 = 320;
const H: i32 = 240;

static BUFS: StaticCell<LvglBuffers<{ (W as usize) * 40 * 2 }>> = StaticCell::new();
static CLICK_COUNT: AtomicU8 = AtomicU8::new(0);

/// Event callback: on click, increment counter and update button label.
///
/// # Safety
/// Called by LVGL with a valid `lv_event_t *`. Accesses LVGL objects — must
/// only be called from the LVGL task (single-task constraint).
unsafe extern "C" fn btn_event_cb(e: *mut lv_event_t) {
    // SAFETY: e is non-null (LVGL guarantee).
    let code = unsafe { lv_event_get_code(e) };
    if code != lv_event_code_t_LV_EVENT_CLICKED {
        return;
    }
    let cnt = CLICK_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
    // SAFETY: target_obj is the button; child 0 is the label.
    let btn = unsafe { lv_event_get_target_obj(e) };
    let label_ptr = unsafe { lv_obj_get_child(btn, 0) };
    if label_ptr.is_null() {
        return;
    }
    // Format "Button: N\0" into a fixed stack buffer.
    let mut buf = [0u8; 32];
    let text = format_in_buf(&mut buf, cnt);
    // SAFETY: buf is NUL-terminated and valid for the duration of the call.
    unsafe { lv_label_set_text(label_ptr, text.as_ptr() as *const _) };
}

/// Write "Button: {cnt}\0" into `buf`, return the filled slice.
fn format_in_buf(buf: &mut [u8; 32], cnt: u8) -> &[u8] {
    use core::fmt::Write;
    struct BufWriter<'a> { buf: &'a mut [u8; 32], pos: usize }
    impl Write for BufWriter<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for b in s.bytes() {
                if self.pos < self.buf.len() - 1 {
                    self.buf[self.pos] = b;
                    self.pos += 1;
                }
            }
            Ok(())
        }
    }
    let mut w = BufWriter { buf, pos: 0 };
    let _ = core::fmt::write(&mut w, format_args!("Button: {}", cnt));
    let end = w.pos;
    w.buf[end] = 0; // NUL terminator
    &w.buf[..=end]
}

fn setup() {
    let _driver = LvglDriver::init(W, H);
    let bufs = BUFS.init(LvglBuffers::new());
    unsafe { lvgl_disp_init(W, H, bufs) };

    let screen = Screen::active().expect("no active screen");

    let btn = Button::new(&screen).expect("button create failed");
    btn.pos(10, 10).size(120, 50);
    btn.on_event(btn_event_cb, lv_event_code_t_LV_EVENT_ALL, core::ptr::null_mut());

    let label = Label::new(&btn).expect("label create failed");
    label.text("Button\0").expect("label text failed");
    label.center();
}

fn main() {
    env_logger::init();
    oxivgl_examples_get_started::run_host(setup);
}
```

**Step 2: Build**
```sh
cargo build -p oxivgl-examples-get-started --bin ex2 --target x86_64-unknown-linux-gnu
```

**Step 3: Run (manual — click button, counter increments in label)**
```sh
LIBCLANG_PATH=/usr/lib64 cargo run -p oxivgl-examples-get-started --bin ex2 --target x86_64-unknown-linux-gnu
```

**Step 4: Commit**
```sh
git add examples/get-started/src/bin/ex2.rs
git commit -m "feat(examples): ex2 — button with click counter"
```

---

## Task 9: Implement Example 3 — Custom Button Styles

**Files:**
- Create: `examples/get-started/src/bin/ex3.rs`

**Step 1: Create `examples/get-started/src/bin/ex3.rs`:**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL Getting Started Example 3: Custom button styles.
//!
//! Rust equivalent of lv_example_get_started_3():
//!   - Two styled buttons with gradients, border, radius
//!   - Pressed-state darkening via ColorFilter
//!   - Second button: red palette + circular radius

use lvgl_rust_sys::lv_state_t_LV_STATE_PRESSED;
use oxivgl::{
    lvgl::LvglDriver,
    lvgl_buffers::{LvglBuffers, lvgl_disp_init},
    widgets::{
        Button, ColorFilter, Label, Palette, Screen, Style,
        darken_filter_cb, palette_lighten, palette_main, GradDir,
    },
};
use static_cell::StaticCell;

const W: i32 = 320;
const H: i32 = 240;
const LV_OPA_COVER: u8 = 255;
const LV_OPA_20: u8 = 51;
const LV_RADIUS_CIRCLE: i32 = 0x7fff;

static BUFS: StaticCell<LvglBuffers<{ (W as usize) * 40 * 2 }>> = StaticCell::new();

struct Ex3View {
    style_btn: Style,
    style_pressed: Style,
    style_red: Style,
    _color_filter: ColorFilter,
    // Widgets are owned by LVGL (children of screen) — held via Child or raw.
    // We don't need to keep them alive here since screen outlives this function.
}

impl Ex3View {
    fn create() -> Self {
        let color_filter = ColorFilter::new(darken_filter_cb);

        let mut style_btn = Style::new();
        style_btn
            .radius(10)
            .bg_opa(LV_OPA_COVER)
            .bg_color(palette_lighten(Palette::Grey, 3))
            .bg_grad_color(palette_main(Palette::Grey))
            .bg_grad_dir(GradDir::Ver)
            .border_color_hex(0x000000)
            .border_opa(LV_OPA_20)
            .border_width(2)
            .text_color_hex(0x000000);

        let mut style_pressed = Style::new();
        style_pressed.color_filter(&color_filter, LV_OPA_20);

        let mut style_red = Style::new();
        style_red
            .bg_color(palette_main(Palette::Red))
            .bg_grad_color(palette_lighten(Palette::Red, 3));

        let screen = Screen::active().expect("no active screen");

        // Button 1 — grey gradient
        let btn1 = Button::new(&screen).expect("btn1 create failed");
        btn1.remove_style_all().pos(10, 10).size(120, 50);
        btn1.add_style(&style_btn, 0);
        btn1.add_style(&style_pressed, lv_state_t_LV_STATE_PRESSED as u32);

        let lbl1 = Label::new(&btn1).expect("lbl1 create failed");
        lbl1.text("Button\0").expect("lbl1 text failed");
        lbl1.center();

        // Button 2 — red, circular
        let btn2 = Button::new(&screen).expect("btn2 create failed");
        btn2.remove_style_all().pos(10, 80).size(120, 50);
        btn2.add_style(&style_btn, 0);
        btn2.add_style(&style_red, 0);
        btn2.add_style(&style_pressed, lv_state_t_LV_STATE_PRESSED as u32);
        // lv_obj_set_style_radius(btn2, LV_RADIUS_CIRCLE, 0) — direct call
        // since Obj::radius sets inline style, not reusable style:
        unsafe {
            lvgl_rust_sys::lv_obj_set_style_radius(btn2.lv_handle(), LV_RADIUS_CIRCLE, 0);
        }

        let lbl2 = Label::new(&btn2).expect("lbl2 create failed");
        lbl2.text("Button 2\0").expect("lbl2 text failed");
        lbl2.center();

        Self {
            style_btn,
            style_pressed,
            style_red,
            _color_filter: color_filter,
        }
    }
}

fn main() {
    env_logger::init();

    let _driver = LvglDriver::init(W, H);
    let bufs = BUFS.init_with(|| LvglBuffers::new());
    unsafe { oxivgl::lvgl_buffers::lvgl_disp_init(W, H, bufs) };

    // Styles must outlive the LVGL widgets; keep them in a local that lives
    // for the entire run loop.
    let _view = Ex3View::create();

    loop {
        unsafe { lvgl_rust_sys::lv_timer_handler() };
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
```

> **Note:** In ex3, `_view` holds the styles for the process lifetime. The `run_host` helper can't be used here because styles must be kept alive in the caller's stack frame. The inline loop is used instead.

**Step 2: Build**
```sh
cargo build -p oxivgl-examples-get-started --bin ex3 --target x86_64-unknown-linux-gnu
```
If `lv_obj_set_style_radius` is not the right name, check with:
```sh
grep "lv_obj_set_style_radius\|lv_style_set_radius" /path/to/lvgl_rust_sys/bindings.rs | head -5
```

**Step 3: Run (manual — two styled buttons, second is circular red)**
```sh
LIBCLANG_PATH=/usr/lib64 cargo run -p oxivgl-examples-get-started --bin ex3 --target x86_64-unknown-linux-gnu
```

**Step 4: Commit**
```sh
git add examples/get-started/src/bin/ex3.rs
git commit -m "feat(examples): ex3 — custom button styles with gradient and press filter"
```

---

## Known Risks / Likely Build Issues

1. **`lv_style_reset`** may not be in bindings — if missing, remove `Style`'s `Drop` impl.
2. **`lv_palette_lighten` signature** — `level` parameter type may be `i32` not `i16`; adjust `palette.rs` accordingly.
3. **`lv_style_set_bg_grad_color` / `lv_style_set_bg_grad_dir`** — may have different names in LVGL 9.x. If missing, check bindings and update or omit gradient from ex3.
4. **`lv_style_set_color_filter_dsc` / `lv_style_set_color_filter_opa`** — may be absent in embedded LVGL config. If missing, omit `ColorFilter` from ex3.
5. **`BUFS StaticCell` init** — `BUFS.init_with(|| ...)` vs `BUFS.init(...)` — use whichever `static_cell` version supports.
6. **`lv_event_get_target_obj`** vs `lv_event_get_target` — use whichever is in bindings.
7. **`Label` text buffer size** — `CString::<20>` limits to 19 bytes. "Button 2\0" = 9 bytes — OK. If longer text is needed, increase the const.
