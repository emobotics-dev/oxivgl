#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 3 — Time picker with subject groups
//!
//! Demonstrates `Subject::new_group`, `subject_get_group_element`, dynamic
//! widget creation/deletion in event callbacks, and `lv_roller_bind_value` /
//! `lv_dropdown_bind_value` / `lv_obj_bind_state_if_eq`.
//!
//! - Four integer subjects: hour, minute, format (12/24), AM/PM.
//! - A group subject watches all four; its observer formats the time label.
//! - Clicking "Set" opens a settings panel with rollers and dropdowns.
//! - The close button deletes the panel and re-enables "Set".
//! - Subjects persist across panel create/delete cycles.

use core::ffi::c_void;
use core::ptr::null_mut;

use lvgl_rust_sys::*;
use oxivgl::{
    enums::{EventCode, ObjState},
    event::Event,
    fonts::MONTSERRAT_30,
    style::{LV_SIZE_CONTENT, lv_pct},
    view::{View, register_event_on},
    widgets::{
        Button, Child, Label, Screen, Subject, WidgetError, observer_get_target_obj,
        subject_get_group_element, subject_get_int_raw,
    },
};

const TIME_FORMAT_12: i32 = 0;
const TIME_FORMAT_24: i32 = 1;
const TIME_AM: i32 = 0;

/// Observer callback: formats the time label from the group subject's members.
unsafe extern "C" fn time_observer_cb(observer: *mut lv_observer_t, subject: *mut lv_subject_t) {
    unsafe {
        let hour = subject_get_int_raw(subject_get_group_element(subject, 0));
        let minute = subject_get_int_raw(subject_get_group_element(subject, 1));
        let format = subject_get_int_raw(subject_get_group_element(subject, 2));
        let am_pm = subject_get_int_raw(subject_get_group_element(subject, 3));
        let label_ptr = observer_get_target_obj(observer);

        if format == TIME_FORMAT_24 {
            lv_label_set_text_fmt(label_ptr, c"%d:%02d".as_ptr(), hour, minute);
        } else {
            let suffix = if am_pm == TIME_AM {
                c"am".as_ptr()
            } else {
                c"pm".as_ptr()
            };
            lv_label_set_text_fmt(label_ptr, c"%d:%02d %s".as_ptr(), hour + 1, minute, suffix);
        }
    }
}

/// Observer callback: updates hour roller options when the format changes.
unsafe extern "C" fn hour_roller_options_update(
    observer: *mut lv_observer_t,
    subject: *mut lv_subject_t,
) {
    unsafe {
        let roller = observer_get_target_obj(observer);
        let prev_selected = lv_roller_get_selected(roller) as i32;
        let v = subject_get_int_raw(subject);

        if v == TIME_FORMAT_12 {
            let sel = (prev_selected - 1).max(0);
            let sel = if sel > 12 { sel - 12 } else { sel };
            lv_roller_set_options(
                roller,
                c"01\n02\n03\n04\n05\n06\n07\n08\n09\n10\n11\n12".as_ptr(),
                lv_roller_mode_t_LV_ROLLER_MODE_NORMAL,
            );
            lv_roller_set_selected(roller, sel as u32, false);
        } else {
            let sel = prev_selected + 1;
            lv_roller_set_options(
                roller,
                c"00\n01\n02\n03\n04\n05\n06\n07\n08\n09\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23".as_ptr(),
                lv_roller_mode_t_LV_ROLLER_MODE_NORMAL,
            );
            lv_roller_set_selected(roller, sel as u32, false);
        }
        lv_obj_send_event(roller, lv_event_code_t_LV_EVENT_VALUE_CHANGED, null_mut());
    }
}

/// Close button callback: deletes the settings panel and re-enables "Set".
unsafe extern "C" fn close_clicked_event_cb(e: *mut lv_event_t) {
    unsafe {
        let set_btn = lv_event_get_user_data(e) as *mut lv_obj_t;
        let close_btn = lv_event_get_target_obj(e);
        let cont = lv_obj_get_parent(close_btn);
        lv_obj_remove_state(set_btn, lv_state_t_LV_STATE_DISABLED);
        lv_obj_delete(cont);
    }
}

struct Observer3 {
    set_btn: Button<'static>,
    _time_label: Label<'static>,
    // Subjects — drop after widgets so observers are removed before deinit.
    hour_subject: Subject,
    minute_subject: Subject,
    format_subject: Subject,
    am_pm_subject: Subject,
    _time_subject: Subject,
}

impl View for Observer3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let hour_subject = Subject::new_int(7);
        let minute_subject = Subject::new_int(45);
        let format_subject = Subject::new_int(TIME_FORMAT_12);
        let am_pm_subject = Subject::new_int(TIME_AM);
        let time_subject = Subject::new_group(&[
            &hour_subject,
            &minute_subject,
            &format_subject,
            &am_pm_subject,
        ]);

        // Time display label updated by the group observer.
        let time_label = Label::new(&screen)?;
        time_label.text_font(MONTSERRAT_30).pos(24, 24);
        time_subject.add_observer_obj(time_observer_cb, &time_label, null_mut());

        // "Set" button opens the settings panel.
        let set_btn = Button::new(&screen)?;
        set_btn.pos(180, 24);
        let set_label = Child::new(Label::new(&set_btn)?);
        set_label.text("Set");

        // Update subjects to verify that the UI reflects the new values.
        hour_subject.set_int(9);
        minute_subject.set_int(30);
        am_pm_subject.set_int(1); // PM

        Ok(Self {
            set_btn,
            _time_label: time_label,
            hour_subject,
            minute_subject,
            format_subject,
            am_pm_subject,
            _time_subject: time_subject,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.set_btn.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.set_btn, EventCode::CLICKED) {
            // Disable "Set" while the panel is open.
            self.set_btn.add_state(ObjState::DISABLED);

            // Create the settings panel with raw LVGL calls.
            // Widgets are short-lived (deleted on close) and owned by LVGL.
            // SAFETY: lv_screen_active() returns a valid screen pointer; all
            // widget create calls follow the same non-null precondition enforced
            // by LVGL's allocator.
            unsafe {
                let screen_ptr = lv_screen_active();
                let cont = lv_obj_create(screen_ptr);
                lv_obj_set_size(cont, lv_pct(100), LV_SIZE_CONTENT);
                lv_obj_align(cont, lv_align_t_LV_ALIGN_BOTTOM_MID as u32, 0, 0);

                // Hour roller — options updated when format subject changes.
                let hour_roller = lv_roller_create(cont);
                lv_obj_add_flag(hour_roller, lv_obj_flag_t_LV_OBJ_FLAG_FLEX_IN_NEW_TRACK);
                lv_subject_add_observer_obj(
                    self.format_subject.raw_ptr(),
                    Some(hour_roller_options_update),
                    hour_roller,
                    null_mut(),
                );
                lv_roller_bind_value(hour_roller, self.hour_subject.raw_ptr());
                lv_obj_set_pos(hour_roller, 0, 0);

                // Minute roller (00–59).
                let min_roller = lv_roller_create(cont);
                lv_roller_set_options(
                    min_roller,
                    c"00\n01\n02\n03\n04\n05\n06\n07\n08\n09\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27\n28\n29\n30\n31\n32\n33\n34\n35\n36\n37\n38\n39\n40\n41\n42\n43\n44\n45\n46\n47\n48\n49\n50\n51\n52\n53\n54\n55\n56\n57\n58\n59".as_ptr(),
                    lv_roller_mode_t_LV_ROLLER_MODE_NORMAL,
                );
                lv_roller_bind_value(min_roller, self.minute_subject.raw_ptr());
                lv_obj_set_pos(min_roller, 64, 0);

                // Format dropdown (12/24).
                let format_dd = lv_dropdown_create(cont);
                lv_dropdown_set_options(format_dd, c"12\n24".as_ptr());
                lv_dropdown_bind_value(format_dd, self.format_subject.raw_ptr());
                lv_obj_set_pos(format_dd, 128, 0);
                lv_obj_set_width(format_dd, 80);

                // AM/PM dropdown — disabled in 24-hour mode.
                let ampm_dd = lv_dropdown_create(cont);
                lv_dropdown_set_options(ampm_dd, c"am\npm".as_ptr());
                lv_dropdown_bind_value(ampm_dd, self.am_pm_subject.raw_ptr());
                lv_obj_bind_state_if_eq(
                    ampm_dd,
                    self.format_subject.raw_ptr(),
                    lv_state_t_LV_STATE_DISABLED,
                    TIME_FORMAT_24,
                );
                lv_obj_set_pos(ampm_dd, 128, 48);
                lv_obj_set_width(ampm_dd, 80);

                // Close button — passes set_btn handle as user_data so it can
                // re-enable the button when clicked.
                let close_btn = lv_button_create(cont);
                lv_obj_align(close_btn, lv_align_t_LV_ALIGN_TOP_RIGHT as u32, 0, 0);
                lv_obj_add_event_cb(
                    close_btn,
                    Some(close_clicked_event_cb),
                    lv_event_code_t_LV_EVENT_CLICKED,
                    self.set_btn.handle() as *mut c_void,
                );

                let close_label = lv_label_create(close_btn);
                lv_label_set_text(close_label, LV_SYMBOL_CLOSE.as_ptr() as *const i8);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer3);
