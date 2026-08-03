// SPDX-License-Identifier: MIT OR Apache-2.0
//! Where the UI runs, and how it ranks against everything else (oxivgl#1).
//!
//! This lives in the example harness, not in oxivgl: thread creation, the
//! priority ladder and core pinning belong to the application or BSP, because
//! only they know what the UI must yield to. oxivgl supplies the blocking wait
//! ([`FlushSync`](oxivgl::flush_pipeline::FlushSync)); placement is here.
//!
//! **Threads, not another `InterruptExecutor`.** An interrupt executor makes the
//! UI preempt *everything*, which is backwards — latency-sensitive work has to
//! win. A thread is preemptible by priority in both directions.
//!
//! Blocking the flush wait only pays off in combination with this ladder. On its
//! own it stops the core being parked; ranked correctly it also stops the UI
//! outranking the work it exists to yield to.

use core::ffi::c_void;

/// Task priorities.
///
/// `#[esp_rtos::main]` starts at **0 — the lowest** — so the app executor must
/// be raised explicitly or the UI threads would outrank the very work they
/// exist to yield to. This is the single easiest thing to get wrong: skip
/// [`raise_app_executor`] and moving the render loop to a thread makes latency
/// *worse*, not better.
///
/// The ladder stays inside 1..=3 on purpose. esp-radio's blob threads are
/// created at the priority the blob asks for (ESP-IDF convention puts them in
/// the low 20s), so keeping the UI down here means it cannot starve the radio.
pub const PRIO_APP: usize = 3;
/// Flush thread — above render, so the panel never starves waiting for pixels.
pub const PRIO_FLUSH: u32 = 2;
/// Render thread — yields to everything above it.
pub const PRIO_RENDER: u32 = 1;

/// LVGL's draw path is the deep one; the flush side only moves bytes. Sized
/// from measurements (~27.7 kB and ~8.6 kB of SRAM actually consumed), not
/// guessed.
pub const RENDER_STACK: usize = 20 * 1024;
/// Stack for the flush thread — it only moves bytes to the panel.
pub const FLUSH_STACK: usize = 8 * 1024;

/// Which core the render thread runs on. LVGL stays single-threaded either
/// way — one thread makes every LVGL call; this only selects which core that
/// thread sits on. Pinned to PRO (0): moving it to APP is a further win on
/// ESP32-S3 but hangs on ESP32, so it is deliberately not offered here.
pub const RENDER_CORE: u32 = 0;

/// Raise the calling thread — the `#[esp_rtos::main]` executor — above the UI.
///
/// Call this **before** spawning the UI threads, or they briefly outrank the
/// app executor.
pub fn raise_app_executor() {
    esp_rtos::CurrentThreadHandle::get().set_priority(PRIO_APP);
}

/// Spawn a native thread pinned to `core`.
///
/// # Safety
/// `entry` must never return: the thread is never joined and never deleted.
pub unsafe fn spawn(
    name: &str,
    entry: extern "C" fn(*mut c_void),
    prio: u32,
    stack: usize,
    core: u32,
) {
    unsafe {
        esp_radio_rtos_driver::task_create(
            name,
            entry,
            core::ptr::null_mut(),
            prio,
            Some(core),
            stack,
        );
    }
}
