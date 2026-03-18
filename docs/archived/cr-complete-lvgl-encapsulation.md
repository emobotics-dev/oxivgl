# Change Request: Complete lvgl_rust_sys encapsulation

Consumer crates (alternator-regulator, oxiforge) should never import `lvgl_rust_sys`
directly. oxivgl is the safe abstraction layer — all raw LVGL FFI should be behind
oxivgl's public API.

Motivation: lets oxiforge re-export oxivgl as a single dependency for consumers
(CR-4 in `oxiforge/docs/cr-altreg-yaml-migration.md`), and eliminates raw `unsafe`
LVGL calls scattered across consumer code.

---

## Current consumer usage of lvgl_rust_sys

Two files in alternator-regulator use `lvgl_rust_sys::*` directly:

### 1. `src/bin/sdl_viewer.rs` — interactive SDL viewer

```rust
use lvgl_rust_sys::*;
unsafe {
    lv_init();
    lv_log_register_print_cb(Some(oxivgl::lvgl::lvgl_log_print));
    lv_tick_set_cb(Some(oxivgl::lvgl::get_tick_ms));
    let disp = lv_sdl_window_create(W, H);
    lv_sdl_window_set_title(disp, c"altreg-ui".as_ptr());
    lv_sdl_mouse_create();
}
// render loop:
let next_ms = unsafe { lv_timer_handler() };
std::thread::sleep(Duration::from_millis(next_ms.max(1) as u64));
```

### 2. `tests/host_ui.rs` — headless snapshot test

```rust
use lvgl_rust_sys::*;
// LvglDriver::init() handles lv_init/display — already encapsulated
unsafe { lv_timer_handler() };
// snapshot capture:
let draw_buf = unsafe {
    lv_snapshot_take(lv_screen_active(), lv_color_format_t_LV_COLOR_FORMAT_RGB565)
};
// read pixel data from draw_buf...
unsafe { lv_draw_buf_destroy(draw_buf) };
```

---

## CR-A: SDL window title + mouse input on `LvglDriver::init_sdl`

### Problem

`LvglDriver::init_sdl()` creates the SDL window but doesn't set a title or create
a mouse input device. Consumers call raw FFI for this.

### Required change

Extend `init_sdl` (or add a builder) to accept an optional window title and
automatically create the SDL mouse device.

```rust
// Option A: extend init_sdl
pub fn init_sdl(w: i32, h: i32, title: &CStr) -> Self { ... }

// Option B: builder
LvglDriver::builder()
    .sdl(w, h)
    .title(c"altreg-ui")
    .mouse(true)
    .build()
```

After this, `sdl_viewer.rs` becomes:

```rust
let _driver = LvglDriver::init_sdl(W, H, c"altreg-ui");
```

No raw FFI needed.

---

## CR-B: `timer_handler()` safe wrapper

### Problem

Every consumer render loop calls `unsafe { lv_timer_handler() }`. This is the most
common raw FFI call.

### Required change

Add a safe wrapper on `LvglDriver`:

```rust
impl LvglDriver {
    /// Drive LVGL timers. Returns recommended delay in ms until next call.
    pub fn timer_handler(&self) -> u32 {
        // SAFETY: LvglDriver existence proves lv_init() was called.
        // Single-task constraint is upheld by the caller.
        unsafe { lv_timer_handler() }
    }
}
```

The `&self` receiver ensures `lv_init()` was called (LvglDriver is the init token).
Consumer render loop becomes:

```rust
let next_ms = driver.timer_handler();
```

Note: `view::run_lvgl` also calls `lv_timer_handler()` directly — update it too.

---

## CR-C: Snapshot API

### Problem

`host_ui.rs` uses raw FFI for screenshots: `lv_snapshot_take`, `lv_screen_active`,
`lv_draw_buf_destroy`, and manual pixel data extraction.

### Required change

Add a snapshot module (host-only, behind `#[cfg(not(target_os = "none"))]`):

```rust
pub struct Snapshot {
    buf: *mut lv_draw_buf_t,
    width: u32,
    height: u32,
}

impl Snapshot {
    /// Capture the active screen as RGB565.
    pub fn take() -> Option<Self> { ... }

    pub fn width(&self) -> u32 { ... }
    pub fn height(&self) -> u32 { ... }

    /// Raw RGB565 pixel data.
    pub fn data(&self) -> &[u8] { ... }

    /// Write as PPM file.
    pub fn write_ppm(&self, path: &Path) -> io::Result<()> { ... }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe { lv_draw_buf_destroy(self.buf) };
    }
}
```

Consumer test becomes:

```rust
let snap = oxivgl::snapshot::Snapshot::take().unwrap();
snap.write_ppm(&path).unwrap();
```

The `write_ppm` helper is currently duplicated in consumer test code — moving it
into oxivgl eliminates that.

---

## Priority

CR-B (timer_handler) is simplest and highest impact — used by every consumer.
CR-A (SDL init) is straightforward.
CR-C (snapshot) is the largest but eliminates the most unsafe consumer code.

## After these CRs

Consumer crates drop their `lvgl_rust_sys` dependency entirely. oxiforge CR-4
re-exports oxivgl only (no `lvgl_rust_sys` re-export needed). The `[workspace]`
conflict is resolved by removing `lvgl_rust_sys` from oxiforge's dependency tree.
