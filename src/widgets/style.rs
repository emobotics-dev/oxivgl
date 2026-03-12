// SPDX-License-Identifier: MIT OR Apache-2.0
use lvgl_rust_sys::*;

use super::palette::GradDir;

/// Owned LVGL style. Wraps `lv_style_t`.
///
/// # Lifetime contract
/// Once passed to [`Obj::add_style`], this struct MUST NOT be moved or dropped
/// while any widget holds a reference to it. Store styles as fields in a `View`
/// struct that lives for the entire LVGL lifetime.
pub struct Style {
    pub(crate) inner: lv_style_t,
}

impl Style {
    pub fn new() -> Self {
        // SAFETY: lv_style_t can be zero-initialized; lv_style_init sets it up.
        let mut inner = unsafe { core::mem::zeroed::<lv_style_t>() };
        unsafe { lv_style_init(&mut inner) };
        Self { inner }
    }

    pub fn radius(&mut self, r: i16) -> &mut Self {
        unsafe { lv_style_set_radius(&mut self.inner, r as lv_coord_t) };
        self
    }

    pub fn bg_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_bg_opa(&mut self.inner, opa as lv_opa_t) };
        self
    }

    pub fn bg_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_color(&mut self.inner, color) };
        self
    }

    pub fn bg_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.bg_color(color)
    }

    pub fn bg_grad_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_grad_color(&mut self.inner, color) };
        self
    }

    pub fn bg_grad_dir(&mut self, dir: GradDir) -> &mut Self {
        unsafe { lv_style_set_bg_grad_dir(&mut self.inner, dir as lv_grad_dir_t) };
        self
    }

    pub fn border_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_border_color(&mut self.inner, color) };
        self
    }

    pub fn border_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.border_color(color)
    }

    pub fn border_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_border_opa(&mut self.inner, opa as lv_opa_t) };
        self
    }

    pub fn border_width(&mut self, w: i16) -> &mut Self {
        unsafe { lv_style_set_border_width(&mut self.inner, w as lv_coord_t) };
        self
    }

    pub fn text_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_text_color(&mut self.inner, color) };
        self
    }

    pub fn text_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.text_color(color)
    }

    pub fn color_filter(&mut self, filter: &ColorFilter, opa: u8) -> &mut Self {
        unsafe {
            lv_style_set_color_filter_dsc(&mut self.inner, &filter.inner);
            lv_style_set_color_filter_opa(&mut self.inner, opa as lv_opa_t);
        }
        self
    }
}

impl Drop for Style {
    fn drop(&mut self) {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_reset(&mut self.inner) };
    }
}

/// Wraps `lv_color_filter_dsc_t` with a C callback function pointer.
pub struct ColorFilter {
    pub(crate) inner: lv_color_filter_dsc_t,
}

impl ColorFilter {
    /// `lv_color_filter_dsc_init` is not available in bindings; set field directly.
    pub fn new(
        cb: unsafe extern "C" fn(
            *const lv_color_filter_dsc_t,
            lv_color_t,
            lv_opa_t,
        ) -> lv_color_t,
    ) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_color_filter_dsc_t>() };
        inner.filter_cb = Some(cb);
        Self { inner }
    }
}

/// Standard "darken" color filter callback — pass to [`ColorFilter::new`].
pub unsafe extern "C" fn darken_filter_cb(
    _dsc: *const lv_color_filter_dsc_t,
    color: lv_color_t,
    opa: lv_opa_t,
) -> lv_color_t {
    // SAFETY: lv_color_darken is a pure color computation.
    lv_color_darken(color, opa)
}
