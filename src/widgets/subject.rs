// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL observer [`Subject`] — an observable value that widgets can bind to.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{ffi::c_void, pin::Pin};

use lvgl_rust_sys::*;

use super::obj::AsLvHandle;

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
    /// Stable pointer array for group subjects; `None` for integer subjects.
    ///
    /// `lv_subject_init_group` stores a pointer into this array, so it must
    /// stay pinned for the lifetime of the subject.
    _group_ptrs: Option<Pin<Box<[*mut lv_subject_t]>>>,
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
        Self {
            inner,
            _group_ptrs: None,
        }
    }

    /// Create a group subject that notifies observers when any member changes.
    ///
    /// The group holds a stable pointer array referencing each member subject.
    /// Member subjects must outlive the group subject.
    pub fn new_group(members: &[&Subject]) -> Self {
        let ptrs: Vec<*mut lv_subject_t> = members.iter().map(|s| s.as_ptr()).collect();
        let mut pinned_ptrs: Pin<Box<[*mut lv_subject_t]>> = Pin::new(ptrs.into_boxed_slice());
        // SAFETY: lv_subject_t is a POD C struct; zero-init is valid before init call.
        let mut inner: Pin<Box<lv_subject_t>> = Box::pin(unsafe { core::mem::zeroed() });
        // SAFETY: both `inner` and `pinned_ptrs` are pinned — addresses are stable.
        // `lv_subject_init_group` stores the array pointer internally; the pinned
        // Box keeps it valid for the lifetime of this Subject.
        unsafe {
            let group_ptr: *mut lv_subject_t =
                Pin::as_mut(&mut inner).get_unchecked_mut() as *mut lv_subject_t;
            let arr_ptr: *mut *mut lv_subject_t = Pin::as_mut(&mut pinned_ptrs)
                .get_unchecked_mut()
                .as_mut_ptr();
            lv_subject_init_group(group_ptr, arr_ptr, members.len() as u32);
        }
        Self {
            inner,
            _group_ptrs: Some(pinned_ptrs),
        }
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

    /// Add a standalone observer callback (not tied to any widget).
    ///
    /// The callback fires whenever the subject's value changes.
    /// The observer is removed when the subject is dropped.
    pub fn add_observer(&self, cb: ObserverCb, user_data: *mut c_void) -> &Self {
        // SAFETY: as_ptr() returns pinned non-null; cb is a valid fn ptr.
        unsafe { lv_subject_add_observer(self.as_ptr(), Some(cb), user_data) };
        self
    }

    /// Add an observer callback tied to a widget's lifetime.
    ///
    /// The observer is automatically removed when the widget is deleted.
    pub fn add_observer_obj(
        &self,
        cb: ObserverCb,
        obj: &impl AsLvHandle,
        user_data: *mut c_void,
    ) -> &Self {
        // SAFETY: as_ptr() pinned non-null; obj.lv_handle() non-null; cb valid fn ptr.
        unsafe { lv_subject_add_observer_obj(self.as_ptr(), Some(cb), obj.lv_handle(), user_data) };
        self
    }

    /// Manually notify all observers without changing the value.
    ///
    /// Useful to force observer callbacks to run, e.g. after external state
    /// changes that did not go through [`set_int`](Self::set_int).
    pub fn notify(&self) -> &Self {
        // SAFETY: as_ptr() returns the pinned, non-null allocation.
        unsafe { lv_subject_notify(self.as_ptr()) };
        self
    }

    /// Raw pointer escape hatch for advanced LVGL interop.
    ///
    /// Returns a raw `*mut lv_subject_t` for passing to raw LVGL APIs that
    /// are not yet wrapped (e.g. dynamic widget creation in event callbacks).
    ///
    /// The pointer is valid for the lifetime of this `Subject`.  Do not store
    /// it beyond the subject's lifetime.
    pub fn raw_ptr(&self) -> *mut lv_subject_t {
        self.as_ptr()
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

/// Observer callback function type for raw LVGL observer callbacks.
pub type ObserverCb = unsafe extern "C" fn(*mut lv_observer_t, *mut lv_subject_t);

/// Get the target widget from an observer (for use in observer callbacks).
///
/// # Safety
///
/// `observer` must be a valid pointer received in an observer callback.
pub unsafe fn observer_get_target_obj(observer: *mut lv_observer_t) -> *mut lv_obj_t {
    // SAFETY: caller guarantees observer is valid.
    unsafe { lv_observer_get_target_obj(observer) }
}

/// Get the integer value from a raw subject pointer (for use in observer callbacks).
///
/// # Safety
///
/// `subject` must be a valid pointer received in an observer callback.
pub unsafe fn subject_get_int_raw(subject: *mut lv_subject_t) -> i32 {
    // SAFETY: caller guarantees subject is valid.
    unsafe { lv_subject_get_int(subject) }
}

/// Get a member subject pointer from a group subject by index.
///
/// For use inside observer callbacks when iterating over group members.
///
/// # Safety
///
/// `subject` must be a valid group subject pointer and `index` must be
/// within the bounds of the group's member list.
pub unsafe fn subject_get_group_element(
    subject: *mut lv_subject_t,
    index: i32,
) -> *mut lv_subject_t {
    // SAFETY: caller guarantees subject is a valid group and index is in bounds.
    unsafe { lv_subject_get_group_element(subject, index) }
}
