#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget ArcLabel 1 — Text curved along circular arcs
//!
//! Three ArcLabel widgets with different radius, angle, and direction settings.

use oxivgl::{
    style::Selector,
    view::View,
    widgets::{Align, ArcLabel, ArcLabelDir, Screen, WidgetError},
};

struct WidgetArclabel1 {
    _al1: ArcLabel<'static>,
    _al2: ArcLabel<'static>,
    _al3: ArcLabel<'static>,
}

impl View for WidgetArclabel1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.bg_color(0xffffff).bg_opa(255);

        // Large clockwise arc across the top
        let al1 = ArcLabel::new(&screen)?;
        al1.set_text_static(c"Hello curved world!");
        al1.set_radius(90);
        al1.set_angle_start(40.0);
        al1.set_angle_size(100.0);
        al1.set_dir(ArcLabelDir::Clockwise);
        al1.size(200, 200);
        al1.align(Align::Center, 0, -30);
        al1.text_color(0x2196F3);

        // Counter-clockwise arc below
        let al2 = ArcLabel::new(&screen)?;
        al2.set_text_static(c"ArcLabel CCW");
        al2.set_radius(70);
        al2.set_angle_start(320.0);
        al2.set_angle_size(140.0);
        al2.set_dir(ArcLabelDir::CounterClockwise);
        al2.size(160, 160);
        al2.align(Align::Center, -50, 30);
        al2.text_color(0xE91E63);

        // Small clockwise arc, right side
        let al3 = ArcLabel::new(&screen)?;
        al3.set_text_static(c"OXIVGL");
        al3.set_radius(45);
        al3.set_angle_start(60.0);
        al3.set_angle_size(120.0);
        al3.set_dir(ArcLabelDir::Clockwise);
        al3.size(110, 110);
        al3.align(Align::Center, 70, 40);
        al3.text_color(0x4CAF50);
        al3.style_text_letter_space(4, Selector::DEFAULT);

        Ok(Self {
            _al1: al1,
            _al2: al2,
            _al3: al3,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetArclabel1);
