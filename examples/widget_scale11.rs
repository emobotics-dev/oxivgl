#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 11 — 24-hour clock face with day/night arcs
//!
//! Round scale with custom hour labels, day/night colored sections, and a
//! `DRAW_TASK_ADDED` handler that highlights 06/12/18/24 labels white.

use oxivgl::{
    draw::DrawTask,
    enums::EventCode,
    event::Event,
    fonts,
    scale_labels,
    style::{
        color_white, palette_darken, palette_main, Palette, Selector, Style, StyleBuilder,
    },
    view::{register_event_on, View},
    widgets::{Align, Label, Obj, Part, Scale, ScaleLabels, ScaleMode, Screen, WidgetError,
        SCALE_LABEL_ROTATE_KEEP_UPRIGHT, SCALE_LABEL_ROTATE_MATCH_TICKS,
    },
};

static HOUR_LABELS: &ScaleLabels = scale_labels!(
    c"01", c"02", c"03", c"04", c"05",
    c"06", c"07", c"08", c"09", c"10",
    c"11", c"12", c"13", c"14", c"15",
    c"16", c"17", c"18", c"19", c"20",
    c"21", c"22", c"23", c"24"
);

struct WidgetScale11 {
    _bg: Obj<'static>,
    scale: Scale<'static>,
    _tick_style: Style,
    _night_style: Style,
    _day_style: Style,
    _today: Label<'static>,
    _sunrise_lbl: Label<'static>,
    _sunrise_time: Label<'static>,
    _sunset_lbl: Label<'static>,
    _sunset_time: Label<'static>,
}

impl WidgetScale11 {
    fn handle_draw_task(draw_task: &DrawTask) {
        let base = draw_task.base();
        if base.part != Part::Indicator {
            return;
        }
        let Some(label_dsc) = draw_task.label_dsc() else { return };
        let Some(text) = label_dsc.text() else { return };

        if text == "06" || text == "12" || text == "18" || text == "24" {
            label_dsc.set_color(color_white());
        } else {
            label_dsc.set_color(palette_darken(Palette::Grey, 1));
        }
    }
}

impl View for WidgetScale11 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Dark circular background
        let bg = Obj::new(&screen)?;
        bg.size(210, 210).center();
        bg.radius(i32::MAX, Selector::DEFAULT);
        bg.style_bg_color(palette_darken(Palette::Grey, 4), Selector::DEFAULT);
        bg.bg_opa(255);
        bg.remove_scrollable();
        bg.pad(0);

        // Scale
        let scale = Scale::new(&bg)?;
        scale.center();
        scale.size(150, 150);
        scale.style_arc_width(5, Selector::DEFAULT);

        scale.set_mode(ScaleMode::RoundOuter);
        scale.set_range(0, 24);
        scale.set_total_tick_count(25);
        scale.set_major_tick_every(1);
        scale.set_angle_range(360);
        scale.set_rotation(105);
        scale.set_label_show(true);
        scale.style_text_font(fonts::MONTSERRAT_12, Part::Indicator);
        scale.style_radial_offset(-6, Part::Indicator);

        // Rotate labels to match tick angles, keep upright
        scale.style_transform_rotation(
            SCALE_LABEL_ROTATE_MATCH_TICKS | SCALE_LABEL_ROTATE_KEEP_UPRIGHT,
            Part::Indicator,
        );

        // Major tick style
        let mut tick_sb = StyleBuilder::new();
        tick_sb.line_color(palette_darken(Palette::Grey, 1))
            .line_width(2)
            .width(10);
        let tick_style = tick_sb.build();
        scale.add_style(&tick_style, Part::Indicator);

        // Night section (blue arc)
        let mut night_sb = StyleBuilder::new();
        night_sb.arc_color(palette_main(Palette::Blue));
        let night_style = night_sb.build();
        let section_night = scale.add_section();
        section_night.set_range(17, 5);
        section_night.set_main_style(&night_style);

        // Day section (dark yellow arc)
        let mut day_sb = StyleBuilder::new();
        day_sb.arc_color(palette_darken(Palette::Yellow, 3));
        let day_style = day_sb.build();
        let section_day = scale.add_section();
        section_day.set_range(5, 17);
        section_day.set_main_style(&day_style);

        // Custom hour labels
        scale.set_text_src(HOUR_LABELS);

        // Enable draw task events for label recoloring
        scale.send_draw_task_events();

        // "TODAY" label
        let today = Label::new(&bg)?;
        today.text("TODAY");
        today.style_text_font(fonts::MONTSERRAT_16, Selector::DEFAULT);
        today.style_text_color(color_white(), Selector::DEFAULT);
        today.align(Align::TopMid, 0, 60);

        // Sunrise
        let sunrise_lbl = Label::new(&bg)?;
        sunrise_lbl.text("SUNRISE");
        sunrise_lbl.style_text_font(fonts::MONTSERRAT_14, Selector::DEFAULT);
        sunrise_lbl.style_text_color(palette_main(Palette::Grey), Selector::DEFAULT);
        sunrise_lbl.align(Align::LeftMid, 37, -10);

        let sunrise_time = Label::new(&bg)?;
        sunrise_time.text("6:43");
        sunrise_time.style_text_font(fonts::MONTSERRAT_20, Selector::DEFAULT);
        sunrise_time.style_text_color(color_white(), Selector::DEFAULT);
        sunrise_time.align_to(&sunrise_lbl, Align::OutBottomMid, 0, 2);

        // Sunset
        let sunset_lbl = Label::new(&bg)?;
        sunset_lbl.text("SUNSET");
        sunset_lbl.style_text_font(fonts::MONTSERRAT_14, Selector::DEFAULT);
        sunset_lbl.style_text_color(palette_main(Palette::Grey), Selector::DEFAULT);
        sunset_lbl.align(Align::RightMid, -37, -10);

        let sunset_time = Label::new(&bg)?;
        sunset_time.text("17:37");
        sunset_time.style_text_font(fonts::MONTSERRAT_20, Selector::DEFAULT);
        sunset_time.style_text_color(color_white(), Selector::DEFAULT);
        sunset_time.align_to(&sunset_lbl, Align::OutBottomMid, 0, 2);

        Ok(Self {
            _bg: bg,
            scale,
            _tick_style: tick_style,
            _night_style: night_style,
            _day_style: day_style,
            _today: today,
            _sunrise_lbl: sunrise_lbl,
            _sunrise_time: sunrise_time,
            _sunset_lbl: sunset_lbl,
            _sunset_time: sunset_time,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.scale.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == EventCode::DRAW_TASK_ADDED {
            if let Some(draw_task) = event.draw_task() {
                Self::handle_draw_task(&draw_task);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetScale11);
