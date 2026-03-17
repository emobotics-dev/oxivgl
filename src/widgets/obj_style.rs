// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style-setting methods for [`Obj`]. These are `impl` blocks on the same type
//! defined in `obj.rs` — no new types introduced.

use alloc::rc::Rc;
use core::ptr::null_mut;

use lvgl_rust_sys::*;

use super::obj::Obj;

impl<'p> Obj<'p> {
    /// Set background color from RGB hex (selector 0).
    pub fn bg_color(&self, color: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_color(self.handle(), lv_color_hex(color), 0) };
        self
    }

    /// Set background opacity (selector 0).
    pub fn bg_opa(&self, opa: u8) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_opa(self.handle(), opa as lv_opa_t, 0) };
        self
    }

    /// Set border width (selector 0).
    pub fn border_width(&self, w: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_border_width(self.handle(), w, 0) };
        self
    }

    /// Set padding on all sides (selector 0).
    pub fn pad(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_all(self.handle(), p, 0) };
        self
    }

    /// Set top padding (selector 0).
    pub fn pad_top(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_top(self.handle(), p, 0) };
        self
    }

    /// Set bottom padding (selector 0).
    pub fn pad_bottom(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_bottom(self.handle(), p, 0) };
        self
    }

    /// Set left padding (selector 0).
    pub fn pad_left(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_left(self.handle(), p, 0) };
        self
    }

    /// Set right padding (selector 0).
    pub fn pad_right(&self, p: i32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_right(self.handle(), p, 0) };
        self
    }

    /// Apply a style to this object for the given selector.
    ///
    /// The style's `Rc` refcount is bumped and stored inside this widget,
    /// keeping the `lv_style_t` alive as long as the widget exists.
    ///
    /// ```ignore
    /// btn.add_style(&style, Selector::DEFAULT);
    /// btn.add_style(&style, ObjState::PRESSED);
    /// slider.add_style(&style, Part::Indicator | ObjState::PRESSED);
    /// ```
    pub fn add_style(&self, style: &crate::style::Style, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // Push clone first: if this panics (OOM), LVGL is not updated and
        // both sides remain consistent.
        self._styles.borrow_mut().push(style.clone());
        // SAFETY: handle non-null; style pointer valid for Rc's lifetime.
        // Pointer obtained via Rc::as_ptr + repr(C) offset-0 guarantee.
        unsafe { lv_obj_add_style(self.handle(), style.lv_ptr(), selector) };
        self
    }

    /// Remove all styles from this object.
    pub fn remove_style_all(&self) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null.
        unsafe { lv_obj_remove_style_all(self.handle()) };
        self._styles.borrow_mut().clear();
        self
    }

    /// Remove styles matching the given selector. Pass `None` for style to
    /// remove all styles for that selector.
    ///
    /// **Note**: when `style` is `None`, LVGL removes all styles for the
    /// selector, but the internal `_styles` Vec is not updated — the Rc
    /// clones remain alive until the widget is dropped. Use
    /// [`remove_style_all`](Self::remove_style_all) for full cleanup.
    pub fn remove_style(
        &self,
        style: Option<&crate::style::Style>,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        let style_ptr = match style {
            Some(s) => s.lv_ptr() as *mut lv_style_t,
            None => null_mut() as *mut lv_style_t,
        };
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_remove_style(self.handle(), style_ptr, selector) };
        // Remove exactly one entry by Rc pointer identity (not retain —
        // the same style can be added multiple times with different selectors).
        if let Some(s) = style {
            let target = Rc::as_ptr(&s.inner);
            let mut styles = self._styles.borrow_mut();
            if let Some(pos) = styles.iter().position(|e| Rc::as_ptr(&e.inner) == target) {
                styles.remove(pos);
            }
        }
        self
    }

    /// Set `clip_corner` — clip overflowing content at rounded corners.
    pub fn style_clip_corner(&self, clip: bool, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_clip_corner(self.handle(), clip, selector) };
        self
    }

    /// Set `translate_x` style property for the given selector.
    pub fn style_translate_x(&self, x: i32, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_translate_x(self.handle(), x, selector) };
        self
    }

    /// Set text color from RGB hex (selector 0).
    pub fn text_color(&self, color: u32) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_text_color(self.handle(), lv_color_hex(color), 0) };
        self
    }

    /// Set text font (selector 0).
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

    /// Set text alignment (selector 0).
    pub fn text_align(&self, align: super::obj::TextAlign) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_text_align(self.handle(), align as lv_text_align_t, 0) };
        self
    }

    /// Set overall opacity (selector 0).
    pub fn opa(&self, opa: u8) -> &Self {
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_opa(self.handle(), opa as lv_opa_t, 0) };
        self
    }

    /// Set opacity for the given style selector.
    pub fn style_opa(&self, opa: u8, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_opa(self.handle(), opa as lv_opa_t, selector) };
        self
    }

    /// Set padding on all sides for the given style selector.
    pub fn style_pad_all(&self, p: i32, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_pad_all(self.handle(), p, selector) };
        self
    }

    /// Set the corner radius for the given style selector.
    /// Use [`RADIUS_MAX`](super::RADIUS_MAX) for a pill/capsule shape.
    pub fn radius(&self, r: i32, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_radius(self.handle(), r, selector) };
        self
    }

    /// Set local `bg_color` style for the given selector (part | state).
    pub fn style_bg_color(&self, color: lv_color_t, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_color(self.handle(), color, selector) };
        self
    }

    /// Set local `bg_grad_color` for the given selector.
    pub fn style_bg_grad_color(
        &self,
        color: lv_color_t,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_grad_color(self.handle(), color, selector) };
        self
    }

    /// Set local `bg_grad_dir` for the given selector.
    pub fn style_bg_grad_dir(
        &self,
        dir: crate::style::GradDir,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_bg_grad_dir(self.handle(), dir as lv_grad_dir_t, selector) };
        self
    }

    /// Set transform rotation in 0.1 degree units for the given selector.
    pub fn style_transform_rotation(
        &self,
        angle: i32,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_transform_rotation(self.handle(), angle, selector) };
        self
    }

    /// Set uniform transform scale (256 = 1.0x) for the given selector.
    pub fn style_transform_scale(&self, scale: i32, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe {
            lv_obj_set_style_transform_scale_x(self.handle(), scale, selector);
            lv_obj_set_style_transform_scale_y(self.handle(), scale, selector);
        };
        self
    }

    /// Set transform pivot X for the given selector.
    pub fn style_transform_pivot_x(&self, x: i32, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_transform_pivot_x(self.handle(), x, selector) };
        self
    }

    /// Set transform pivot Y for the given selector.
    pub fn style_transform_pivot_y(&self, y: i32, selector: impl Into<crate::style::Selector>) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_style_transform_pivot_y(self.handle(), y, selector) };
        self
    }

    /// Set base text direction for the given selector.
    pub fn set_style_base_dir(
        &self,
        dir: super::obj::BaseDir,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
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

    /// Set image recolor tint.
    pub fn style_image_recolor(
        &self,
        color: lv_color_t,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        unsafe { lv_obj_set_style_image_recolor(self.handle(), color, selector) };
        self
    }

    /// Set radial offset for parts on round scales (in pixels).
    pub fn style_radial_offset(
        &self,
        offset: i32,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        unsafe { lv_obj_set_style_radial_offset(self.handle(), offset, selector) };
        self
    }

    /// Set line opacity for a part (0–255).
    pub fn style_line_opa(
        &self,
        opa: u8,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        unsafe { lv_obj_set_style_line_opa(self.handle(), opa as lv_opa_t, selector) };
        self
    }

    /// Set image recolor opacity (0–255).
    pub fn style_image_recolor_opa(
        &self,
        opa: u8,
        selector: impl Into<crate::style::Selector>,
    ) -> &Self {
        let selector = selector.into().raw();
        assert_ne!(self.handle(), null_mut(), "Obj handle cannot be null");
        unsafe { lv_obj_set_style_image_recolor_opa(self.handle(), opa as lv_opa_t, selector) };
        self
    }
}
