<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# The render pipeline: blocking the flush wait

Everything here is measured on the bench, both boards, and reproducible with
`examples/widget_scale10.rs` built `--features perf-probe`.

Context: oxivgl#1, and the cost model in m5stack-core's
`docs/lvgl-ui-performance.md`.

## The problem

LVGL calls `flush_wait_cb` on its synchronous C stack, so the render thread cannot
`.await` there — it has to *block*. oxivgl's original wait blocked with the
Xtensa `waiti 0` instruction:

```rust
loop {
    if let Ok(drv) = FLUSH_OPERATION.try_receive() { /* … */ return; }
    unsafe { core::arch::asm!("waiti 0") };
}
```

`waiti 0` does not yield — it **halts the core** until the next interrupt. The
scheduler is never entered, so for the entire 15–30 ms panel transfer *nothing
runs but ISRs*. It is invisible in CPU-load measurements because the core is not
busy; it is stopped.

## Measured

Same board, same UI, same load — a continuously swept gauge needle. The only
difference is the pipeline. A probe task wakes on a fixed 10 ms period and
records how late it actually ran, standing in for latency-sensitive application
work; it should see **100 wakeups per second**.

### Fire27 (ESP32)

| | wakeups/s | mean late | max late | >5 ms | >20 ms | flush |
|---|---:|---:|---:|---:|---:|---|
| `waiti` + shared executor | 65–70 | 5872 µs | 36 997 µs | 13 | 10 | 33 ops/s, 400 kB/s |
| threads + semaphore | **100** | **84 µs** | **190 µs** | **0** | **0** | 33 ops/s, 417 kB/s |

### CoreS3 (ESP32-S3)

| | wakeups/s | mean late | max late | >5 ms | >20 ms | flush |
|---|---:|---:|---:|---:|---:|---|
| `waiti` + shared executor | 83–95 | 1124–2317 µs | 13 214–22 390 µs | 9–13 | 0–1 | 33 ops/s, 397 kB/s |
| threads + semaphore | **100** | **69 µs** | **329 µs** | **0** | **0** | 33 ops/s, 403 kB/s |

> The `waiti` + shared executor row is a **historical baseline**: it measured the
> blocking wait running inside an async task on a shared executor. The shipped design
> never does that (see "The split loop" below), so this row's configuration is no
> longer reachable — it stays here unmodified for the before/after comparison.

Two things to read off these numbers.

**Throughput is unchanged.** Flush ops/s and kB/s are the same either way, so the
latency is not bought with frame rate. That is the signature of a *blocking*
problem rather than a throughput one — the baseline was never CPU-starved, it
was parked.

**The ESP32 gains most.** It starts worse (a third of its deadlines missed
outright, worst case 37 ms) and ends level with the S3. A design that looks
adequate on ESP32-S3 can be badly starved on ESP32.

## The design

The blocking primitive is **injected**, not chosen by oxivgl:

```rust
pub trait FlushSync: Sync {
    fn wait(&self);    // render thread, LVGL's synchronous C stack
    fn signal(&self);  // flush context — may be interrupt context
}
```

| implementation | blocks by | while waiting |
|---|---|---|
| `SemaphoreFlushSync` (`rtos-sem`) | RTOS semaphore | the render thread leaves the run queue |
| `WaitiFlushSync` (**deprecated**) | `waiti 0` | the core is parked |

`SemaphoreFlushSync` is what an application should register. `WaitiFlushSync` is
deprecated and remains only as the fallback when nothing is registered, for an
application that links no scheduler at all.

Under the split loop it is not merely slower, it is the wrong shape: the refresh
blocks from the executor's idle hook, so parking the core there halts the whole
scheduler rather than only the render thread — the opposite of what the split
buys. The board harness now registers `SemaphoreFlushSync` in **every** mode:
`leak_isr()` in the stock pipeline, whose flush runs on an `InterruptExecutor`,
and `leak_thread()` in `threaded`, whose flush has its own thread. Previously
only `threaded` registered one, so the stock modes silently fell back to
`waiti`.

### Why this stays RTOS-agnostic

`SemaphoreFlushSync` is built on `esp-radio-rtos-driver`, which is **not an
RTOS** — it is Espressif's scheduler *interface* crate (its own dependencies are
`cfg-if`, `esp-sync` and `portable-atomic`; it depends on neither esp-hal nor
esp-rtos). It declares `esp_rtos_semaphore_*` symbols that a scheduler registers
at link time, the same shape as `#[global_allocator]`:

* zero implementations linked → undefined-symbol **link error**;
* two → duplicate-symbol **link error**.

So depending on the interface does not pick a scheduler — the application's
scheduler crate does. esp-rtos is the usual one; ArielOS and ESP-IDF/FreeRTOS
shims implement the same interface.

**Linking requirement.** esp-rtos registers those symbols from a module gated
behind its `esp-radio` feature, so the *consumer* must enable it. Despite the
name it pulls no radio blob — it is esp-rtos's FreeRTOS-compat IPC objects:

```toml
esp-rtos = { version = "0.4", features = ["embassy", "esp-radio", "esp-alloc"] }
```

### LVGL stays single-threaded

The flush side only moves bytes. `lv_display_flush_ready` is still called from
the render thread, inside `FlushSync::wait`'s caller — never from the flush
context. That is what keeps `LV_USE_OS LV_OS_NONE` correct even with the flush on
its own thread.

## The split loop: async events, blocking refresh

`FlushSync::wait` runs on LVGL's synchronous C stack — a genuine block, not
something an async task can `.await`. A task that calls `lv_timer_handler` directly
stalls its whole executor for the transfer, taking every other task sharing that
executor down with it (that is the `waiti` + shared executor row above, at executor
scope rather than core scope).

So the render thread splits in two. `Ui::refresh()` is the *only* place
`lv_timer_handler` (and, in DIRECT mode, `scanout::wait_presented()`) runs — a plain
blocking call made directly on the render thread, never from inside an async task.
`Ui::run_events` / `run_events_nav` are the async side: widget setup, `View::update`,
input waits — no call that can block.

The two meet at the executor's idle hook: `esp_rtos::embassy::Executor::run_with_callbacks`
invokes `Callbacks::on_idle` exactly when the executor is about to sleep, and the
application wires `on_idle` to call `Ui::refresh()` there. The executor still parks
between refreshes — no busy-poll — and the blocking refresh never runs while an async
task is mid-poll. `run_app`/`Ui::run`-family loops skip this split; they call
`lv_timer_handler` inline and so are only correct where the flush wait doesn't block
(host, or a non-blocking pipeline).

## Placement is the application's job

Blocking the wait is only half the result. The other half is the priority
ladder, and oxivgl deliberately does not own it — only the application knows
what the UI must yield to. See `examples/common/src/sched.rs`:

| | priority | |
|---|---|---|
| esp-radio blob threads | ~20+ | untouched, always win |
| app / latency-sensitive work | **3** | raised via `CurrentThreadHandle::set_priority` |
| flush thread | **2** | above render, so the panel never starves |
| render thread | **1** | yields to everything above |
| idle | 0 | |

**`#[esp_rtos::main]` starts at priority 0 — the lowest.** Moving the render loop
to a thread *without* raising the app executor first makes latency worse, not
better, because the render thread then outranks the very work it exists to yield
to. This is the single easiest thing to get wrong.

### RGB / scan-out panels

SPI copies dirty stripes onto a bus. An RGB panel DMA-scans a framebuffer
already, so that copy is waste. `scanout` is the other pipeline:
`Ui::init` with `Buffers::full` puts LVGL in `DIRECT` mode against two
full-screen buffers,
and on the last flush of a frame `ScanOut::present` swaps the scan pointer
at vblank. No stripe buffers, no flush thread. The render thread remains so
LVGL stays off the application executor.

Threads, not another `InterruptExecutor`: an interrupt executor makes the UI
preempt *everything*, which is backwards. A thread is preemptible by priority in
both directions.

## The broadest measurement: `oxivgl::demo::benchmark`

Everything below measures one example under one configuration. LVGL's benchmark
demo measures the whole pipeline across ~30 scenes — fills, borders, shadows,
images, text, arcs, masking, blending, scrolling — and reports averaged FPS,
CPU, render time and flush time per scene, so a regression can be attributed to
a drawing primitive rather than to "the UI feels slow". It is the first thing to
run when render performance is in question, and the last word on whether a
pipeline change helped.

```sh
./run_benchmark.sh host       # SDL window
./run_benchmark.sh fire27     # flash + monitor (ESP32)
./run_benchmark.sh cores3     # flash + monitor (ESP32-S3)
```

It is not in the default build: the demo is enabled by `LV_USE_DEMO_BENCHMARK`
in the application's `lv_conf.h`, so it needs the separate configuration
directory `examples/conf-benchmark`, which the script selects through
`DEP_LV_CONFIG_PATH`. It costs about 900 KB of flash and must never ship in a
production image. See `oxivgl::demo` for the API, the two preconditions it
enforces, and why the demo cannot be called directly.

## Reproducing

```sh
cargo +esp -Zbuild-std=alloc,core build --target xtensa-esp32-none-elf \
    --release --features fire27,perf-probe --example widget_scale10
```

Flash and capture through m5stack-core's HIL harness, which locks the board by a
stable identity and fails a truncated capture rather than reading it as a pass:

```sh
export M5STACK_HIL_CONFIG=…/hil.toml
tools/hil.sh --board fire27 --ensure-image <elf> --capture 20
```

Never address a board by `/dev/ttyACM*` — the index is assigned by discovery
order and renumbers on replug (`conventions/testing.md` §1, §8.1).

Two limitations of this setup, stated so results are not over-read:

* **`--ensure-image` still cannot *verify* an image, for a different reason
  than it first appears.** With m5stack-core v0.5.0's `identity` feature the
  examples now do name themselves, and the marks match exactly
  (`oxivgl/ex/<commit>` on both the board and the ELF). What does not match is
  the descriptor's ELF hash: the board reports
  `app_elf_sha256=000000000000` while the image carries a real hash, so the
  guard reports "flash did not take" even though the flash plainly worked (the
  app runs and captures are clean). The harness's own message names this case
  and says not to paper over it, so it is recorded here rather than worked
  around — see the upstream issue. Until it is resolved, use `--capture`, and
  treat the mode string in each stats line (`[shared executor]` versus
  `[threads+semaphore]`) as the image fingerprint.

  Note also that the identity prefix is a fixed `oxivgl/ex`, not
  `CARGO_PKG_NAME/CARGO_BIN_NAME`: the mark must fit `EspAppDesc::version`'s 31
  bytes, and `oxivgl/widget_buttonmatrix1/` alone is 28. The git mark, not the
  example name, is what distinguishes builds.
* **The CoreS3 console ring can overrun during the flash gap**
  (`[CONSOLE-DROP …]`). The harness declares such a capture unusable; re-capture
  without reflashing rather than reading the partial transcript.

## Two knobs this also exposes

* `display::set_refresh_period(ms)` retunes LVGL's redraw timer at runtime,
  rather than through the application-wide `lv_conf.h`. The stock 32 ms caps the
  display at 31 fps before any drawing cost is counted.
* `view::RenderConfig` sets the loop cadence, and `view::Ui` separates display
  setup (`init`) from the blocking step (`refresh`) and the async event loop
  (`run_events`/`run_events_nav`) — which is what lets an application put the
  blocking half on a thread of its own while keeping this pipeline.
