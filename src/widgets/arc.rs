// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{cell::Cell, ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    obj::{Align, AsLvHandle, Obj},
    to_lvgl, WidgetError, LVGL_SCALE,
};

/// LVGL arc widget. Value range is normalized: call
/// [`set_range`](Arc::set_range) with a physical maximum, then
/// [`set_value`](Arc::set_value) with the physical value in the same unit.
///
/// For a ready-made gauge ring see [`Arc::gauge_ring`].
///
/// # Examples
///
/// ```no_run
/// use oxivgl::widgets::{Arc, Screen};
///
/// let screen = Screen::active().unwrap();
/// let arc = Arc::new(&screen).unwrap();
/// arc.set_range(150.0); // 0–150 A
/// arc.set_value(75.0);  // 50 %
/// ```
#[derive(Debug)]
pub struct Arc<'p> {
    obj: Obj<'p>,
    max: Cell<f32>,
}

impl<'p> AsLvHandle for Arc<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Arc<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Arc<'p> {
    /// Create a new arc widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_arc_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Arc {
                obj: Obj::from_raw(handle),
                max: Cell::new(0.0),
            })
        }
    }

    /// Set range maximum. Min is always 0. Must be called before `set_value`.
    pub fn set_range(&self, max: f32) -> &Self {
        self.max.set(max);
        // SAFETY: handle non-null (from Arc::new/gauge_ring, both null-check).
        unsafe { lv_arc_set_range(self.obj.handle(), 0, LVGL_SCALE) };
        self
    }

    /// Set current value in physical units (mapped via `max` set by
    /// [`set_range`](Arc::set_range)).
    pub fn set_value(&self, v: f32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_value(self.obj.handle(), to_lvgl(v, self.max.get())) };
        self
    }

    /// Get current value in physical units.
    pub fn get_value(&self) -> f32 {
        let max = self.max.get();
        if max == 0.0 {
            return 0.0;
        }
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        let raw = unsafe { lv_arc_get_value(self.obj.handle()) };
        (raw as f32 / LVGL_SCALE as f32) * max
    }

    /// Set the arc start angle rotation in degrees.
    pub fn set_rotation(&self, rotation: i32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_rotation(self.obj.handle(), rotation) };
        self
    }

    /// Set background arc start and end angles.
    pub fn set_bg_angles(&self, start: i32, end: i32) -> &Self {
        // SAFETY: handle non-null (from Arc::new/gauge_ring).
        unsafe { lv_arc_set_bg_angles(self.obj.handle(), start, end) };
        self
    }

    /// Create an arc pre-configured as a display-only gauge ring (no knob, not
    /// clickable).
    ///
    /// - `size`: diameter in px (arc is centered in parent).
    /// - `arc_width`: ring thickness in px.
    /// - `range_max`: physical maximum (e.g. `150.0` for 0–150 A).
    /// - `track_color` / `indicator_color`: RGB hex colors.
    /// - `rotation`: start angle in degrees (0 = 3 o'clock, CW; 150 ≈ 7
    ///   o'clock).
    /// - `sweep`: arc extent in degrees (e.g. `200` for a 200° arc).
    pub fn gauge_ring(
        parent: &impl AsLvHandle,
        size: i32,
        arc_width: i32,
        range_max: f32,
        track_color: u32,
        indicator_color: u32,
        rotation: i32,
        sweep: i32,
    ) -> Result<Self, WidgetError> {
        let arc = Arc::new(parent)?;
        arc.max.set(range_max);
        let h = arc.obj.handle();
        // SAFETY: h non-null (Arc::new null-checks); all LVGL style/arc fns safe with
        // valid ptr.
        unsafe {
            lv_obj_set_size(h, size, size);
            lv_obj_align(h, Align::Center as lv_align_t, 0, 0);
            lv_arc_set_rotation(h, rotation);
            lv_arc_set_bg_angles(h, 0, sweep);
            lv_arc_set_range(h, 0, LVGL_SCALE);
            lv_arc_set_value(h, 0);
            // Background track
            lv_obj_set_style_arc_width(h, arc_width, lv_part_t_LV_PART_MAIN as u32);
            lv_obj_set_style_arc_color(h, lv_color_hex(track_color), lv_part_t_LV_PART_MAIN as u32);
            // Indicator
            lv_obj_set_style_arc_width(h, arc_width, lv_part_t_LV_PART_INDICATOR as u32);
            lv_obj_set_style_arc_color(
                h,
                lv_color_hex(indicator_color),
                lv_part_t_LV_PART_INDICATOR as u32,
            );
            // Hide knob
            lv_obj_set_style_pad_all(h, 0, lv_part_t_LV_PART_KNOB as u32);
            lv_obj_set_style_opa(
                h,
                super::Opa::TRANSP.0 as lv_opa_t,
                lv_part_t_LV_PART_KNOB as u32,
            );
            // Not interactive
            lv_obj_remove_flag(h, super::ObjFlag::CLICKABLE.0);
        }
        Ok(arc)
    }
}
