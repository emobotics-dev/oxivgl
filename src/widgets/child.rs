// SPDX-License-Identifier: MIT OR Apache-2.0
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::AsLvHandle;

/// Non-owning wrapper for a widget whose LVGL object is owned by a parent container.
///
/// LVGL automatically deletes children when their parent is deleted. Wrapping child
/// widgets in `Child<W>` suppresses Rust's `Drop` on the inner widget, preventing the
/// double-free that would otherwise occur when both LVGL and Rust try to delete the
/// same object.
///
/// Use [`Child::new`] for children that must be retained for later method calls (e.g.
/// `set_value`). For decoration-only children configured once and never accessed again,
/// use [`detach`] instead.
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Arc, Child, Label, Obj, Screen};
///
/// struct Gauge<'p> {
///     container: Obj<'p>,
///     arc:       Child<Arc<'p>>,
///     label:     Child<Label<'p>>,
/// }
/// ```
#[repr(transparent)]
pub struct Child<W>(ManuallyDrop<W>);

impl<W> Child<W> {
    /// Wrap `widget` as a non-owning child.
    ///
    /// The caller must ensure the widget's LVGL parent outlives this `Child`.
    /// LVGL will delete the object when the parent is deleted; Rust's `Drop` is suppressed.
    pub fn new(widget: W) -> Self {
        Child(ManuallyDrop::new(widget))
    }
}

impl<W> Deref for Child<W> {
    type Target = W;
    fn deref(&self) -> &W {
        &self.0
    }
}

impl<W> DerefMut for Child<W> {
    fn deref_mut(&mut self) -> &mut W {
        &mut self.0
    }
}

impl<W: Default> Default for Child<W> {
    fn default() -> Self {
        Child(ManuallyDrop::new(W::default()))
    }
}

impl<W: core::fmt::Debug> core::fmt::Debug for Child<W> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&*self.0, f)
    }
}

impl<W: AsLvHandle> AsLvHandle for Child<W> {
    fn lv_handle(&self) -> *mut lvgl_rust_sys::lv_obj_t {
        self.0.lv_handle()
    }
}

/// Consume `widget` without running its destructor.
///
/// Use for decoration-only children that are configured once and never accessed again.
/// LVGL will delete the object when its parent is deleted.
///
/// For children that need to be retained for later calls, use [`Child`] instead.
pub fn detach<W>(widget: W) {
    core::mem::forget(widget);
}
