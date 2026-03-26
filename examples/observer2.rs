#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 2 — PIN login screen with state bindings
//!
//! Demonstrates `add_observer`, `add_observer_obj`, `bind_state_if_eq`,
//! `bind_state_if_not_eq`, and `bind_checked`.
//!
//! - A textarea in password mode accepts a PIN.
//! - Pressing Enter checks whether the text equals `"hello"`.
//! - An info label is updated by an observer callback.
//! - A "LOG OUT" button clears the auth state.
//! - A "START ENGINE" checkable button is two-way bound to `engine_subject`.

use lvgl_rust_sys::*;
use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    view::{View, register_event_on},
    widgets::{
        Align, Button, Child, Keyboard, KeyboardMode, Label, Screen, Subject, Textarea, WidgetError,
        observer_get_target_obj, subject_get_int_raw,
    },
};

const LOGGED_OUT: i32 = 0;
const LOGGED_IN: i32 = 1;
const AUTH_FAILED: i32 = 2;

/// Observer callback for the engine subject — fires when engine state changes.
unsafe extern "C" fn engine_state_observer_cb(
    _observer: *mut lv_observer_t,
    subject: *mut lv_subject_t,
) {
    unsafe {
        let v = subject_get_int_raw(subject);
        // In a real app, would set/clear a GPIO pin.
        let _ = v;
    }
}

/// Observer callback for the info label — updates label text based on auth state.
unsafe extern "C" fn info_label_observer_cb(
    observer: *mut lv_observer_t,
    subject: *mut lv_subject_t,
) {
    unsafe {
        let label_ptr = observer_get_target_obj(observer);
        let state = subject_get_int_raw(subject);
        let text = match state {
            LOGGED_IN => c"Login successful",
            AUTH_FAILED => c"Login failed",
            _ => c"Logged out",
        };
        lv_label_set_text(label_ptr, text.as_ptr());
    }
}

struct Observer2 {
    ta: Textarea<'static>,
    _kb: Keyboard<'static>,
    btn_logout: Button<'static>,
    _info_label: Label<'static>,
    _btn_engine: Button<'static>,
    // Subjects last — drop after widgets so observers are removed before deinit.
    _engine_subject: Subject,
    auth_state_subject: Subject,
}

impl View for Observer2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let auth_state_subject = Subject::new_int(LOGGED_OUT);
        let engine_subject = Subject::new_int(0);

        // Textarea in password mode, disabled when logged in.
        let ta = Textarea::new(&screen)?;
        ta.set_password_mode(true)
            .set_one_line(true)
            .set_placeholder_text("Password")
            .set_max_length(20)
            .size(200, 40)
            .align(Align::TopMid, 0, 20);
        ta.bind_state_if_eq(&auth_state_subject, ObjState::DISABLED, LOGGED_IN);
        ta.bubble_events();

        // Keyboard attached to textarea.
        let kb = Keyboard::new(&screen)?;
        kb.set_mode(KeyboardMode::TextLower);
        kb.set_textarea(&ta);

        // LOG OUT button — disabled when not logged in.
        let btn_logout = Button::new(&screen)?;
        btn_logout.size(120, 40).align(Align::TopLeft, 10, 70);
        let lbl_logout = Child::new(Label::new(&btn_logout)?);
        lbl_logout.text("LOG OUT").center();
        btn_logout.bind_state_if_not_eq(&auth_state_subject, ObjState::DISABLED, LOGGED_IN);
        btn_logout.bubble_events();

        // Info label — text updated by observer callback.
        let info_label = Label::new(&screen)?;
        info_label.text("Logged out").align(Align::TopLeft, 10, 120);
        auth_state_subject.add_observer_obj(
            info_label_observer_cb,
            &info_label,
            core::ptr::null_mut(),
        );

        // START ENGINE checkable button — two-way bound to engine_subject.
        let btn_engine = Button::new(&screen)?;
        btn_engine
            .add_flag(ObjFlag::CHECKABLE)
            .size(160, 40)
            .align(Align::TopLeft, 10, 160);
        let lbl_engine = Child::new(Label::new(&btn_engine)?);
        lbl_engine.text("START ENGINE").center();
        btn_engine.bind_checked(&engine_subject);

        // App-level observer for engine state (no widget).
        engine_subject.add_observer(engine_state_observer_cb, core::ptr::null_mut());

        Ok(Self {
            ta,
            _kb: kb,
            btn_logout,
            _info_label: info_label,
            _btn_engine: btn_engine,
            _engine_subject: engine_subject,
            auth_state_subject,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.ta.handle());
        register_event_on(self, self.btn_logout.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.ta, EventCode::READY) {
            let text_ptr = unsafe { lv_textarea_get_text(self.ta.handle()) };
            let text = unsafe { core::ffi::CStr::from_ptr(text_ptr) };
            if text == c"hello" {
                self.auth_state_subject.set_int(LOGGED_IN);
            } else {
                self.auth_state_subject.set_int(AUTH_FAILED);
            }
        }
        if event.matches(&self.btn_logout, EventCode::CLICKED) {
            self.auth_state_subject.set_int(LOGGED_OUT);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer2);
