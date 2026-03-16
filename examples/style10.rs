#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style 10 — Transition

extern crate alloc;

use oxivgl::{
    anim::anim_path_linear,
    style::{
        palette_darken, palette_main, props, Palette, Selector, Style, StyleBuilder, TransitionDsc,
    },
    view::View,
    enums::ObjState,
    widgets::{Obj, Screen, WidgetError},
};

struct Style10 {
    _obj: Obj<'static>,
    _style_def: Style,
    _style_pr: Style,
}

static TRANS_PROPS: [props::lv_style_prop_t; 4] =
    [props::BG_COLOR, props::BORDER_COLOR, props::BORDER_WIDTH, 0];

impl View for Style10 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let trans_def = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 100, 200);

        let trans_pr = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 500, 0);

        let mut builder_def = StyleBuilder::new();
        builder_def.transition(trans_def);
        let style_def = builder_def.build();

        let mut builder_pr = StyleBuilder::new();
        builder_pr
            .bg_color(palette_main(Palette::Red))
            .border_width(6)
            .border_color(palette_darken(Palette::Red, 3))
            .transition(trans_pr);
        let style_pr = builder_pr.build();

        let obj = Obj::new(&screen)?;
        obj.add_style(&style_def, Selector::DEFAULT);
        obj.add_style(&style_pr, ObjState::PRESSED);
        obj.center();

        Ok(Self {
            _obj: obj,
            _style_def: style_def,
            _style_pr: style_pr,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Style10);
