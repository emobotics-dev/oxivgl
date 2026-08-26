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
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

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
static PRESENTED: AtomicBool = AtomicBool::new(false);

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

/// LVGL DIRECT flush: pixels are already in `px_map`. Last area of a frame:
/// queue the swap only. Vblank wait is [`wait_after_handler`].
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
        PRESENTED.store(true, Ordering::Release);
    }
    unsafe { lv_display_flush_ready(disp) };
}

/// If this tick presented a frame, park on that vblank. Returns whether it waited.
pub(crate) fn wait_after_handler() -> bool {
    if !PRESENTED.swap(false, Ordering::AcqRel) {
        return false;
    }
    if let Some(scan) = scan_out() {
        scan.wait_presented();
        true
    } else {
        false
    }
}
