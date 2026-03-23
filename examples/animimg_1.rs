#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! AnimImg 1 — Animated image cycling through frames
//!
//! Demonstrates the `AnimImg` widget with a two-frame animation using the
//! cogwheel image asset. Both frames use the same image; in a real
//! application each frame would be a distinct image. The animation loops
//! infinitely with a 1-second cycle.

use oxivgl::{
    anim::ANIM_REPEAT_INFINITE,
    view::View,
    widgets::{AnimImg, Screen, WidgetError, lv_image_dsc_t},
};

// Declare the extern image symbol directly so we can take its address
// in a static context. image_declare! generates a function, which cannot
// be called in const/static initializers.
unsafe extern "C" {
    #[allow(non_upper_case_globals)]
    static img_cogwheel_argb: lv_image_dsc_t;
}

/// Wrapper to make `*const c_void` usable in a `static`.
#[repr(transparent)]
struct SyncPtr(*const core::ffi::c_void);
// SAFETY: image descriptors are immutable compile-time data.
unsafe impl Sync for SyncPtr {}

/// Static array of frame pointers — must be `'static` because LVGL stores
/// the raw pointer (spec §3.1).
static FRAME_PTRS: [SyncPtr; 2] = [
    SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
    SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
];

struct AnimImg1 {
    _animimg: AnimImg<'static>,
}

impl View for AnimImg1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let animimg = AnimImg::new(&screen)?;
        animimg.center();
        // SAFETY: FRAME_PTRS has the same layout as [*const c_void; 2]
        // due to #[repr(transparent)] on SyncPtr.
        let frames: &'static [*const core::ffi::c_void] =
            unsafe { core::slice::from_raw_parts(FRAME_PTRS.as_ptr().cast(), FRAME_PTRS.len()) };
        animimg
            .set_src(frames)
            .set_duration(1000)
            .set_repeat_count(ANIM_REPEAT_INFINITE)
            .start();

        Ok(Self { _animimg: animimg })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(AnimImg1);
