// SPDX-License-Identifier: MIT OR Apache-2.0
//! View navigation stack with modal overlay support.
//!
//! `Navigator` manages a stack of [`View`](crate::view::View) instances,
//! supporting push/pop/replace transitions and modal overlays. Only the
//! topmost view (and any active modal) have live LVGL widgets.
//!
//! Views cannot call Navigator methods directly (the navigator owns
//! the view). Instead, [`View::update`](crate::view::View::update) and
//! [`View::on_event`](crate::view::View::on_event) return
//! [`NavAction`](crate::view::NavAction), which the render loop dispatches.

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use embassy_time::Duration;
use oxivgl_sys::*;

use crate::driver::get_tick_ms;

use crate::view::{
    AnyView, NavAction, NavigationError, View,
    take_pending_event_action,
};
use crate::widgets::{AsLvHandle, Obj, Screen, ScreenAnim};

/// Entry on the navigation stack, pairing a type-erased view with its
/// owning screen object (if any).
struct ViewEntry {
    view: Box<dyn AnyView>,
    /// The LVGL screen created for this view. `None` for the root view
    /// which uses LVGL's default screen.
    screen: Option<Obj<'static>>,
}

/// View navigation stack with modal overlay support.
///
/// The navigator owns all view instances. Views lower in the stack
/// have their widget trees destroyed but their struct state preserved.
/// Only the topmost view (and any active modal) have live widgets.
///
/// # Usage
///
/// For single-screen applications, use
/// [`run_app`](crate::view::run_app) directly. `Navigator` is for
/// multi-screen applications that need push/pop/replace/modal.
pub struct Navigator {
    /// Full-screen navigation stack. Index 0 is the root view.
    stack: Vec<ViewEntry>,
    /// Currently active modal, if any. Rendered on `lv_layer_top()`.
    modal: Option<Box<dyn AnyView>>,
    /// Currently active global toast, if any. Rendered as a child of
    /// `lv_layer_sys()` so it persists across page switches and stays
    /// above any modal. See [`Navigator::show_toast`].
    toast: Option<Box<dyn AnyView>>,
    /// The container the active toast was created into. Owned here
    /// (rather than by the toast view) so dismissal deletes exactly the
    /// toast's widgets and nothing else on the system layer.
    toast_container: Option<Obj<'static>>,
    /// Auto-dismiss deadline for the active toast, in `get_tick_ms` units
    /// (wrap-aware u32 milliseconds). Compared via `wrapping_sub`.
    toast_deadline_ms: Option<u32>,
}

impl Navigator {
    /// Create a new empty navigator.
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            modal: None,
            toast: None,
            toast_container: None,
            toast_deadline_ms: None,
        }
    }

    /// Push the initial root view. Called once during setup.
    ///
    /// The root view uses the default LVGL screen. Its widgets are
    /// created immediately.
    pub fn push_root(&mut self, view: impl View) {
        let mut boxed: Box<dyn AnyView> = Box::new(view);

        // Use the default active screen as the container. Child suppresses
        // Drop so the LVGL screen is never deleted by Rust.
        let screen_handle = unsafe { lv_screen_active() };
        assert!(!screen_handle.is_null(), "no active screen");
        let container = Obj::from_raw_non_owning(screen_handle);

        boxed
            .create(&container)
            .expect("root view create failed");

        // register_events() default calls register_event_on(self, lv_screen_active()).
        // lv_screen_active() is the default screen — correct at this point.
        boxed.register_events();
        boxed.did_show();

        self.stack.push(ViewEntry {
            view: boxed,
            screen: None,
        });
    }

    /// Push a new view onto the stack.
    ///
    /// 1. Calls `will_hide()` on the current top view.
    /// 2. Creates a new LVGL screen for the new view.
    /// 3. Loads the new screen (makes it active).
    /// 4. Calls `create(container)` on the new view.
    /// 5. Registers event handlers.
    /// 6. Cleans the old screen's widget tree (preserving view state).
    /// 7. Calls `did_show()` on the new view.
    pub fn push(&mut self, view: impl View, anim: Option<ScreenAnim>) {
        self.push_boxed(Box::new(view), anim);
    }

    /// Push a boxed (type-erased) view.
    fn push_boxed(&mut self, mut boxed: Box<dyn AnyView>, anim: Option<ScreenAnim>) {
        // Notify current top view.
        if let Some(top) = self.stack.last_mut() {
            top.view.will_hide();
        }

        // Capture the old screen handle BEFORE loading the new screen,
        // because lv_screen_active() will change after Screen::load.
        let old_screen_h = self.stack.last().map(|top| {
            top.screen
                .as_ref()
                .map(|s| s.lv_handle())
                .unwrap_or_else(|| {
                    // Root view uses the LVGL default screen.
                    unsafe { lv_screen_active() }
                })
        });

        // Create a new screen for the incoming view.
        let new_screen = Screen::create();

        // Load the new screen BEFORE create/register_events so that
        // lv_screen_active() returns the new screen during those calls.
        if let Some(ref a) = anim {
            Screen::load(&new_screen, a, false);
        } else {
            Screen::load_instant(&new_screen);
        }

        boxed
            .create(&new_screen)
            .expect("pushed view create failed");

        // register_events() calls register_event_on(self, lv_screen_active()).
        // Since we loaded new_screen above, lv_screen_active() == new_screen.
        boxed.register_events();

        // Clean the old screen's children (widget tree) to free memory,
        // but keep the screen object alive for potential pop animation.
        // SAFETY: old_screen_h was captured above while still valid. The old
        // screen object is still alive (just no longer active). lv_obj_clean
        // deletes all children but keeps the screen itself. Note: any Obj
        // wrappers held by the old view now contain stale pointers — their
        // Drop uses lv_obj_is_valid() as a guard (see spec-memory-lifetime §8.1).
        if let Some(h) = old_screen_h {
            unsafe { lv_obj_clean(h) };
        }

        boxed.did_show();

        self.stack.push(ViewEntry {
            view: boxed,
            screen: Some(new_screen),
        });
    }

    /// Pop the current view and return to the previous one.
    ///
    /// Returns `Err(NavigationError::StackEmpty)` if only the root view
    /// remains (the root cannot be popped).
    pub fn pop(&mut self, anim: Option<ScreenAnim>) -> Result<(), NavigationError> {
        if self.stack.len() <= 1 {
            return Err(NavigationError::StackEmpty);
        }

        // Remove the top view — will_hide + drop.
        let mut popped = self.stack.pop().unwrap();
        popped.view.will_hide();

        // Rebuild the now-top view's widgets.
        let top = self.stack.last_mut().unwrap();

        // Load the restored screen BEFORE dropping the popped screen.
        // This ensures lv_screen_active() returns the correct screen
        // during create/register_events, and avoids the undefined state
        // of having no active screen.
        let container_handle = if let Some(ref top_screen) = top.screen {
            if let Some(ref a) = anim {
                Screen::load(top_screen, a, false);
            } else {
                Screen::load_instant(top_screen);
            }
            top_screen.lv_handle()
        } else {
            // Root view: load the default LVGL screen. We must get its
            // handle BEFORE dropping popped (which deletes popped.screen).
            // SAFETY: lv_display_get_default/lv_display_get_screen returns
            // the LVGL default screen (index 0), which is always valid.
            let default_screen = unsafe {
                let disp = lv_display_get_default();
                lv_display_get_screen_active(disp)
            };
            // The default screen may be behind the popped screen. Load it
            // before dropping so it becomes active.
            Screen::load_instant(&Obj::from_raw_non_owning(default_screen));
            default_screen
        };

        // Now safe to drop the popped view and its screen.
        drop(popped);

        // Non-owning handle — the screen is owned by the ViewEntry, not
        // this temporary. Child suppresses Drop so no screen deletion.
        let container = Obj::from_raw_non_owning(container_handle);
        top.view
            .create(&container)
            .map_err(NavigationError::CreateFailed)?;

        top.view.register_events();
        top.view.did_show();
        Ok(())
    }

    /// Replace the current view without preserving it on the stack.
    ///
    /// The current view is dropped. The new view takes its place at the
    /// same stack depth.
    pub fn replace(&mut self, view: impl View, anim: Option<ScreenAnim>) {
        self.replace_boxed(Box::new(view), anim);
    }

    /// Replace with a boxed view.
    fn replace_boxed(&mut self, mut boxed: Box<dyn AnyView>, anim: Option<ScreenAnim>) {
        // Notify the view being replaced so it can save state if needed.
        if let Some(top) = self.stack.last_mut() {
            top.view.will_hide();
        }

        // Create a new screen and load it BEFORE dropping the old view,
        // ensuring there is always a valid active screen.
        let new_screen = Screen::create();
        if let Some(ref a) = anim {
            Screen::load(&new_screen, a, false);
        } else {
            Screen::load_instant(&new_screen);
        }

        // Now safe to drop the old view and its screen.
        self.stack.pop();

        boxed
            .create(&new_screen)
            .expect("replaced view create failed");
        boxed.register_events();
        boxed.did_show();

        self.stack.push(ViewEntry {
            view: boxed,
            screen: Some(new_screen),
        });
    }

    /// Show a modal overlay on top of the current view.
    ///
    /// The current view's widget tree stays alive and visible underneath.
    /// The modal's widgets are created on `lv_layer_top()`.
    ///
    /// Only one modal can be active at a time. Calling `modal()` while
    /// a modal is already open replaces it.
    pub fn modal(&mut self, view: impl View) {
        self.modal_boxed(Box::new(view));
    }

    /// Show a boxed modal.
    fn modal_boxed(&mut self, mut boxed: Box<dyn AnyView>) {
        // Dismiss any existing modal first.
        if self.modal.is_some() {
            let _ = self.dismiss_modal();
        }

        let layer_top = Screen::layer_top();
        boxed
            .create(&layer_top)
            .expect("modal view create failed");
        // For modals, register_events default would register on
        // lv_screen_active() which is the background view's screen.
        // Modal views should override register_events to register on
        // the layer_top container instead, or use EVENT_BUBBLE.
        // We call register_events() and trust the view's override.
        boxed.register_events();
        boxed.did_show();
        self.modal = Some(boxed);
    }

    /// Dismiss the current modal overlay.
    ///
    /// Cleans `lv_layer_top()` children. Returns `Err` if no modal is active.
    pub fn dismiss_modal(&mut self) -> Result<(), NavigationError> {
        if let Some(mut modal) = self.modal.take() {
            modal.will_hide();
            // SAFETY: lv_layer_top() returns the global overlay object (valid
            // after lv_init). lv_obj_clean deletes all children. Any Obj
            // wrappers in the modal view now hold stale pointers — their Drop
            // uses lv_obj_is_valid() as a guard (spec-memory-lifetime §8.1).
            // We clean before dropping the modal so LVGL removes its widgets
            // from the display immediately.
            let layer = unsafe { lv_layer_top() };
            unsafe { lv_obj_clean(layer) };
            // modal is dropped here — Obj::drop guards prevent double-free.
            Ok(())
        } else {
            Err(NavigationError::NoActiveModal)
        }
    }

    /// Whether a modal is currently showing.
    pub fn has_modal(&self) -> bool {
        self.modal.is_some()
    }

    /// Show a global passive status overlay on the system layer.
    ///
    /// Unlike [`modal`](Self::modal), the toast:
    /// - lives on `lv_layer_sys()` so it persists across `push` /
    ///   `replace` / `pop` and is unaffected by modal dismissal;
    /// - is **passive** — `register_events` is never called, and every
    ///   widget the view creates has the `CLICKABLE` flag cleared so
    ///   touches pass through to the view beneath;
    /// - has its auto-dismiss timer owned by the navigator. If
    ///   `duration` is `Some`, the toast is dismissed automatically the
    ///   next time [`tick_toast`](Self::tick_toast) runs after the
    ///   deadline. `None` means the caller must dismiss explicitly.
    ///
    /// Calling `show_toast` while one is already active replaces it.
    pub fn show_toast(&mut self, view: impl View, duration: Option<Duration>) {
        self.show_toast_boxed(Box::new(view), duration);
    }

    fn show_toast_boxed(
        &mut self,
        mut boxed: Box<dyn AnyView>,
        duration: Option<Duration>,
    ) {
        // Replace any active toast first.
        if self.toast.is_some() {
            let _ = self.dismiss_toast();
        }

        // Create a dedicated container as a child of lv_layer_sys() so
        // dismissal deletes only the toast's widgets — the system layer
        // itself stays alive (it is LVGL-owned).
        let sys_layer = unsafe { lv_layer_sys() };
        assert!(!sys_layer.is_null(), "lv_layer_sys returned NULL");
        // SAFETY: sys_layer is a valid LVGL container.
        let container_handle = unsafe { lv_obj_create(sys_layer) };
        assert!(!container_handle.is_null(), "toast container creation failed");
        let container = Obj::from_raw(container_handle);

        if let Err(e) = boxed.create(&container) {
            warn!("nav show_toast: create failed: {:?}", e);
            // Drop the container so we leave the sys layer clean.
            drop(container);
            return;
        }

        // Strip CLICKABLE from the container and all its descendants so
        // touches pass through to whatever is beneath the toast. This
        // enforces the passivity contract regardless of what the view did.
        // SAFETY: container_handle is the freshly-created object above;
        // its tree is what the view just populated.
        unsafe { remove_clickable_recursive(container_handle) };

        // Intentionally do NOT call boxed.register_events(): the default
        // impl registers on lv_screen_active() (the background view's
        // screen) and would dangle across page switches — exactly the
        // bug this toast surface exists to avoid.

        boxed.did_show();

        self.toast = Some(boxed);
        self.toast_container = Some(container);
        self.toast_deadline_ms = duration.map(|d| {
            // Saturate the embassy Duration into u32 ms (≈49.7 days max),
            // then wrap-add to the current tick. The compare in tick_toast
            // uses `wrapping_sub` so wrap-around is correct as long as the
            // duration is < ~25 days.
            let ms = d.as_millis().min(u32::MAX as u64) as u32;
            get_tick_ms().wrapping_add(ms)
        });
    }

    /// Dismiss the active toast overlay.
    ///
    /// Returns `Err(NoActiveToast)` if none is showing.
    pub fn dismiss_toast(&mut self) -> Result<(), NavigationError> {
        let mut toast = match self.toast.take() {
            Some(t) => t,
            None => return Err(NavigationError::NoActiveToast),
        };
        self.toast_deadline_ms = None;
        toast.will_hide();

        // Delete the toast's container (and thus its widget subtree).
        // The toast view's internal Obj wrappers now hold stale pointers;
        // Obj::Drop uses lv_obj_is_valid as a guard (spec §8.1).
        if let Some(container) = self.toast_container.take() {
            let handle = container.lv_handle();
            // Suppress container's own Drop — we delete it explicitly.
            core::mem::forget(container);
            // SAFETY: handle was returned by lv_obj_create above; checked
            // valid here in case lv_obj_clean on the sys layer destroyed
            // it externally between show and dismiss.
            unsafe {
                if lv_obj_is_valid(handle) {
                    lv_obj_delete(handle);
                }
            }
        }
        drop(toast);
        Ok(())
    }

    /// Whether a toast is currently showing.
    pub fn has_toast(&self) -> bool {
        self.toast.is_some()
    }

    /// Get a mutable reference to the active toast view, if any.
    pub fn active_toast_mut(&mut self) -> Option<&mut dyn AnyView> {
        self.toast.as_mut().map(|t| &mut **t as &mut dyn AnyView)
    }

    /// Maintenance tick for the toast slot — call once per render-loop
    /// iteration.
    ///
    /// - Dismisses the toast if its auto-dismiss deadline has passed.
    /// - Self-heals the slot if the toast container was destroyed
    ///   externally (e.g. some other code cleared the system layer):
    ///   drops the orphaned view so the slot is reusable.
    pub fn tick_toast(&mut self) {
        if self.toast.is_none() {
            return;
        }

        // External-destruction guard: if the container handle is no
        // longer valid, drop the view + clear the slot.
        if let Some(container) = self.toast_container.as_ref() {
            let handle = container.lv_handle();
            // SAFETY: lv_obj_is_valid handles any pointer (returns false
            // for freed objects).
            if !unsafe { lv_obj_is_valid(handle) } {
                if let Some(mut t) = self.toast.take() {
                    t.will_hide();
                }
                // Suppress the container's Drop — its target is already gone.
                if let Some(orphan) = self.toast_container.take() {
                    core::mem::forget(orphan);
                }
                self.toast_deadline_ms = None;
                return;
            }
        }

        // Wrap-aware compare: `now - deadline >= 0` (as i32) means we've
        // reached the deadline, robust to u32 wrap.
        if let Some(deadline) = self.toast_deadline_ms
            && get_tick_ms().wrapping_sub(deadline) as i32 >= 0
        {
            let _ = self.dismiss_toast();
        }
    }

    /// Number of views on the stack.
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Get a mutable reference to the active (topmost) view.
    pub fn active_view_mut(&mut self) -> Option<&mut dyn AnyView> {
        self.stack
            .last_mut()
            .map(|e| &mut *e.view as &mut dyn AnyView)
    }

    /// Get a mutable reference to the active modal, if any.
    pub fn active_modal_mut(&mut self) -> Option<&mut dyn AnyView> {
        self.modal.as_mut().map(|m| &mut **m as &mut dyn AnyView)
    }

    /// Process a [`NavAction`] returned by a view.
    pub fn process_action(&mut self, action: NavAction) {
        match action {
            NavAction::None => {}
            NavAction::Push(view, anim) => self.push_boxed(view, anim),
            NavAction::Pop(anim) => {
                if let Err(e) = self.pop(anim) {
                    warn!("nav pop failed: {}", e);
                }
            }
            NavAction::Replace(view, anim) => self.replace_boxed(view, anim),
            NavAction::Modal(view) => self.modal_boxed(view),
            NavAction::DismissModal => {
                if let Err(e) = self.dismiss_modal() {
                    warn!("nav dismiss_modal failed: {}", e);
                }
            }
            NavAction::ShowToast(view, duration) => self.show_toast_boxed(view, duration),
            NavAction::DismissToast => {
                if let Err(e) = self.dismiss_toast() {
                    warn!("nav dismiss_toast failed: {}", e);
                }
            }
        }
    }

    /// Process any pending event action stashed by the on_event trampoline.
    /// Returns `true` if an event action was processed.
    pub fn process_pending_event_action(&mut self) -> bool {
        if let Some(action) = take_pending_event_action() {
            self.process_action(action);
            true
        } else {
            false
        }
    }
}

impl Default for Navigator {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip `CLICKABLE` and `CLICK_FOCUSABLE` from `obj` and every descendant.
///
/// Used to make a toast subtree input-transparent regardless of what the
/// toast view did when it built its widgets.
///
/// # Safety
/// `obj` must be a valid `lv_obj_t*` whose subtree is fully constructed
/// (no concurrent mutation from another task).
unsafe fn remove_clickable_recursive(obj: *mut lv_obj_t) {
    if obj.is_null() {
        return;
    }
    let flags = crate::enums::ObjFlag::CLICKABLE.0 | crate::enums::ObjFlag::CLICK_FOCUSABLE.0;
    unsafe {
        lv_obj_remove_flag(obj, flags);
        let n = lv_obj_get_child_count(obj);
        for i in 0..n {
            let child = lv_obj_get_child(obj, i as i32);
            remove_clickable_recursive(child);
        }
    }
}
