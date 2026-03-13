// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style-setting methods for [`Obj`]. These are `impl` blocks on the same type
//! defined in `obj.rs` — no new types introduced.

use core::ptr::null_mut;

use lvgl_rust_sys::*;

use super::obj::Obj;

impl<'p> Obj<'p> {
    pub fn bg_color(&self, color: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_color(self.handle(), lv_color_hex(color), 0) };
        self
    }

    pub fn bg_opa(&self, opa: u8) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_opa(self.handle(), opa as lv_opa_t, 0) };
        self
    }

    pub fn border_width(&self, w: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_border_width(self.handle(), w, 0) };
        self
    }

    pub fn pad(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_all(self.handle(), p, 0) };
        self
    }

    pub fn pad_top(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_top(self.handle(), p, 0) };
        self
    }

    pub fn pad_bottom(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_bottom(self.handle(), p, 0) };
        self
    }

    pub fn pad_left(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_left(self.handle(), p, 0) };
        self
    }

    pub fn pad_right(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_right(self.handle(), p, 0) };
        self
    }

    /// Apply a style to this object for the given selector.
    /// Pass `0` for default state, `ObjState::PRESSED.0` for pressed.
    ///
    /// The `style` must outlive this object (see [`Style`](super::Style) docs).
    pub fn add_style(&self, style: &super::Style, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null; style.inner pointer valid for style's lifetime.
        unsafe { lv_obj_add_style(self.handle(), &style.inner as *const lv_style_t, selector) };
        self
    }

    /// Remove all styles from this object.
    pub fn remove_style_all(&self) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_obj_remove_style_all(self.handle()) };
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_text_color(self.handle(), lv_color_hex(color), 0) };
        self
    }

    pub fn text_font(&self, font: crate::fonts::Font) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        assert_ne!(font.as_ptr(), null_mut(), "Font pointer cannot be null");
        // SAFETY: handle and font pointer non-null (asserted above).
        unsafe { lv_obj_set_style_text_font(self.handle(), font.as_ptr(), 0) };
        self
    }

    /// Alias for [`text_font`](Self::text_font).
    pub fn font(&self, font: crate::fonts::Font) -> &Self {
        self.text_font(font)
    }

    pub fn text_align(&self, align: super::obj::TextAlign) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_text_align(self.handle(), align as lv_text_align_t, 0) };
        self
    }

    pub fn opa(&self, opa: u8) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_opa(self.handle(), opa as lv_opa_t, 0) };
        self
    }

    /// Set the corner radius for the given style selector (0 = default state).
    /// Use `0x7fff` for a pill/capsule shape.
    pub fn radius(&self, r: i32, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_radius(self.handle(), r, selector) };
        self
    }

    /// Set local `bg_color` style for the given selector (part | state).
    pub fn style_bg_color(&self, color: lv_color_t, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_color(self.handle(), color, selector) };
        self
    }

    /// Set local `bg_grad_color` for the given selector.
    pub fn style_bg_grad_color(&self, color: lv_color_t, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_grad_color(self.handle(), color, selector) };
        self
    }

    /// Set local `bg_grad_dir` for the given selector.
    pub fn style_bg_grad_dir(&self, dir: u32, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_grad_dir(self.handle(), dir as lv_grad_dir_t, selector) };
        self
    }

    /// Set transform rotation in 0.1 degree units for the given selector.
    pub fn style_transform_rotation(&self, angle: i32, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_transform_rotation(self.handle(), angle, selector) };
        self
    }

    /// Set uniform transform scale (256 = 1.0x) for the given selector.
    pub fn style_transform_scale(&self, scale: i32, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe {
            lv_obj_set_style_transform_scale_x(self.handle(), scale, selector);
            lv_obj_set_style_transform_scale_y(self.handle(), scale, selector);
        };
        self
    }

    /// Set transform pivot X for the given selector.
    pub fn style_transform_pivot_x(&self, x: i32, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_transform_pivot_x(self.handle(), x, selector) };
        self
    }

    /// Set transform pivot Y for the given selector.
    pub fn style_transform_pivot_y(&self, y: i32, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_transform_pivot_y(self.handle(), y, selector) };
        self
    }

    pub fn set_style_base_dir(&self, dir: super::obj::BaseDir, selector: u32) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_base_dir(self.handle(), dir as lv_base_dir_t, selector) };
        self
    }

    /// Set `lv_obj_set_style_line_width` for the given LVGL style part.
    pub fn line_width(&self, part: super::obj::Part, width: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_line_width(self.handle(), width, part as u32) };
        self
    }
}
