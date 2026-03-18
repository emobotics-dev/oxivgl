# Design: Complete lvgl_rust_sys Encapsulation

Implements CR-A, CR-B, CR-C from `docs/cr-complete-lvgl-encapsulation.md`.
Goal: consumer crates never import `lvgl_rust_sys` directly.

Note: this spec supersedes the CR's `write_ppm` with `write_png` (PNG is a better format, user chose it).

## CR-B: `timer_handler()` wrapper

Add to `LvglDriver` in `src/driver.rs`:

```rust
impl LvglDriver {
    /// Drive LVGL timers. Returns recommended delay in ms until next call.
    pub fn timer_handler(&self) -> u32 {
        // SAFETY: LvglDriver existence proves lv_init() was called.
        unsafe { lv_timer_handler() }
    }
}
```

- `&self` receiver — init token guarantees lv_init() was called
- Update `view::run_lvgl` line 123: rename `_driver` to `driver`, call `driver.timer_handler()`
- Update consumer `sdl_viewer.rs` render loop
- Update `examples/common/src/host.rs`: `run_host_loop()` and `pump()` call raw `lv_timer_handler()` — thread `&LvglDriver` through as parameter. The `host_main!` macro creates the driver, so it passes the reference into `run_host_loop`.

## CR-A: SDL builder

New `SdlBuilder` in `src/driver.rs`, gated `#[cfg(not(target_os = "none"))]`:

```rust
pub struct SdlBuilder {
    w: i32,
    h: i32,
    title: Option<&'static CStr>,
    mouse: bool,
}

impl LvglDriver {
    /// Start building an SDL-backed LVGL driver.
    pub fn sdl(w: i32, h: i32) -> SdlBuilder {
        SdlBuilder { w, h, title: None, mouse: true }
    }
}

impl SdlBuilder {
    /// Set SDL window title. Default: no title.
    pub fn title(mut self, t: &'static CStr) -> Self {
        self.title = Some(t);
        self
    }

    /// Enable/disable SDL mouse input device. Default: true.
    pub fn mouse(mut self, enabled: bool) -> Self {
        self.mouse = enabled;
        self
    }

    /// Build the driver. Calls init_common(), creates SDL window,
    /// optionally sets title, optionally creates mouse device.
    pub fn build(self) -> LvglDriver { ... }
}
```

Implementation notes:
- Extract shared init sequence (`lv_init`, `lv_log_register_print_cb`, `lv_tick_set_cb`) into private `init_common()` to avoid three-way duplication between `init()`, `init_sdl()`, and `build()`.
- Prerequisite: verify `lv_sdl_window_set_title` and `lv_sdl_mouse_create` exist in `lvgl_rust_sys` bindings. If missing, add them to the sys crate first.
- Default: mouse enabled (most common), no title.
- Remove `init_sdl(w, h)` — only 1-2 call sites, clean break.
- `'static` lifetime on title is conservative but safe (unclear if LVGL copies the string).
- Consumer `sdl_viewer.rs` becomes:
  ```rust
  let driver = LvglDriver::sdl(W, H).title(c"altreg-ui").build();
  ```

## CR-C: Snapshot API

New `src/snapshot.rs`, gated `#[cfg(not(target_os = "none"))]`.

```rust
/// Screen capture. Contains raw LVGL draw buffer.
/// `!Send + !Sync` by design (LVGL is single-task).
pub struct Snapshot {
    buf: *mut lv_draw_buf_t,
    width: u32,
    height: u32,
}

impl Snapshot {
    /// Capture the active screen as RGB565. Returns None if snapshot fails.
    /// Requires `&LvglDriver` to prove LVGL is initialized.
    pub fn take(driver: &LvglDriver) -> Option<Self> { ... }

    pub fn width(&self) -> u32 { ... }
    pub fn height(&self) -> u32 { ... }

    /// Raw RGB565 pixel data. Uses `buf.header.stride` for correct row pitch
    /// (stride may differ from width * 2 due to alignment padding).
    pub fn data(&self) -> &[u8] { ... }

    /// Write as PNG file. Converts RGB565 to RGB8 internally.
    /// Requires `png` feature.
    #[cfg(feature = "png")]
    pub fn write_png(&self, path: &Path) -> io::Result<()> { ... }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        unsafe { lv_draw_buf_destroy(self.buf) };
    }
}
```

- `take(&LvglDriver)` — init token prevents UB from calling before `lv_init()`
- Feature `png` in `Cargo.toml`: adds `png` crate dep, enables `write_png()`
- `data()` must use `buf.header.stride` (not assume `width * 2`) for correct row pitch
- RGB565-to-RGB8 conversion in `write_png()` (currently duplicated in consumers)
- `pub mod snapshot` in `lib.rs` (cfg-gated), not in prelude (host-only utility)
- Refactor `examples/common/src/host.rs::capture()` to use `Snapshot`
- Consumer `host_ui.rs` switches to `Snapshot::take(&driver)` + `write_png()`

## Consumer updates

After all CRs:
- `sdl_viewer.rs`: builder init + `driver.timer_handler()`, no raw FFI
- `host_ui.rs`: `Snapshot::take(&driver)` + `write_png()`, no raw FFI
- `examples/common/src/host.rs`: `run_host_loop(&driver)`, `pump(&driver)`, `capture()` uses `Snapshot`
- Consumer crates drop `lvgl_rust_sys` from `[dependencies]`

## Test plan

- Unit test: `timer_handler` returns non-zero after init
- Integration test: `SdlBuilder` builds without panic (with/without mouse, with/without title)
- Integration test: `Snapshot::take()` + `write_png()` roundtrip
- Update existing tests in `tests/integration.rs` and `tests/leak_check.rs` to use `driver.timer_handler()` instead of raw `lv_timer_handler()`

## Execution order

CR-B -> CR-A -> CR-C (each builds on previous; CR-B simplest/highest impact)

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| `timer_handler` receiver | `&self` | Init token sufficient; `&mut` complicates render loops |
| SDL config | Builder pattern | Future-proof for additional SDL options |
| Mouse default | Enabled | Most common use case |
| `init_sdl` | Remove (not deprecate) | Only 1-2 call sites, clean break |
| PNG support | Optional feature `png` | Keeps dep tree minimal for non-snapshot consumers |
| PNG vs PPM | PNG (supersedes CR) | Better format, user preference |
| `Snapshot::take` | `take(&LvglDriver)` | Requires init token, prevents UB |
| Snapshot in prelude | No | Host-only utility, not a core type |
| Snapshot Send/Sync | `!Send + !Sync` (default from raw ptr) | Correct — LVGL is single-task |
| Snapshot scope | Active screen only | All current consumers use `lv_screen_active()` |
| Example refactor | Yes | Eliminates duplicate snapshot code |
| Stride handling | Use `buf.header.stride` | Correct for padded buffers |
