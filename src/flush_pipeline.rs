// SPDX-License-Identifier: MIT OR Apache-2.0
//! ESP32 flush pipeline: async DMA transfer between LVGL and the display driver.
//!
//! LVGL's `flush_callback` (called from the render task) sends pixel data through
//! [`DRAW_OPERATION`] to [`flush_frame_buffer`] (running on a high-priority
//! interrupt executor or its own RTOS thread), which forwards it to the board's
//! [`DisplayOutput`] implementation. Completion is handed back through a
//! [`FlushSync`], which is what unblocks the render task.
//!
//! # Why the wait is injected
//!
//! `flush_wait_cb` runs on LVGL's *synchronous* C stack, so it cannot `.await`
//! — it needs a real blocking primitive. Which primitive is available depends on
//! the scheduler the application links, so oxivgl takes it as a parameter rather
//! than picking one. See [`FlushSync`] for the trade-off between the two shipped
//! implementations; the short version is that the default [`WaitiFlushSync`]
//! *parks the core* for the whole transfer, and [`SemaphoreFlushSync`] does not.
//!
//! # What keeps LVGL single-threaded
//!
//! The flush side only moves bytes. `lv_display_flush_ready` is still called
//! from the render task, in [`FlushSync::wait`]'s caller — never from the flush
//! context. That is what lets `LV_USE_OS LV_OS_NONE` stay correct even when the
//! flush runs on its own thread.

use core::cell::UnsafeCell;
use core::slice::from_raw_parts;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use oxivgl_sys::{lv_area_t, lv_display_flush_ready, lv_display_t};

/// Error type for display output operations.
#[derive(Debug)]
pub enum UiError {
    /// Display output failed.
    Display,
}

/// Trait abstracting the raw pixel-data display output.
/// Defined in ui; implemented by the board layer.
#[allow(async_fn_in_trait)]
pub trait DisplayOutput {
    /// Write raw pixel data to the display.
    async fn show_raw_data(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
        data: &[u8],
    ) -> Result<(), UiError>;
}

// ---------------------------------------------------------------------------
// FlushSync — the render↔flush handoff primitive
// ---------------------------------------------------------------------------

/// The blocking handoff between the flush context and the LVGL render task.
///
/// LVGL calls `flush_wait_cb` on its synchronous C stack, so the render task
/// cannot `.await` there — it must *block*. This trait is that blocking
/// primitive, supplied by the application so oxivgl need not depend on a
/// particular scheduler.
///
/// Two implementations ship with the library:
///
/// | | blocks by | while waiting |
/// |---|---|---|
/// | [`WaitiFlushSync`] (default) | `waiti 0` | **the core is parked** — the scheduler is never entered, so nothing runs but ISRs |
/// | [`SemaphoreFlushSync`] (`rtos-sem`) | an RTOS semaphore | the render task leaves the run queue; every other thread runs normally |
///
/// A panel transfer is 15–30 ms, so the difference is not subtle: with
/// `waiti 0` that is 15–30 ms in which no application work happens at all.
/// Prefer [`SemaphoreFlushSync`] whenever a scheduler is linked.
///
/// # Implementing this trait
///
/// [`wait`](Self::wait) is called on the render task and must block until a
/// [`signal`](Self::signal) that has not yet been consumed. A `signal` that
/// arrives *before* the matching `wait` must not be lost — the render task may
/// reach `wait` after the transfer already finished. A binary/counting
/// semaphore with capacity 1 has exactly this behaviour.
///
/// [`signal`](Self::signal) is called from the flush context, which may be
/// interrupt context (an `InterruptExecutor` task) — so an implementation must
/// use whatever ISR-safe form its scheduler requires.
pub trait FlushSync: Sync {
    /// Block the render task until the pending flush completes.
    ///
    /// Called from LVGL's `flush_wait_cb`, on the render task's stack.
    fn wait(&self);

    /// Release the render task blocked in [`wait`](Self::wait).
    ///
    /// Called from the flush context once the pixel data has reached the panel.
    /// May run in interrupt context — see the trait docs.
    fn signal(&self);
}

/// The default [`FlushSync`]: blocks with the Xtensa `waiti 0` instruction.
///
/// **This parks the core.** `waiti 0` halts the CPU until the next interrupt
/// rather than yielding, so the scheduler is never entered and for the entire
/// 15–30 ms panel transfer nothing runs except ISRs. It is invisible in CPU
/// load measurements because the core is not busy — it is stopped.
///
/// It is the default only because it needs no scheduler at all. Any application
/// that links one should use [`SemaphoreFlushSync`] instead.
///
/// # The wakeup can be one interrupt late
///
/// [`signal`](FlushSync::signal) can land in the window between `wait`'s check
/// of the flag and the `waiti 0` that follows it. The core then sleeps until the
/// *next* interrupt rather than returning at once, and the loop only notices the
/// completion on the following pass. It is bounded — some interrupt always
/// arrives, so this is added latency and never a hang — and the behaviour is
/// unchanged from before the flush wait was made injectable. Closing the window
/// properly needs the flag tested with interrupts masked and `waiti` entered
/// atomically from that state; [`SemaphoreFlushSync`] sidesteps it entirely by
/// letting the scheduler do the waiting.
#[derive(Debug)]
pub struct WaitiFlushSync {
    /// Set by `signal`, consumed by `wait`. Carries a `signal` that arrives
    /// before its `wait`, so the handoff cannot be missed.
    pending: AtomicBool,
}

impl WaitiFlushSync {
    /// Create the `waiti`-based sync.
    pub const fn new() -> Self {
        Self { pending: AtomicBool::new(false) }
    }
}

impl Default for WaitiFlushSync {
    fn default() -> Self {
        Self::new()
    }
}

impl FlushSync for WaitiFlushSync {
    #[cfg_attr(feature = "esp-hal", esp_hal::ram)]
    fn wait(&self) {
        loop {
            // Acquire pairs with the Release in `signal`, so the pixel data the
            // flush side wrote is visible before we report completion to LVGL.
            if self.pending.swap(false, Ordering::Acquire) {
                return;
            }
            // SAFETY: parks the core until the next interrupt. Xtensa `waiti 0`;
            // RISC-V `wfi`. No critical section is held.
            #[cfg(all(target_os = "none", target_arch = "xtensa"))]
            unsafe {
                core::arch::asm!("waiti 0")
            };
            #[cfg(all(target_os = "none", target_arch = "riscv32"))]
            unsafe {
                core::arch::asm!("wfi")
            };
        }
    }

    #[cfg_attr(feature = "esp-hal", esp_hal::ram)]
    fn signal(&self) {
        self.pending.store(true, Ordering::Release);
    }
}

/// A [`FlushSync`] backed by an RTOS semaphore — the one to use.
///
/// The render task blocks in the scheduler instead of halting the core, so the
/// 15–30 ms panel transfer costs the rest of the system nothing. Measured
/// against a 10 ms-period probe task on a CoreS3, swapping [`WaitiFlushSync`]
/// for this (with a render thread below the application's priority) moved the
/// probe from 4700 µs mean / 86 of 100 wakeups to 119 µs / 100 of 100.
///
/// # Linking requirement
///
/// Requires the `rtos-sem` feature **and** a scheduler that registers an
/// `esp-radio-rtos-driver` semaphore implementation. oxivgl deliberately does
/// not depend on a scheduler, so the application supplies it:
///
/// ```toml
/// # the app's Cargo.toml — esp-rtos registers the semaphore symbols from a
/// # module gated behind this feature. It pulls no radio blob; the feature name
/// # refers to esp-rtos's FreeRTOS-compat IPC objects.
/// esp-rtos = { version = "0.4", features = ["embassy", "esp-radio", "esp-alloc"] }
/// ```
///
/// The interface is scheduler-neutral by construction — ArielOS and
/// ESP-IDF/FreeRTOS shims register the same symbols. With none linked, the
/// build fails at link time on undefined `esp_rtos_semaphore_*`, which is the
/// honest failure mode: a missing scheduler is a build error, never a silent
/// fallback to parking the core.
///
/// # Blocking the wait is only half the win
///
/// This stops the core being parked, which alone is a large improvement. Coming
/// all the way down to ~119 µs additionally needs the render loop on its own
/// thread *below* the latency-sensitive work: `#[esp_rtos::main]` starts at
/// priority **0**, the lowest, so an application that does not raise itself
/// would sit *under* the render thread and be starved by it. Thread creation
/// and the priority ladder are the application's to own; see the threaded
/// harness in `examples/common` for a worked ladder.
#[cfg(feature = "rtos-sem")]
#[derive(Debug)]
pub struct SemaphoreFlushSync {
    /// Leaked at construction, so the semaphore outlives every borrow.
    sem: esp_radio_rtos_driver::semaphore::SemaphorePtr,
    /// Whether [`signal`](FlushSync::signal) runs in interrupt context. Giving a
    /// semaphore from an ISR requires the `_from_isr` form; using the wrong one
    /// is a scheduler-level error, so the caller states which context applies.
    from_isr: bool,
}

// SAFETY: `sem` is an opaque handle to an RTOS semaphore, which is precisely an
// object built to be used from several threads and from interrupt context at
// once — the scheduler provides the internal synchronisation, and every
// `SemaphoreHandle` method takes `&self` for that reason. Sharing it is the
// intended use (esp-radio drives the same interface this way); `NonNull` is
// merely not `Sync` by default because the compiler cannot know that.
#[cfg(feature = "rtos-sem")]
unsafe impl Sync for SemaphoreFlushSync {}

#[cfg(feature = "rtos-sem")]
impl SemaphoreFlushSync {
    /// Create the semaphore and leak it, for a flush that runs **in interrupt
    /// context** — i.e. [`flush_frame_buffer`] spawned on an
    /// `InterruptExecutor`. This is the shape the stock board harness uses.
    pub fn leak_isr() -> &'static Self {
        Self::leak(true)
    }

    /// Create the semaphore and leak it, for a flush that runs **on its own
    /// RTOS thread** rather than an interrupt executor. Pair this with a flush
    /// thread ranked just above the render thread.
    pub fn leak_thread() -> &'static Self {
        Self::leak(false)
    }

    fn leak(from_isr: bool) -> &'static Self {
        use esp_radio_rtos_driver::semaphore::{SemaphoreHandle, SemaphoreKind};
        // Capacity 1, initially empty: a `give` that lands before its `take`
        // stays pending, so a transfer that finishes before the render task
        // reaches `wait` is not lost.
        let sem = SemaphoreHandle::new(SemaphoreKind::Counting { max: 1, initial: 0 });
        // Leak: the semaphore is registered with LVGL for the display lifetime
        // and must never be dropped while the callbacks can still run.
        alloc::boxed::Box::leak(alloc::boxed::Box::new(Self { sem: sem.leak(), from_isr }))
    }

    /// Borrow the leaked semaphore for one call.
    fn handle(&self) -> &esp_radio_rtos_driver::semaphore::SemaphoreHandle {
        // SAFETY: `self.sem` came from `SemaphoreHandle::leak` in `leak` above
        // and is never deleted, so the pointee outlives this borrow.
        unsafe { esp_radio_rtos_driver::semaphore::SemaphoreHandle::ref_from_ptr(&self.sem) }
    }
}

#[cfg(feature = "rtos-sem")]
impl FlushSync for SemaphoreFlushSync {
    fn wait(&self) {
        // `None` = wait forever. A lost completion would hang the UI rather
        // than corrupt it; the flush side always signals, including on error.
        self.handle().take(None);
    }

    fn signal(&self) {
        let ok = if self.from_isr {
            self.handle().try_give_from_isr(None)
        } else {
            self.handle().give()
        };
        if !ok {
            error!("flush completion semaphore rejected the give");
        }
    }
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

/// Fallback used when the application registers nothing.
static DEFAULT_SYNC: WaitiFlushSync = WaitiFlushSync::new();

/// Holds the registered [`FlushSync`]. A `&dyn` is a fat pointer, so it cannot
/// live in an `AtomicPtr`; `set` publishes it instead.
struct FlushSyncSlot {
    /// Written only by [`set_flush_sync`], before any reader exists.
    sync: UnsafeCell<Option<&'static dyn FlushSync>>,
    /// Publishes `sync`: `Release` on write, `Acquire` on read.
    set: AtomicBool,
}

// SAFETY: `sync` is written once by `set_flush_sync` and thereafter only read.
// The `set` flag orders the write before every read (Release/Acquire), so the
// flush context — a different task, thread or ISR — observes a fully-written
// reference or none at all.
unsafe impl Sync for FlushSyncSlot {}

static FLUSH_SYNC: FlushSyncSlot =
    FlushSyncSlot { sync: UnsafeCell::new(None), set: AtomicBool::new(false) };

/// Register the [`FlushSync`] the render task blocks on.
///
/// Call **before** [`lvgl_disp_init`](crate::display::lvgl_disp_init) and before
/// spawning [`flush_frame_buffer`]; the registration is read by both. Calling it
/// once is the intended use — a second call is rejected rather than allowed to
/// race a live pipeline.
///
/// With no registration the pipeline falls back to [`WaitiFlushSync`], which
/// parks the core for the whole transfer. See [`FlushSync`].
///
/// Returns `false` if a sync was already registered (the existing one is kept).
pub fn set_flush_sync(sync: &'static dyn FlushSync) -> bool {
    if FLUSH_SYNC.set.load(Ordering::Acquire) {
        error!("set_flush_sync called twice — keeping the first registration");
        return false;
    }
    // SAFETY: no reader can observe the cell until `set` is published below,
    // and this is the only writer. See the `unsafe impl Sync` comment.
    unsafe { *FLUSH_SYNC.sync.get() = Some(sync) };
    FLUSH_SYNC.set.store(true, Ordering::Release);
    true
}

/// The registered [`FlushSync`], or the [`WaitiFlushSync`] fallback.
#[cfg_attr(feature = "esp-hal", esp_hal::ram)]
fn flush_sync() -> &'static dyn FlushSync {
    if FLUSH_SYNC.set.load(Ordering::Acquire) {
        // SAFETY: `set` is true, so `set_flush_sync` completed its write and the
        // Acquire above orders it before this read. The cell is never written
        // again. See the `unsafe impl Sync` comment.
        if let Some(sync) = unsafe { *FLUSH_SYNC.sync.get() } {
            return sync;
        }
    }
    &DEFAULT_SYNC
}

// ---------------------------------------------------------------------------
// Draw pipeline
// ---------------------------------------------------------------------------

/// Pixel data produced by LVGL's flush callback, consumed by [`flush_frame_buffer`].
#[derive(Debug)]
pub struct DrawOperation {
    /// Points into a `static mut LvglBuf` — the `'static` lifetime is truthful.
    /// Aliasing safety: LVGL's `flushing` flag (see `wait_for_flushing` in
    /// lv_refr.c) prevents buffer reuse until `lv_display_flush_ready()` clears it.
    /// `flush_frame_buffer` consumes this ref before calling [`FlushSync::signal`],
    /// which unblocks `wait_callback` → `flush_ready`. Do not store outside the
    /// flush pipeline.
    pub data: &'static [u8],
    /// X offset of the area in pixels.
    pub x: u16,
    /// Y offset of the area in pixels.
    pub y: u16,
    /// Width of the area in pixels.
    pub w: u16,
    /// Height of the area in pixels.
    pub h: u16,
}

// SAFETY: moved from the render task to the flush context and never aliased —
// LVGL's `flushing` flag holds the buffer until `lv_display_flush_ready`.
unsafe impl Send for DrawOperation {}

// NOTE: single-display limit — this static couples LVGL's flush pipeline to one
// display. A second simultaneous LVGL display is not supported.

/// Channel carrying rendered pixel stripes from LVGL to the flush task.
pub static DRAW_OPERATION: Channel<CriticalSectionRawMutex, DrawOperation, 1> = Channel::new();

/// Async flush task: receives pixel data from LVGL, forwards to [`DisplayOutput`].
///
/// Spawn this on a high-priority interrupt executor, or on its own RTOS thread
/// ranked just above the render thread. Signals
/// [`DISPLAY_READY`](super::display::DISPLAY_READY) once ready, then loops
/// forever consuming [`DRAW_OPERATION`] and writing to the display.
///
/// The context this runs in must match the registered [`FlushSync`] — see
/// [`SemaphoreFlushSync::leak_isr`] versus [`SemaphoreFlushSync::leak_thread`].
#[cfg_attr(feature = "esp-hal", esp_hal::ram)]
pub async fn flush_frame_buffer(mut display_driver: impl DisplayOutput) -> ! {
    debug!("Starting flush task");
    super::display::DISPLAY_READY.signal(());
    // Say so once, here rather than in `flush_sync`, which is on the per-flush
    // path. Falling back is legitimate but expensive enough to be worth
    // surfacing: measured on a Fire27, parking the core costs a 10 ms-period
    // task ~30 of every 100 wakeups. Someone who never reads a changelog should
    // still find out.
    if !FLUSH_SYNC.set.load(Ordering::Acquire) {
        warn!(
            "flush wait: no FlushSync registered — falling back to WaitiFlushSync, \
             which parks the core for the whole panel transfer. Register a \
             blocking one (see oxivgl::flush_pipeline::SemaphoreFlushSync) if a \
             scheduler is linked."
        );
    }
    let sync = flush_sync();
    loop {
        debug!("Flushing frame buffer");
        let DrawOperation { data, x, y, w, h } = DRAW_OPERATION.receive().await;
        if let Err(_e) = display_driver.show_raw_data(x, y, w, h, data).await {
            error!("show_raw_data failed");
        }

        // DO NOT call LVGL from here — this may be interrupt context, and LVGL
        // is single-threaded. Release the render task instead; it calls
        // lv_display_flush_ready() itself in wait_callback.
        // Signalled even on error: a dropped completion hangs the UI forever.
        sync.signal();
        debug!("Flush done");
    }
}

/// LVGL wait callback: blocks on the registered [`FlushSync`] until the flush
/// completes, then reports completion to LVGL.
#[cfg_attr(feature = "esp-hal", esp_hal::ram)]
pub(crate) unsafe extern "C" fn wait_callback(disp: *mut lv_display_t) {
    if disp.is_null() {
        error!("wait_callback: null disp");
        return;
    }
    flush_sync().wait();
    // SAFETY: `disp` is LVGL's own display pointer, valid for the display
    // lifetime, and this runs on the render task — the only context that
    // touches LVGL. The flush side has finished with the buffer by now, which
    // is exactly what the wait above established.
    unsafe {
        lv_display_flush_ready(disp);
    }
}

/// LVGL flush callback: packages pixel data and sends to the flush task.
#[cfg_attr(feature = "esp-hal", esp_hal::ram)]
pub(crate) unsafe extern "C" fn flush_callback(
    disp: *mut lv_display_t,
    area_p: *const lv_area_t,
    px_map: *mut u8,
) {
    if disp.is_null() || area_p.is_null() || px_map.is_null() {
        error!("flush_callback: null disp, area_p, or px_map");
        return;
    }
    // SAFETY: area_p is non-null (checked above); LVGL guarantees the lv_area_t
    // reference is valid for the duration of this callback.
    let area = unsafe { &*area_p };
    if area.x2 < area.x1 || area.y2 < area.y1 {
        error!("flush_callback: invalid area");
        return;
    }

    let w = (area.x2 - area.x1 + 1) as u16;
    let h = (area.y2 - area.y1 + 1) as u16;

    debug!("Flushing {} x {} ({};{} .. {};{})", w, h, area.x1, area.y1, area.x2, area.y2);

    let Some(len_pixels) = (w as usize).checked_mul(h as usize) else {
        error!("flush_callback: w*h overflowed");
        return;
    };

    // px_map is already byte-swapped by LVGL (RGB565_SWAPPED format).
    // Interpret as RGB565 bytes (2 per pixel).
    let data_bytes = len_pixels * 2;
    let op = DrawOperation {
        // SAFETY: px_map is non-null (checked above); points into one of the `static mut
        // LvglBuf` buffers registered via `lv_display_set_buffers` in `lvgl_disp_init` —
        // this is what makes the `'static` lifetime truthful. The aliasing invariant
        // (no concurrent LVGL writes) is upheld by the `flushing` flag; see the
        // `DrawOperation::data` doc comment.
        data: unsafe { from_raw_parts(px_map, data_bytes) },
        x: area.x1 as u16,
        y: area.y1 as u16,
        w,
        h,
    };
    // Believed unreachable: wait_callback blocks until flush_frame_buffer
    // drains the channel, so it is always empty when LVGL calls flush_callback.
    // If this fires, it indicates a protocol violation (e.g. flush task not
    // spawned). No recovery: calling flush_ready would lie (data not flushed),
    // not calling it deadlocks wait_callback. Log and let it hang — the error
    // message will be visible in the log output.
    if let Err(_e) = DRAW_OPERATION.try_send(op) {
        error!("DRAW_OPERATION channel full — should be unreachable");
    }
}
