#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Arc 3 — Interactive pie chart
//!
//! Five colored arc segments forming a pie chart. Clicking a slice animates
//! it outward; clicking again or clicking another slice animates it back.

extern crate alloc;
use alloc::vec::Vec;

use oxivgl::{
    anim::{anim_set_x, anim_set_y, Anim},
    enums::{EventCode, ObjFlag},
    event::Event,
    math::{trigo_cos, trigo_sin, TRIGO_SHIFT},
    style::{palette_main, Palette},
    view::{register_event_on, View},
    widgets::{Align, Arc, ArcMode, Label, Obj, Part, Screen, WidgetError},
};

const CHART_SIZE: i32 = 150;
const SLICE_OFFSET: i32 = 20;
const NUM_SLICES: usize = 5;

struct SliceInfo {
    mid_angle: i32,
    home_x: i32,
    home_y: i32,
    out: bool,
}

struct WidgetArc3 {
    _container: Obj<'static>,
    arcs: [Arc<'static>; NUM_SLICES],
    _labels: [Label<'static>; NUM_SLICES],
    slices: [SliceInfo; NUM_SLICES],
    active: Option<usize>,
}

const PERCENTAGES: [i32; NUM_SLICES] = [12, 18, 26, 24, 20];
const COLORS: [Palette; NUM_SLICES] = [
    Palette::Red,
    Palette::Blue,
    Palette::Green,
    Palette::Orange,
    Palette::BlueGrey,
];

impl WidgetArc3 {
    fn animate_slice(&self, idx: usize, to_x: i32, to_y: i32) {
        let arc = &self.arcs[idx];
        let cur_x = arc.get_x();
        let cur_y = arc.get_y();

        let mut ax = Anim::new();
        ax.set_var(arc)
            .set_exec_cb(Some(anim_set_x))
            .set_values(cur_x, to_x)
            .set_duration(200);
        ax.start();

        let mut ay = Anim::new();
        ay.set_var(arc)
            .set_exec_cb(Some(anim_set_y))
            .set_values(cur_y, to_y)
            .set_duration(200);
        ay.start();
    }
}

impl View for WidgetArc3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Slices container — transparent, non-scrollable
        let cont = Obj::new(&screen)?;
        let cont_size = CHART_SIZE + 2 * SLICE_OFFSET;
        cont.size(cont_size, cont_size).center();
        cont.pad(0);
        cont.border_width(0);
        cont.bg_opa(0);
        cont.remove_flag(ObjFlag::SCROLLABLE);

        let mut angle_accum: f32 = 0.0;
        let mut arcs_vec = Vec::with_capacity(NUM_SLICES);
        let mut labels_vec = Vec::with_capacity(NUM_SLICES);
        let mut slices_vec = Vec::with_capacity(NUM_SLICES);

        for i in 0..NUM_SLICES {
            let pct = PERCENTAGES[i];
            let slice_angle = (pct as f32 * 360.0) / 100.0;
            let start = (angle_accum + 0.5) as i32;
            angle_accum += slice_angle;
            let mut end = (angle_accum + 0.5) as i32;
            if end > 360 {
                end = 360;
            }

            let arc = Arc::new(&cont)?;
            arc.size(CHART_SIZE, CHART_SIZE).center();
            arc.set_mode(ArcMode::Normal);
            arc.set_bg_start_angle(start);
            arc.set_bg_end_angle(end);

            // Main part = filled pie, indicator = invisible
            arc.style_arc_width(CHART_SIZE / 2, Part::Main);
            arc.style_arc_width(0, Part::Indicator);
            arc.style_arc_color(palette_main(COLORS[i]), Part::Main);
            arc.style_arc_rounded(false, Part::Main);
            arc.remove_style(None, Part::Knob);
            arc.add_flag(ObjFlag::ADV_HITTEST);
            arc.bubble_events();

            // Percentage label at midpoint
            let label = Label::new(&arc)?;
            let mut buf = heapless::String::<8>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}%", pct));
            label.text(&buf);

            let mid_angle = start + (end - start) / 2;
            let radius = CHART_SIZE / 4;
            let x_off = (radius * trigo_cos(mid_angle)) >> TRIGO_SHIFT;
            let y_off = (radius * trigo_sin(mid_angle)) >> TRIGO_SHIFT;
            label.align(Align::Center, x_off, y_off);

            let home_x = arc.get_x();
            let home_y = arc.get_y();

            arcs_vec.push(arc);
            labels_vec.push(label);
            slices_vec.push(SliceInfo { mid_angle, home_x, home_y, out: false });
        }

        let arcs: [Arc<'static>; NUM_SLICES] = arcs_vec.try_into().ok()
            .ok_or(WidgetError::LvglNullPointer)?;
        let labels: [Label<'static>; NUM_SLICES] = labels_vec.try_into().ok()
            .ok_or(WidgetError::LvglNullPointer)?;
        let slices: [SliceInfo; NUM_SLICES] = slices_vec.try_into().ok()
            .ok_or(WidgetError::LvglNullPointer)?;

        Ok(Self {
            _container: cont,
            arcs,
            _labels: labels,
            slices,
            active: None,
        })
    }

    fn register_events(&mut self) {
        let handles: [_; NUM_SLICES] = core::array::from_fn(|i| self.arcs[i].handle());
        for h in handles {
            register_event_on(self, h);
        }
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::CLICKED {
            return;
        }
        let target = event.target_handle();

        // Find which slice was clicked
        let Some(idx) = self.arcs.iter().position(|a| a.handle() == target) else {
            return;
        };

        // If another slice is currently out, animate it back
        if let Some(prev) = self.active {
            if prev != idx && self.slices[prev].out {
                let info = &self.slices[prev];
                self.animate_slice(prev, info.home_x, info.home_y);
                self.slices[prev].out = false;
            }
        }

        let info = &self.slices[idx];
        if info.out {
            // Animate back
            self.animate_slice(idx, info.home_x, info.home_y);
            self.slices[idx].out = false;
            self.active = None;
        } else {
            // Animate out
            let x_off = (SLICE_OFFSET * trigo_cos(info.mid_angle)) >> TRIGO_SHIFT;
            let y_off = (SLICE_OFFSET * trigo_sin(info.mid_angle)) >> TRIGO_SHIFT;
            self.animate_slice(idx, info.home_x + x_off, info.home_y + y_off);
            self.slices[idx].out = true;
            self.active = Some(idx);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetArc3);
