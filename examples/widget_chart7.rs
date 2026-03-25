#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 7 — Scatter chart with live data
//!
//! A scatter plot with 50 data points. A timer adds new random points every
//! 100 ms. Simplified from LVGL original (no custom draw coloring).

use oxivgl::{
    style::Selector,
    timer::Timer,
    view::View,
    widgets::{Chart, ChartAxis, ChartSeries, ChartType, Part, Screen, WidgetError},
};

/// Simple LCG pseudo-random for deterministic scatter data.
fn pseudo_rand(seed: &mut u32, min: i32, max: i32) -> i32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let range = (max - min + 1) as u32;
    min + ((*seed >> 16) % range) as i32
}

struct WidgetChart7 {
    _screen: Screen,
    chart: Chart<'static>,
    ser: ChartSeries,
    timer: Timer,
    seed: u32,
}

impl View for WidgetChart7 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let chart = Chart::new(&screen)?;
        chart.size(200, 150);
        chart.center();
        chart.set_type(ChartType::Scatter);
        chart.set_point_count(50);
        chart.set_axis_range(ChartAxis::PrimaryX, 0, 200);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 1000);
        // Hide connecting lines — show points only
        chart.style_line_width(0, Selector::from(Part::Items));
        chart.style_size(4, 4, Selector::from(Part::Indicator));

        let color = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser = chart.add_series(color, ChartAxis::PrimaryY);

        let mut seed: u32 = 42;
        for _ in 0..50 {
            let x = pseudo_rand(&mut seed, 0, 200);
            let y = pseudo_rand(&mut seed, 0, 1000);
            chart.set_next_value2(&ser, x, y);
        }

        let timer = Timer::new(100)?;

        Ok(Self { _screen: screen, chart, ser, timer, seed })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        if self.timer.triggered() {
            let x = pseudo_rand(&mut self.seed, 0, 200);
            let y = pseudo_rand(&mut self.seed, 0, 1000);
            self.chart.set_next_value2(&self.ser, x, y);
        }
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetChart7);
