// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tests for the navigator-level global toast overlay.

use crate::common::{ensure_init, pump};

use embassy_time::Duration;
use oxivgl::enums::ObjFlag;
use oxivgl::navigator::Navigator;
use oxivgl::view::{NavigationError, View};
use oxivgl::widgets::{Button, Label, Obj, WidgetError};

// ── Test fixtures ────────────────────────────────────────────────────────────

/// A trivial full-screen view used as the root in these tests.
#[derive(Default)]
struct EmptyRoot;
impl View for EmptyRoot {
    fn create(&mut self, _container: &Obj<'static>) -> Result<(), WidgetError> {
        Ok(())
    }
}

/// A passive toast view: one Label.
#[derive(Default)]
struct LabelToast;
impl View for LabelToast {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let lbl = Label::new(container)?;
        lbl.text("status");
        Ok(())
    }
}

/// A toast view that builds a Button — used to verify Navigator strips
/// CLICKABLE regardless of what the view did.
#[derive(Default)]
struct ButtonToast;
impl View for ButtonToast {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let _btn = Button::new(container)?;
        Ok(())
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn sys_layer_child_count() -> u32 {
    // SAFETY: lv_layer_sys() returns a valid LVGL global after init.
    unsafe { oxivgl_sys::lv_obj_get_child_count(oxivgl_sys::lv_layer_sys()) }
}

fn fresh_navigator() -> Navigator {
    ensure_init();
    // Establish a fresh active screen (other tests may have left LVGL
    // without one) and clear any residue from the system layer.
    // SAFETY: LVGL initialised; lv_obj_create(NULL) creates a screen.
    unsafe {
        let new_screen = oxivgl_sys::lv_obj_create(core::ptr::null_mut());
        oxivgl_sys::lv_screen_load(new_screen);
        oxivgl_sys::lv_obj_clean(oxivgl_sys::lv_layer_sys());
    }
    let mut nav = Navigator::new();
    nav.push_root(EmptyRoot);
    nav
}

/// Walk `obj` and every descendant; assert `CLICKABLE` is cleared.
fn assert_subtree_not_clickable(obj: *mut oxivgl_sys::lv_obj_t) {
    // SAFETY: caller passes a valid handle; we don't mutate the tree here.
    unsafe {
        let flags = ObjFlag::CLICKABLE.0;
        assert_eq!(
            oxivgl_sys::lv_obj_has_flag(obj, flags),
            false,
            "subtree node still has CLICKABLE set",
        );
        let n = oxivgl_sys::lv_obj_get_child_count(obj);
        for i in 0..n {
            let c = oxivgl_sys::lv_obj_get_child(obj, i as i32);
            assert_subtree_not_clickable(c);
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn show_then_dismiss_clears_sys_layer() {
    let mut nav = fresh_navigator();
    assert!(!nav.has_toast());
    assert_eq!(sys_layer_child_count(), 0);

    nav.show_toast(LabelToast, None);
    pump();
    assert!(nav.has_toast());
    assert_eq!(sys_layer_child_count(), 1);

    nav.dismiss_toast().expect("dismiss_toast");
    pump();
    assert!(!nav.has_toast());
    assert_eq!(sys_layer_child_count(), 0);
}

#[test]
fn dismiss_without_toast_is_error() {
    let mut nav = fresh_navigator();
    match nav.dismiss_toast() {
        Err(NavigationError::NoActiveToast) => {}
        other => panic!("expected NoActiveToast, got {:?}", other),
    }
}

#[test]
fn auto_dismiss_after_duration() {
    let mut nav = fresh_navigator();
    nav.show_toast(LabelToast, Some(Duration::from_millis(40)));
    assert!(nav.has_toast());

    // Before the deadline, tick must NOT dismiss.
    std::thread::sleep(std::time::Duration::from_millis(10));
    nav.tick_toast();
    assert!(nav.has_toast(), "toast dismissed too early");

    // After the deadline, tick must dismiss.
    std::thread::sleep(std::time::Duration::from_millis(60));
    nav.tick_toast();
    assert!(!nav.has_toast(), "toast not auto-dismissed");
    pump();
    assert_eq!(sys_layer_child_count(), 0);
}

#[test]
fn persists_across_push_replace_pop() {
    // Push twice up front so we can `pop` back to a non-root entry,
    // sidestepping a pre-existing pop-to-root edge case in Navigator
    // unrelated to the toast feature.
    let mut nav = fresh_navigator();
    nav.push(EmptyRoot, None);
    nav.show_toast(LabelToast, None);
    let baseline = sys_layer_child_count();
    assert_eq!(baseline, 1);

    nav.push(EmptyRoot, None);
    assert!(nav.has_toast(), "toast slot cleared by push");
    assert_eq!(sys_layer_child_count(), baseline, "toast widgets removed by push");

    nav.replace(EmptyRoot, None);
    assert!(nav.has_toast(), "toast slot cleared by replace");
    assert_eq!(sys_layer_child_count(), baseline, "toast widgets removed by replace");

    nav.pop(None).expect("pop");
    assert!(nav.has_toast(), "toast slot cleared by pop");
    assert_eq!(sys_layer_child_count(), baseline, "toast widgets removed by pop");

    nav.dismiss_toast().expect("dismiss_toast");
    assert_eq!(sys_layer_child_count(), 0);
}

#[test]
fn show_toast_replaces_existing() {
    let mut nav = fresh_navigator();
    nav.show_toast(LabelToast, None);
    assert_eq!(sys_layer_child_count(), 1);

    nav.show_toast(LabelToast, None);
    // Old toast container deleted, new one created — net count still 1.
    assert_eq!(sys_layer_child_count(), 1);

    nav.dismiss_toast().expect("dismiss");
    assert_eq!(sys_layer_child_count(), 0);
}

#[test]
fn input_transparency_strips_clickable() {
    let mut nav = fresh_navigator();
    nav.show_toast(ButtonToast, None);
    pump();

    // SAFETY: lv_layer_sys() is a valid LVGL global.
    let sys = unsafe { oxivgl_sys::lv_layer_sys() };
    assert_eq!(
        unsafe { oxivgl_sys::lv_obj_get_child_count(sys) },
        1,
        "expected exactly one toast container",
    );
    let container = unsafe { oxivgl_sys::lv_obj_get_child(sys, 0) };
    assert_subtree_not_clickable(container);

    nav.dismiss_toast().expect("dismiss");
}
