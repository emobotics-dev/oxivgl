// SPDX-License-Identifier: MIT OR Apache-2.0
//! Measurement apparatus for the threaded render pipeline (oxivgl#1).
//!
//! Enabled by the `perf-probe` feature, so only a build that is being measured
//! pays for it. Two numbers matter:
//!
//! * **Probe latency** — the headline. A task wakes on a fixed 10 ms period and
//!   records how late it actually ran. It stands in for the latency-sensitive
//!   application work the UI must not disturb. With the stock pipeline the
//!   render task parks the core in `waiti 0` for the whole panel transfer, so
//!   this task simply does not run for 15-30 ms at a stretch.
//! * **Flush throughput** — ops/s and kB/s actually pushed to the panel,
//!   counted in the display driver where they can be observed honestly.
//!
//! Deliberately *not* reported as a frame rate: with PARTIAL render mode LVGL
//! splits one refresh into however many stripes the dirty area needs, so
//! flush-ops is not frames and calling it fps would overclaim.

use core::sync::atomic::{AtomicU32, Ordering::Relaxed};

/// Panel transfers completed.
pub static FLUSH_OPS: AtomicU32 = AtomicU32::new(0);
/// Bytes pushed to the panel.
pub static FLUSH_BYTES: AtomicU32 = AtomicU32::new(0);

/// Record one completed panel transfer.
pub fn record_flush(bytes: usize) {
    FLUSH_OPS.fetch_add(1, Relaxed);
    FLUSH_BYTES.fetch_add(bytes as u32, Relaxed);
}

/// Wakeup-latency accumulator. Plain atomics rather than a lock: the probe
/// writes, the stats task drains, and neither may block the other.
pub struct Latency {
    count: AtomicU32,
    sum_us: AtomicU32,
    max_us: AtomicU32,
    over_5ms: AtomicU32,
    over_20ms: AtomicU32,
}

/// One drained interval.
#[derive(Debug, Default, Clone, Copy)]
pub struct LatencySample {
    /// Wakeups observed.
    pub count: u32,
    /// Mean lateness, microseconds.
    pub mean_us: u32,
    /// Worst lateness, microseconds.
    pub max_us: u32,
    /// Wakeups later than 5 ms.
    pub over_5ms: u32,
    /// Wakeups later than 20 ms — a missed deadline by any measure.
    pub over_20ms: u32,
}

impl Latency {
    /// Create an empty accumulator.
    pub const fn new() -> Self {
        Self {
            count: AtomicU32::new(0),
            sum_us: AtomicU32::new(0),
            max_us: AtomicU32::new(0),
            over_5ms: AtomicU32::new(0),
            over_20ms: AtomicU32::new(0),
        }
    }

    /// Record one wakeup that was `us` microseconds late.
    pub fn record(&self, us: u32) {
        self.count.fetch_add(1, Relaxed);
        self.sum_us.fetch_add(us, Relaxed);
        self.max_us.fetch_max(us, Relaxed);
        if us > 5_000 {
            self.over_5ms.fetch_add(1, Relaxed);
        }
        if us > 20_000 {
            self.over_20ms.fetch_add(1, Relaxed);
        }
    }

    /// Drain and reset.
    pub fn take(&self) -> LatencySample {
        let count = self.count.swap(0, Relaxed);
        let sum_us = self.sum_us.swap(0, Relaxed);
        LatencySample {
            count,
            mean_us: if count > 0 { sum_us / count } else { 0 },
            max_us: self.max_us.swap(0, Relaxed),
            over_5ms: self.over_5ms.swap(0, Relaxed),
            over_20ms: self.over_20ms.swap(0, Relaxed),
        }
    }
}

impl Default for Latency {
    fn default() -> Self {
        Self::new()
    }
}

/// The probe's latency record.
pub static LATENCY: Latency = Latency::new();

/// Drain the flush counters, returning `(ops, kbytes)` for the interval.
pub fn take_flush() -> (u32, u32) {
    (FLUSH_OPS.swap(0, Relaxed), FLUSH_BYTES.swap(0, Relaxed) / 1024)
}
