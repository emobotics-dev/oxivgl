// SPDX-License-Identifier: MIT OR Apache-2.0
// Formerly `lvgl_buffers` — renamed for clarity.
//! DMA-aligned render buffers and embedded display initialisation.
//!
//! Buffer types (`LvglBuf`, `LvglBuffers`) are target-independent.
//! `lvgl_disp_init` registers them with LVGL and (on ESP32) wires up the
//! flush pipeline from the `flush_pipeline` module.

use core::ffi::c_void;
use core::sync::atomic::{AtomicPtr, Ordering};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use oxivgl_sys::{
    lv_display_create, lv_display_set_buffers, lv_display_set_color_format, lv_display_t,
    lv_display_get_refr_timer, lv_timer_set_period,
    lv_color_format_t_LV_COLOR_FORMAT_RGB565_SWAPPED,
    lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
};
#[cfg(any(feature = "esp-hal", feature = "rtos-sem"))]
use oxivgl_sys::{lv_display_set_flush_cb, lv_display_set_flush_wait_cb};

/// Number of pixel rows per render stripe. Large value trades stack RAM for fewer flush calls.
// NOTE: this is a lot of buffer — reduces available stack RAM intentionally; easy to shrink later.
pub const COLOR_BUF_LINES: usize = 40;

/// Aligned render buffer; `BYTES` = `screen_w × COLOR_BUF_LINES × 2` (RGB565).
/// Caller allocates as a `static mut` so the pointer is valid for the LVGL display lifetime.
#[repr(align(16))]
pub struct LvglBuf<const BYTES: usize>(pub [u8; BYTES]);

impl<const BYTES: usize> core::fmt::Debug for LvglBuf<BYTES> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LvglBuf").finish_non_exhaustive()
    }
}

impl<const BYTES: usize> LvglBuf<BYTES> {
    /// Create a zeroed render buffer.
    pub const fn new() -> Self { Self([0; BYTES]) }
}

/// Pair of DMA-aligned render buffers. Parameterised by byte size so the caller
/// controls allocation using the actual screen width:
/// `LvglBuffers::<{SCREEN_W as usize * COLOR_BUF_LINES * 2}>`
pub struct LvglBuffers<const BYTES: usize> {
    /// First render buffer.
    pub buf1: LvglBuf<BYTES>,
    /// Second render buffer (double-buffering).
    pub buf2: LvglBuf<BYTES>,
}

impl<const BYTES: usize> core::fmt::Debug for LvglBuffers<BYTES> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LvglBuffers").finish_non_exhaustive()
    }
}

impl<const BYTES: usize> LvglBuffers<BYTES> {
    /// Create zeroed double-buffered render buffers.
    pub const fn new() -> Self { Self { buf1: LvglBuf::new(), buf2: LvglBuf::new() } }
}

/// Signalled by the flush task (ESP32) or immediately (host) once the display
/// driver is ready. [`crate::view::run_app`] waits on this before entering
/// the render loop.
pub static DISPLAY_READY: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// The display created by [`lvgl_disp_init`], kept so the refresh period can be
/// retuned at runtime. Null until init runs. Single-display limit, as above.
static ACTIVE_DISPLAY: AtomicPtr<lv_display_t> = AtomicPtr::new(core::ptr::null_mut());

/// Record the active display so [`set_refresh_period`] can find it.
pub(crate) fn set_active_display(disp: *mut lv_display_t) {
    ACTIVE_DISPLAY.store(disp, Ordering::Release);
}

/// Set LVGL's redraw period, in milliseconds, at runtime.
///
/// This is the frame-rate ceiling: LVGL renders at most one frame per period,
/// so the stock `LV_DEF_REFR_PERIOD` of 32 ms caps the display at **31 fps
/// before any drawing cost is counted** — under 30 fps once real draw load is
/// added. Lowering the period raises the ceiling and costs proportionally more
/// CPU; raising it is the single most effective way to buy CPU back (holding
/// 31 fps instead of 59 roughly halves render load).
///
/// Setting it here rather than in `lv_conf.h` keeps the choice per-application:
/// `lv_conf.h` is owned by the application but shared by everything it builds,
/// whereas this is per-display and changeable while running.
///
/// Must be called after the display exists and from the LVGL task, like every
/// other LVGL call. Returns `false` if no display has been initialised yet, or
/// if the display has no refresh timer.
pub fn set_refresh_period(ms: u32) -> bool {
    let disp = ACTIVE_DISPLAY.load(Ordering::Acquire);
    if disp.is_null() {
        error!("set_refresh_period: no display initialised yet");
        return false;
    }
    // SAFETY: `disp` came from `lv_display_create` and is valid for the display
    // lifetime; called on the LVGL task, the only context that touches LVGL.
    unsafe {
        let timer = lv_display_get_refr_timer(disp);
        if timer.is_null() {
            error!("set_refresh_period: display has no refresh timer");
            return false;
        }
        lv_timer_set_period(timer, ms);
    }
    true
}

/// Register render buffers with LVGL and wire up the flush pipeline.
///
/// # Safety
/// `lv_init()` must have been called. Call at most once. `bufs` must be
/// `'static` so the pointers remain valid for the LVGL display lifetime.
/// `w` and `h` are the display resolution in pixels.
pub unsafe fn lvgl_disp_init<const BYTES: usize>(
    w: i32,
    h: i32,
    bufs: &'static mut LvglBuffers<BYTES>,
) {
    // SAFETY: addr_of_mut! obtains raw pointers without creating &mut references.
    // Caller guarantees single-call, lv_init() precondition, and 'static lifetime.
    unsafe {
        let buf1_ptr = core::ptr::addr_of_mut!(bufs.buf1) as *mut c_void;
        let buf2_ptr = core::ptr::addr_of_mut!(bufs.buf2) as *mut c_void;

        assert_eq!(buf1_ptr as usize % 4, 0, "DMA buffer must be 4-byte aligned");

        let disp = lv_display_create(w, h);
        assert!(!disp.is_null(), "lv_display_create returned NULL");

        lv_display_set_color_format(disp, lv_color_format_t_LV_COLOR_FORMAT_RGB565_SWAPPED);
        lv_display_set_buffers(
            disp,
            buf1_ptr,
            buf2_ptr,
            BYTES as u32,
            lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
        );
        set_active_display(disp);
        #[cfg(any(feature = "esp-hal", feature = "rtos-sem"))]
        {
            use crate::flush_pipeline::{flush_callback, wait_callback};
            lv_display_set_flush_cb(disp, Some(flush_callback));
            lv_display_set_flush_wait_cb(disp, Some(wait_callback));
        }
        // Without a flush pipeline the flush task never runs; signal ready immediately.
        #[cfg(not(any(feature = "esp-hal", feature = "rtos-sem")))]
        DISPLAY_READY.signal(());
    }
}
