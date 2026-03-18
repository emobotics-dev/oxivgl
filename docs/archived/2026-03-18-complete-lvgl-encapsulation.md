# Complete lvgl_rust_sys Encapsulation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wrap all remaining raw `lvgl_rust_sys` FFI behind oxivgl's safe API so consumers never import `lvgl_rust_sys` directly.

**Architecture:** Three incremental CRs: timer_handler wrapper (CR-B), SDL builder (CR-A), snapshot API (CR-C). Each adds safe wrappers in `src/driver.rs` or new modules, then updates consumers and tests.

**Tech Stack:** Rust (no_std/std), lvgl_rust_sys FFI, png crate (optional feature)

**Spec:** `docs/superpowers/specs/2026-03-18-complete-lvgl-encapsulation-design.md`

---

### Task 1: CR-B — `timer_handler()` safe wrapper

**Files:**
- Modify: `src/driver.rs:11-43` (add method to `impl LvglDriver`)
- Modify: `src/view.rs:119-125` (replace raw FFI call)
- Test: `tests/integration.rs:58-64` (update `pump()`)
- Test: `tests/leak_check.rs:90-96` (update `pump()`)

- [ ] **Step 1: Write failing test — timer_handler returns non-zero**

Add to `tests/integration.rs` after the `pump()` function:

```rust
#[test]
fn timer_handler_callable() {
    ensure_init();
    // SAFETY: single-threaded, LVGL initialised via ensure_init().
    let driver = unsafe { DRIVER.as_ref().unwrap() };
    // timer_handler may return 0 if all timers just fired — just verify no panic.
    let _ms = driver.timer_handler();
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./run_tests.sh int`
Expected: FAIL — `timer_handler` method does not exist on `LvglDriver`.

Note: test name changed from spec's `timer_handler_returns_delay` to `timer_handler_callable` because `lv_timer_handler` may return 0 if all timers just fired.

- [ ] **Step 3: Implement `timer_handler` on `LvglDriver`**

In `src/driver.rs`, add inside `impl LvglDriver` block (after line 42):

```rust
    /// Drive LVGL timers. Returns recommended delay in ms until next call.
    ///
    /// # Safety contract
    ///
    /// Safe because `LvglDriver` existence proves `lv_init()` was called.
    /// Caller must uphold the single-task constraint (no concurrent LVGL calls).
    pub fn timer_handler(&self) -> u32 {
        // SAFETY: LvglDriver is the init token — lv_init() was called.
        unsafe { lv_timer_handler() }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./run_tests.sh int`
Expected: PASS for `timer_handler_returns_delay`.

- [ ] **Step 5: Update `view::run_lvgl` to use `timer_handler`**

In `src/view.rs`, line 94 change `_driver` to `driver`:
```rust
    let driver = LvglDriver::init(w, h);
```

Lines 119-125 (the complete `for` block), replace the old text:
```rust
            // SAFETY: lv_init() was called inside LvglDriver::init(); no other task
            // calls LVGL concurrently (single-task constraint).
            unsafe { lv_timer_handler() };
```
with:
```rust
            driver.timer_handler();
```
(Keep the surrounding `for _ in 0..16 {`, `debug!(...)`, `Timer::after(...)`, and closing `}` unchanged.)

Do NOT remove `use lvgl_rust_sys::*;` from `view.rs` — `lv_screen_active()`, `lv_obj_add_event_cb()`, `LV_DEF_REFR_PERIOD`, `lv_event_get_user_data`, `lv_event_t` are still used.

- [ ] **Step 6: Update `tests/integration.rs` `pump()` to use driver**

Replace the `pump()` function (lines 58-64):
```rust
fn pump() {
    // SAFETY: LVGL initialised, single-threaded.
    let driver = unsafe { DRIVER.as_ref().unwrap() };
    driver.timer_handler();
    unsafe { lvgl_rust_sys::lv_refr_now(core::ptr::null_mut()) };
}
```

Note: `lv_refr_now` has no safe wrapper yet — keep raw call for now.

- [ ] **Step 7: Update `tests/leak_check.rs` `pump()` to use driver**

Replace the `pump()` function (lines 90-96):
```rust
fn pump() {
    // SAFETY: LVGL initialised, single-threaded.
    let driver = unsafe { DRIVER.as_ref().unwrap() };
    driver.timer_handler();
    unsafe { lvgl_rust_sys::lv_refr_now(core::ptr::null_mut()) };
}
```

- [ ] **Step 8: Update `examples/common/src/host.rs` — thread driver through**

Change `run_host_loop` and `pump` to accept `&LvglDriver` (lines 12-28):
```rust
/// Drive the LVGL timer loop. Call after creating all widgets. Never returns.
pub fn run_host_loop(driver: &LvglDriver) -> ! {
    loop {
        driver.timer_handler();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Pump the LVGL timer `n` times (5 ms each).
pub fn pump(driver: &LvglDriver, n: u32) {
    for _ in 0..n {
        driver.timer_handler();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
```

Add import at top of file:
```rust
use oxivgl::driver::LvglDriver;
```

Remove `use lvgl_rust_sys::*;` if no other uses remain. Check: `lv_screen_load`, `lv_obj_create` in `fresh_screen()` (line 31-34) and `lv_snapshot_take` etc. in `capture()` still use it — keep import for now.

- [ ] **Step 9: Update `host_main!` macro to pass driver**

In `examples/common/src/host.rs`, update the macro (lines 96-140). Change `_driver` to `driver` and pass to `pump` and `run_host_loop`:

```rust
            let driver = if screenshot_only {
                LvglDriver::init(W, H)
            } else {
                LvglDriver::init_sdl(W, H)
            };
```

```rust
            pump(&driver, 10);
            capture(name, &dir);
```

```rust
            run_host_loop(&driver);
```

- [ ] **Step 10: Run all tests**

Run: `./run_tests.sh all`
Expected: all pass.

- [ ] **Step 11: Commit**

```bash
git add src/driver.rs src/view.rs tests/integration.rs tests/leak_check.rs examples/common/src/host.rs
git commit -m "feat: add LvglDriver::timer_handler() safe wrapper (CR-B)

Replaces raw lv_timer_handler() calls in view loop, tests, and examples."
```

---

### Task 2: CR-A — Extract `init_common()` helper

**Files:**
- Modify: `src/driver.rs:11-43` (extract shared init, refactor init/init_sdl)

- [ ] **Step 1: Run existing tests to verify baseline**

Run: `./run_tests.sh all`
Expected: all pass.

- [ ] **Step 2: Extract `init_common()` in `src/driver.rs`**

Add private helper before `impl LvglDriver`:
```rust
/// Shared LVGL init sequence: lv_init + log + tick callbacks.
/// Called by all public constructors.
fn init_common() {
    // SAFETY: lv_init is called exactly once (LvglDriver is non-Clone,
    // and all constructors go through this path).
    unsafe {
        lv_init();
        lv_log_register_print_cb(Some(lvgl_log_print));
        lv_tick_set_cb(Some(get_tick_ms));
    }
}
```

Refactor `init()`:
```rust
    pub fn init(w: i32, h: i32) -> Self {
        init_common();
        #[cfg(not(target_os = "none"))]
        // SAFETY: lv_init() was called in init_common() above.
        unsafe { init_host_display(w, h) };
        let _ = (w, h);
        Self
    }
```

Refactor `init_sdl()`:
```rust
    #[cfg(not(target_os = "none"))]
    pub fn init_sdl(w: i32, h: i32) -> Self {
        init_common();
        // SAFETY: lv_init() was called in init_common() above.
        unsafe { init_sdl_display(w, h) };
        Self
    }
```

- [ ] **Step 3: Run tests to verify refactor is correct**

Run: `./run_tests.sh all`
Expected: all pass — pure refactor, no behavior change.

- [ ] **Step 4: Commit**

```bash
git add src/driver.rs
git commit -m "refactor: extract init_common() to deduplicate LvglDriver constructors"
```

---

### Task 3: CR-A — Verify SDL FFI functions exist

**Files:**
- Check: lvgl_rust_sys bindings for `lv_sdl_window_set_title`, `lv_sdl_mouse_create`

- [ ] **Step 1: Search for `lv_sdl_window_set_title` in bindings**

```bash
grep -rn "lv_sdl_window_set_title" $(find . -path '*/lvgl_rust_sys*' -name '*.rs' -o -name '*.h') 2>/dev/null
```

- [ ] **Step 2: Search for `lv_sdl_mouse_create` in bindings**

```bash
grep -rn "lv_sdl_mouse_create" $(find . -path '*/lvgl_rust_sys*' -name '*.rs' -o -name '*.h') 2>/dev/null
```

- [ ] **Step 3: Handle results**

**If both functions exist in bindings:** proceed to Task 4 — no changes needed.

**If `lv_sdl_window_set_title` is missing:** in Task 4's `SdlBuilder::build()`, skip the title-setting call and add `// TODO: lv_sdl_window_set_title not in bindings — add to lvgl_rust_sys`. The `title()` builder method still compiles (stores the value), it just has no effect yet.

**If `lv_sdl_mouse_create` is missing:** in Task 4's `SdlBuilder::build()`, skip the mouse-creation call and add `// TODO: lv_sdl_mouse_create not in bindings — add to lvgl_rust_sys`. The `mouse()` builder method still compiles, it just has no effect yet.

Document any missing functions as a follow-up issue.

---

### Task 4: CR-A — SDL builder

**Files:**
- Modify: `src/driver.rs` (add `SdlBuilder` struct + impl)
- Modify: `examples/common/src/host.rs:106-110` (update `host_main!` macro)
- Test: `tests/integration.rs` (add builder test)

- [ ] **Step 1: Write failing test — SdlBuilder API compiles and chains**

Add to `tests/integration.rs`:
```rust
#[test]
fn sdl_builder_api() {
    // Verify builder API compiles and chains. Can't call build() since
    // LVGL is already initialised by ensure_init().
    let _builder = oxivgl::driver::LvglDriver::sdl(320, 240)
        .title(c"test")
        .mouse(false);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./run_tests.sh int`
Expected: FAIL — `sdl` method doesn't exist on `LvglDriver`.

- [ ] **Step 3: Implement `SdlBuilder`**

In `src/driver.rs`, add after `impl LvglDriver` block, gated host-only:

```rust
/// Builder for SDL-backed LVGL driver (interactive host demos).
#[cfg(not(target_os = "none"))]
pub struct SdlBuilder {
    w: i32,
    h: i32,
    title: Option<&'static core::ffi::CStr>,
    mouse: bool,
}

#[cfg(not(target_os = "none"))]
impl LvglDriver {
    /// Start building an SDL-backed LVGL driver.
    pub fn sdl(w: i32, h: i32) -> SdlBuilder {
        SdlBuilder { w, h, title: None, mouse: true }
    }
}

#[cfg(not(target_os = "none"))]
impl SdlBuilder {
    /// Set SDL window title. Default: no title.
    pub fn title(mut self, t: &'static core::ffi::CStr) -> Self {
        self.title = Some(t);
        self
    }

    /// Enable/disable SDL mouse input device. Default: enabled.
    pub fn mouse(mut self, enabled: bool) -> Self {
        self.mouse = enabled;
        self
    }

    /// Build the driver. Initialises LVGL, creates SDL window.
    pub fn build(self) -> LvglDriver {
        init_common();
        // SAFETY: lv_init() was called in init_common().
        let disp = unsafe { lv_sdl_window_create(self.w, self.h) };
        assert!(!disp.is_null(), "lv_sdl_window_create returned NULL");
        if let Some(title) = self.title {
            // SAFETY: disp is valid, title is a valid CStr.
            unsafe { lv_sdl_window_set_title(disp, title.as_ptr()) };
        }
        if self.mouse {
            // SAFETY: LVGL and SDL display are initialised.
            unsafe { lv_sdl_mouse_create() };
        }
        LvglDriver
    }
}
```

Note: if `lv_sdl_window_set_title` or `lv_sdl_mouse_create` are missing from bindings (Task 3), skip those calls and add a TODO.

- [ ] **Step 4: Run test to verify it passes**

Run: `./run_tests.sh int`
Expected: PASS for `sdl_builder_api`.

- [ ] **Step 5: Update `host_main!` macro to use builder**

In `examples/common/src/host.rs`, update the macro's driver creation (lines 106-110):
```rust
            let driver = if screenshot_only {
                LvglDriver::init(W, H)
            } else {
                LvglDriver::sdl(W, H).title(c"oxivgl").mouse(true).build()
            };
```

- [ ] **Step 6: Remove `init_sdl` method and `init_sdl_display` helper**

In `src/driver.rs`:
1. Remove the `pub fn init_sdl(w, h)` method (lines 32-42 after Task 2 refactor).
2. Remove the private `unsafe fn init_sdl_display(w, h)` function (lines 87-93).

The `SdlBuilder::build()` inlines the SDL window creation with the same `lv_sdl_window_create` call and the same null-pointer assert. The `assert!(!disp.is_null(), ...)` check from `init_sdl_display` is preserved in the builder's `build()` method.

- [ ] **Step 7: Run all tests + verify host example**

Run: `./run_tests.sh all`
Then: `./run_host.sh -s getting_started1` (captures screenshot via macro)
Expected: all pass, screenshot generated.

- [ ] **Step 8: Commit**

```bash
git add src/driver.rs examples/common/src/host.rs
git commit -m "feat: add SdlBuilder for LvglDriver SDL init (CR-A)

Replace init_sdl() with builder pattern: LvglDriver::sdl(w, h).title().mouse().build().
Supports window title and mouse input device configuration."
```

---

### Task 5: CR-C — Snapshot API

**Files:**
- Create: `src/snapshot.rs`
- Modify: `src/lib.rs` (add `pub mod snapshot`)
- Modify: `Cargo.toml` (add `png` optional feature)
- Test: `tests/integration.rs` (add snapshot test)

- [ ] **Step 1: Add `png` feature to `Cargo.toml`**

In `Cargo.toml`, add to `[features]`:
```toml
png = ["dep:png"]
```

Add to `[dependencies]`:
```toml
png = { version = "0.17", optional = true }
```

Note: `png` is only used on host (snapshot is cfg-gated), but the feature itself is unconditional — it's the `snapshot` module that's gated.

- [ ] **Step 2: Write failing test — Snapshot::take returns Some**

Add to `tests/integration.rs`:
```rust
#[test]
fn snapshot_take_returns_some() {
    let screen = fresh_screen();
    let _label = Label::new(&screen).unwrap();
    pump();
    let driver = unsafe { DRIVER.as_ref().unwrap() };
    let snap = oxivgl::snapshot::Snapshot::take(driver);
    assert!(snap.is_some(), "Snapshot::take should succeed after init");
    let snap = snap.unwrap();
    assert_eq!(snap.width(), 320);
    assert_eq!(snap.height(), 240);
    assert!(!snap.data().is_empty());
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `./run_tests.sh int`
Expected: FAIL — `snapshot` module doesn't exist.

- [ ] **Step 4: Create `src/snapshot.rs`**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Screen capture API (host-only).

use std::path::Path;

use lvgl_rust_sys::*;

use crate::driver::LvglDriver;

/// Captured screen snapshot. Owns the LVGL draw buffer.
///
/// `!Send + !Sync` by design — LVGL is single-task.
pub struct Snapshot {
    buf: *mut lv_draw_buf_t,
    width: u32,
    height: u32,
}

impl Snapshot {
    /// Capture the active screen as RGB565.
    ///
    /// Requires `&LvglDriver` to prove LVGL is initialised.
    /// Returns `None` if the snapshot allocation fails.
    pub fn take(_driver: &LvglDriver) -> Option<Self> {
        // SAFETY: LvglDriver proves lv_init() was called.
        // lv_screen_active() returns the current screen.
        // lv_snapshot_take allocates a draw buffer.
        let buf = unsafe {
            lv_snapshot_take(
                lv_screen_active(),
                lv_color_format_t_LV_COLOR_FORMAT_RGB565,
            )
        };
        if buf.is_null() {
            return None;
        }
        // SAFETY: buf is non-null and points to a valid lv_draw_buf_t.
        let header = unsafe { &(*buf).header };
        Some(Self {
            buf,
            width: header.w(),
            height: header.h(),
        })
    }

    /// Snapshot width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Snapshot height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Raw RGB565 pixel data. Row stride may include alignment padding.
    pub fn data(&self) -> &[u8] {
        // SAFETY: buf is valid (non-null, allocated by lv_snapshot_take).
        let buf = unsafe { &*self.buf };
        unsafe { std::slice::from_raw_parts(buf.data, buf.data_size as usize) }
    }

    /// Row stride in bytes (may differ from `width * 2` due to alignment).
    fn stride(&self) -> usize {
        // SAFETY: buf is valid.
        let buf = unsafe { &*self.buf };
        buf.header.stride as usize
    }

    /// Write snapshot as PNG. Converts RGB565 to RGB8.
    ///
    /// Requires the `png` feature.
    #[cfg(feature = "png")]
    pub fn write_png(&self, path: &Path) -> std::io::Result<()> {
        let w = self.width as usize;
        let h = self.height as usize;
        let stride = self.stride();
        let data = self.data();

        let mut rgb = Vec::with_capacity(w * h * 3);
        for row in 0..h {
            for col in 0..w {
                let off = row * stride + col * 2;
                let p = u16::from_le_bytes([data[off], data[off + 1]]);
                let r = ((p >> 11) & 0x1F) as u8;
                let g = ((p >> 5) & 0x3F) as u8;
                let b = (p & 0x1F) as u8;
                rgb.push((r << 3) | (r >> 2));
                rgb.push((g << 2) | (g >> 4));
                rgb.push((b << 3) | (b >> 2));
            }
        }

        let file = std::fs::File::create(path)?;
        let buf = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(buf, self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|e| std::io::Error::other(e))?;
        writer
            .write_image_data(&rgb)
            .map_err(|e| std::io::Error::other(e))?;
        Ok(())
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        // SAFETY: buf was allocated by lv_snapshot_take in take().
        unsafe { lv_draw_buf_destroy(self.buf) };
    }
}
```

- [ ] **Step 5: Add module to `lib.rs`**

In `src/lib.rs`, add after the `pub mod widgets;` line:
```rust
/// Screen capture (host-only).
#[cfg(not(target_os = "none"))]
pub mod snapshot;
```

- [ ] **Step 6: Run test to verify it passes**

Run: `./run_tests.sh int`
Expected: PASS for `snapshot_take_returns_some`.

- [ ] **Step 7: Write PNG roundtrip test**

Add to `tests/integration.rs` (requires `png` feature — tests already run on host):
```rust
#[test]
fn snapshot_write_png() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("PNG test");
    pump();
    let driver = unsafe { DRIVER.as_ref().unwrap() };
    let snap = oxivgl::snapshot::Snapshot::take(driver).unwrap();

    let dir = std::env::temp_dir().join("oxivgl-test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("snapshot_test.png");
    snap.write_png(&path).unwrap();
    assert!(path.exists(), "PNG file should be written");
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
    // Cleanup
    let _ = std::fs::remove_file(&path);
}
```

Gate with `#[cfg(feature = "png")]` so existing `run_tests.sh` keeps working without changes:
```rust
#[cfg(feature = "png")]
#[test]
fn snapshot_write_png() {
```

To run manually: `LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu --test integration --features png -- --test-threads=1 snapshot_write_png`

- [ ] **Step 8: Run test**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu --test integration --features png -- --test-threads=1 snapshot_write_png`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add src/snapshot.rs src/lib.rs Cargo.toml tests/integration.rs
git commit -m "feat: add Snapshot API for screen capture (CR-C)

Host-only module with take(), data(), write_png(). PNG output behind
optional 'png' feature. Uses buf.header.stride for correct row pitch."
```

---

### Task 6: Refactor `examples/common` to use Snapshot API

**Files:**
- Modify: `examples/common/src/host.rs:36-86` (replace `capture()` and `write_png()`)
- Modify: `examples/common/Cargo.toml` (add `png` feature on oxivgl, remove `png` direct dep)

- [ ] **Step 1: Update `examples/common/Cargo.toml`**

In the `[target.'cfg(not(target_arch = "xtensa"))'.dependencies]` section (line 8-9), change oxivgl to include `png` feature:
```toml
oxivgl = { path = "../..", features = ["log-04", "png"] }
```

In the same target-gated section, remove the standalone `png` dep (line 12):
```toml
# png = "0.17"  — now provided via oxivgl's png feature
```

- [ ] **Step 2: Rewrite `capture()` to use Snapshot**

In `examples/common/src/host.rs`, replace `capture()` and the private `write_png()` (lines 36-86):

```rust
/// Capture current screen as a PNG file under `<dir>/<name>.png`.
pub fn capture(driver: &LvglDriver, name: &str, dir: &str) {
    use oxivgl::snapshot::Snapshot;

    let snap = Snapshot::take(driver).expect("lv_snapshot_take returned NULL");
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.png"));
    snap.write_png(&path).expect("PNG write failed");
    println!("Screenshot: {}", path.display());
}
```

Delete the private `write_png()` function entirely.

- [ ] **Step 3: Update `host_main!` macro — pass driver to capture**

In the macro, update the `capture` call:
```rust
            capture(&driver, name, &dir);
```

- [ ] **Step 4: Check remaining `lvgl_rust_sys` uses in host.rs**

`fresh_screen()` (lines 31-34) still uses `lv_screen_load` and `lv_obj_create`. These have no safe wrapper yet — keep the import for now.

- [ ] **Step 5: Run host example screenshot**

Run: `./run_host.sh -s getting_started1`
Expected: screenshot generated, no errors.

- [ ] **Step 6: Commit**

```bash
git add examples/common/src/host.rs examples/common/Cargo.toml
git commit -m "refactor: examples/common uses Snapshot API, drops png dep

capture() now uses oxivgl::snapshot::Snapshot. Removes duplicate
RGB565-to-PNG conversion code."
```

---

### Task 7: Final verification + cleanup

**Files:**
- Check: all test suites
- Check: remaining `lvgl_rust_sys` imports in consumer code

- [ ] **Step 1: Run full test suite**

Run: `./run_tests.sh all`
Expected: all pass.

- [ ] **Step 2: Verify host example works**

Run: `./run_host.sh -s getting_started1`
Expected: screenshot generated.

- [ ] **Step 3: Check remaining `lvgl_rust_sys` usage**

```bash
grep -rn "lvgl_rust_sys" tests/ examples/common/src/ src/
```

Catalogue what remains. Expected remaining uses:
- `src/*.rs` — internal to oxivgl (correct, this is the abstraction layer)
- `tests/*.rs` — `lv_refr_now`, `lv_obj_create`, `lv_screen_load`, `lv_event_t` (future CRs)
- `examples/common/src/host.rs` — `lv_screen_load`, `lv_obj_create` in `fresh_screen()` (future CR)

These are out of scope for the current CRs but should be noted for follow-up.

- [ ] **Step 4: Verify embedded check still works**

Run: `cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04`
Expected: no errors (snapshot module is cfg-gated host-only).

- [ ] **Step 5: Commit any remaining cleanup**

If any dead imports or minor fixes needed:
```bash
git add -u
git commit -m "chore: cleanup dead imports after lvgl_rust_sys encapsulation"
```

---

## Notes

**Scope:** This plan covers oxivgl and `examples/common` only. Consumer crate updates (alternator-regulator's `sdl_viewer.rs`, `host_ui.rs`, `ui/mod.rs`) are a separate follow-up plan.

**Stride change:** The new `Snapshot::write_png()` uses `buf.header.stride` for row pitch, while the old `capture()` in `examples/common` used `w * 2`. This is more correct (handles alignment padding) but may produce slightly different PNG output if LVGL adds padding. In practice, RGB565 at typical resolutions has no padding, so output should be identical.

**Task 1 Step 9 vs Task 4 Step 5:** Both modify the `host_main!` macro's driver creation block. Task 1 renames `_driver` → `driver` and threads it to `pump`/`run_host_loop` (but keeps `LvglDriver::init_sdl()`). Task 4 then replaces `LvglDriver::init_sdl()` with the builder. These are sequential — Task 4 layers on top of Task 1's changes.
