#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 2 — PIN login screen with state bindings
//!
//! Demonstrates `bind_state_if_eq`, `bind_state_if_not_eq`, `bind_checked`,
//! `bind_text_map`, and `on_change`.
//!
//! - A textarea in password mode accepts a PIN.
//! - Pressing Enter checks whether the text equals `"hello"`.
//! - An info label is reactively updated via `bind_text_map`.
//! - A "LOG OUT" button clears the auth state.
//! - A "START ENGINE" checkable button is two-way bound to `engine_subject`.

use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    view::{View, register_event_on},
    widgets::{
        Align, Button, Child, Keyboard, KeyboardMode, Label, Screen, Subject, Textarea,
        WidgetError,
    },
};

const LOGGED_OUT: i32 = 0;
const LOGGED_IN: i32 = 1;
const AUTH_FAILED: i32 = 2;

struct Observer2 {
    ta: Textarea<'static>,
    btn_logout: Button<'static>,
    _info_label: Label<'static>,
    _kb: Keyboard<'static>,
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

        // Info label — reactively updated via bind_text_map.
        let info_label = Label::new(&screen)?;
        info_label.text("Logged out").align(Align::TopLeft, 10, 120);
        info_label.bind_text_map(&auth_state_subject, |state| match state {
            LOGGED_IN => "Login successful",
            AUTH_FAILED => "Login failed",
            _ => "Logged out",
        });

        // START ENGINE checkable button — two-way bound to engine_subject.
        let btn_engine = Button::new(&screen)?;
        btn_engine
            .add_flag(ObjFlag::CHECKABLE)
            .size(160, 40)
            .align(Align::TopLeft, 10, 160);
        let lbl_engine = Child::new(Label::new(&btn_engine)?);
        lbl_engine.text("START ENGINE").center();
        btn_engine.bind_checked(&engine_subject);

        // Engine observer — fires when engine state changes.
        engine_subject.on_change(|_value| {
            // In a real app, would set/clear a GPIO pin.
        });

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
            if self.ta.get_text() == Some("hello") {
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
