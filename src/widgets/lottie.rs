// SPDX-License-Identifier: MIT OR Apache-2.0
use alloc::vec::Vec;
use core::{cell::RefCell, ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL Lottie animation widget.
///
/// Plays a Lottie animation (`.json`) using the bundled ThorVG renderer.
/// Requires `LV_USE_LOTTIE = 1` and `LV_USE_THORVG_INTERNAL = 1` in `lv_conf.h`.
///
/// The render buffer (ARGB8888, 4 bytes/px) is owned by this struct and
/// automatically freed when the widget is dropped.
///
/// # Usage
///
/// ```no_run
/// use oxivgl::widgets::{Lottie, Screen};
///
/// let screen = Screen::active().unwrap();
/// let lottie = Lottie::new(&screen).unwrap();
/// // JSON data can be embedded with include_bytes!("my_animation.json")
/// lottie.set_buffer(64, 64).set_src_data(b"{}");
/// lottie.center();
/// ```
#[derive(Debug)]
pub struct Lottie<'p> {
    obj: Obj<'p>,
    /// ARGB8888 render buffer passed to LVGL; kept alive alongside the widget.
    buf: RefCell<Vec<u8>>,
}

impl<'p> AsLvHandle for Lottie<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Lottie<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Lottie<'p> {
    /// Create a Lottie widget as a child of `parent`.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via LvglDriver.
        let handle = unsafe { lv_lottie_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Lottie { obj: Obj::from_raw(handle), buf: RefCell::new(Vec::new()) })
        }
    }

    /// Allocate an ARGB8888 render buffer of `w × h` pixels and register it
    /// with LVGL. This also sets the displayed size of the animation.
    ///
    /// Must be called before [`set_src_data`](Self::set_src_data) or
    /// [`set_src_file`](Self::set_src_file).
    pub fn set_buffer(&self, w: i32, h: i32) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Lottie handle cannot be null");
        assert!(w > 0 && h > 0, "Buffer dimensions must be positive");
        let size = (w * h * 4) as usize;
        let mut buf = self.buf.borrow_mut();
        buf.resize(size, 0);
        // SAFETY: handle non-null; buf ptr is valid and stable for this call.
        // LVGL stores the pointer: buf lives in RefCell alongside obj, so it
        // outlives the widget (obj is dropped first in declaration order).
        unsafe {
            lv_lottie_set_buffer(
                self.obj.handle(),
                w,
                h,
                buf.as_mut_ptr() as *mut core::ffi::c_void,
            )
        };
        self
    }

    /// Load the animation from an in-memory byte slice (JSON data).
    ///
    /// ThorVG copies the data internally; the slice does not need to stay alive.
    pub fn set_src_data(&self, data: &[u8]) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Lottie handle cannot be null");
        // SAFETY: handle non-null; data ptr and len valid; ThorVG copies internally.
        unsafe {
            lv_lottie_set_src_data(
                self.obj.handle(),
                data.as_ptr() as *const core::ffi::c_void,
                data.len(),
            )
        };
        self
    }

    /// Load the animation from a filesystem path (JSON file).
    ///
    /// Note: Lottie does not use LVGL's virtual filesystem — the path is
    /// passed directly to the OS. Host-only in practice.
    pub fn set_src_file(&self, path: &str) -> &Self {
        assert_ne!(self.obj.handle(), null_mut(), "Lottie handle cannot be null");
        let mut bytes: Vec<u8> = Vec::with_capacity(path.len() + 1);
        bytes.extend_from_slice(path.as_bytes());
        bytes.push(0);
        // SAFETY: handle non-null; bytes is NUL-terminated.
        unsafe {
            lv_lottie_set_src_file(
                self.obj.handle(),
                bytes.as_ptr() as *const core::ffi::c_char,
            )
        };
        self
    }

    /// Return the underlying `lv_anim_t` that drives playback.
    ///
    /// The pointer is owned by LVGL and valid for the widget's lifetime.
    pub fn get_anim(&self) -> *mut lv_anim_t {
        assert_ne!(self.obj.handle(), null_mut(), "Lottie handle cannot be null");
        // SAFETY: handle non-null; LVGL returns a pointer into widget-private state.
        unsafe { lv_lottie_get_anim(self.obj.handle()) }
    }
}
