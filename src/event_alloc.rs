// SPDX-License-Identifier: MIT OR Apache-2.0
//! Keep LVGL's event lists out of LVGL's heap.
//!
//! An event descriptor (`lv_malloc(sizeof(lv_event_dsc_t))`, 12 bytes) and the
//! pointer array that holds it live for exactly as long as the callback is
//! registered, and that is arbitrary: `lv_label_mark_need_refr_text` registers a
//! callback on the **display** and removes it on the next layout pass, so any
//! drawing churns both allocations continuously.
//!
//! That breaks a consumer who lends LVGL a pool for the duration of some
//! operation — the pattern `oxivgl::mem::reserve_pool` exists to serve, and the
//! one `lv_demo_benchmark` needs on boards whose resident pool cannot hold it.
//! Such a pool can only be handed back when it is empty (`lv_tlsf_remove_pool`
//! asserts one single free block), and a block that lands there cannot be
//! trimmed away afterwards: `block_can_split` needs `16 + size` while
//! `block_size_min` is 12, so even a two-entry list occupies a 12-byte block
//! permanently. Measured on ESP32: the surviving block *was*
//! `lv_display_get_default()->event_list.array.data`, confirmed by address
//! identity.
//!
//! Reserving capacity instead was tried and rejected — it protects only
//! displays, self-destructs when a list empties (`lv_array_deinit` frees the
//! reservation), leaves the descriptors in LVGL's heap regardless, and signals
//! an exceeded bound only through `LV_LOG_WARN`, which the log level can compile
//! out. Routing has no cap to exceed, covers every object, and costs nothing for
//! applications that never lend a pool.
//!
//! Unlike [`crate::render_scratch`] this routing is **unconditional**. A guarded
//! route would leave the default build with the original defect and nothing to
//! observe it, which is the objection that sank the reservation. The memory is
//! small — an ESP32 running the benchmark peaks at 63 concurrent display
//! callbacks, so well under 1.5 KiB of descriptors plus array.
//!
//! `oxivgl-sys` patches `lv_event.c` to route both ends. The array cannot be
//! routed by swapping `lv_array_*` calls, because those allocate and free
//! internally through `lv_realloc`/`lv_free` — `lv_array_deinit` would hand a
//! block from here to `lv_free`. The patch therefore carries its own array
//! helpers; `lv_array.c` stays on LVGL's heap, since the scale widget, the
//! vector path ops and `lv_circle_buf` share it.

use crate::render_scratch::{alloc_internal, free_internal, size_internal};
use core::ffi::c_void;

/// Allocate an event descriptor or event-list array.
///
/// # Safety
/// Exported for the patched C call sites; not intended to be called from Rust.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxivgl_event_malloc(size: usize) -> *mut c_void {
    if size == 0 {
        return core::ptr::null_mut();
    }
    alloc_internal(size, false)
}

/// Grow or shrink an event-list array, preserving its contents.
///
/// A NULL `ptr` allocates, mirroring `realloc`. Returns NULL on failure with the
/// original block untouched, so the caller can keep using it — which
/// `event_array_resize` relies on to fail without losing the list.
///
/// # Safety
/// `ptr` must be NULL or a live pointer from [`oxivgl_event_malloc`] /
/// [`oxivgl_event_realloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxivgl_event_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
    if ptr.is_null() {
        return unsafe { oxivgl_event_malloc(size) };
    }
    if size == 0 {
        unsafe { free_internal(ptr) };
        return core::ptr::null_mut();
    }

    // SAFETY: `ptr` is live and came from `alloc_internal`, so it carries a size
    // header.
    let old = unsafe { size_internal(ptr) };
    let new = alloc_internal(size, false);
    if new.is_null() {
        return core::ptr::null_mut();
    }

    // SAFETY: both blocks are live and at least `min(old, size)` bytes long, and
    // they cannot overlap — `new` is a fresh allocation.
    unsafe {
        core::ptr::copy_nonoverlapping(ptr.cast::<u8>(), new.cast::<u8>(), old.min(size));
        free_internal(ptr);
    }
    new
}

/// Free an event descriptor or event-list array.
///
/// # Safety
/// `ptr` must be NULL or a live pointer from [`oxivgl_event_malloc`] /
/// [`oxivgl_event_realloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn oxivgl_event_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: non-null and from this module's allocator, per the contract above.
    unsafe { free_internal(ptr) };
}
