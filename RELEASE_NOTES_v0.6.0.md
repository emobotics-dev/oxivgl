# oxivgl v0.6.0

This release adds **ESP32-S3 (M5Stack CoreS3) support**, lets **LVGL's heap live
in PSRAM** — a region whose address is only known at run time — and removes a
class of **silent failure** from the bundled configuration, the leak suite, and
the test tooling.

## Highlights

### 🎛️ Two boards, one harness — ESP32-S3 (CoreS3) support (#119)

oxivgl now targets both the M5Stack **Fire27** (ESP32) and **CoreS3**
(ESP32-S3). A new `esp32s3` library feature is all `src/` needs — it is
chip-agnostic, generic over `DisplayOutput`.

The example harness was ported onto the
[`m5stack-core`](https://github.com/emobotics-dev/m5stack-core) BSP, replacing
~400 lines of hand-rolled Fire27 SPI/GPIO/button bring-up with the BSP's
`Board::split` + `board::display` + input/console/heap loops. Both boards share
one code path and diverge only where they physically differ — the CoreS3 resets
and powers its panel over I2C (AW9523B + AXP2101) and takes FT6336U **touch**
input, where the Fire27 uses a GPIO reset and three **buttons**. Pick a board
with the `fire27` / `cores3` cargo feature; the input model follows
automatically.

```sh
./run_fire27.sh getting_started1   # M5Stack Fire27 (ESP32)
./run_cores3.sh getting_started1   # M5Stack CoreS3 (ESP32-S3)
```

The public `example_main!` / `_nav!` / `_psram!` macros are unchanged, so **no
example file changed**. HIL-validated on both boards.

### 🧠 Runtime memory pools — LVGL's heap in PSRAM (#116)

`LV_MEM_ADR` is a compile-time constant, but on ESP32-S3 the PSRAM window maps
*after* the flash rodata mmap: its base shifts whenever the binary's size
changes, so no constant can name it. Hardcoding one either panics on boot or
works until the next build.

`oxivgl::mem::reserve_pool` takes a region discovered at run time and registers
it with LVGL's heap during driver init:

```rust
// after the BSP maps PSRAM, before the render loop starts
let split = m5stack_core::mem::psram_split(peripherals.PSRAM, Some(512 * 1024))?;
oxivgl::mem::reserve_pool(split.private)?;   // no unsafe, no cast
```

It takes `&'static mut [MaybeUninit<u8>]` rather than raw parts, because LVGL
never releases a pool — the requirement is in the type, and the call site needs
no `unsafe`. Rejections come back as a typed `MemError` instead of LVGL's
log-and-return-NULL, and `MemError::TooLarge` names the specific `lv_conf.h`
macro at fault.

**Draw buffers are kept out of the pool automatically.** TLSF pools are
fungible, and LVGL's default draw-buffer allocator is a plain `lv_malloc` — so
layer, canvas and snapshot buffers are eligible to land in PSRAM, which the
original ESP32 cannot DMA from at all. With the guard disabled, the *first*
64×64 draw buffer allocates inside the pool; with it, they route through the
Rust global allocator and stay in internal DMA-capable RAM. Verified on
hardware.

Requires `LV_USE_STDLIB_MALLOC = LV_STDLIB_BUILTIN`. Under `LV_STDLIB_CLIB`,
`lv_mem_add_pool` is a no-op returning NULL — a pool would be accepted and
silently ignored — so the module is not compiled at all on that backend.

### 🔬 Leak detection now sees LVGL's C heap (#118)

The leak suite measured only a Rust `#[global_allocator]` — blind to the exact
failure the wrappers exist to prevent. If a `Drop` impl stopped calling
`lv_obj_delete`, the wrapper struct still freed, the Rust balance still
zeroed, and the test still passed while LVGL's heap grew every iteration.

Every leak test now *also* asserts on `lv_mem_monitor().total_size - free_size`,
so that regression fails the suite. Both heaps assert an **exact zero with no
noise floor** — the one-shot LVGL init costs are identified and excluded by
name rather than absorbed into a tolerance — and the sensitivity is itself
tested: a leak of **one byte per iteration**, and the smallest allocation LVGL
can make, are each *required* to fail. Gated on `LV_STDLIB_BUILTIN` (now the
bundled default); compiled out under CLIB rather than degrading to an assertion
that can only pass.

### 💥 LVGL assertions now panic instead of spinning

LVGL's default `LV_ASSERT_HANDLER` is `while(1);`. Every assertion — a failed
allocation, a NULL object, a corrupt style — became an indefinite spin with no
message and no backtrace. On host a CI job sits at its timeout reporting no
failing test; on target the device wedges. Downstream traced a recurring-freeze
bug class to exactly this.

The bundled configs now route assertions through `oxivgl_lv_assert_handler`,
which panics — a failing test on host, the BSP panic handler on target — with
LVGL's own log line (expression, file, line) immediately above it. The symbol is
prefixed deliberately so it cannot collide with an application's own unprefixed
`lv_assert_handler`. Opt-in: `lv_conf.h` belongs to the application, and one
that prefers halting can keep LVGL's default.

## ⚠️ Upgrade notes

- **The bundled `lv_conf.h` files now select `LV_STDLIB_BUILTIN`.** If you copied
  `examples/conf/lv_conf.h` or `oxivgl-sys/default-conf/lv_conf.h`, LVGL now
  allocates from its own static TLSF pool sized by `LV_MEM_SIZE` rather than
  from the Rust global allocator. **Size `LV_MEM_SIZE` for your workload** — it
  is a real static array, and too small a value now fails allocations instead of
  falling back to the system heap.
- **Set `LV_MEM_POOL_EXPAND_SIZE` if you register a runtime pool.** LVGL patches
  TLSF so its largest indexable block is `LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE`
  rounded up, and it defaults to `0` — so a 512 KiB pool is rejected outright
  unless this is raised. It costs about 0.5 KB of index, not the pool.
- **`esp-hal` moved to `=1.1.1`**, matching `m5stack-core`.
- **Examples now depend on the `m5stack-core` BSP** and are selected with the
  `fire27` / `cores3` feature. This affects only building the examples;
  **library consumers are unaffected**. The workspace redirects the esp-hal
  family to the emobotics fork via `[patch.crates-io]` (the BSP's DMA flush needs
  its SPI/DMA fixes), but `cargo publish` strips `[patch]` and oxivgl's own
  `esp-hal` dependency stays a plain crates.io version — a downstream
  `cargo add oxivgl` resolves stock esp-hal with no fork and no git sources.

## Fixed

- **`tests/leak_check.rs` did not compile.** Two hand-declared libc externs
  (`write`, `open`) diverged from their real signatures, which newer rustc
  rejects. The leak suite had also never run in CI, which is how that went
  unnoticed. It runs now (and covers the C heap — see above).
- **The doc audit could not fail.** `run_docs.sh --check` piped rustdoc into
  `grep "warning:"` then `|| true; exit 0`, discarding every `error:` line and
  the exit status. Broken intra-doc links are errors and a failed doc build emits
  no HTML at all — so it reported a clean audit while `oxivgl::mem` was entirely
  absent from the generated docs.
- **Docs that asserted `LV_STDLIB_CLIB`** were corrected, including a README
  claim that leak detection "catches leaks in both Rust and LVGL's C code across
  the FFI boundary". It never did across FFI via `GlobalAlloc`; the C side is now
  covered through `lv_mem_monitor` instead.
- **`bindings_docsrs.rs`** still reported CLIB, which would have published
  docs.rs without `oxivgl::mem`. Regenerated, and `tools/regen-docsrs-bindings.sh`
  now exists so it does not drift again.
- **`translation::add_static`** now documents that it must be called once per
  pack — each call allocates on LVGL's heap with no per-pack removal, so
  repeated registration grows the heap without bound. Surfaced by the new C-heap
  leak coverage.

## Known limitations

- **No performance figures are claimed here.** The improvements that motivated
  the PSRAM work were measured downstream on CoreS3 and are being re-measured
  against this implementation (emobotics-dev/alternator-regulator#137), which
  differs from the hardcoded-address prototype in ways that move allocations
  between pools. The mechanism is verified on hardware; the numbers belong to
  the workload that owns them.
