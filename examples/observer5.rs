#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 5 — Firmware update state machine with timers.
//!
//! Port of `lv_example_observer_5.c`. Demonstrates observer-driven UI state
//! changes, raw LVGL timers simulating async operations (connect / download),
//! and a Win widget whose content is rebuilt on each state transition.
//!
//! State machine: IDLE → CONNECTING → CONNECTED → DOWNLOADING → READY (or CANCEL).

use core::ffi::c_void;
use core::ptr::null_mut;

use lvgl_rust_sys::*;
use oxivgl::{
    enums::EventCode,
    event::Event,
    style::lv_pct,
    view::{View, register_event_on},
    widgets::{Button, Label, Screen, Subject, WidgetError},
};

// Firmware update states.
const FW_IDLE: i32 = 0;
const FW_CONNECTING: i32 = 1;
const FW_CONNECTED: i32 = 2;
const FW_DOWNLOADING: i32 = 3;
const FW_CANCEL: i32 = 4;
const FW_READY: i32 = 5;

/// Raw subject pointers used by C-style callbacks that cannot capture state.
static mut FW_STATUS_PTR: *mut lv_subject_t = null_mut();
static mut FW_PERCENT_PTR: *mut lv_subject_t = null_mut();

// ---------------------------------------------------------------------------
// Timer callbacks
// ---------------------------------------------------------------------------

/// Fired 2 s after CONNECTING; transitions to CONNECTED (unless cancelled).
unsafe extern "C" fn connect_timer_cb(t: *mut lv_timer_t) {
    unsafe {
        if lv_subject_get_int(FW_STATUS_PTR) != FW_CANCEL {
            lv_subject_set_int(FW_STATUS_PTR, FW_CONNECTED);
        }
        lv_timer_delete(t);
    }
}

/// Fired every 50 ms during DOWNLOADING; increments percent until 100.
unsafe extern "C" fn download_timer_cb(t: *mut lv_timer_t) {
    unsafe {
        if lv_subject_get_int(FW_STATUS_PTR) == FW_CANCEL {
            lv_timer_delete(t);
            return;
        }
        let v = lv_subject_get_int(FW_PERCENT_PTR);
        if v < 100 {
            lv_subject_set_int(FW_PERCENT_PTR, v + 1);
        } else {
            lv_subject_set_int(FW_STATUS_PTR, FW_READY);
            lv_timer_delete(t);
        }
    }
}

// ---------------------------------------------------------------------------
// Event callbacks
// ---------------------------------------------------------------------------

/// Close button inside the Win header: cancel the firmware update.
unsafe extern "C" fn fw_update_close_event_cb(e: *mut lv_event_t) {
    unsafe {
        let _e = e;
        lv_subject_set_int(FW_STATUS_PTR, FW_CANCEL);
    }
}

/// Restart button inside the READY content: delete the Win and reset to IDLE.
unsafe extern "C" fn restart_btn_click_event_cb(e: *mut lv_event_t) {
    unsafe {
        let win = lv_event_get_user_data(e) as *mut lv_obj_t;
        lv_obj_delete(win);
        lv_subject_set_int(FW_STATUS_PTR, FW_IDLE);
    }
}

// ---------------------------------------------------------------------------
// Observer callbacks
// ---------------------------------------------------------------------------

/// Win-bound observer: rebuilds the content area on each state change.
unsafe extern "C" fn fw_update_win_observer_cb(
    observer: *mut lv_observer_t,
    subject: *mut lv_subject_t,
) {
    unsafe {
        let win = lv_observer_get_target_obj(observer);
        let cont = lv_win_get_content(win);
        let status = lv_subject_get_int(FW_STATUS_PTR);

        if status == FW_IDLE {
            lv_obj_clean(cont);
            let spinner = lv_spinner_create(cont);
            lv_obj_center(spinner);
            lv_obj_set_size(spinner, 130, 130);

            let label = lv_label_create(cont);
            lv_label_set_text(label, c"Connecting".as_ptr());
            lv_obj_center(label);

            // Transition immediately to CONNECTING to start the timer.
            lv_subject_set_int(subject, FW_CONNECTING);
        } else if status == FW_DOWNLOADING {
            lv_obj_clean(cont);
            let arc = lv_arc_create(cont);
            lv_arc_bind_value(arc, FW_PERCENT_PTR);
            lv_obj_center(arc);
            lv_obj_set_size(arc, 130, 130);
            lv_obj_remove_flag(arc, lv_obj_flag_t_LV_OBJ_FLAG_CLICKABLE);

            let label = lv_label_create(cont);
            lv_label_bind_text(label, FW_PERCENT_PTR, c"%d %%".as_ptr());
            lv_obj_center(label);
        } else if status == FW_READY {
            lv_obj_clean(cont);
            let label = lv_label_create(cont);
            lv_label_set_text(label, c"Firmware update is ready".as_ptr());
            lv_obj_align(label, lv_align_t_LV_ALIGN_CENTER as u32, 0, -20);

            let btn = lv_button_create(cont);
            lv_obj_align(btn, lv_align_t_LV_ALIGN_CENTER as u32, 0, 20);
            lv_obj_add_event_cb(
                btn,
                Some(restart_btn_click_event_cb),
                lv_event_code_t_LV_EVENT_CLICKED,
                win as *mut c_void,
            );
            let btn_label = lv_label_create(btn);
            lv_label_set_text(btn_label, c"Restart".as_ptr());
        } else if status == FW_CANCEL {
            lv_obj_delete(win);
        }
    }
}

/// Global app-level observer: manages timers for CONNECTING and CONNECTED.
unsafe extern "C" fn fw_upload_manager_observer_cb(
    _observer: *mut lv_observer_t,
    _subject: *mut lv_subject_t,
) {
    unsafe {
        let state = lv_subject_get_int(FW_STATUS_PTR);
        if state == FW_CONNECTING {
            lv_timer_create(Some(connect_timer_cb), 2000, null_mut());
        } else if state == FW_CONNECTED {
            lv_subject_set_int(FW_PERCENT_PTR, 0);
            lv_subject_set_int(FW_STATUS_PTR, FW_DOWNLOADING);
            lv_timer_create(Some(download_timer_cb), 50, null_mut());
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: open the firmware update Win
// ---------------------------------------------------------------------------

/// Creates the Win widget and attaches the win-bound observer.
///
/// Called from the View's `on_event` handler when the main button is clicked.
///
/// SAFETY: `FW_STATUS_PTR` must be initialised (done in `View::create`).
unsafe fn open_fw_update_win() {
    unsafe {
        let screen = lv_screen_active();
        let win = lv_win_create(screen);
        lv_obj_set_size(win, lv_pct(90), lv_pct(90));
        lv_obj_set_height(lv_win_get_header(win), 40);
        lv_obj_set_style_radius(win, 8, 0);
        lv_obj_set_style_shadow_width(win, 24, 0);
        lv_obj_set_style_shadow_offset_x(win, 2, 0);
        lv_obj_set_style_shadow_offset_y(win, 3, 0);
        lv_obj_set_style_shadow_color(win, lv_color_hex3(0x888), 0);

        lv_win_add_title(win, c"Firmware update".as_ptr());

        let close_btn = lv_win_add_button(win, LV_SYMBOL_CLOSE.as_ptr() as *const c_void, 40);
        lv_obj_add_event_cb(
            close_btn,
            Some(fw_update_close_event_cb),
            lv_event_code_t_LV_EVENT_CLICKED,
            null_mut(),
        );
        lv_obj_center(win);

        // Set IDLE to trigger the win observer which begins the sequence.
        lv_subject_set_int(FW_STATUS_PTR, FW_IDLE);
        lv_subject_add_observer_obj(
            FW_STATUS_PTR,
            Some(fw_update_win_observer_cb),
            win,
            null_mut(),
        );
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

struct Observer5 {
    _btn: Button<'static>,
    _btn_label: Label<'static>,
    // Subjects must outlive any widgets/observers bound to them.
    // Prefixed with `_` to silence dead_code; Drop deinits the subject.
    _fw_status_subject: Subject,
    _fw_percent_subject: Subject,
}

impl View for Observer5 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let fw_status_subject = Subject::new_int(FW_IDLE);
        let fw_percent_subject = Subject::new_int(0);

        // Publish raw pointers into statics so callbacks can access them.
        // SAFETY: single-threaded LVGL; these are written once here and read
        // only from LVGL callbacks called on the same task.
        unsafe {
            FW_STATUS_PTR = fw_status_subject.raw_ptr();
            FW_PERCENT_PTR = fw_percent_subject.raw_ptr();
        }

        // Global app-level observer: manages timers.
        fw_status_subject.add_observer(fw_upload_manager_observer_cb, null_mut());

        // "Firmware update" button — centered on screen.
        let btn = Button::new(&screen)?;
        btn.center();
        let btn_label = Label::new(&btn)?;
        btn_label.text("Firmware update").center();

        Ok(Self {
            _btn: btn,
            _btn_label: btn_label,
            _fw_status_subject: fw_status_subject,
            _fw_percent_subject: fw_percent_subject,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self._btn.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self._btn, EventCode::CLICKED) {
            // SAFETY: FW_STATUS_PTR initialised in create().
            unsafe { open_fw_update_win() };
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer5);
