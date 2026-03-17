#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Arc 3 — Donut chart
//!
//! Three colored arc segments forming a donut chart (simplified from LVGL's
//! interactive pie chart example).

extern crate alloc;

use oxivgl::{
    style::{color_make, Style, StyleBuilder},
    view::View,
    widgets::{Arc, ArcMode, ObjFlag, Opa, Part, Screen, WidgetError},
};

struct WidgetArc3 {
    _arc1: Arc<'static>,
    _arc2: Arc<'static>,
    _arc3: Arc<'static>,
    _style_red: Style,
    _style_green: Style,
    _style_blue: Style,
    _style_bg: Style,
    _style_knob: Style,
}

impl View for WidgetArc3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Indicator styles — each segment gets a different color
        let mut b = StyleBuilder::new();
        b.arc_color(color_make(0xe0, 0x40, 0x40)).arc_width(20);
        let style_red = b.build();

        let mut b = StyleBuilder::new();
        b.arc_color(color_make(0x40, 0xb0, 0x40)).arc_width(20);
        let style_green = b.build();

        let mut b = StyleBuilder::new();
        b.arc_color(color_make(0x40, 0x60, 0xe0)).arc_width(20);
        let style_blue = b.build();

        // Transparent background arc
        let mut b = StyleBuilder::new();
        b.arc_width(0);
        let style_bg = b.build();

        // Hidden knob
        let mut b = StyleBuilder::new();
        b.pad_all(0);
        let style_knob = b.build();

        // Helper: create one donut segment
        let make_seg = |start: i32, end: i32, ind_style: &Style| -> Result<Arc<'static>, WidgetError> {
            let arc = Arc::new(&screen)?;
            arc.size(150, 150);
            arc.center();
            arc.set_mode(ArcMode::Normal);
            arc.set_bg_start_angle(start);
            arc.set_bg_end_angle(end);
            arc.set_range_raw(start, end);
            arc.set_value_raw(end); // fill entire segment
            arc.remove_flag(ObjFlag::CLICKABLE);
            arc.remove_flag(ObjFlag::SCROLLABLE);
            arc.add_style(ind_style, Part::Indicator);
            arc.add_style(&style_bg, Part::Main);
            arc.add_style(&style_knob, Part::Knob);
            arc.style_opa(Opa::TRANSP.0, Part::Knob);
            Ok(arc)
        };

        let arc1 = make_seg(0, 120, &style_red)?;
        let arc2 = make_seg(120, 240, &style_green)?;
        let arc3 = make_seg(240, 360, &style_blue)?;

        Ok(Self {
            _arc1: arc1,
            _arc2: arc2,
            _arc3: arc3,
            _style_red: style_red,
            _style_green: style_green,
            _style_blue: style_blue,
            _style_bg: style_bg,
            _style_knob: style_knob,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetArc3);
