// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// Type-safe wrapper for `lv_chart_type_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ChartType {
    /// Don't draw the series.
    None = lv_chart_type_t_LV_CHART_TYPE_NONE,
    /// Connect points with lines.
    Line = lv_chart_type_t_LV_CHART_TYPE_LINE,
    /// Connect points with smooth curves.
    Curve = lv_chart_type_t_LV_CHART_TYPE_CURVE,
    /// Draw columns.
    Bar = lv_chart_type_t_LV_CHART_TYPE_BAR,
    /// Stacked bars.
    Stacked = lv_chart_type_t_LV_CHART_TYPE_STACKED,
    /// Draw points and lines in 2D (x,y coordinates).
    Scatter = lv_chart_type_t_LV_CHART_TYPE_SCATTER,
}

/// Type-safe wrapper for `lv_chart_axis_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ChartAxis {
    /// Primary Y axis.
    PrimaryY = 0x00,
    /// Secondary Y axis.
    SecondaryY = 0x01,
    /// Primary X axis.
    PrimaryX = 0x02,
    /// Secondary X axis.
    SecondaryX = 0x04,
}

/// Opaque handle to a chart data series.
///
/// Returned by [`Chart::add_series`]. The series is owned by LVGL and freed
/// when the parent chart is deleted.
#[derive(Debug)]
pub struct ChartSeries {
    ptr: *mut lv_chart_series_t,
}

/// LVGL chart widget — line, bar, or scatter plots.
#[derive(Debug)]
pub struct Chart<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Chart<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Chart<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Chart<'p> {
    /// Create a new chart widget.
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        // SAFETY: parent_ptr non-null (asserted above); lv_init() called via
        // LvglDriver.
        let handle = unsafe { lv_chart_create(parent_ptr) };
        if handle.is_null() { Err(WidgetError::LvglNullPointer) } else { Ok(Chart { obj: Obj::from_raw(handle) }) }
    }

    /// Set the chart type (line, bar, scatter, or none).
    pub fn set_type(&self, t: ChartType) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()).
        unsafe { lv_chart_set_type(self.lv_handle(), t as lv_chart_type_t) };
        self
    }

    /// Set the number of data points per series.
    pub fn set_point_count(&self, count: u32) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()).
        unsafe { lv_chart_set_point_count(self.lv_handle(), count) };
        self
    }

    /// Set the value range for a given axis.
    pub fn set_axis_range(&self, axis: ChartAxis, min: i32, max: i32) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()).
        unsafe { lv_chart_set_axis_range(self.lv_handle(), axis as lv_chart_axis_t, min, max) };
        self
    }

    /// Add a data series bound to the given axis. Returns a handle for
    /// subsequent value operations.
    pub fn add_series(&self, color: lv_color_t, axis: ChartAxis) -> ChartSeries {
        // SAFETY: lv_handle() is non-null (checked in new()).
        let ptr = unsafe { lv_chart_add_series(self.lv_handle(), color, axis as lv_chart_axis_t) };
        ChartSeries { ptr }
    }

    /// Set a specific point's x/y values by index (scatter plots).
    pub fn set_series_value_by_id2(&self, series: &ChartSeries, id: u32, x: i32, y: i32) -> &Self {
        // SAFETY: lv_handle() and series.ptr are non-null (created by LVGL).
        unsafe { lv_chart_set_series_value_by_id2(self.lv_handle(), series.ptr, id, x, y) };
        self
    }

    /// Append the next value to a series (shift mode).
    pub fn set_next_value(&self, series: &ChartSeries, value: i32) -> &Self {
        // SAFETY: lv_handle() and series.ptr are non-null (created by LVGL).
        unsafe { lv_chart_set_next_value(self.lv_handle(), series.ptr, value) };
        self
    }

    /// Append the next x/y value pair to a series (scatter, shift mode).
    pub fn set_next_value2(&self, series: &ChartSeries, x: i32, y: i32) -> &Self {
        // SAFETY: lv_handle() and series.ptr are non-null (created by LVGL).
        unsafe { lv_chart_set_next_value2(self.lv_handle(), series.ptr, x, y) };
        self
    }

    /// Refresh the chart — call after externally modifying series data.
    pub fn refresh(&self) -> &Self {
        // SAFETY: lv_handle() is non-null (checked in new()).
        unsafe { lv_chart_refresh(self.lv_handle()) };
        self
    }
}
