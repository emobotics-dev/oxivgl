// SPDX-License-Identifier: MIT OR Apache-2.0
//! Runtime memory pools for LVGL's heap.
//!
//! LVGL's built-in allocator manages a TLSF heap whose primary pool is the
//! `LV_MEM_SIZE` static array. [`reserve_pool`] hands it a *second* region whose
//! address is not known until run time.
//!
//! That is the whole point: `LV_MEM_ADR` is a compile-time constant, but on
//! ESP32-S3 the PSRAM window maps after the flash rodata mmap, so its base moves
//! whenever the binary's size changes. No constant can name it. A region
//! obtained at run time — from a BSP that maps PSRAM and hands back the raw
//! span — can be registered here instead.
//!
//! ```no_run
//! # use core::mem::MaybeUninit;
//! # fn psram() -> &'static mut [MaybeUninit<u8>] { unimplemented!() }
//! // Before starting the LVGL task:
//! oxivgl::mem::reserve_pool(psram()).expect("LVGL pool");
//! ```
//!
//! The pool is registered with LVGL during driver initialisation, so
//! [`reserve_pool`] must be called *before* the render loop starts — see
//! [`MemError::TooLate`].
//!
//! # Availability
//!
//! This module exists only when the application's `lv_conf.h` selects
//! `LV_USE_STDLIB_MALLOC = LV_STDLIB_BUILTIN`. Under `LV_STDLIB_CLIB` LVGL
//! delegates to libc `malloc` and implements `lv_mem_add_pool` as a no-op that
//! returns NULL, so a pool would be accepted and silently ignored. The build
//! script gates the module rather than let that happen.
//!
//! # Constraints on the memory you supply
//!
//! These are not enforceable here — the region is handed to a C allocator that
//! subdivides it opaquely — so they are the caller's responsibility:
//!
//! - **No DMA buffers may end up in it.** The original ESP32 cannot DMA from
//!   PSRAM at all, and the ESP32-S3 only slowly. LVGL's draw buffers must stay
//!   in internal, DMA-capable RAM.
//! - **No atomics may live in it.** Atomic read-modify-write instructions
//!   misbehave against PSRAM-backed addresses on ESP32 and ESP32-S3. This holds
//!   for LVGL while `LV_USE_OS` is `LV_OS_NONE`, which allocates no atomics;
//!   enabling an OS integration would need this revisited.
//! - **It must outlive LVGL**, hence the `'static` bound. Pools are never
//!   removed: `lv_mem_remove_pool` while allocations are still live in the
//!   region would corrupt the heap, so this module does not expose it.

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use thiserror_no_std::Error;

use oxivgl_sys::{
    lv_draw_buf_get_font_handlers, lv_draw_buf_get_handlers, lv_draw_buf_get_image_handlers,
    lv_mem_add_pool, LV_DRAW_BUF_ALIGN, LV_MEM_POOL_EXPAND_SIZE, LV_MEM_SIZE,
};

/// TLSF aligns every allocation and pool base to this many bytes
/// (`ALIGN_SIZE` in `lv_tlsf.c`: 4 on 32-bit, 8 on 64-bit).
const ALIGN_SIZE: usize = core::mem::size_of::<usize>();

/// Bytes TLSF spends on a pool's own bookkeeping — a free block header plus the
/// sentinel (`lv_tlsf_pool_overhead()` = `2 * block_header_overhead`).
const POOL_OVERHEAD: usize = 2 * core::mem::size_of::<usize>();

/// Smallest usable free block (`block_size_min` in `lv_tlsf.c`:
/// `sizeof(block_header_t) - sizeof(block_header_t *)`).
const BLOCK_SIZE_MIN: usize = 3 * core::mem::size_of::<usize>();

/// Largest block TLSF can index, i.e. the largest pool it will accept.
///
/// LVGL patches stock TLSF to derive this from the application's `lv_conf.h`
/// (`lv_tlsf.c`: `TLSF_MAX_POOL_SIZE = LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE`,
/// then `FL_INDEX_MAX = TLSF_LOG2_CEIL(...)`), rather than using stock TLSF's
/// fixed 1 GiB ceiling. `TLSF_LOG2_CEIL` rounds up to a power of two.
///
/// So a pool larger than `LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE` (rounded up)
/// is rejected — which is why registering PSRAM requires sizing
/// `LV_MEM_POOL_EXPAND_SIZE`, not merely shrinking `LV_MEM_SIZE`.
const BLOCK_SIZE_MAX: usize =
    ((LV_MEM_SIZE as usize) + (LV_MEM_POOL_EXPAND_SIZE as usize)).next_power_of_two();

/// Reason a runtime memory pool could not be registered with LVGL.
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemError {
    /// A pool was already reserved. Only one runtime pool is supported.
    #[error("a runtime pool has already been reserved")]
    AlreadyReserved,
    /// LVGL has already been initialised, so the pool would be registered after
    /// allocations had begun. Reserve before starting the render loop.
    #[error("LVGL is already initialised — reserve the pool before starting the render loop")]
    TooLate,
    /// The region's base address is not `ALIGN_SIZE`-aligned; TLSF rejects it.
    #[error("pool base is not aligned to {0} bytes")]
    Misaligned(usize),
    /// The region is too small to hold TLSF's pool bookkeeping plus one block.
    #[error("pool is too small: need more than {0} bytes")]
    TooSmall(usize),
    /// The region exceeds the largest block TLSF was compiled to index. Raise
    /// `LV_MEM_POOL_EXPAND_SIZE` in `lv_conf.h` to at least the pool size.
    #[error("pool exceeds TLSF's maximum of {0} bytes — raise LV_MEM_POOL_EXPAND_SIZE")]
    TooLarge(usize),
    /// LVGL refused the pool despite it passing validation. Indicates the
    /// allocator backend is not the built-in one, or a TLSF invariant this
    /// wrapper does not model.
    #[error("LVGL rejected the pool")]
    Rejected,
}

#[cfg(feature = "defmt")]
impl defmt::Format for MemError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            MemError::AlreadyReserved => defmt::write!(f, "a runtime pool has already been reserved"),
            MemError::TooLate => defmt::write!(f, "LVGL already initialised — reserve earlier"),
            MemError::Misaligned(a) => defmt::write!(f, "pool base is not aligned to {} bytes", a),
            MemError::TooSmall(n) => defmt::write!(f, "pool too small: need more than {} bytes", n),
            MemError::TooLarge(n) => {
                defmt::write!(f, "pool exceeds TLSF maximum of {} bytes — raise LV_MEM_POOL_EXPAND_SIZE", n)
            }
            MemError::Rejected => defmt::write!(f, "LVGL rejected the pool"),
        }
    }
}

/// Base address of the reserved-but-not-yet-registered pool, or 0 for none.
static PENDING_PTR: AtomicUsize = AtomicUsize::new(0);
/// Length in bytes of the pool described by `PENDING_PTR`.
static PENDING_LEN: AtomicUsize = AtomicUsize::new(0);
/// Set once the pending pool has been handed to LVGL.
static APPLIED: AtomicBool = AtomicBool::new(false);

/// Reserve a memory region for LVGL's heap, registered when the driver
/// initialises.
///
/// The region must be `'static` because LVGL keeps it forever: pools are never
/// removed (see the module docs). Taking a slice rather than a raw pointer keeps
/// the call site free of `unsafe` and encodes that requirement in the type.
///
/// Call this before the render loop starts. It does not touch LVGL — it only
/// records the region, which driver initialisation then registers.
///
/// # Errors
///
/// [`MemError::Misaligned`], [`MemError::TooSmall`] or [`MemError::TooLarge`] if
/// the region cannot be a TLSF pool; [`MemError::AlreadyReserved`] on a second
/// call; [`MemError::TooLate`] if LVGL has already been initialised.
pub fn reserve_pool(mem: &'static mut [MaybeUninit<u8>]) -> Result<(), MemError> {
    let len = mem.len();
    let ptr = mem.as_mut_ptr() as usize;
    // SAFETY: `mem` is a `'static` exclusive borrow, so the region is valid for
    // LVGL's lifetime and no other Rust code can alias it once it is handed over.
    unsafe { reserve_pool_raw(ptr as *mut u8, len) }
}

/// Reserve a memory region by raw parts, for callers whose allocator hands back
/// a pointer and length rather than a slice.
///
/// Prefer [`reserve_pool`], which enforces the requirements below in the type
/// system.
///
/// # Safety
///
/// The caller must guarantee that:
/// - `ptr` is valid for reads and writes of `len` bytes,
/// - the region lives for the remainder of the program (LVGL never releases it),
/// - nothing else — no allocator, no DMA engine, no other `reserve_pool_raw`
///   call — uses that memory.
///
/// # Errors
///
/// As [`reserve_pool`].
pub unsafe fn reserve_pool_raw(ptr: *mut u8, len: usize) -> Result<(), MemError> {
    if APPLIED.load(Ordering::Acquire) {
        return Err(MemError::TooLate);
    }
    if PENDING_PTR.load(Ordering::Relaxed) != 0 {
        return Err(MemError::AlreadyReserved);
    }

    let addr = ptr as usize;
    if !addr.is_multiple_of(ALIGN_SIZE) {
        return Err(MemError::Misaligned(ALIGN_SIZE));
    }

    // Mirror `lv_tlsf_add_pool`'s own accept/reject test so the caller gets a
    // typed error instead of a log line and a NULL return.
    let usable = len
        .checked_sub(POOL_OVERHEAD)
        .map(|n| n & !(ALIGN_SIZE - 1))
        .unwrap_or(0);
    if usable < BLOCK_SIZE_MIN {
        return Err(MemError::TooSmall(POOL_OVERHEAD + BLOCK_SIZE_MIN));
    }
    if usable > BLOCK_SIZE_MAX {
        return Err(MemError::TooLarge(BLOCK_SIZE_MAX));
    }

    PENDING_LEN.store(len, Ordering::Relaxed);
    PENDING_PTR.store(addr, Ordering::Release);
    Ok(())
}

/// Register the reserved pool with LVGL. Called by driver initialisation once
/// `lv_init()` has run and before any widget is created.
///
/// # Panics
///
/// If LVGL refuses the pool. Validation in [`reserve_pool`] has already ruled
/// out every reason it should, so a refusal here means the heap is not in the
/// state this module assumes — and continuing would silently leave the pool
/// unused, which is the failure this module exists to prevent.
pub(crate) fn apply_pending() {
    let addr = PENDING_PTR.load(Ordering::Acquire);
    if addr == 0 {
        APPLIED.store(true, Ordering::Release);
        return;
    }
    let len = PENDING_LEN.load(Ordering::Relaxed);

    // SAFETY: `lv_init()` has run (this is called from driver init), and the
    // region was validated and guaranteed `'static` by `reserve_pool*`.
    let pool = unsafe { lv_mem_add_pool(addr as *mut core::ffi::c_void, len) };
    assert!(
        !pool.is_null(),
        "LVGL rejected the reserved memory pool — the pool would have been \
         silently unused (is LV_USE_STDLIB_MALLOC = LV_STDLIB_BUILTIN?)"
    );

    // The pool is now part of the same TLSF heap as everything else, so LVGL's
    // draw buffers could be served from it. They must not be — keep them out.
    install_draw_buf_guard();

    APPLIED.store(true, Ordering::Release);
}

// ── Draw-buffer guard ────────────────────────────────────────────────────────

/// Bytes reserved ahead of each draw buffer to record its allocation size.
///
/// LVGL's free callback receives only a pointer, but Rust's deallocator needs
/// the original `Layout`. LVGL frees `draw_buf->unaligned_data` — precisely the
/// pointer [`draw_buf_malloc`] returned, never the aligned one — so stashing the
/// size immediately before it round-trips reliably.
const SIZE_HEADER: usize = core::mem::size_of::<usize>();

/// Route LVGL's draw-buffer allocations through the Rust global allocator
/// instead of its own heap.
///
/// TLSF pools are fungible: once a runtime pool is registered, `lv_malloc` may
/// satisfy *any* allocation from it, and LVGL's default draw-buffer allocator
/// (`buf_malloc` in `lv_draw_buf.c`) is a plain `lv_malloc`. Layer, canvas,
/// snapshot and image-decoder buffers would therefore be eligible to land in the
/// runtime pool — which is wrong when that pool is PSRAM, because the original
/// ESP32 cannot DMA from PSRAM at all and the ESP32-S3 only slowly.
///
/// The global allocator is the right target because it is internal-only by
/// construction: a BSP that hands out a private PSRAM region consumes the PSRAM
/// peripheral by value, so the same PSRAM cannot also back the global heap.
///
/// Only `buf_malloc_cb` / `buf_free_cb` are replaced. The remaining handlers
/// (`buf_copy`, `align_pointer`, `width_to_stride`) keep LVGL's defaults, which
/// are file-static in `lv_draw_buf.c` and cannot be named from Rust — hence
/// patching the fields rather than calling `lv_draw_buf_handlers_init`.
fn install_draw_buf_guard() {
    // SAFETY: called from driver init after lv_init(), which has already
    // populated the three handler sets with LVGL's defaults. Single-task
    // constraint means no other code is touching them concurrently.
    unsafe {
        for handlers in [
            lv_draw_buf_get_handlers(),
            lv_draw_buf_get_font_handlers(),
            lv_draw_buf_get_image_handlers(),
        ] {
            if handlers.is_null() {
                continue;
            }
            (*handlers).buf_malloc_cb = Some(draw_buf_malloc);
            (*handlers).buf_free_cb = Some(draw_buf_free);
        }
    }
}

/// Allocate a draw buffer from the Rust global allocator.
///
/// Mirrors LVGL's own `buf_malloc`, which over-allocates by
/// `LV_DRAW_BUF_ALIGN - 1` so the caller can align the pointer upwards
/// afterwards, and adds a header recording the size for [`draw_buf_free`].
///
/// Returns NULL on failure or on a size that would overflow, which is what LVGL
/// expects from an allocation failure.
unsafe extern "C" fn draw_buf_malloc(
    size: usize,
    _color_format: oxivgl_sys::lv_color_format_t,
) -> *mut core::ffi::c_void {
    let Some(total) = size
        .checked_add(LV_DRAW_BUF_ALIGN as usize - 1)
        .and_then(|n| n.checked_add(SIZE_HEADER))
    else {
        return core::ptr::null_mut();
    };
    let Ok(layout) = alloc::alloc::Layout::from_size_align(total, core::mem::align_of::<usize>())
    else {
        return core::ptr::null_mut();
    };

    // SAFETY: `layout` has non-zero size (SIZE_HEADER > 0).
    let base = unsafe { alloc::alloc::alloc(layout) };
    if base.is_null() {
        return core::ptr::null_mut();
    }
    // SAFETY: `base` is a fresh allocation of at least SIZE_HEADER bytes, and is
    // `usize`-aligned by the layout above.
    unsafe {
        base.cast::<usize>().write(total);
        base.add(SIZE_HEADER).cast()
    }
}

/// Free a draw buffer allocated by [`draw_buf_malloc`].
unsafe extern "C" fn draw_buf_free(buf: *mut core::ffi::c_void) {
    if buf.is_null() {
        return;
    }
    // SAFETY: LVGL frees `unaligned_data`, i.e. exactly the pointer
    // `draw_buf_malloc` returned, so the header sits SIZE_HEADER bytes below it
    // and holds the total size passed to `alloc`.
    unsafe {
        let base = buf.cast::<u8>().sub(SIZE_HEADER);
        let total = base.cast::<usize>().read();
        let layout = alloc::alloc::Layout::from_size_align_unchecked(
            total,
            core::mem::align_of::<usize>(),
        );
        alloc::alloc::dealloc(base, layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_size_max_is_a_power_of_two_covering_the_configured_heap() {
        assert!(BLOCK_SIZE_MAX.is_power_of_two());
        assert!(BLOCK_SIZE_MAX >= (LV_MEM_SIZE as usize) + (LV_MEM_POOL_EXPAND_SIZE as usize));
    }

    #[test]
    fn misaligned_base_is_rejected() {
        // 1 is never a multiple of ALIGN_SIZE (4 or 8).
        let err = unsafe { reserve_pool_raw(1 as *mut u8, 64 * 1024) }.unwrap_err();
        assert_eq!(err, MemError::Misaligned(ALIGN_SIZE));
    }

    #[test]
    fn too_small_is_rejected_without_underflowing() {
        // Smaller than POOL_OVERHEAD — must report TooSmall, not wrap around
        // into a huge `usize` and pass the upper bound check.
        let err = unsafe { reserve_pool_raw(ALIGN_SIZE as *mut u8, 1) }.unwrap_err();
        assert_eq!(err, MemError::TooSmall(POOL_OVERHEAD + BLOCK_SIZE_MIN));
    }

    #[test]
    fn oversized_pool_is_rejected_naming_the_limit() {
        let err =
            unsafe { reserve_pool_raw(ALIGN_SIZE as *mut u8, BLOCK_SIZE_MAX * 4) }.unwrap_err();
        assert_eq!(err, MemError::TooLarge(BLOCK_SIZE_MAX));
    }

    /// The draw-buffer guard's size header must survive the malloc/free
    /// round-trip: LVGL hands back the same pointer we returned, and we recover
    /// the layout from the bytes just below it.
    #[test]
    fn draw_buf_alloc_round_trips_through_the_size_header() {
        const CF: oxivgl_sys::lv_color_format_t =
            oxivgl_sys::lv_color_format_t_LV_COLOR_FORMAT_ARGB8888;
        for size in [1usize, 17, 4096, 100_000] {
            // SAFETY: mirrors LVGL's call sequence — allocate, use, free the
            // exact pointer returned.
            unsafe {
                let p = draw_buf_malloc(size, CF);
                assert!(!p.is_null(), "allocation of {size} failed");

                // The caller must be able to align upwards and still have `size`
                // usable bytes, which is what the LV_DRAW_BUF_ALIGN slack is for.
                let aligned = (p as usize).next_multiple_of(LV_DRAW_BUF_ALIGN as usize);
                assert!(aligned - (p as usize) < LV_DRAW_BUF_ALIGN as usize);

                // Touch the whole usable span so a short allocation would be
                // caught by the allocator or a sanitiser.
                core::ptr::write_bytes(aligned as *mut u8, 0xA5, size);

                draw_buf_free(p);
            }
        }
    }

    #[test]
    fn draw_buf_alloc_returns_null_rather_than_overflowing() {
        const CF: oxivgl_sys::lv_color_format_t =
            oxivgl_sys::lv_color_format_t_LV_COLOR_FORMAT_ARGB8888;
        // SAFETY: the size overflows before any allocation is attempted.
        let p = unsafe { draw_buf_malloc(usize::MAX, CF) };
        assert!(p.is_null());
    }

    #[test]
    fn draw_buf_free_tolerates_null() {
        // LVGL can call free on a failed allocation; must not fault.
        unsafe { draw_buf_free(core::ptr::null_mut()) };
    }
}
