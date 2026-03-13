#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 10 — Transition

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        anim_path_linear, palette_darken, palette_main, props, Obj, ObjState, Palette, Screen,
        Selector, Style, TransitionDsc, WidgetError,
    },
};

struct Style10 {
    _obj: Obj<'static>,
    _style_def: Box<Style>,
    _style_pr: Box<Style>,
    _trans_def: Box<TransitionDsc>,
    _trans_pr: Box<TransitionDsc>,
}

static TRANS_PROPS: [props::lv_style_prop_t; 4] =
    [props::BG_COLOR, props::BORDER_COLOR, props::BORDER_WIDTH, 0];

impl View for Style10 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let trans_def = Box::new(TransitionDsc::new(
            &TRANS_PROPS,
            Some(anim_path_linear),
            100,
            200,
        ));

        let trans_pr = Box::new(TransitionDsc::new(
            &TRANS_PROPS,
            Some(anim_path_linear),
            500,
            0,
        ));

        let mut style_def = Box::new(Style::new());
        style_def.transition(&trans_def);

        let mut style_pr = Box::new(Style::new());
        style_pr
            .bg_color(palette_main(Palette::Red))
            .border_width(6)
            .border_color(palette_darken(Palette::Red, 3))
            .transition(&trans_pr);

        let obj = Obj::new(&screen)?;
        obj.add_style(&style_def, Selector::DEFAULT);
        obj.add_style(&style_pr, ObjState::PRESSED);
        obj.center();

        Ok(Self {
            _obj: obj,
            _style_def: style_def,
            _style_pr: style_pr,
            _trans_def: trans_def,
            _trans_pr: trans_pr,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style10);
