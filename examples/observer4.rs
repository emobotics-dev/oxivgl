#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 4 — Tab navigation with animated transitions
//!
//! Three tab buttons switch content (sliders/dropdowns/rollers) with animated
//! transitions. Widget values persist across tab switches via subjects.

use core::ffi::c_void;
use core::ptr::null_mut;

use lvgl_rust_sys::*;
use oxivgl::{
    view::View,
    widgets::{Screen, Subject, WidgetError},
};

// Static pointers for subjects — set in create(), used in callbacks.
static mut CURRENT_TAB_PTR: *mut lv_subject_t = null_mut();
static mut SLIDER_PTRS: [*mut lv_subject_t; 4] = [null_mut(); 4];
static mut DROPDOWN_PTRS: [*mut lv_subject_t; 3] = [null_mut(); 3];
static mut ROLLER_PTRS: [*mut lv_subject_t; 2] = [null_mut(); 2];

struct Observer4 {
    _screen: Screen,
    // Subjects held for drop-order lifetime; prefixed _ to suppress dead_code.
    _current_tab_subject: Subject,
    _slider_subjects: [Subject; 4],
    _dropdown_subjects: [Subject; 3],
    _roller_subjects: [Subject; 2],
}

// Animation exec callback — sets x position.
unsafe extern "C" fn anim_set_x_cb(obj: *mut c_void, v: i32) {
    unsafe { lv_obj_set_x(obj as *mut lv_obj_t, v) };
}

// Animation get-value callback — reads current x aligned position.
unsafe extern "C" fn anim_get_x_cb(a: *mut lv_anim_t) -> i32 {
    unsafe { lv_obj_get_x_aligned((*a).var as *const lv_obj_t) }
}

// Content area observer — animates out old children, creates new tab content,
// animates in new children.
unsafe extern "C" fn cont_observer_cb(observer: *mut lv_observer_t, subject: *mut lv_subject_t) {
    unsafe {
        let prev_v = lv_subject_get_previous_int(subject);
        let cur_v = lv_subject_get_int(subject);
        let cont = lv_observer_get_target_obj(observer);

        // Animate out existing children.
        let mut a: lv_anim_t = core::mem::zeroed();
        lv_anim_init(&mut a);
        lv_anim_set_duration(&mut a, 300);
        lv_anim_set_path_cb(&mut a, Some(lv_anim_path_ease_in_out));
        lv_anim_set_exec_cb(&mut a, Some(anim_set_x_cb));
        lv_anim_set_get_value_cb(&mut a, Some(anim_get_x_cb));
        lv_anim_set_completed_cb(&mut a, Some(lv_obj_delete_anim_completed_cb));

        let child_cnt_prev = lv_obj_get_child_count(cont);
        let mut delay: u32 = 0;
        for i in 0..child_cnt_prev {
            let child = lv_obj_get_child(cont, i as i32);
            lv_anim_set_var(&mut a, child as *mut c_void);
            if prev_v < cur_v {
                lv_anim_set_values(&mut a, 0, -20);
            } else {
                lv_anim_set_values(&mut a, 0, 20);
            }
            lv_anim_set_delay(&mut a, delay);
            lv_anim_start(&a);
            lv_obj_fade_out(child, 200, delay);
            delay += 50;
        }

        // Create new widgets based on current tab.
        if cur_v == 0 {
            for i in 0..4u32 {
                let slider = lv_slider_create(cont);
                lv_slider_bind_value(slider, SLIDER_PTRS[i as usize]);
                lv_obj_align(
                    slider,
                    lv_align_t_LV_ALIGN_TOP_MID as u32,
                    0,
                    (10 + i * 30) as i32,
                );
            }
        }
        if cur_v == 1 {
            for i in 0..3u32 {
                let dropdown = lv_dropdown_create(cont);
                lv_dropdown_bind_value(dropdown, DROPDOWN_PTRS[i as usize]);
                lv_obj_align(dropdown, lv_align_t_LV_ALIGN_TOP_MID as u32, 0, (i * 50) as i32);
            }
        }
        if cur_v == 2 {
            for i in 0..2u32 {
                let roller = lv_roller_create(cont);
                lv_roller_bind_value(roller, ROLLER_PTRS[i as usize]);
                lv_obj_align(
                    roller,
                    lv_align_t_LV_ALIGN_CENTER as u32,
                    -80 + (i as i32) * 160,
                    0,
                );
            }
        }

        // Animate in new widgets.
        lv_anim_set_completed_cb(&mut a, None);
        let new_cnt = lv_obj_get_child_count(cont);
        for i in child_cnt_prev..new_cnt {
            let child = lv_obj_get_child(cont, i as i32);
            lv_anim_set_var(&mut a, child as *mut c_void);
            if prev_v < cur_v {
                lv_anim_set_values(&mut a, 20, 0);
            } else {
                lv_anim_set_values(&mut a, -20, 0);
            }
            lv_anim_set_delay(&mut a, delay);
            lv_anim_start(&a);
            lv_obj_fade_in(child, 200, delay);
            delay += 50;
        }
    }
}

// Button click — sets current tab to the clicked button's index.
unsafe extern "C" fn btn_click_event_cb(e: *mut lv_event_t) {
    unsafe {
        let btn = lv_event_get_target_obj(e);
        let idx = lv_obj_get_index(btn);
        lv_subject_set_int(CURRENT_TAB_PTR, idx as i32);
    }
}

// Button observer — toggles checked state based on current tab.
unsafe extern "C" fn btn_observer_cb(observer: *mut lv_observer_t, subject: *mut lv_subject_t) {
    unsafe {
        let prev_v = lv_subject_get_previous_int(subject);
        let cur_v = lv_subject_get_int(subject);
        let btn = lv_observer_get_target_obj(observer);
        let idx = lv_obj_get_index(btn) as i32;
        if idx == prev_v {
            lv_obj_remove_state(btn, lv_state_t_LV_STATE_CHECKED);
        }
        if idx == cur_v {
            lv_obj_add_state(btn, lv_state_t_LV_STATE_CHECKED);
        }
    }
}

// Indicator observer — animates indicator position to align with active tab.
unsafe extern "C" fn indicator_observer_cb(
    observer: *mut lv_observer_t,
    subject: *mut lv_subject_t,
) {
    unsafe {
        let cur_v = lv_subject_get_int(subject);
        let indicator = lv_observer_get_target_obj(observer);
        let footer = lv_obj_get_parent(indicator);
        let btn_act = lv_obj_get_child(footer, cur_v);
        lv_obj_set_width(indicator, lv_obj_get_width(btn_act));

        let mut a: lv_anim_t = core::mem::zeroed();
        lv_anim_init(&mut a);
        lv_anim_set_exec_cb(&mut a, Some(anim_set_x_cb));
        lv_anim_set_duration(&mut a, 300);
        lv_anim_set_path_cb(&mut a, Some(lv_anim_path_ease_in_out));
        lv_anim_set_var(&mut a, indicator as *mut c_void);
        lv_anim_set_values(&mut a, lv_obj_get_x(indicator), lv_obj_get_x(btn_act));
        lv_anim_start(&a);
    }
}

// Helper: create a tab button with observer and click handler.
unsafe fn btn_create(parent: *mut lv_obj_t, text: &core::ffi::CStr) {
    unsafe {
        let btn = lv_button_create(parent);
        lv_obj_set_flex_grow(btn, 1);
        lv_obj_set_height(btn, lv_pct(100));
        lv_obj_set_style_radius(btn, 0, 0);
        lv_subject_add_observer_obj(CURRENT_TAB_PTR, Some(btn_observer_cb), btn, null_mut());
        lv_obj_add_event_cb(
            btn,
            Some(btn_click_event_cb),
            lv_event_code_t_LV_EVENT_CLICKED,
            null_mut(),
        );
        let label = lv_label_create(btn);
        lv_label_set_text(label, text.as_ptr());
        lv_obj_center(label);
    }
}

impl View for Observer4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let current_tab_subject = Subject::new_int(0);
        let slider_subjects = core::array::from_fn(|_| Subject::new_int(0));
        let dropdown_subjects = core::array::from_fn(|_| Subject::new_int(0));
        let roller_subjects = core::array::from_fn(|_| Subject::new_int(0));

        // Store raw pointers in statics for callbacks.
        // SAFETY: Subjects live in the View struct for the app's lifetime.
        unsafe {
            CURRENT_TAB_PTR = current_tab_subject.raw_ptr();
            for (i, s) in slider_subjects.iter().enumerate() {
                SLIDER_PTRS[i] = s.raw_ptr();
            }
            for (i, s) in dropdown_subjects.iter().enumerate() {
                DROPDOWN_PTRS[i] = s.raw_ptr();
            }
            for (i, s) in roller_subjects.iter().enumerate() {
                ROLLER_PTRS[i] = s.raw_ptr();
            }
        }

        // Build entire UI with raw LVGL — callbacks need raw subject pointers.
        // SAFETY: lv_screen_active() returns valid screen; all create calls non-null.
        unsafe {
            let screen_ptr = lv_screen_active();

            let main_cont = lv_obj_create(screen_ptr);
            lv_obj_remove_style_all(main_cont);
            lv_obj_set_size(main_cont, lv_pct(100), lv_pct(100));
            lv_obj_set_style_pad_all(main_cont, 0, 0);
            lv_obj_set_flex_flow(main_cont, lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);

            let cont = lv_obj_create(main_cont);
            lv_obj_remove_style_all(cont);
            lv_obj_set_flex_grow(cont, 1);
            lv_obj_set_style_pad_all(cont, 8, 0);
            lv_obj_set_width(cont, lv_pct(100));
            lv_subject_add_observer_obj(
                CURRENT_TAB_PTR,
                Some(cont_observer_cb),
                cont,
                null_mut(),
            );
            lv_obj_set_scroll_dir(cont, lv_dir_t_LV_DIR_VER);

            let footer = lv_obj_create(main_cont);
            lv_obj_remove_style_all(footer);
            lv_obj_set_style_pad_column(footer, 8, 0);
            lv_obj_set_style_pad_all(footer, 8, 0);
            lv_obj_set_flex_flow(footer, lv_flex_flow_t_LV_FLEX_FLOW_ROW);
            lv_obj_set_size(footer, lv_pct(100), 60);
            lv_obj_align(footer, lv_align_t_LV_ALIGN_BOTTOM_MID as u32, 0, 0);

            btn_create(footer, c"First");
            btn_create(footer, c"Second");
            btn_create(footer, c"Third");

            let indicator = lv_obj_create(footer);
            lv_obj_remove_style(indicator, null_mut(), 0);
            lv_obj_set_style_bg_opa(indicator, 40, 0); // LV_OPA_40
            lv_subject_add_observer_obj(
                CURRENT_TAB_PTR,
                Some(indicator_observer_cb),
                indicator,
                null_mut(),
            );
            lv_obj_set_height(indicator, 10);
            lv_obj_align(indicator, lv_align_t_LV_ALIGN_BOTTOM_LEFT as u32, 0, 0);
            lv_obj_add_flag(indicator, lv_obj_flag_t_LV_OBJ_FLAG_IGNORE_LAYOUT);

            // Ensure indicator has correct size before first notification.
            lv_obj_update_layout(indicator);
            lv_subject_notify(CURRENT_TAB_PTR);
        }

        Ok(Self {
            _screen: screen,
            _current_tab_subject: current_tab_subject,
            _slider_subjects: slider_subjects,
            _dropdown_subjects: dropdown_subjects,
            _roller_subjects: roller_subjects,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer4);
