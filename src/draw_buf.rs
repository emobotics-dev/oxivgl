// SPDX-License-Identifier: MIT OR Apache-2.0
//! Owned LVGL draw buffer — wraps `lv_draw_buf_t`.

use lvgl_rust_sys::*;

/// LVGL pixel color format.
///
/// Passed to [`DrawBuf::create`] to specify the pixel layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorFormat(pub u32);

impl ColorFormat {
    /// 16-bit RGB (5-6-5), no alpha channel.
    pub const RGB565: Self = Self(lv_color_format_t_LV_COLOR_FORMAT_RGB565 as u32);
    /// 32-bit ARGB (8-8-8-8), with full alpha.
    pub const ARGB8888: Self = Self(lv_color_format_t_LV_COLOR_FORMAT_ARGB8888 as u32);
}

/// Owned LVGL draw buffer. Allocated by LVGL on [`create`](DrawBuf::create) and freed on `Drop`.
///
/// Pass to [`Canvas::new`](crate::widgets::Canvas::new) — `Canvas` takes ownership
/// and ensures the buffer outlives the LVGL canvas object.
pub struct DrawBuf {
    ptr: *mut lv_draw_buf_t,
}

impl DrawBuf {
    /// Allocate a draw buffer of the given dimensions and color format.
    ///
    /// Returns `None` if LVGL allocation fails (OOM).
    pub fn create(w: u32, h: u32, cf: ColorFormat) -> Option<Self> {
        // SAFETY: lv_draw_buf_create allocates and zero-initialises the buffer;
        // returns null on allocation failure. We check before storing.
        let ptr = unsafe { lv_draw_buf_create(w, h, cf.0, 0) };
        if ptr.is_null() { None } else { Some(Self { ptr }) }
    }

    /// Raw LVGL pointer. Valid for the lifetime of this `DrawBuf`.
    pub(crate) fn as_ptr(&self) -> *mut lv_draw_buf_t {
        self.ptr
    }

    /// Convert buffer contents to an `lv_image_dsc_t` for use with [`DrawImageDsc`](crate::draw::DrawImageDsc).
    ///
    /// The returned descriptor borrows from `self`; it is valid only while
    /// this `DrawBuf` is alive.
    pub fn image_dsc(&self) -> lv_image_dsc_t {
        // SAFETY: ptr is a valid lv_draw_buf_t; lv_draw_buf_to_image fills the
        // image descriptor with a pointer into the buffer's pixel data.
        let mut img = unsafe { core::mem::zeroed::<lv_image_dsc_t>() };
        unsafe { lv_draw_buf_to_image(self.ptr, &mut img) };
        img
    }
}

impl Drop for DrawBuf {
    fn drop(&mut self) {
        // SAFETY: ptr was allocated by lv_draw_buf_create and has not been freed.
        // Canvas::drop (via lv_obj_delete) runs before DrawBuf::drop because
        // Canvas is declared after DrawBuf in every View struct (reverse field order).
        unsafe { lv_draw_buf_destroy(self.ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_format_rgb565_value() {
        // Sanity-check that the constant matches the raw binding value.
        assert_eq!(
            ColorFormat::RGB565.0,
            lv_color_format_t_LV_COLOR_FORMAT_RGB565 as u32
        );
    }

    #[test]
    fn color_format_argb8888_value() {
        assert_eq!(
            ColorFormat::ARGB8888.0,
            lv_color_format_t_LV_COLOR_FORMAT_ARGB8888 as u32
        );
    }
}
