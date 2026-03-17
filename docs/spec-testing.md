# Testing Specification

When and how to test oxivgl.

---

## 1. Test Tiers

| Tier | Location | Runs with | What it catches |
|---|---|---|---|
| **Unit** | `src/**/*.rs` `#[test]` | `./run_tests.sh unit` | Pure logic: enum values, value mapping, bitflags, grid helpers |
| **Doc** | `src/**/*.rs` doc blocks | `./run_tests.sh unit` | Stale doc examples, wrong import paths |
| **Integration** | `tests/integration.rs` | `./run_tests.sh int` | Full LVGL lifecycle: widget create/drop, style add/remove, events, animations |
| **Leak** | `tests/leak_check.rs` | `./run_tests.sh leak` | Memory leaks across the FFI boundary via `mallinfo2()` |
| **Visual** | `./run_host.sh -s` | CI screenshot step | Compile + run all examples; catches stale imports, runtime crashes, visual regressions |

All tiers run on host (x86\_64) without hardware. CI runs all of them.

---

## 2. When to Write Tests

### New wrapper method

Every new public method on a widget wrapper should have at least one
integration test exercising the happy path. If the method stores a
pointer in LVGL, add a leak test confirming no memory is leaked when
the widget is dropped.

### New widget type

Add integration tests covering: construction, basic property setting,
style application, and drop. Add a leak test for the widget type. Port
at least one LVGL example to serve as a visual test.

### Spec compliance fix

When fixing a spec violation (e.g. adding a `'static` bound, fixing
drop ordering), add a test that would have caught the bug. Integration
tests are the right tier for lifecycle and ordering issues.

### Refactoring

No new tests needed if the refactoring is pure restructuring (module
moves, renames). But run all tiers to verify nothing broke.

---

## 3. Integration Test Patterns

Integration tests share a common harness in `tests/integration.rs`:

- `fresh_screen()` — creates a fresh LVGL screen for each test.
- `pump()` — runs one LVGL render cycle.
- Tests run sequentially (`--test-threads=1`) because LVGL is
  single-threaded.
- `SDL_VIDEODRIVER=dummy` is set by `run_tests.sh`.

### Testing drop ordering

Create widgets + styles, drop them in various orders, call `pump()`
after each drop. If LVGL crashes or leaks, the test fails.

### Testing events

Use `Obj::send_event()` to simulate events. Check state via
`AtomicBool` flags in `unsafe extern "C"` callback stubs.

---

## 4. Leak Test Patterns

Leak tests use `mallinfo2()` to measure heap before and after:

```
assert_no_leak("WidgetName", expected_alloc_count, |screen| {
    let widget = Widget::new(screen).unwrap();
    // exercise the widget
    drop(widget);
});
```

The closure receives a fresh screen. The harness checks that heap usage
returns to baseline after the closure runs.

Add a leak test for every new widget type and for any wrapper that
allocates (Box, Vec, Rc).

---

## 5. Visual Tests

Every example in `run_host.sh` is compiled and executed headlessly in
CI. This catches:

- Compile errors (stale imports, missing `no_std` imports)
- Runtime crashes (null pointers, wrong LVGL API usage)
- Visual regressions (compare screenshots against LVGL docs)

No explicit test code needed — the examples ARE the tests. Adding a new
example to `ALL_EXAMPLES` in `run_host.sh` is sufficient.

---

## 6. Doc Tests

Use ```` ``` ```` (bare) for examples that can run standalone (pure value
computations). Use ```` ```no_run ```` for examples that need LVGL init
(compile-checked but not executed). Use ```` ```ignore ```` only for
incomplete snippets that reference undefined variables like `self`.

Prefer `no_run` over `ignore` — it catches stale imports and type
errors.

---

## 7. Portability

Every change to `src/` must build on both targets:

```sh
# Host (std):
LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu

# Embedded (no_std):
cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04
```

Common `no_std` traps:
- Missing `use alloc::vec::Vec` (implicit in std prelude, not in
  `no_std`)
- `format!()` requires `alloc::string::String` — use
  `heapless::String` instead
- `Arc` unavailable on Xtensa (no hardware atomics) — use `Rc`
- `std::thread`, `std::time` — not available
