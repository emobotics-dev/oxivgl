// SPDX-License-Identifier: MIT OR Apache-2.0
//! Keep LVGL's transient per-frame render scratch in internal DRAM.
//!
//! The software renderer allocates a scratch buffer *per draw op, every frame*,
//! and frees it within the same draw call — draw-task descriptors
//! (`lv_draw_add_task`), scanline masks (arc, line, fill, border, triangle,
//! rect-mask), the box-shadow blur/mask buffers, and image mask/transform
//! buffers. All go through a plain `lv_malloc` / `lv_free` with **no callback
//! hook** — unlike draw *buffers*, which expose `buf_malloc_cb` and are
//! redirected by [`crate::mem`]'s buffer guard.
//!
//! Once a runtime pool is registered (see [`crate::mem::reserve_pool`]) LVGL's
//! TLSF heap is one fungible arena, so this scratch becomes eligible to land in
//! the pool. When that pool is PSRAM it is expensive: serving per-frame
//! alloc/free churn against PSRAM-resident TLSF metadata halves render
//! throughput on the original ESP32 (issue #124). It must stay in internal DRAM,
//! exactly like the pixel draw buffers — so render cost tracks dirty-region
//! complexity, not total UI size.
//!
//! The only clean seam is the allocation site, so `oxivgl-sys` patches the SW
//! draw sources at build time to route these calls through the three symbols
//! defined here. While the guard is [`active`](activate) the scratch comes from
//! the Rust global allocator — internal, DMA-capable DRAM by construction (a BSP
//! that hands out a private PSRAM region consumes the peripheral, so the same
//! PSRAM cannot also back the global heap). Otherwise — under `LV_STDLIB_CLIB`,
//! or `LV_STDLIB_BUILTIN` with no pool registered — the calls delegate straight
//! to LVGL's allocator, so behaviour is byte-for-byte unchanged.
//!
//! Only *transient* per-frame scratch is routed. LVGL's gradient and
//! circle-mask *caches* — allocated once and read across frames, and freed
//! cross-function — are intentionally left on LVGL's allocator: they are not the
//! per-frame churn this targets, and their alloc/free pairing does not fit the
//! single-regime invariant below.

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};

/// Bytes reserved ahead of each block to record its allocation size. The free
/// callback receives only a pointer, but Rust's deallocator needs the original
/// size, so it is stashed immediately before the returned pointer — the same
/// technique the draw-buffer guard uses in [`crate::mem`].
const SIZE_HEADER: usize = core::mem::size_of::<usize>();

/// Alignment of a returned block. LVGL uses the pointer as-is (it does not
/// re-align this scratch afterward), so this must satisfy the largest scratch
/// type's alignment directly. `usize` alignment (4 on the 32-bit targets, 8 on
/// host) covers it: no routed scratch carries a member wider than a pointer on
/// 32-bit. Matches `draw_buf_malloc`.
const ALIGN: usize = core::mem::align_of::<usize>();

/// Set once, before the first frame, when a runtime pool is registered.
///
/// Until then — and always under CLIB or when no pool is registered — the
/// callbacks delegate to LVGL's allocator. Because the flip happens before any
/// rendering (in [`crate::mem::apply_pending`], driver-init time), every scratch
/// block is allocated and freed under one regime, so the free callback can trust
/// this flag without tagging individual allocations.
static GUARD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Activate render-scratch rerouting.
///
/// Called from driver init when a pool has been registered — the same trigger,
/// and the same pre-render moment, as the draw-buffer guard.
#[cfg(lvgl_builtin_malloc)]
pub(crate) fn activate() {
    GUARD_ACTIVE.store(true, Ordering::Release);
}

/// Allocate `size` bytes from the Rust global allocator with a size header for
/// [`free_internal`]. `zeroed` mirrors `lv_malloc` vs `lv_malloc_zeroed`.
/// Returns NULL on overflow or allocation failure, as LVGL expects.
fn alloc_internal(size: usize, zeroed: bool) -> *mut c_void {
    let Some(total) = size.checked_add(SIZE_HEADER) else {
        return core::ptr::null_mut();
    };
    let Ok(layout) = alloc::alloc::Layout::from_size_align(total, ALIGN) else {
        return core::ptr::null_mut();
    };
    // SAFETY: `total >= SIZE_HEADER > 0`, so the layout has non-zero size.
    let base = unsafe {
        if zeroed {
            alloc::alloc::alloc_zeroed(layout)
        } else {
            alloc::alloc::alloc(layout)
        }
    };
    if base.is_null() {
        return core::ptr::null_mut();
    }
    // SAFETY: `base` is a fresh, `usize`-aligned allocation of at least
    // `SIZE_HEADER` bytes. Writing the header never touches the user region, so
    // the `zeroed` guarantee above is preserved.
    unsafe {
        base.cast::<usize>().write(total);
        base.add(SIZE_HEADER).cast()
    }
}

/// Free a non-null pointer returned by [`alloc_internal`].
///
/// # Safety
/// `ptr` must be a pointer returned by [`alloc_internal`] and not yet freed.
unsafe fn free_internal(ptr: *mut c_void) {
    // SAFETY: the header sits `SIZE_HEADER` bytes below the returned pointer and
    // records the total size passed to the allocator.
    unsafe {
        let base = ptr.cast::<u8>().sub(SIZE_HEADER);
        let total = base.cast::<usize>().read();
        let layout = alloc::alloc::Layout::from_size_align_unchecked(total, ALIGN);
        alloc::alloc::dealloc(base, layout);
    }
}

/// Allocate uninitialised render scratch. Replaces `lv_malloc` at the routed SW
/// draw sites via the build-time patch in `oxivgl-sys`.
///
/// # Safety
/// Exported for the patched C call sites; not intended to be called from Rust.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxivgl_render_scratch_malloc(size: usize) -> *mut c_void {
    if GUARD_ACTIVE.load(Ordering::Acquire) {
        alloc_internal(size, false)
    } else {
        // SAFETY: valid at any point after `lv_init`, on the single LVGL task.
        unsafe { oxivgl_sys::lv_malloc(size) }
    }
}

/// Allocate zeroed render scratch. Replaces `lv_malloc_zeroed` (the draw-task
/// descriptor site) via the build-time patch in `oxivgl-sys`.
///
/// # Safety
/// Exported for the patched C call sites; not intended to be called from Rust.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxivgl_render_scratch_zalloc(size: usize) -> *mut c_void {
    if GUARD_ACTIVE.load(Ordering::Acquire) {
        alloc_internal(size, true)
    } else {
        // SAFETY: valid at any point after `lv_init`, on the single LVGL task.
        unsafe { oxivgl_sys::lv_malloc_zeroed(size) }
    }
}

/// Free render scratch. Replaces `lv_free` at the routed SW draw sites.
///
/// The guard flag does not change between a block's allocation and its free (it
/// flips once, before rendering), so the same branch is taken as at allocation.
///
/// # Safety
/// `ptr` must be NULL or a pointer previously returned by
/// [`oxivgl_render_scratch_malloc`] / [`oxivgl_render_scratch_zalloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxivgl_render_scratch_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    if GUARD_ACTIVE.load(Ordering::Acquire) {
        // SAFETY: allocated by the active branch of a routed allocator.
        unsafe { free_internal(ptr) }
    } else {
        // SAFETY: allocated by LVGL, on the single LVGL task.
        unsafe { oxivgl_sys::lv_free(ptr) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A zeroed block round-trips through the size header: the region is zeroed,
    /// writable, and freed with the size the header preserved (an incorrect size
    /// would trip the allocator under Miri/ASan).
    #[test]
    fn zalloc_zeroes_and_round_trips_through_the_size_header() {
        const SIZE: usize = 96;
        let p = alloc_internal(SIZE, true).cast::<u8>();
        assert!(!p.is_null());
        // SAFETY: `SIZE` bytes were requested and zeroed.
        unsafe {
            for i in 0..SIZE {
                assert_eq!(*p.add(i), 0, "byte {i} not zeroed");
                *p.add(i) = 0xAB;
            }
            free_internal(p.cast());
        }
    }

    /// A non-zeroed block round-trips through the header too (no zeroing
    /// guarantee, but the size must survive the alloc/free cycle).
    #[test]
    fn malloc_round_trips_through_the_size_header() {
        const SIZE: usize = 64;
        let p = alloc_internal(SIZE, false).cast::<u8>();
        assert!(!p.is_null());
        // SAFETY: `SIZE` bytes were requested; write then free.
        unsafe {
            for i in 0..SIZE {
                *p.add(i) = 0xCD;
            }
            free_internal(p.cast());
        }
    }

    /// A size that would overflow the header addition returns NULL rather than
    /// wrapping — matching an LVGL allocation failure.
    #[test]
    fn alloc_returns_null_rather_than_overflowing() {
        assert!(alloc_internal(usize::MAX, false).is_null());
        assert!(alloc_internal(usize::MAX, true).is_null());
    }
}
