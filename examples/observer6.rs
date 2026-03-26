#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 6 — Light/dark theme switching.
//!
//! Port of `lv_example_observer_6.c`. A `theme_subject` (LIGHT=0, DARK=1)
//! drives global style changes via `add_observer_with_target`. Observer
//! callbacks receive leaked `PanelStyles` / `ButtonStyles` structs as targets,
//! modify their `lv_style_t` fields in-place, then call
//! `lv_obj_report_style_change` so LVGL re-draws affected widgets.
//!
//! Clicking any button toggles between light and dark themes.

extern crate alloc;

use core::ffi::c_void;
use core::ptr::null_mut;

use lvgl_rust_sys::*;
use oxivgl::{
    style::lv_pct,
    view::View,
    widgets::{Screen, Subject, WidgetError, observer_get_target},
};

// ---------------------------------------------------------------------------
// Theme constants
// ---------------------------------------------------------------------------

/// Light theme index.
const THEME_LIGHT: i32 = 0;
/// Dark theme index.
const THEME_DARK: i32 = 1;

// ---------------------------------------------------------------------------
// Static theme subject pointer (accessed by C-style callbacks)
// ---------------------------------------------------------------------------

/// Raw pointer to the theme subject; set once in [`Observer6::create`].
///
/// SAFETY: written once before any observer fires; thereafter read-only from
/// LVGL callbacks on the single LVGL task.
static mut THEME_PTR: *mut lv_subject_t = null_mut();

// ---------------------------------------------------------------------------
// Style structs
// ---------------------------------------------------------------------------

/// Panel style bundle: main background + scrollbar.
#[repr(C)]
struct PanelStyles {
    style_main: lv_style_t,
    style_scrollbar: lv_style_t,
}

/// Button style bundle: main gradient + pressed overlay.
#[repr(C)]
struct ButtonStyles {
    style_main: lv_style_t,
    style_pressed: lv_style_t,
}

// ---------------------------------------------------------------------------
// Observer callbacks
// ---------------------------------------------------------------------------

/// Update panel colors when the theme subject changes.
unsafe extern "C" fn panel_style_observer_cb(
    observer: *mut lv_observer_t,
    _subject: *mut lv_subject_t,
) {
    unsafe {
        let m = lv_subject_get_int(THEME_PTR);
        // SAFETY: target is a valid `*mut PanelStyles` set in `init_panel_styles`.
        let styles = observer_get_target(observer) as *mut PanelStyles;
        if m == THEME_LIGHT {
            lv_style_set_bg_color(&mut (*styles).style_main, lv_color_hex3(0xfff));
            lv_style_set_shadow_color(&mut (*styles).style_main, lv_color_hex3(0x888));
            lv_style_set_text_color(&mut (*styles).style_main, lv_color_hex3(0x222));
            lv_style_set_bg_color(&mut (*styles).style_scrollbar, lv_color_hex3(0x888));
        }
        if m == THEME_DARK {
            lv_style_set_bg_color(&mut (*styles).style_main, lv_color_hex(0x040038));
            lv_style_set_shadow_color(&mut (*styles).style_main, lv_color_hex3(0xaaa));
            lv_style_set_text_color(&mut (*styles).style_main, lv_color_hex3(0xeee));
            lv_style_set_bg_color(&mut (*styles).style_scrollbar, lv_color_hex3(0xaaa));
        }
        lv_obj_report_style_change(&mut (*styles).style_main);
        lv_obj_report_style_change(&mut (*styles).style_scrollbar);
    }
}

/// Update button colors when the theme subject changes.
unsafe extern "C" fn button_style_observer_cb(
    observer: *mut lv_observer_t,
    _subject: *mut lv_subject_t,
) {
    unsafe {
        let m = lv_subject_get_int(THEME_PTR);
        // SAFETY: target is a valid `*mut ButtonStyles` set in `init_button_styles`.
        let styles = observer_get_target(observer) as *mut ButtonStyles;
        if m == THEME_LIGHT {
            lv_style_set_bg_color(&mut (*styles).style_main, lv_color_hex(0x3379de));
            lv_style_set_bg_grad_color(&mut (*styles).style_main, lv_color_hex(0xd249a5));
            lv_style_set_shadow_color(&mut (*styles).style_main, lv_color_hex(0x3379de));
            lv_style_set_text_color(&mut (*styles).style_main, lv_color_hex3(0xfff));
            lv_style_set_color_filter_opa(&mut (*styles).style_pressed, 178); // LV_OPA_70
        }
        if m == THEME_DARK {
            lv_style_set_bg_color(&mut (*styles).style_main, lv_color_hex(0xde1382));
            lv_style_set_bg_grad_color(&mut (*styles).style_main, lv_color_hex(0x4b0c72));
            lv_style_set_shadow_color(&mut (*styles).style_main, lv_color_hex(0x4b0c72));
            lv_style_set_text_color(&mut (*styles).style_main, lv_color_hex3(0xfff));
            lv_style_set_color_filter_opa(&mut (*styles).style_pressed, 76); // LV_OPA_30
        }
        lv_obj_report_style_change(&mut (*styles).style_main);
        lv_obj_report_style_change(&mut (*styles).style_pressed);
    }
}

/// Toggle the theme between light and dark on button click.
unsafe extern "C" fn switch_theme_event_cb(_e: *mut lv_event_t) {
    unsafe {
        // SAFETY: THEME_PTR initialised in Observer6::create before any event fires.
        let m = lv_subject_get_int(THEME_PTR);
        if m == THEME_LIGHT {
            lv_subject_set_int(THEME_PTR, THEME_DARK);
        } else {
            lv_subject_set_int(THEME_PTR, THEME_LIGHT);
        }
    }
}

// ---------------------------------------------------------------------------
// Style initialisation helpers
// ---------------------------------------------------------------------------

/// Allocate and leak a `PanelStyles`, register its observer, return the pointer.
///
/// # Safety
///
/// `THEME_PTR` must be initialised before calling this function.
unsafe fn init_panel_styles() -> *mut PanelStyles {
    // SAFETY: PanelStyles contains lv_style_t which are initialised by
    // lv_style_init below; zeroed memory is a valid pre-init state.
    let styles = alloc::boxed::Box::leak(alloc::boxed::Box::new(PanelStyles {
        style_main: unsafe { core::mem::zeroed() },
        style_scrollbar: unsafe { core::mem::zeroed() },
    }));
    unsafe {
        lv_style_init(&mut styles.style_main);
        lv_style_set_radius(&mut styles.style_main, 12);
        lv_style_set_bg_opa(&mut styles.style_main, 255); // LV_OPA_COVER
        lv_style_set_shadow_width(&mut styles.style_main, 24);
        lv_style_set_shadow_offset_x(&mut styles.style_main, 4);
        lv_style_set_shadow_offset_y(&mut styles.style_main, 6);
        lv_style_set_pad_all(&mut styles.style_main, 12);
        lv_style_set_pad_gap(&mut styles.style_main, 16);

        lv_style_init(&mut styles.style_scrollbar);
        lv_style_set_width(&mut styles.style_scrollbar, 4);
        lv_style_set_radius(&mut styles.style_scrollbar, 2);
        lv_style_set_pad_right(&mut styles.style_scrollbar, 8);
        lv_style_set_pad_ver(&mut styles.style_scrollbar, 8);
        lv_style_set_bg_opa(&mut styles.style_scrollbar, 127); // LV_OPA_50

        // Register observer: panel colors update when theme changes.
        // SAFETY: THEME_PTR is non-null (guaranteed by caller);
        // `styles` is leaked so its address is permanently valid.
        lv_subject_add_observer_with_target(
            THEME_PTR,
            Some(panel_style_observer_cb),
            styles as *mut PanelStyles as *mut c_void,
            null_mut(),
        );
    }
    styles
}

/// Create a panel object with the given styles applied.
///
/// # Safety
///
/// `parent` and `styles` must be valid non-null pointers.
unsafe fn create_panel(parent: *mut lv_obj_t, styles: *mut PanelStyles) -> *mut lv_obj_t {
    unsafe {
        let panel = lv_obj_create(parent);
        lv_obj_remove_style_all(panel);
        lv_obj_add_style(panel, &mut (*styles).style_main, 0);
        lv_obj_add_style(panel, &mut (*styles).style_scrollbar, lv_part_t_LV_PART_SCROLLBAR);
        panel
    }
}

/// Allocate and leak a `ButtonStyles`, register its observer, return the pointer.
///
/// # Safety
///
/// `THEME_PTR` must be initialised before calling this function.
unsafe fn init_button_styles() -> *mut ButtonStyles {
    // SAFETY: ButtonStyles contains lv_style_t initialised by lv_style_init;
    // zeroed memory is valid before init.
    let styles = alloc::boxed::Box::leak(alloc::boxed::Box::new(ButtonStyles {
        style_main: unsafe { core::mem::zeroed() },
        style_pressed: unsafe { core::mem::zeroed() },
    }));
    unsafe {
        lv_style_init(&mut styles.style_main);
        lv_style_set_radius(&mut styles.style_main, 0x7FFF); // LV_RADIUS_CIRCLE
        lv_style_set_bg_opa(&mut styles.style_main, 255); // LV_OPA_COVER
        lv_style_set_bg_grad_dir(&mut styles.style_main, lv_grad_dir_t_LV_GRAD_DIR_HOR);
        lv_style_set_shadow_width(&mut styles.style_main, 24);
        lv_style_set_shadow_offset_y(&mut styles.style_main, 6);
        lv_style_set_pad_hor(&mut styles.style_main, 32);
        lv_style_set_pad_ver(&mut styles.style_main, 12);

        lv_style_init(&mut styles.style_pressed);
        // Apply a shade color filter when pressed.
        // SAFETY: lv_color_filter_shade is a static C object valid for program lifetime.
        lv_style_set_color_filter_dsc(
            &mut (*styles).style_pressed,
            &lv_color_filter_shade as *const lv_color_filter_dsc_t,
        );

        // Register observer: button colors update when theme changes.
        // SAFETY: THEME_PTR is non-null (guaranteed by caller);
        // `styles` is leaked so its address is permanently valid.
        lv_subject_add_observer_with_target(
            THEME_PTR,
            Some(button_style_observer_cb),
            styles as *mut ButtonStyles as *mut c_void,
            null_mut(),
        );
    }
    styles
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

/// Light/dark theme switching example driven by the observer API.
struct Observer6 {
    _screen: Screen,
    /// Subject must outlive all bound observers and widgets.
    _theme_subject: Subject,
}

impl View for Observer6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        let theme_subject = Subject::new_int(THEME_DARK);

        // SAFETY: single-threaded LVGL; written once before any observer fires.
        unsafe {
            THEME_PTR = theme_subject.raw_ptr();

            let panel_styles = init_panel_styles();
            let button_styles = init_button_styles();

            let screen_ptr = lv_screen_active();
            let panel = create_panel(screen_ptr, panel_styles);
            lv_obj_set_flex_flow(panel, lv_flex_flow_t_LV_FLEX_FLOW_COLUMN);
            lv_obj_set_flex_align(
                panel,
                lv_flex_align_t_LV_FLEX_ALIGN_START,
                lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
                lv_flex_align_t_LV_FLEX_ALIGN_CENTER,
            );
            lv_obj_set_size(panel, lv_pct(90), lv_pct(90));
            lv_obj_center(panel);

            // Create 10 buttons that all toggle the theme when clicked.
            for i in 1..=10i32 {
                let btn = lv_button_create(panel);
                lv_obj_remove_style_all(btn);
                lv_obj_add_style(btn, &mut (*button_styles).style_main, 0);
                lv_obj_add_style(
                    btn,
                    &mut (*button_styles).style_pressed,
                    lv_state_t_LV_STATE_PRESSED,
                );
                lv_obj_add_event_cb(
                    btn,
                    Some(switch_theme_event_cb),
                    lv_event_code_t_LV_EVENT_CLICKED,
                    null_mut(),
                );
                let label = lv_label_create(btn);
                lv_label_set_text_fmt(label, c"Button %d".as_ptr(), i);
            }
        }

        Ok(Self {
            _screen: screen,
            _theme_subject: theme_subject,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer6);
