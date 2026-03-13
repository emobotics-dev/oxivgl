// SPDX-License-Identifier: MIT OR Apache-2.0
use core::ffi::c_void;
use lvgl_rust_sys::*;

use super::obj::AsLvHandle;

/// Stack-local animation builder. LVGL copies the descriptor on `start()`,
/// so this can be dropped after starting.
pub struct Anim {
    pub(crate) inner: lv_anim_t,
}

impl Anim {
    pub fn new() -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_anim_t>() };
        unsafe { lv_anim_init(&mut inner) };
        Self { inner }
    }

    /// Set the animated variable (the raw `lv_obj_t*` pointer).
    pub fn set_var(&mut self, obj: &impl AsLvHandle) -> &mut Self {
        unsafe { lv_anim_set_var(&mut self.inner, obj.lv_handle() as *mut c_void) };
        self
    }

    pub fn set_values(&mut self, start: i32, end: i32) -> &mut Self {
        unsafe { lv_anim_set_values(&mut self.inner, start, end) };
        self
    }

    pub fn set_duration(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_duration(&mut self.inner, ms) };
        self
    }

    pub fn set_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_delay(&mut self.inner, ms) };
        self
    }

    pub fn set_exec_cb(&mut self, cb: lv_anim_exec_xcb_t) -> &mut Self {
        unsafe { lv_anim_set_exec_cb(&mut self.inner, cb) };
        self
    }

    pub fn set_custom_exec_cb(&mut self, cb: lv_anim_custom_exec_cb_t) -> &mut Self {
        unsafe { lv_anim_set_custom_exec_cb(&mut self.inner, cb) };
        self
    }

    pub fn set_path_cb(&mut self, cb: lv_anim_path_cb_t) -> &mut Self {
        unsafe { lv_anim_set_path_cb(&mut self.inner, cb) };
        self
    }

    pub fn set_reverse_duration(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_reverse_duration(&mut self.inner, ms) };
        self
    }

    pub fn set_reverse_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_reverse_delay(&mut self.inner, ms) };
        self
    }

    pub fn set_repeat_count(&mut self, cnt: u32) -> &mut Self {
        unsafe { lv_anim_set_repeat_count(&mut self.inner, cnt) };
        self
    }

    pub fn set_repeat_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_repeat_delay(&mut self.inner, ms) };
        self
    }

    /// Start the animation. LVGL copies the descriptor internally.
    pub fn start(&self) {
        unsafe { lv_anim_start(&self.inner) };
    }
}

/// Linear animation path — wraps `lv_anim_path_linear`.
pub unsafe extern "C" fn anim_path_linear(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_linear(a) }
}

pub unsafe extern "C" fn anim_path_overshoot(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_overshoot(a) }
}

pub unsafe extern "C" fn anim_path_ease_in(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_in(a) }
}

pub unsafe extern "C" fn anim_path_ease_out(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_out(a) }
}

pub unsafe extern "C" fn anim_path_ease_in_out(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_in_out(a) }
}

pub unsafe extern "C" fn anim_path_bounce(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_bounce(a) }
}

/// `LV_ANIM_REPEAT_INFINITE`
pub const ANIM_REPEAT_INFINITE: u32 = LV_ANIM_REPEAT_INFINITE;

// ── Common animation exec callbacks (lv_anim_exec_xcb_t) ──

/// Exec callback: `lv_obj_set_x(var, v)`.
pub unsafe extern "C" fn anim_set_x(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_x(var as *mut lv_obj_t, v) };
}

/// Exec callback: `lv_obj_set_size(var, v, v)` — uniform width+height.
pub unsafe extern "C" fn anim_set_size(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_size(var as *mut lv_obj_t, v, v) };
}

// ── Common animation custom exec callbacks (lv_anim_custom_exec_cb_t) ──

/// Custom exec callback: `lv_obj_set_width(anim.var, v)`.
pub unsafe extern "C" fn anim_set_width(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_obj_set_width((*a).var as *mut lv_obj_t, v) };
}

/// Custom exec callback: `lv_obj_set_height(anim.var, v)`.
pub unsafe extern "C" fn anim_set_height(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_obj_set_height((*a).var as *mut lv_obj_t, v) };
}

/// Custom exec callback: `lv_slider_set_value(anim.var, v, false)`.
pub unsafe extern "C" fn anim_set_slider_value(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_slider_set_value((*a).var as *mut lv_obj_t, v, false) };
}
