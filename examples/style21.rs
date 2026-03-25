#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 21 — Material-design cards with shadow, rotation & scale transforms
//!
//! Two card objects with rounded corners and drop shadow. An Arc controls
//! rotation transform; a Slider controls scale transform on both cards.

use oxivgl::{
    enums::EventCode,
    event::Event,
    layout::FlexFlow,
    style::{color_make, Selector},
    view::View,
    widgets::{Align, Arc, Label, Obj, Screen, Slider, WidgetError},
};

struct Style21 {
    card1: Obj<'static>,
    card2: Obj<'static>,
    arc: Arc<'static>,
    slider: Slider<'static>,
    _lbl1: Label<'static>,
    _lbl2: Label<'static>,
    _arc_lbl: Label<'static>,
    _slider_lbl: Label<'static>,
}

impl View for Style21 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.bg_color(0xeeeeee).bg_opa(255);

        // --- Card 1 ---
        let card1 = Obj::new(&screen)?;
        card1.size(120, 80);
        card1.align(Align::TopLeft, 20, 20);
        card1.bg_color(0xffffff).bg_opa(255);
        card1.radius(12, Selector::DEFAULT);
        card1.border_width(0);
        card1.style_shadow_width(20, Selector::DEFAULT);
        card1.style_shadow_color(color_make(0x88, 0x88, 0x88), Selector::DEFAULT);
        card1.style_shadow_offset_x(2, Selector::DEFAULT);
        card1.style_shadow_offset_y(4, Selector::DEFAULT);
        card1.style_shadow_spread(0, Selector::DEFAULT);
        card1.style_shadow_opa(200, Selector::DEFAULT);
        card1.style_transform_pivot_x(60, Selector::DEFAULT);
        card1.style_transform_pivot_y(40, Selector::DEFAULT);

        let lbl1 = Label::new(&card1)?;
        lbl1.text("Card A").center();

        // --- Card 2 ---
        let card2 = Obj::new(&screen)?;
        card2.size(120, 80);
        card2.align(Align::TopRight, -20, 20);
        card2.bg_color(0xffffff).bg_opa(255);
        card2.radius(12, Selector::DEFAULT);
        card2.border_width(0);
        card2.style_shadow_width(20, Selector::DEFAULT);
        card2.style_shadow_color(color_make(0x88, 0x88, 0x88), Selector::DEFAULT);
        card2.style_shadow_offset_x(2, Selector::DEFAULT);
        card2.style_shadow_offset_y(4, Selector::DEFAULT);
        card2.style_shadow_spread(0, Selector::DEFAULT);
        card2.style_shadow_opa(200, Selector::DEFAULT);
        card2.style_transform_pivot_x(60, Selector::DEFAULT);
        card2.style_transform_pivot_y(40, Selector::DEFAULT);

        let lbl2 = Label::new(&card2)?;
        lbl2.text("Card B").center();

        // --- Controls container (bottom half) ---
        let controls = Obj::new(&screen)?;
        controls.size(280, 100);
        controls.align(Align::BottomMid, 0, -10);
        controls.bg_opa(0);
        controls.border_width(0);
        controls.set_flex_flow(FlexFlow::Column);
        controls.pad(4);
        controls.style_pad_row(8, Selector::DEFAULT);

        // Arc — controls rotation
        let arc_row = Obj::new(&controls)?;
        arc_row.size(260, 40);
        arc_row.bg_opa(0).border_width(0);
        arc_row.set_flex_flow(FlexFlow::Row);

        let arc_lbl = Label::new(&arc_row)?;
        arc_lbl.text("Rot:");

        let arc = Arc::new(&arc_row)?;
        arc.size(40, 40);
        arc.set_rotation(270);
        arc.set_bg_angles(0, 360);
        arc.set_range_raw(0, 3600); // 0..360 degrees in 0.1 units
        arc.set_value_raw(450); // 45 deg for screenshot
        arc.bubble_events();

        // Slider — controls scale
        let slider_row = Obj::new(&controls)?;
        slider_row.size(260, 40);
        slider_row.bg_opa(0).border_width(0);
        slider_row.set_flex_flow(FlexFlow::Row);

        let slider_lbl = Label::new(&slider_row)?;
        slider_lbl.text("Scl:");

        let slider = Slider::new(&slider_row)?;
        slider.size(180, 10);
        slider.center();
        slider.set_range(128, 512); // 0.5x .. 2.0x  (256 = 1.0)
        slider.set_value(256); // 1.0x for default
        slider.bubble_events();

        // Apply initial transforms for screenshot
        card1.style_transform_rotation(450, Selector::DEFAULT);
        card2.style_transform_rotation(450, Selector::DEFAULT);

        Ok(Self {
            card1,
            card2,
            arc,
            slider,
            _lbl1: lbl1,
            _lbl2: lbl2,
            _arc_lbl: arc_lbl,
            _slider_lbl: slider_lbl,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.arc, EventCode::VALUE_CHANGED) {
            let angle = self.arc.get_value_raw();
            self.card1
                .style_transform_rotation(angle, Selector::DEFAULT);
            self.card2
                .style_transform_rotation(angle, Selector::DEFAULT);
        }
        if event.matches(&self.slider, EventCode::VALUE_CHANGED) {
            let scale = self.slider.get_value();
            self.card1
                .style_transform_scale_x(scale, Selector::DEFAULT);
            self.card1
                .style_transform_scale_y(scale, Selector::DEFAULT);
            self.card2
                .style_transform_scale_x(scale, Selector::DEFAULT);
            self.card2
                .style_transform_scale_y(scale, Selector::DEFAULT);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style21);
