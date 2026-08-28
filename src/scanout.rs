// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scan-out pipeline for panels that DMA-scan a framebuffer (RGB parallel, DSI).
//!
//! SPI and similar buses copy dirty stripes onto the wire — that is
//! the flush pipeline (`esp-hal` / `rtos-sem`). A scan-out panel already
//! has the pixels: LVGL
//! draws in `DIRECT` mode into the buffers the scan engine reads, and on the
//! last flush of a frame the application points the engine at the finished
//! buffer and waits for vblank. No stripe copy, no flush thread.

use core::cell::RefCell;
use core::ffi::c_void;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use oxivgl_sys::{lv_area_t, lv_display_flush_is_last, lv_display_flush_ready, lv_display_t};

/// A continuously-scanning panel. Implemented by the board, called from LVGL's
/// flush callback on the render thread.
pub trait ScanOut: Sync {
    /// Point the scan engine at `fb` from the next vblank. Must not block.
    fn present(&self, fb: NonNull<u8>);

    /// Block until that buffer is being scanned. Render thread, **after**
    /// `lv_timer_handler` — not inside the flush callback (that wait was
    /// counted as LVGL flush time and then slept again).
    fn wait_presented(&self);
}

static SCAN_OUT: Mutex<CriticalSectionRawMutex, RefCell<Option<&'static dyn ScanOut>>> =
    Mutex::new(RefCell::new(None));

/// Display whose last flush is queued but not yet on the panel.
static AWAITING: AtomicPtr<c_void> = AtomicPtr::new(core::ptr::null_mut());

/// Report the queued frame as flushed. Call when the swap has landed - from
/// the panel's vblank handler, interrupt context included.
///
/// Until this is called LVGL considers the flush outstanding, which is what
/// makes its own `wait_for_flushing` mean something.
pub fn notify_presented() {
    let disp = AWAITING.swap(core::ptr::null_mut(), Ordering::AcqRel);
    if !disp.is_null() {
        // SAFETY: the pointer LVGL passed to the flush callback; still valid.
        unsafe { lv_display_flush_ready(disp.cast()) };
    }
}

/// Register the scan-out panel. Call before [`crate::view::Ui::init`].
///
/// Returns `false` if one was already registered (the existing one is kept).
pub fn set_scan_out(scan: &'static dyn ScanOut) -> bool {
    SCAN_OUT.lock(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_some() {
            return false;
        }
        *slot = Some(scan);
        true
    })
}

fn scan_out() -> Option<&'static dyn ScanOut> {
    SCAN_OUT.lock(|slot| *slot.borrow())
}

/// Block until the last [`ScanOut::present`] is on the pins. Render thread,
/// after `lv_timer_handler` — never inside flush (that wait was counted as
/// LVGL flush time).
pub fn wait_presented() {
    if let Some(scan) = scan_out() {
        scan.wait_presented();
    }
}

/// LVGL DIRECT flush: pixels are already in `px_map`. Last area of a frame:
/// queue the swap only. The render thread waits vblank after the handler.
pub(crate) unsafe extern "C" fn flush_callback(
    disp: *mut lv_display_t,
    _area: *const lv_area_t,
    px_map: *mut u8,
) {
    if disp.is_null() || px_map.is_null() {
        return;
    }
    // SAFETY: `disp` is LVGL's display; this runs on the render thread.
    let last = unsafe { lv_display_flush_is_last(disp) };
    if last && let (Some(scan), Some(fb)) = (scan_out(), NonNull::new(px_map)) {
        scan.present(fb);
        // Do not report the flush yet. LVGL calls `wait_for_flushing` before
        // synchronising the two buffers in DIRECT double-buffered mode, and
        // reporting at queue time makes that a no-op - LVGL then treats a
        // buffer the panel is still scanning as free. The board calls
        // `notify_presented` when the swap has actually landed.
        //
        // Waiting here instead would be simpler and is wrong: the wait counts
        // as LVGL flush time and the render thread sleeps on it a second time.
        AWAITING.store(disp.cast(), Ordering::Release);
        return;
    }
    unsafe { lv_display_flush_ready(disp) };
}
