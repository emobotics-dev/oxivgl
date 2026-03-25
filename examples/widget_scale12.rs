#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Scale 12 — Compass gauge with animated rotation
//!
//! Round scale displaying 8 compass directions (N/NE/E/SE/S/SW/W/NW) with a
//! line needle and continuous rotation animation (full circle in ~10 s).

use oxivgl::{
    anim::{Anim, ANIM_REPEAT_INFINITE, anim_path_linear, anim_set_scale_rotation},
    fonts,
    scale_labels,
    style::{
        Palette, Selector, StyleBuilder, color_white, palette_darken, palette_main,
    },
    view::View,
    widgets::{
        Align, Label, Line, Obj, Part, Scale, ScaleLabels, ScaleMode, Screen, WidgetError,
        SCALE_LABEL_ROTATE_KEEP_UPRIGHT, SCALE_LABEL_ROTATE_MATCH_TICKS,
    },
};

static COMPASS_LABELS: &ScaleLabels = scale_labels!(
    c"N", c"NE", c"E", c"SE", c"S", c"SW", c"W", c"NW"
);

struct WidgetScale12 {
    _bg: Obj<'static>,
    _scale: Scale<'static>,
    _needle: Line<'static>,
    _needle_style: oxivgl::style::Style,
    _tick_style: oxivgl::style::Style,
    _heading_lbl: Label<'static>,
}

impl View for WidgetScale12 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Dark circular background
        let bg = Obj::new(&screen)?;
        bg.size(220, 220).center();
        bg.radius(i32::MAX, Selector::DEFAULT);
        bg.style_bg_color(palette_darken(Palette::Grey, 4), Selector::DEFAULT);
        bg.bg_opa(255);
        bg.remove_scrollable();
        bg.pad(0);

        // Scale — round inner, 8 compass points
        let scale = Scale::new(&bg)?;
        scale.center();
        scale.size(200, 200);
        scale.style_arc_width(3, Selector::DEFAULT);

        scale.set_mode(ScaleMode::RoundInner);
        scale.set_range(0, 360);
        scale.set_total_tick_count(33); // 32 minor divisions + closing tick
        scale.set_major_tick_every(4);  // major every 45°
        scale.set_angle_range(360);
        scale.set_rotation(270); // N at top (initial)
        scale.set_label_show(true);
        scale.style_text_font(fonts::MONTSERRAT_14, Part::Indicator);
        scale.style_text_color(color_white(), Part::Indicator);

        // Rotate labels to match tick angles, keep upright
        scale.style_transform_rotation(
            SCALE_LABEL_ROTATE_MATCH_TICKS | SCALE_LABEL_ROTATE_KEEP_UPRIGHT,
            Part::Indicator,
        );

        // Major tick style
        let mut tick_sb = StyleBuilder::new();
        tick_sb
            .line_color(palette_main(Palette::Cyan))
            .line_width(2)
            .width(12);
        let tick_style = tick_sb.build();
        scale.add_style(&tick_style, Part::Indicator);

        // Custom compass labels
        scale.set_text_src(COMPASS_LABELS);

        // Needle (line from center outward)
        let needle = Line::new(&scale)?;
        let mut needle_sb = StyleBuilder::new();
        needle_sb.line_color(palette_main(Palette::Red)).line_width(3);
        let needle_style = needle_sb.build();
        needle.add_style(&needle_style, Selector::DEFAULT);
        scale.set_line_needle_value(&needle, 80, 0);

        // "HEADING" label
        let heading_lbl = Label::new(&bg)?;
        heading_lbl.text("COMPASS");
        heading_lbl.style_text_font(fonts::MONTSERRAT_16, Selector::DEFAULT);
        heading_lbl.style_text_color(color_white(), Selector::DEFAULT);
        heading_lbl.align(Align::Center, 0, 30);

        // Animation: rotate scale 0 -> 3600 (ten full circles) over 10s, infinite repeat
        let mut anim = Anim::new();
        anim.set_var(&scale)
            .set_values(0, 3600)
            .set_duration(100_000)
            .set_exec_cb(Some(anim_set_scale_rotation))
            .set_path_cb(Some(anim_path_linear))
            .set_repeat_count(ANIM_REPEAT_INFINITE);
        anim.start();

        Ok(Self {
            _bg: bg,
            _scale: scale,
            _needle: needle,
            _needle_style: needle_style,
            _tick_style: tick_style,
            _heading_lbl: heading_lbl,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetScale12);
