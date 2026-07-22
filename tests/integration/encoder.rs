// SPDX-License-Identifier: MIT OR Apache-2.0
//! Encoder input device tests.
//!
//! Validates that [`EncoderIndev`] registers an ENCODER device wired to its
//! [`EncoderState`], that dropping it unregisters cleanly, and that the three
//! producer channels behave per the LVGL encoder model:
//! - [`turn`](EncoderState::turn) moves focus in *navigate* mode and changes
//!   value in *edit* mode, with multi-step deltas preserved;
//! - [`click`](EncoderState::click) enters edit mode on an editable widget;
//! - [`long_press`](EncoderState::long_press) toggles edit mode (the only way to
//!   *leave* edit in a multi-object group).
//!
//! [`EncoderState`]: oxivgl::indev::EncoderState

use crate::common::{fresh_screen, pump};

use oxivgl::enums::ObjState;
use oxivgl::group::Group;
use oxivgl::indev::{EncoderIndev, EncoderState};
use oxivgl::widgets::Slider;

// Each test uses its own `'static` state so `find_encoder` can identify the
// device it created by user-data pointer, independent of other tests.
static ENC_REG: EncoderState = EncoderState::new();
static ENC_FOCUS: EncoderState = EncoderState::new();
static ENC_MULTI: EncoderState = EncoderState::new();
static ENC_EDIT: EncoderState = EncoderState::new();
static ENC_PENDING: EncoderState = EncoderState::new();

/// Find the registered ENCODER indev whose user data points at `state`.
fn find_encoder(state: &EncoderState) -> Option<*mut oxivgl_sys::lv_indev_t> {
    let target = state as *const EncoderState as *const core::ffi::c_void;
    // SAFETY: lv_indev_get_next(NULL) walks the global indev list; get_type and
    // get_user_data are safe on any non-null indev.
    unsafe {
        let mut indev = oxivgl_sys::lv_indev_get_next(core::ptr::null_mut());
        while !indev.is_null() {
            let is_encoder = oxivgl_sys::lv_indev_get_type(indev)
                == oxivgl_sys::lv_indev_type_t_LV_INDEV_TYPE_ENCODER;
            if is_encoder
                && oxivgl_sys::lv_indev_get_user_data(indev) as *const core::ffi::c_void == target
            {
                return Some(indev);
            }
            indev = oxivgl_sys::lv_indev_get_next(indev);
        }
        None
    }
}

/// Whether the group bound to `indev` is currently in edit mode.
fn editing(indev: *mut oxivgl_sys::lv_indev_t) -> bool {
    // SAFETY: indev is a live encoder device; lv_indev_get_group returns its
    // bound group (non-null here, set via set_group), lv_group_get_editing reads
    // the editing flag.
    unsafe {
        let g = oxivgl_sys::lv_indev_get_group(indev);
        !g.is_null() && oxivgl_sys::lv_group_get_editing(g)
    }
}

#[test]
fn encoder_registers_as_encoder_and_unregisters_on_drop() {
    let _screen = fresh_screen();
    assert!(find_encoder(&ENC_REG).is_none(), "no encoder before creation");

    let enc = EncoderIndev::new(&ENC_REG).unwrap();
    pump();
    assert!(
        find_encoder(&ENC_REG).is_some(),
        "EncoderIndev::new registers an ENCODER device wired to its EncoderState",
    );

    drop(enc);
    pump();
    assert!(
        find_encoder(&ENC_REG).is_none(),
        "dropping EncoderIndev removes the device from LVGL",
    );
}

#[test]
fn encoder_turn_moves_group_focus_in_navigate_mode() {
    let screen = fresh_screen();

    let group = Group::new().unwrap();
    let a = Slider::new(&screen).unwrap();
    let b = Slider::new(&screen).unwrap();
    let c = Slider::new(&screen).unwrap();
    group.add_obj(&a);
    group.add_obj(&b);
    group.add_obj(&c);

    // EVENT mode: LVGL never polls; we drain via read().
    let enc = EncoderIndev::new_event(&ENC_FOCUS).unwrap();
    enc.set_group(&group);
    pump();
    let indev = find_encoder(&ENC_FOCUS).expect("encoder present");
    assert!(!editing(indev), "starts in navigate mode");
    assert!(a.has_state(ObjState::FOCUSED), "first member focused on add");

    // One +1 turn advances focus one step (navigate).
    ENC_FOCUS.turn(1);
    assert!(ENC_FOCUS.has_pending(), "turn is pending until read");
    enc.read();
    pump();
    assert!(!ENC_FOCUS.has_pending(), "read() drained the turn");
    assert!(b.has_state(ObjState::FOCUSED), "turn(1) → next member focused");
    assert!(!a.has_state(ObjState::FOCUSED), "first member loses focus");

    drop(enc);
}

#[test]
fn encoder_multi_step_turn_moves_focus_by_count() {
    let screen = fresh_screen();

    let group = Group::new().unwrap();
    let items: Vec<_> = (0..4).map(|_| Slider::new(&screen).unwrap()).collect();
    for it in &items {
        group.add_obj(it);
    }

    let enc = EncoderIndev::new_event(&ENC_MULTI).unwrap();
    enc.set_group(&group);
    pump();
    assert!(items[0].has_state(ObjState::FOCUSED), "first member focused on add");

    // A multi-tap "+" arrives as a single accumulated delta: turn(3) must move
    // focus three steps, not one — the count is preserved end to end.
    ENC_MULTI.turn(3);
    enc.read();
    pump();
    assert!(
        items[3].has_state(ObjState::FOCUSED),
        "turn(3) advances focus by three, not one",
    );
    assert!(!items[0].has_state(ObjState::FOCUSED));

    drop(enc);
}

#[test]
fn encoder_click_enters_edit_and_long_press_leaves() {
    let screen = fresh_screen();

    let group = Group::new().unwrap();
    // Two editable widgets: edit-mode toggling requires an editable focus and a
    // group with more than one object (LVGL refuses to leave edit otherwise).
    let a = Slider::new(&screen).unwrap();
    let b = Slider::new(&screen).unwrap();
    group.add_obj(&a);
    group.add_obj(&b);

    let enc = EncoderIndev::new_event(&ENC_EDIT).unwrap();
    enc.set_group(&group);
    pump();
    let indev = find_encoder(&ENC_EDIT).expect("encoder present");
    assert!(!editing(indev), "starts in navigate mode");

    // A click on an editable focused widget enters edit mode.
    ENC_EDIT.click();
    enc.read();
    pump();
    assert!(editing(indev), "click enters edit mode on an editable widget");

    // A long press toggles edit mode off — the direct route for the BSP's
    // pre-decoded Long event (issue #127).
    ENC_EDIT.long_press();
    enc.read();
    pump();
    assert!(!editing(indev), "long_press toggles edit mode back off");

    // And toggles it on again (it is a toggle, not a one-way leave).
    ENC_EDIT.long_press();
    enc.read();
    pump();
    assert!(editing(indev), "long_press toggles edit mode on again");

    drop(enc);
}

#[test]
fn encoder_has_pending_tracks_all_three_channels() {
    let _screen = fresh_screen();
    let enc = EncoderIndev::new_event(&ENC_PENDING).unwrap();
    pump();

    assert!(!ENC_PENDING.has_pending(), "idle: nothing pending");

    ENC_PENDING.turn(1);
    assert!(ENC_PENDING.has_pending(), "turn is pending");
    enc.read();
    assert!(!ENC_PENDING.has_pending(), "turn drained");

    ENC_PENDING.click();
    assert!(ENC_PENDING.has_pending(), "click is pending");
    enc.read(); // PRESSED
    enc.read(); // RELEASED
    assert!(!ENC_PENDING.has_pending(), "click fully drained (press + release)");

    ENC_PENDING.long_press();
    assert!(ENC_PENDING.has_pending(), "long press is pending");
    enc.read();
    assert!(!ENC_PENDING.has_pending(), "long press consumed");

    drop(enc);
}
