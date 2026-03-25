#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Chart 4 — Bar chart with value-based coloring
//!
//! A bar chart where each bar's color is interpolated between red (low) and
//! green (high) based on its Y value. Uses `DRAW_TASK_ADDED` events with
//! `with_fill_dsc` to recolor bars during rendering.

use oxivgl::{
    enums::EventCode,
    event::Event,
    style::{color_make, color_mix, palette_main, Palette, Selector},
    view::{register_event_on, View},
    widgets::{Chart, ChartAxis, ChartSeries, ChartType, Part, Screen, WidgetError},
};

const NUM_POINTS: usize = 24;

/// Simple LCG pseudo-random.
fn pseudo_rand(seed: &mut u32, min: i32, max: i32) -> i32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let range = (max - min + 1) as u32;
    min + ((*seed >> 16) % range) as i32
}

struct WidgetChart4 {
    _screen: Screen,
    chart: Chart<'static>,
    _ser: ChartSeries,
    values: [i32; NUM_POINTS],
}

impl View for WidgetChart4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let chart = Chart::new(&screen)?;
        chart.set_type(ChartType::Bar);
        chart.set_point_count(NUM_POINTS as u32);
        chart.style_pad_column(2, Selector::DEFAULT);
        chart.size(260, 160);
        chart.center();

        let color = color_make(0xff, 0, 0);
        let ser = chart.add_series(color, ChartAxis::PrimaryY);

        chart.send_draw_task_events();

        let mut values = [0i32; NUM_POINTS];
        let mut seed: u32 = 42;
        for i in 0..NUM_POINTS {
            values[i] = pseudo_rand(&mut seed, 10, 90);
            chart.set_next_value(&ser, values[i]);
        }

        Ok(Self { _screen: screen, chart, _ser: ser, values })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.chart.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::DRAW_TASK_ADDED {
            return;
        }
        let Some(task) = event.draw_task() else { return };
        let base = task.base();
        if base.part != Part::Items {
            return;
        }
        let idx = base.id2 as usize;
        if idx < NUM_POINTS {
            let value = self.values[idx];
            let ratio = ((value * 255) / 100) as u8;
            let green = palette_main(Palette::Green);
            let red = palette_main(Palette::Red);
            task.with_fill_dsc(|dsc| {
                dsc.set_color(color_mix(green, red, ratio));
            });
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetChart4);
