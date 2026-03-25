#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 1 — Basic line chart
//!
//! Two data series (green on primary Y, red on secondary Y), 10 points each.
//! Series 1 uses `set_next_value`; series 2 uses `set_series_value_by_id`.
//! Shadow style on data-point items.

use oxivgl::{
    style::Selector,
    view::View,
    widgets::{Chart, ChartAxis, ChartType, Part, Screen, WidgetError},
};

struct WidgetChart1 {
    _screen: Screen,
    _chart: Chart<'static>,
}

impl View for WidgetChart1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let chart = Chart::new(&screen)?;
        chart.size(200, 150);
        chart.center();
        chart.set_type(ChartType::Line);
        chart.set_point_count(10);

        chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
        chart.set_axis_range(ChartAxis::SecondaryY, 0, 100);

        // Shadow on data points
        chart.style_shadow_width(8, Selector::from(Part::Items));
        chart.style_shadow_opa(255, Selector::from(Part::Items));
        chart.style_shadow_offset_x(0, Selector::from(Part::Items));

        // Green series on primary Y
        let color_green = oxivgl::style::palette_main(oxivgl::style::Palette::Green);
        let ser1 = chart.add_series(color_green, ChartAxis::PrimaryY);
        for &v in &[25, 38, 15, 42, 30, 48, 12, 35, 28, 45] {
            chart.set_next_value(&ser1, v);
        }

        // Red series on secondary Y — use set_series_value_by_id
        let color_red = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser2 = chart.add_series(color_red, ChartAxis::SecondaryY);
        for (i, &v) in [65, 72, 55, 80, 68, 75, 58, 85, 62, 70].iter().enumerate() {
            chart.set_series_value_by_id(&ser2, i as u32, v);
        }
        chart.refresh();

        Ok(Self {
            _screen: screen,
            _chart: chart,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetChart1);
