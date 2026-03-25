#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 6 — Cursor on clicked point
//!
//! A line chart with a cursor crosshair. Clicking a data point moves the
//! cursor to that location. A label above says "Click on a point".

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::View,
    widgets::{Align, Chart, ChartAxis, ChartCursor, ChartSeries, ChartType, Label, Screen, WidgetError},
};

struct WidgetChart6 {
    _screen: Screen,
    chart: Chart<'static>,
    cursor: ChartCursor,
    _ser: ChartSeries,
    _label: Label<'static>,
}

impl View for WidgetChart6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let chart = Chart::new(&screen)?;
        chart.size(200, 150);
        chart.align(Align::Center, 0, 10);
        chart.set_type(ChartType::Line);
        chart.bubble_events();

        let color_blue = oxivgl::style::palette_main(oxivgl::style::Palette::Blue);
        let cursor = chart.add_cursor(color_blue, 0x01 | 0x08); // LEFT | BOTTOM

        let color_red = oxivgl::style::palette_main(oxivgl::style::Palette::Red);
        let ser = chart.add_series(color_red, ChartAxis::PrimaryY);
        for &v in &[32, 58, 17, 73, 45, 82, 29, 65, 41, 70] {
            chart.set_next_value(&ser, v);
        }

        let label = Label::new(&screen)?;
        label.text("Click on a point");
        label.align(Align::TopMid, 0, 5);

        Ok(Self { _screen: screen, chart, cursor, _ser: ser, _label: label })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.chart, EventCode::VALUE_CHANGED) {
            if let Some(id) = self.chart.get_pressed_point() {
                self.chart.set_cursor_point(&self.cursor, None, id);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetChart6);
