#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 3 — Round scale with needle
//!
//! Round gauge with animated line needle sweeping 0–100.

use oxivgl::{
    style::{color_make, Selector, StyleBuilder},
    view::View,
    enums::Opa,
    widgets::{Line, Part, Scale, ScaleMode, Screen, WidgetError, RADIUS_MAX},
};

struct WidgetScale3 {
    scale: Scale<'static>,
    needle: Line<'static>,
    value: i32,
    ascending: bool,
}

impl View for WidgetScale3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let scale = Scale::new(&screen)?;
        scale.size(200, 200).center();
        scale
            .set_mode(ScaleMode::RoundInner)
            .set_rotation(135)
            .set_angle_range(270)
            .set_range(0, 100)
            .set_total_tick_count(21)
            .set_major_tick_every(5)
            .set_label_show(true)
            .set_tick_length(Part::Items, 5)
            .set_tick_length(Part::Indicator, 10);

        // Transparent bg, no border, round corners
        let mut sb = StyleBuilder::new();
        sb.bg_opa(Opa::TRANSP.0).border_width(0).radius(RADIUS_MAX as i16);
        let scale_style = sb.build();
        scale.add_style(&scale_style, Selector::DEFAULT);

        // Create needle line as child of scale
        let needle = Line::new(&scale)?;
        let mut sb = StyleBuilder::new();
        sb.line_width(3)
            .line_color(color_make(0x33, 0x33, 0x33))
            .line_rounded(true);
        let needle_style = sb.build();
        needle.add_style(&needle_style, Selector::DEFAULT);

        // Initial needle position
        scale.set_line_needle_value(&needle, 80, 50);

        Ok(Self {
            scale,
            needle,
            value: 50,
            ascending: true,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        if self.ascending {
            self.value += 1;
            if self.value >= 100 {
                self.ascending = false;
            }
        } else {
            self.value -= 1;
            if self.value <= 0 {
                self.ascending = true;
            }
        }
        self.scale
            .set_line_needle_value(&self.needle, 80, self.value);
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetScale3);
