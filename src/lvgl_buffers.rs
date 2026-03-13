// SPDX-License-Identifier: MIT OR Apache-2.0
use core::ffi::c_void;
#[cfg(feature = "esp-hal")]
use core::slice::from_raw_parts;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
#[cfg(feature = "esp-hal")]
use embassy_sync::channel::Channel;
use lvgl_rust_sys::{
    lv_display_create, lv_display_set_buffers, lv_display_set_color_format,
    lv_color_format_t_LV_COLOR_FORMAT_RGB565_SWAPPED,
    lv_display_render_mode_t_LV_DISPLAY_RENDER_MODE_PARTIAL,
};
#[cfg(feature = "esp-hal")]
use lvgl_rust_sys::{
    lv_area_t, lv_display_flush_ready, lv_display_set_flush_cb, lv_display_set_flush_wait_cb,
    lv_display_t,
};

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
    async fn show_raw_data(&mut self, x: u16, y: u16, w: u16, h: u16, data: &[u8]) -> Result<(), UiError>;
}

/// Number of pixel rows per render stripe. Large value trades stack RAM for fewer flush calls.
// NOTE: this is a lot of buffer — reduces available stack RAM intentionally; easy to shrink later.
pub const COLOR_BUF_LINES: usize = 40;

/// Aligned render buffer; `BYTES` = `screen_w × COLOR_BUF_LINES × 2` (RGB565).
/// Caller allocates as a `static mut` so the pointer is valid for the LVGL display lifetime.
#[repr(align(16))]
pub struct LvglBuf<const BYTES: usize>(pub [u8; BYTES]);

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

impl<const BYTES: usize> LvglBuffers<BYTES> {
    /// Create zeroed double-buffered render buffers.
    pub const fn new() -> Self { Self { buf1: LvglBuf::new(), buf2: LvglBuf::new() } }
}

#[cfg(feature = "esp-hal")]
#[derive(Debug)]
pub struct LvDispDrv(pub(crate) *mut lv_display_t); // encapsulate to allow impl Send
#[cfg(feature = "esp-hal")]
unsafe impl Send for LvDispDrv {}
#[cfg(feature = "esp-hal")]
unsafe impl Sync for LvDispDrv {}

#[cfg(feature = "esp-hal")]
#[derive(Debug)]
pub struct DrawOperation {
    disp_drv: LvDispDrv,
    /// Points into a `static mut LvglBuf` — the `'static` lifetime is truthful.
    /// Aliasing safety: LVGL's `flushing` flag (see `wait_for_flushing` in
    /// lv_refr.c) prevents buffer reuse until `lv_display_flush_ready()` clears it.
    /// `flush_frame_buffer` consumes this ref before signaling `FLUSH_OPERATION`,
    /// which unblocks `wait_callback` → `flush_ready`. Do not store outside the
    /// flush pipeline.
    pub data: &'static [u8],
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

// NOTE: single-display limit — these statics couple LVGL's flush pipeline to one
// display. A second simultaneous LVGL display is not supported.
#[cfg(feature = "esp-hal")]
pub static DRAW_OPERATION: Channel<CriticalSectionRawMutex, DrawOperation, 1> = Channel::new();
#[cfg(feature = "esp-hal")]
pub static FLUSH_OPERATION: Channel<CriticalSectionRawMutex, LvDispDrv, 1> = Channel::new();

/// Signalled by `flush_frame_buffer` once the display driver is ready.
/// `run_lvgl` waits on this before entering the render loop.
pub static DISPLAY_READY: Signal<CriticalSectionRawMutex, ()> = Signal::new();

/// SAFETY: `lv_init()` must have been called. Call at most once. `bufs` must be
/// `'static` so the pointers remain valid for the LVGL display lifetime.
/// `w` and `h` are the display resolution in pixels.
pub unsafe fn lvgl_disp_init<const BYTES: usize>(w: i32, h: i32, bufs: &'static mut LvglBuffers<BYTES>) {
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
        #[cfg(feature = "esp-hal")]
        lv_display_set_flush_cb(disp, Some(flush_callback));
        #[cfg(feature = "esp-hal")]
        lv_display_set_flush_wait_cb(disp, Some(wait_callback));
        // On non-esp-hal targets the flush task never runs; signal ready immediately.
        #[cfg(not(feature = "esp-hal"))]
        DISPLAY_READY.signal(());
    }
}

#[cfg(feature = "esp-hal")]
#[esp_hal::ram]
pub async fn flush_frame_buffer(mut display_driver: impl DisplayOutput) -> ! {
    debug!("Starting flush task");
    DISPLAY_READY.signal(());
    let flush_sender = FLUSH_OPERATION.sender();
    loop {
        debug!("Flushing frame buffer");
        let draw_operation = DRAW_OPERATION.receive().await;
        let DrawOperation { disp_drv, data, x, y, w, h } = draw_operation;
        if let Err(_e) = display_driver.show_raw_data(x, y, w, h, data).await {
            error!("show_raw_data failed");
        }

        // DO NOT call LVGL from interrupt context.
        // Just notify the LVGL thread to call lv_display_flush_ready().
        flush_sender.send(disp_drv).await;
        debug!("Flush done");
    }
}

#[cfg(feature = "esp-hal")]
#[esp_hal::ram]
unsafe extern "C" fn wait_callback(_disp: *mut lv_display_t) {
    // Wait for flush_frame_buffer (interrupt executor) to complete the SPI
    // transfer. Use `waiti 0` to sleep until the next interrupt instead of
    // busy-spinning — this lets the DMA/SPI completion interrupt wake us
    // and avoids wasting CPU cycles.
    // Use try_receive (non-blocking) + waiti rather than receive().await:
    // wait_callback runs on the LVGL task stack, not in an async context.
    // A blocking .await here would deadlock; try_receive + waiti 0 suspends
    // the core until the next interrupt without holding a critical section.
    loop {
        if let Ok(drv) = FLUSH_OPERATION.try_receive() {
            // SAFETY: drv.0 is the lv_display_t pointer originally supplied by LVGL to
            // flush_callback and stored in LvDispDrv; it remains valid for the display lifetime.
            unsafe {
                lv_display_flush_ready(drv.0);
            }
            return;
        }
        // SAFETY: `waiti 0` is a valid Xtensa instruction; executing it inside an
        // interrupt executor is safe — it just suspends the core until the next interrupt.
        #[cfg(target_os = "none")]
        unsafe { core::arch::asm!("waiti 0") };
    }
}

#[cfg(feature = "esp-hal")]
#[esp_hal::ram]
unsafe extern "C" fn flush_callback(
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
        disp_drv: LvDispDrv(disp),
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
    if let Err(_e) = DRAW_OPERATION.try_send(op) {
        error!("DRAW_OPERATION channel full");
    }
}
