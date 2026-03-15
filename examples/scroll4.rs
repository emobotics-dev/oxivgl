#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scroll 4 — Custom scrollbar styling
//!
//! Replaces default scrollbar style with a blue rounded bar that widens and
//! becomes fully opaque when actively scrolling.

extern crate alloc;

use alloc::boxed::Box;
use oxivgl::{
    view::View,
    widgets::{
        anim_path_linear, palette_darken, palette_main, props, Label, Obj, ObjState, Palette,
        Part, Screen, ScrollbarMode, Style, TransitionDsc, WidgetError,
    },
};

struct Scroll4 {
    _obj: Obj<'static>,
    _label: Label<'static>,
    _style: Box<Style>,
    _style_scrolled: Box<Style>,
    _transition: Box<TransitionDsc>,
}

/// Transition property list: opacity + width + sentinel.
static TRANS_PROPS: [props::lv_style_prop_t; 3] = [props::BG_OPA, props::WIDTH, props::LAST];

impl View for Scroll4 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let obj = Obj::new(&screen)?;
        obj.size(200, 100).center();

        let label = Label::new(&obj)?;
        // Constrain label width so text wraps and overflows vertically
        label.width(200);
        label.text_long(concat!(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n",
            "Etiam dictum, tortor vestibulum lacinia laoreet, ",
            "mi neque consectetur neque, vel mattis odio dolor egestas ligula.\n",
            "Sed vestibulum sapien nulla, id convallis ex porttitor nec.\n",
            "Duis et massa eu libero accumsan faucibus a in arcu.\n",
            "Ut pulvinar odio lorem, vel tempus turpis condimentum quis.\n",
            "Sed nisl augue, venenatis in blandit et, gravida ac tortor.\n",
            "Etiam dapibus elementum suscipit.\n",
            "Proin mollis sollicitudin convallis.\n",
            "Integer dapibus tempus arcu nec viverra.\n",
            "Donec molestie nulla enim, eu interdum velit placerat quis.\n",
            "Donec id efficitur risus, at molestie turpis.\n",
            "Suspendisse vestibulum consectetur nunc ut commodo.\n",
            "Fusce molestie rhoncus nisi sit amet tincidunt.\n",
            "Suspendisse a nunc ut magna ornare volutpat.",
        ));

        // Force scrollbar always visible (C example relies on interactive scrolling)
        obj.set_scrollbar_mode(ScrollbarMode::On);

        let transition = Box::new(TransitionDsc::new(
            &TRANS_PROPS,
            Some(anim_path_linear),
            200,
            0,
        ));

        let mut style = Box::new(Style::new());
        style
            .width(4)
            .length(20)
            .pad_right(5)
            .pad_top(5)
            .radius(2)
            .bg_opa(178) // LV_OPA_70
            .bg_color(palette_main(Palette::Blue))
            .border_color(palette_darken(Palette::Blue, 3))
            .border_width(2)
            .shadow_width(8)
            .shadow_spread(2)
            .shadow_color(palette_darken(Palette::Blue, 1))
            .transition(&transition);

        let mut style_scrolled = Box::new(Style::new());
        style_scrolled.width(8).bg_opa(255); // LV_OPA_COVER

        obj.add_style(&style, Part::Scrollbar);
        obj.add_style(&style_scrolled, Part::Scrollbar | ObjState::SCROLLED);

        Ok(Self {
            _obj: obj,
            _label: label,
            _style: style,
            _style_scrolled: style_scrolled,
            _transition: transition,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Scroll4);
