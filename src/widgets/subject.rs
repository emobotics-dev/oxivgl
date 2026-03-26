// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL observer [`Subject`] — an observable value that widgets can bind to.

use alloc::boxed::Box;
use core::pin::Pin;

use lvgl_rust_sys::*;

/// An observable value that LVGL widgets can bind to via the observer API.
///
/// `Subject` owns a heap-allocated, pinned `lv_subject_t`, guaranteeing a
/// stable memory address for the lifetime of this object.  Widget bindings
/// established with e.g. [`Slider::bind_value`](super::Slider::bind_value)
/// store a raw pointer to this allocation — the `Pin<Box<_>>` prevents moves
/// that would invalidate the pointer.
///
/// # Drop order
///
/// Subjects should outlive any widgets bound to them.  Both drop orders are
/// safe (LVGL handles cleanup in either case), but dropping a subject first
/// calls `lv_subject_deinit`, which removes all observer linkage before the
/// widget is deleted.
///
/// # Thread safety
///
/// `Subject` is `!Send + !Sync` — LVGL must be driven from a single task.
pub struct Subject {
    inner: Pin<Box<lv_subject_t>>,
}

impl core::fmt::Debug for Subject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Subject").finish_non_exhaustive()
    }
}

impl Subject {
    /// Create a new integer subject with the given initial value.
    pub fn new_int(value: i32) -> Self {
        // SAFETY: lv_subject_t is a POD C struct; zero-init is a valid
        // uninitialised state before lv_subject_init_int is called.
        let mut inner: Pin<Box<lv_subject_t>> = Box::pin(unsafe { core::mem::zeroed() });
        // SAFETY: We hold the only reference to `inner`; it is pinned so the
        // address will not change.  lv_subject_init_int writes into the struct
        // via the pointer and does not retain it beyond the call.
        unsafe {
            let ptr: *mut lv_subject_t =
                Pin::as_mut(&mut inner).get_unchecked_mut() as *mut lv_subject_t;
            lv_subject_init_int(ptr, value);
        }
        Self { inner }
    }

    /// Set the subject value and notify all bound observers.
    pub fn set_int(&self, value: i32) -> &Self {
        // SAFETY: as_ptr() returns the pinned, non-null allocation.
        unsafe { lv_subject_set_int(self.as_ptr(), value) };
        self
    }

    /// Get the current integer value.
    pub fn get_int(&self) -> i32 {
        // SAFETY: as_ptr() returns the pinned, non-null allocation.
        unsafe { lv_subject_get_int(self.as_ptr()) }
    }

    /// Get the previous integer value (value before the last `set_int` call).
    pub fn get_previous_int(&self) -> i32 {
        // SAFETY: as_ptr() returns the pinned, non-null allocation.
        unsafe { lv_subject_get_previous_int(self.as_ptr()) }
    }

    /// Return a raw mutable pointer to the underlying `lv_subject_t`.
    ///
    /// The pointer is valid for the lifetime of this `Subject`.  Callers must
    /// not store the pointer beyond the subject's lifetime.
    pub(crate) fn as_ptr(&self) -> *mut lv_subject_t {
        // Cast away the shared-reference immutability — LVGL's C API takes
        // `*mut lv_subject_t` even for read-only operations.
        // SAFETY: The inner Box is pinned; the address is stable.  We only
        // hand this out to LVGL FFI calls executed on the single LVGL task.
        &*self.inner as *const lv_subject_t as *mut lv_subject_t
    }
}

impl Drop for Subject {
    fn drop(&mut self) {
        // SAFETY: inner is the pinned allocation initialised by new_int.
        // lv_subject_deinit removes all observers and frees LVGL-internal
        // linked-list nodes; it is safe to call even if no observers exist.
        unsafe { lv_subject_deinit(self.as_ptr()) };
    }
}
