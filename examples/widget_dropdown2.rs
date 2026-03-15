#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Dropdown 2 — Drop-down in four directions
//!
//! Four dropdowns opening in each cardinal direction.

use oxivgl::{
    view::View,
    widgets::{Align, DdDir, Dropdown, Screen, WidgetError},
};

struct WidgetDropdown2 {
    _dd_top: Dropdown<'static>,
    _dd_bottom: Dropdown<'static>,
    _dd_right: Dropdown<'static>,
    _dd_left: Dropdown<'static>,
}

const OPTS: &str = "Apple\nBanana\nOrange\nMelon";

impl View for WidgetDropdown2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Default (opens downward)
        let dd_top = Dropdown::new(&screen)?;
        dd_top.set_options(OPTS);
        dd_top.align(Align::TopMid, 0, 10);

        // Opens upward
        let dd_bottom = Dropdown::new(&screen)?;
        dd_bottom.set_options(OPTS);
        dd_bottom.set_dir(DdDir::Top);
        dd_bottom.align(Align::BottomMid, 0, -10);

        // Opens to the right
        let dd_right = Dropdown::new(&screen)?;
        dd_right.set_options(OPTS);
        dd_right.set_dir(DdDir::Right);
        dd_right.align(Align::LeftMid, 10, 0);

        // Opens to the left
        let dd_left = Dropdown::new(&screen)?;
        dd_left.set_options(OPTS);
        dd_left.set_dir(DdDir::Left);
        dd_left.align(Align::RightMid, -10, 0);

        Ok(Self {
            _dd_top: dd_top,
            _dd_bottom: dd_bottom,
            _dd_right: dd_right,
            _dd_left: dd_left,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetDropdown2);
