// SPDX-License-Identifier: MIT OR Apache-2.0
//! Verifies that a reserved runtime pool actually reaches LVGL's heap, and that
//! draw buffers are kept out of it.
//!
//! This is a separate test binary on purpose: `LvglDriver::init()` may run only
//! once per process, and the pool has to be reserved *before* it. A single test
//! function therefore covers the whole sequence.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo test --test mem_pool
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

#![cfg(lvgl_builtin_malloc)]

use core::mem::MaybeUninit;

use oxivgl::mem::{self, MemError};

/// Comfortably larger than any incidental allocation, and well under TLSF's
/// `block_size_max` (`LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE`, rounded up).
const POOL_SIZE: usize = 128 * 1024;

#[test]
fn reserved_pool_reaches_lvgl_and_excludes_draw_buffers() {
    // A leaked heap allocation is genuinely `'static`, which is what the pool
    // contract requires — LVGL never releases a pool.
    let pool: &'static mut [MaybeUninit<u8>] =
        Box::leak(vec![MaybeUninit::<u8>::uninit(); POOL_SIZE].into_boxed_slice());
    let pool_start = pool.as_ptr() as usize;
    let pool_end = pool_start + POOL_SIZE;

    mem::reserve_pool(pool).expect("pool should be accepted");

    // A second reservation must be refused rather than silently replacing the
    // first — LVGL would otherwise keep a region nothing tracks.
    let mut spare = vec![MaybeUninit::<u8>::uninit(); 4096];
    let err = unsafe { mem::reserve_pool_raw(spare.as_mut_ptr().cast(), spare.len()) }
        .expect_err("second reservation must be refused");
    assert_eq!(err, MemError::AlreadyReserved);

    // Registration happens here, inside driver init, right after lv_init().
    let _driver = oxivgl::driver::LvglDriver::init(320, 240);

    // ── The pool reached LVGL's heap ──────────────────────────────────────
    let mut mon = unsafe { core::mem::zeroed::<oxivgl_sys::lv_mem_monitor_t>() };
    unsafe { oxivgl_sys::lv_mem_monitor(&mut mon) };

    // `lv_mem_monitor` walks every pool in `state.pool_ll`, which holds the
    // initial `LV_MEM_SIZE` array plus anything added via `lv_mem_add_pool`.
    let baseline = oxivgl_sys::LV_MEM_SIZE as usize;
    assert!(
        mon.total_size > baseline,
        "heap did not grow: total_size {} <= LV_MEM_SIZE {baseline} — the pool \
         was not registered",
        mon.total_size
    );
    // Not an exact +POOL_SIZE: `lv_mem_walker` accumulates each *block's* payload
    // size, which excludes that block's header, so `total_size` under-reports the
    // raw pool bytes by one header per live block. The shortfall therefore tracks
    // how many blocks LVGL's init happens to leave behind. 95% is far above that
    // while still failing loudly if the pool were not registered.
    let expected_min = baseline + POOL_SIZE * 95 / 100;
    assert!(
        mon.total_size >= expected_min,
        "heap grew by less than the pool: total_size {} < {expected_min} \
         (baseline {baseline} + 95% of {POOL_SIZE})",
        mon.total_size,
    );

    // ── Draw buffers stay out of the pool ─────────────────────────────────
    // The guard routes them through the Rust global allocator, so their memory
    // must fall outside the region we handed to LVGL. Without it, TLSF would be
    // free to satisfy these from the pool — which on ESP32 means memory the DMA
    // engine cannot reach.
    let cf = oxivgl_sys::lv_color_format_t_LV_COLOR_FORMAT_ARGB8888;
    for (w, h) in [(64, 64), (256, 128)] {
        // SAFETY: LVGL is initialised; the buffer is destroyed below.
        let buf = unsafe { oxivgl_sys::lv_draw_buf_create(w, h, cf, 0) };
        assert!(!buf.is_null(), "draw buf {w}x{h} allocation failed");

        let data = unsafe { (*buf).data } as usize;
        assert!(
            data < pool_start || data >= pool_end,
            "draw buf {w}x{h} landed inside the runtime pool at {data:#x} \
             (pool {pool_start:#x}..{pool_end:#x}) — it would be un-DMA-able on ESP32"
        );

        // SAFETY: created directly above and not referenced elsewhere.
        unsafe { oxivgl_sys::lv_draw_buf_destroy(buf) };
    }

    // ── Reserving after init is refused ───────────────────────────────────
    let err = unsafe { mem::reserve_pool_raw(spare.as_mut_ptr().cast(), spare.len()) }
        .expect_err("reservation after init must be refused");
    assert_eq!(err, MemError::TooLate);
}
