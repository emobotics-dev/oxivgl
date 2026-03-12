// SPDX-License-Identifier: GPL-3.0-only
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{Align, AsLvHandle, Obj},
};

/// Type-safe wrapper for `lv_scale_mode_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ScaleMode {
    HorizontalTop = 0,
    HorizontalBottom = 1,
    VerticalLeft = 2,
    VerticalRight = 4,
    RoundInner = 8,
    RoundOuter = 16,
}

/// LVGL scale widget (tick marks only, no arc). Use
/// [`tick_ring`](Scale::tick_ring) for the pre-configured round gauge variant.
#[derive(Debug)]
pub struct Scale<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Scale<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Scale<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Scale<'p> {
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_scale_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Scale { obj: Obj::from_raw(handle) }) }
    }

    /// Create a tick-mark ring scale (no arc drawn, transparent background).
    ///
    /// - `size`: diameter in px; centered in parent.
    /// - `mode`: e.g. `LV_SCALE_MODE_ROUND_INNER` (ticks point inward).
    /// - `rotation` / `sweep`: same convention as
    ///   [`Arc::gauge_ring`](super::Arc::gauge_ring).
    /// - `range_max`: integer range maximum (ticks labeled 0..range_max).
    /// - `total_ticks`: total number of tick marks.
    /// - `major_every`: every N-th tick is a major (longer, labeled if
    ///   `show_labels=true`).
    /// - `major_len` / `minor_len`: tick length in px.
    /// - `major_color` / `minor_color`: RGB hex colors.
    #[allow(clippy::too_many_arguments)]
    pub fn tick_ring(
        parent: &impl AsLvHandle,
        size: i32,
        mode: ScaleMode,
        rotation: i32,
        sweep: i32,
        range_max: i32,
        total_ticks: u32,
        major_every: u32,
        show_labels: bool,
        major_len: i32,
        minor_len: i32,
        major_color: u32,
        minor_color: u32,
    ) -> Result<Self, WidgetError> {
        let scale = Scale::new(parent)?;
        let h = scale.obj.handle();
        // SAFETY: h non-null (Scale::new null-checks); all LVGL style/scale fns safe
        // with valid ptr.
        unsafe {
            lv_obj_set_size(h, size, size);
            lv_obj_align(h, Align::Center as lv_align_t, 0, 0);
            lv_scale_set_mode(h, mode as lv_scale_mode_t);
            lv_scale_set_rotation(h, rotation);
            lv_scale_set_angle_range(h, sweep as u32);
            lv_scale_set_range(h, 0, range_max);
            lv_scale_set_total_tick_count(h, total_ticks);
            lv_scale_set_major_tick_every(h, major_every);
            lv_scale_set_label_show(h, show_labels);
            // No ring; explicit line_width=1 so tick outer end = radius_edge-1 (1px inset
            // from arc outer edge)
            lv_obj_set_style_arc_width(h, 0, lv_part_t_LV_PART_MAIN as u32);
            lv_obj_set_style_line_width(h, 1, lv_part_t_LV_PART_MAIN as u32);
            lv_obj_set_style_bg_opa(h, _lv_opacity_level_t_LV_OPA_TRANSP as lv_opa_t, 0);
            lv_obj_set_style_border_width(h, 0, 0);
            lv_obj_set_style_pad_all(h, 0, 0);
            // Minor ticks
            lv_obj_set_style_length(h, minor_len, lv_part_t_LV_PART_ITEMS as u32);
            lv_obj_set_style_line_color(h, lv_color_hex(minor_color), lv_part_t_LV_PART_ITEMS as u32);
            lv_obj_set_style_line_width(h, 1, lv_part_t_LV_PART_ITEMS as u32);
            // Major ticks
            lv_obj_set_style_length(h, major_len, lv_part_t_LV_PART_INDICATOR as u32);
            lv_obj_set_style_line_color(h, lv_color_hex(major_color), lv_part_t_LV_PART_INDICATOR as u32);
            lv_obj_set_style_line_width(h, 2, lv_part_t_LV_PART_INDICATOR as u32);
        }
        Ok(scale)
    }
}
