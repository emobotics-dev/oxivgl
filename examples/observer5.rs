#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 5 — Firmware update state machine
//!
//! Demonstrates a multi-state UI driven by integer subjects and poll-based
//! timers. A "Firmware update" button opens a window that steps through:
//! IDLE → CONNECTING (spinner) → DOWNLOADING (progress arc) → READY
//! (restart prompt). The window's close button cancels at any stage.

use core::ffi::c_void;
use core::ptr::null_mut;

use oxivgl::{
    enums::{EventCode, ObjFlag},
    event::Event,
    style::{Selector, color_make},
    timer::Timer,
    view::View,
    widgets::{
        Align, Arc, AsLvHandle, Button, Child, Label, Screen, Spinner, Subject, Win,
        WidgetError,
    },
};
use oxivgl::symbols;

// --- State constants ---
const STATE_IDLE: i32 = 0;
const STATE_CONNECTING: i32 = 1;
const STATE_CONNECTED: i32 = 2;
const STATE_DOWNLOADING: i32 = 3;
const STATE_CANCEL: i32 = 4;
const STATE_READY: i32 = 5;

struct Observer5 {
    start_btn: Button<'static>,
    _start_label: Child<Label<'static>>,

    // Dynamic window state (None = closed)
    win: Option<Win<'static>>,
    close_btn_handle: *mut c_void,
    restart_btn_handle: *mut c_void,

    // Last polled state — used to detect transitions
    last_state: i32,

    // Timers (created / dropped per phase)
    connect_timer: Option<Timer>,
    download_timer: Option<Timer>,

    // Subjects — must be last so they drop after all widgets that observe them
    download_pct_subject: Subject,
    status_subject: Subject,
}

impl View for Observer5 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let download_pct_subject = Subject::new_int(0);
        let status_subject = Subject::new_int(STATE_IDLE);

        // Start button, centered on screen.
        let start_btn = Button::new(&screen)?;
        start_btn.center();
        let start_label = Child::new(Label::new(&start_btn)?);
        start_label.text("Firmware update").center();

        Ok(Self {
            start_btn,
            _start_label: start_label,
            win: None,
            close_btn_handle: null_mut(),
            restart_btn_handle: null_mut(),
            last_state: STATE_IDLE,
            connect_timer: None,
            download_timer: None,
            download_pct_subject,
            status_subject,
        })
    }

    fn on_event(&mut self, event: &Event) {
        // Start button — open the firmware update window.
        if event.matches(&self.start_btn, EventCode::CLICKED) && self.win.is_none() {
            let screen = match Screen::active() {
                Some(s) => s,
                None => return,
            };

            let win = match Win::new(&screen) {
                Ok(w) => w,
                Err(_) => return,
            };

            // Style: rounded corners + drop shadow.
            let shadow_color = color_make(0x88, 0x88, 0x88);
            win.radius(8, Selector::DEFAULT)
                .style_shadow_width(24, Selector::DEFAULT)
                .style_shadow_offset_x(2, Selector::DEFAULT)
                .style_shadow_offset_y(3, Selector::DEFAULT)
                .style_shadow_color(shadow_color, Selector::DEFAULT);

            // Title and close button in the header.
            win.add_title("Firmware update");
            let close_btn = win.add_button(&symbols::CLOSE, 40);
            close_btn.bubble_events();
            self.close_btn_handle = close_btn.lv_handle() as *mut c_void;

            self.win = Some(win);

            // Kick off the state machine.
            self.status_subject.set_int(STATE_IDLE);
            self.last_state = -1; // force re-entry into IDLE handling
        }

        // Close button — cancel the update.
        if !self.close_btn_handle.is_null()
            && event.target_handle() as *mut c_void == self.close_btn_handle
            && event.code() == EventCode::CLICKED
        {
            self.status_subject.set_int(STATE_CANCEL);
        }

        // Restart button — close window and return to idle.
        if !self.restart_btn_handle.is_null()
            && event.target_handle() as *mut c_void == self.restart_btn_handle
            && event.code() == EventCode::CLICKED
        {
            self.connect_timer = None;
            self.download_timer = None;
            self.close_btn_handle = null_mut();
            self.restart_btn_handle = null_mut();
            self.win = None;
            self.status_subject.set_int(STATE_IDLE);
            self.last_state = STATE_IDLE;
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        let status = self.status_subject.get_int();

        // Detect state transitions.
        if status != self.last_state {
            self.last_state = status;

            match status {
                STATE_IDLE => {
                    // Transition: show connecting UI, then wait for timer.
                    if let Some(ref win) = self.win {
                        let content = win.get_content();
                        content.clean();
                    }
                    if self.win.is_some() {
                        self.show_connecting()?;
                    }
                    self.status_subject.set_int(STATE_CONNECTING);
                    self.last_state = STATE_CONNECTING;
                }

                STATE_CONNECTING => {
                    // 2-second one-shot timer before moving to CONNECTED.
                    let timer = Timer::new(2000)?;
                    timer.set_repeat_count(1);
                    self.connect_timer = Some(timer);
                }

                STATE_CONNECTED => {
                    // Reset progress and move immediately to downloading.
                    self.download_pct_subject.set_int(0);
                    self.status_subject.set_int(STATE_DOWNLOADING);
                    self.last_state = STATE_DOWNLOADING;
                }

                STATE_DOWNLOADING => {
                    // Replace window content with arc + percentage label.
                    if let Some(ref win) = self.win {
                        let content = win.get_content();
                        content.clean();
                        // Child wrappers suppress Rust Drop — LVGL parent owns these.
                        let arc = Child::new(Arc::new(&*content)?);
                        arc.size(130, 130).center();
                        arc.set_range_raw(0, 100)
                            .remove_flag(ObjFlag::CLICKABLE);
                        arc.bind_value(&self.download_pct_subject);
                        let pct_label = Child::new(Label::new(&*content)?);
                        pct_label.center();
                        pct_label.bind_text(&self.download_pct_subject, c"%d %%");
                    }
                    // 50 ms repeating timer to increment progress.
                    self.download_timer = Some(Timer::new(50)?);
                }

                STATE_READY => {
                    // Show "firmware ready" message and restart button.
                    if let Some(ref win) = self.win {
                        let content = win.get_content();
                        content.clean();
                        // Child wrappers suppress Rust Drop — LVGL parent owns these.
                        let ready_label = Child::new(Label::new(&*content)?);
                        ready_label
                            .text("Firmware update is ready")
                            .align(Align::TopMid, 0, 20);
                        let restart_btn = Child::new(Button::new(&*content)?);
                        restart_btn.align(Align::TopMid, 0, 60);
                        restart_btn.bubble_events();
                        self.restart_btn_handle = restart_btn.lv_handle() as *mut c_void;
                        let restart_lbl = Child::new(Label::new(&restart_btn)?);
                        restart_lbl.text("Restart").center();
                    }
                }

                STATE_CANCEL => {
                    // Tear down everything.
                    self.connect_timer = None;
                    self.download_timer = None;
                    self.close_btn_handle = null_mut();
                    self.restart_btn_handle = null_mut();
                    self.win = None;
                    self.last_state = STATE_IDLE;
                }

                _ => {}
            }
        }

        // Poll: CONNECTING — wait for 2-second timer, then move to CONNECTED.
        if status == STATE_CONNECTING {
            if let Some(ref timer) = self.connect_timer {
                if timer.triggered() {
                    self.connect_timer = None;
                    self.status_subject.set_int(STATE_CONNECTED);
                }
            }
        }

        // Poll: DOWNLOADING — increment progress every 50 ms.
        if status == STATE_DOWNLOADING {
            if let Some(ref timer) = self.download_timer {
                if timer.triggered() {
                    let pct = self.download_pct_subject.get_int();
                    let next = pct + 1;
                    if next >= 100 {
                        self.download_timer = None;
                        self.download_pct_subject.set_int(100);
                        self.status_subject.set_int(STATE_READY);
                    } else {
                        self.download_pct_subject.set_int(next);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Observer5 {
    /// Populate the window content area with the connecting spinner and label.
    fn show_connecting(&self) -> Result<(), WidgetError> {
        let win = match self.win.as_ref() {
            Some(w) => w,
            None => return Ok(()),
        };
        let content = win.get_content();
        // Wrap in Child to suppress Rust Drop — LVGL parent owns these widgets.
        let spinner = Child::new(Spinner::new(&*content)?);
        spinner.size(130, 130).center();
        let lbl = Child::new(Label::new(&*content)?);
        lbl.text("Connecting").align(Align::Center, 0, 80);
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer5);
