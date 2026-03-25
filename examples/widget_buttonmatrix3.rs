#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Buttonmatrix 3 — Pagination
//!
//! A pill-shaped button group with left/right arrows and numbered pages 1-5.
//! Only one page number can be checked at a time. Arrows navigate between
//! pages.

use oxivgl::{
    btnmatrix_map,
    enums::EventCode,
    event::Event,
    style::{BorderSide, Selector, StyleBuilder},
    view::View,
    widgets::{
        Align, Buttonmatrix, ButtonmatrixCtrl, ButtonmatrixMap, Part, RADIUS_MAX, Screen,
        WidgetError,
    },
};

static MAP: &ButtonmatrixMap = btnmatrix_map!(
    c"\xEF\x81\x93", c"1", c"2", c"3", c"4", c"5", c"\xEF\x81\x94"
);

struct WidgetButtonmatrix3 {
    _screen: Screen,
    btnm: Buttonmatrix<'static>,
    _style_bg: oxivgl::style::Style,
    _style_btn: oxivgl::style::Style,
}

impl View for WidgetButtonmatrix3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut style_bg = StyleBuilder::new();
        style_bg
            .pad_all(0)
            .radius(RADIUS_MAX as i16)
            .border_width(0);
        let style_bg = style_bg.build();

        let mut style_btn = StyleBuilder::new();
        style_btn
            .radius(0)
            .border_width(1)
            .border_opa(128)
            .border_color(oxivgl::style::palette_main(oxivgl::style::Palette::Grey))
            .border_side(BorderSide::INTERNAL);
        let style_btn = style_btn.build();

        let btnm = Buttonmatrix::new(&screen)?;
        btnm.set_map(MAP);
        btnm.add_style(&style_bg, Selector::DEFAULT);
        btnm.add_style(&style_btn, Selector::from(Part::Items));
        // Clip corners to respect the pill shape
        btnm.style_clip_corner(true, Selector::DEFAULT);
        // Zero gap between buttons
        btnm.style_pad_column(0, Selector::DEFAULT);
        btnm.size(225, 35);
        btnm.align(Align::Center, 0, 0);

        // All checkable, except arrows (btn 0 and 6)
        btnm.set_button_ctrl_all(ButtonmatrixCtrl::CHECKABLE);
        btnm.clear_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE);
        btnm.clear_button_ctrl(6, ButtonmatrixCtrl::CHECKABLE);

        // One-checked mode, start on page 1 (btn_id=1)
        btnm.set_one_checked(true);
        btnm.set_button_ctrl(1, ButtonmatrixCtrl::CHECKED);

        btnm.bubble_events();

        Ok(Self {
            _screen: screen,
            btnm,
            _style_bg: style_bg,
            _style_btn: style_btn,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.btnm, EventCode::VALUE_CHANGED) {
            let id = self.btnm.get_selected_button();
            let prev = id == 0;
            let next = id == 6;
            if prev || next {
                // Find currently checked page (buttons 1-5)
                let mut current = 1u32;
                for i in 1..=5 {
                    if self.btnm.has_button_ctrl(i, ButtonmatrixCtrl::CHECKED) {
                        current = i;
                        break;
                    }
                }
                let target = if prev && current > 1 {
                    current - 1
                } else if next && current < 5 {
                    current + 1
                } else {
                    current
                };
                self.btnm.set_button_ctrl(target, ButtonmatrixCtrl::CHECKED);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetButtonmatrix3);
