# oxivgl v0.8.0

One defect and what it took to fix properly: the LVGL render task **parked the
CPU core** for the entire 15–30 ms panel transfer.

## The flush wait is injected, not chosen (#1)

LVGL calls `flush_wait_cb` on its *synchronous* C stack, so the render task
cannot `.await` — it has to block. The original wait used the Xtensa `waiti 0`
instruction, which does not yield: it **halts the core** until the next
interrupt. The scheduler is never entered, so for the whole transfer nothing
runs but ISRs. It is invisible in CPU-load measurements because the core is not
busy — it is stopped.

Which primitive is available depends on the scheduler the application links, so
oxivgl now takes it as a parameter:

```rust
pub trait FlushSync: Sync {
    fn wait(&self);    // render task, LVGL's synchronous C stack
    fn signal(&self);  // flush context — may be interrupt context
}
```

| implementation | blocks by | while waiting |
|---|---|---|
| `WaitiFlushSync` (default) | `waiti 0` | the core is **parked** |
| `SemaphoreFlushSync` (`rtos-sem`) | RTOS semaphore | the render task leaves the run queue |

Measured on Fire27, same UI, with a 10 ms probe task standing in for
latency-sensitive work — it should see 100 wakeups per second:

| | wakeups/s | mean late | max late | flush |
|---|---:|---:|---:|---|
| `waiti` + shared executor | 65–70 | 5872 µs | 36 997 µs | 33 ops/s, 400 kB/s |
| threads + semaphore | **100** | **84 µs** | **190 µs** | 33 ops/s, 417 kB/s |

Flush throughput is **identical either way** — the latency is not bought with
frame rate. That is the signature of a blocking problem, not a throughput one:
the baseline was never CPU-starved, it was parked.

`rtos-sem` pulls `esp-radio-rtos-driver`, which is **not an RTOS** but
Espressif's scheduler *interface* crate — it declares symbols a scheduler
registers at link time, so linking none is a link error rather than a silent
fallback. `lv_display_flush_ready` is still called from the render task, which
keeps `LV_USE_OS LV_OS_NONE` correct even with the flush on its own thread.

## A render loop that does not cost a period per frame

The loop ran a fixed `4 × (lv_timer_handler + sleep LV_DEF_REFR_PERIOD/4)` per
`update()`, so a cycle cost `LV_DEF_REFR_PERIOD` **plus** render time — 31 fps at
the stock 32 ms before drawing anything. It now paces on the delay
`lv_timer_handler` returns, bounded by `RenderConfig::max_idle_ms`.

`Ui` splits display setup from the loop, so an application can place the loop on
a thread of its own:

```rust
Ui::init(W, H, bufs)
    .run(MyView::default(), RenderConfig::default().with_target_fps(60))
    .await
```

Placement stays the application's job — only it knows what the UI must yield to.
`examples/common/src/sched.rs` shows the bench ladder (app 3 / flush 2 /
render 1). `#[esp_rtos::main]` starts at priority **0**, the lowest, so moving
the render loop to a thread *without* raising the app executor first makes
latency worse, not better.

## ⚠️ Upgrade notes

- **Render-loop timing changes for every existing `run_app` caller.** Signatures
  are unchanged and all 196 examples build untouched, but the pacing differs —
  that is the fix. Re-take any timing measurements against 0.7.0 or earlier.
- **The `waiti` fallback is still the default.** It now warns once at flush-task
  start when no `FlushSync` is registered. To opt in, enable `rtos-sem` and
  register a `SemaphoreFlushSync`.
- **`rtos-sem` needs the consumer to enable `esp-rtos/esp-radio`** — that is
  where the semaphore symbols are registered. Despite the name it pulls no radio
  blob:
  ```toml
  esp-rtos = { version = "0.3", features = ["embassy", "esp-radio", "esp-alloc"] }
  ```
- **`oxivgl-build` is now 0.1.1** and oxivgl requires it — `build.rs` calls the
  new `emit_identity_mark`.
- **Examples move to `m5stack-core` 0.6.0 and esp-hal 1.1.2.** The esp32s3 PAC
  renamed `USB0` and `EXT_WAKEUP1`, which the previously pinned esp-hal rev
  predates, so the cores3 example build stopped compiling inside esp-hal. Both
  repositories now name the same fork rev, so esp-hal resolves to exactly one
  copy. Affects building the examples only; library consumers are unaffected —
  the published crate depends on neither m5stack-core nor the fork.

## Fixed

- **`with_target_fps` capped idling at the whole refresh period** while the
  default caps at a quarter. The cap, not the refresh period, binds: measured on
  CoreS3 at 31 fps with a 10 ms cap against 19 fps at 50 ms.
- **`run_lvgl` was documented but never existed** — in the README and CLAUDE.md,
  across several releases. The entry point is `run_app`, or `Ui`.
- **`RenderConfig` claimed the defaults left cadence unchanged.** They do not.

## Known limitations

- **`--ensure-image` cannot verify a flashed image.** The marks match, but the
  board reports `app_elf_sha256` as zeros while the image carries a real hash,
  so the guard reports "flash did not take" on a flash that worked. Use
  `--capture` and treat the mode string in each stats line as the fingerprint.
- **`WaitiFlushSync`'s wakeup can be one interrupt late.** A `signal` landing
  between the flag check and `waiti 0` defers the wakeup to the next interrupt.
  Bounded and pre-existing; `SemaphoreFlushSync` sidesteps it.
- **The numbers above are one workload.** Flush ops/s is a coarse proxy for
  frame rate — `PARTIAL` mode splits a refresh into as many stripes as the dirty
  area needs, so they are deliberately not reported as fps.
