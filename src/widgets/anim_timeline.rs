// SPDX-License-Identifier: MIT OR Apache-2.0
use lvgl_rust_sys::*;

use super::anim::Anim;

/// Owning wrapper around `lv_anim_timeline_t*`. Calls `lv_anim_timeline_delete` on drop.
pub struct AnimTimeline {
    handle: *mut lv_anim_timeline_t,
}

impl AnimTimeline {
    pub fn new() -> Self {
        let handle = unsafe { lv_anim_timeline_create() };
        assert!(!handle.is_null(), "lv_anim_timeline_create returned NULL");
        Self { handle }
    }

    pub fn add(&mut self, start_time: u32, anim: &Anim) -> &mut Self {
        unsafe { lv_anim_timeline_add(self.handle, start_time, &anim.inner) };
        self
    }

    pub fn start(&self) -> u32 {
        unsafe { lv_anim_timeline_start(self.handle) }
    }

    pub fn pause(&self) {
        unsafe { lv_anim_timeline_pause(self.handle) }
    }

    pub fn set_reverse(&self, reverse: bool) {
        unsafe { lv_anim_timeline_set_reverse(self.handle, reverse) }
    }

    pub fn set_progress(&self, progress: u16) {
        unsafe { lv_anim_timeline_set_progress(self.handle, progress) }
    }

    pub fn handle(&self) -> *mut lv_anim_timeline_t {
        self.handle
    }
}

impl Drop for AnimTimeline {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { lv_anim_timeline_delete(self.handle) };
        }
    }
}

/// `LV_ANIM_TIMELINE_PROGRESS_MAX`
pub const ANIM_TIMELINE_PROGRESS_MAX: u16 = LV_ANIM_TIMELINE_PROGRESS_MAX as u16;
