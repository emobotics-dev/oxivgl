#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 8 — Circular line chart with gap
//!
//! A line chart in circular update mode with 80 data points. A timer adds
//! new values every 300 ms, overwriting the oldest. Three points ahead of
//! the write cursor are blanked to create a visible gap.

use oxivgl::{
    style::Selector,
    timer::Timer,
    view::View,
    widgets::{Chart, ChartAxis, ChartSeries, ChartType, ChartUpdateMode, Part, Screen, WidgetError, CHART_POINT_NONE},
};

/// Simple LCG pseudo-random.
fn pseudo_rand(seed: &mut u32, min: i32, max: i32) -> i32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let range = (max - min + 1) as u32;
    min + ((*seed >> 16) % range) as i32
}

struct WidgetChart8 {
    _screen: Screen,
    chart: Chart<'static>,
    ser: ChartSeries,
    timer: Timer,
    seed: u32,
}

impl View for WidgetChart8 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let chart = Chart::new(&screen)?;
        chart.size(280, 150);
        chart.center();
        chart.set_type(ChartType::Line);
        chart.set_update_mode(ChartUpdateMode::Circular);
        chart.set_point_count(80);
        chart.style_size(0, 0, Selector::from(Part::Indicator));

        let color = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser = chart.add_series(color, ChartAxis::PrimaryY);

        let mut seed: u32 = 123;
        for _ in 0..80 {
            chart.set_next_value(&ser, pseudo_rand(&mut seed, 10, 90));
        }

        let timer = Timer::new(300)?;

        Ok(Self { _screen: screen, chart, ser, timer, seed })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        if self.timer.triggered() {
            self.chart.set_next_value(&self.ser, pseudo_rand(&mut self.seed, 10, 90));

            // Blank 3 points ahead of write cursor to create gap
            let p = self.chart.get_point_count();
            let s = self.chart.get_x_start_point(&self.ser);
            for offset in 1..=3 {
                let idx = (s + offset) % p;
                self.chart.set_series_value_by_id(&self.ser, idx, CHART_POINT_NONE as i32);
            }
            self.chart.refresh();
        }
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetChart8);
