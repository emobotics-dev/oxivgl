// SPDX-License-Identifier: MIT OR Apache-2.0
//! Style system: builders, selectors, themes, gradients, and color palettes.

mod grad;
mod palette;
mod selector;
mod style;
mod theme;

pub use grad::{GradDsc, GradExtend};
pub use palette::{
    color_black, color_brightness, color_darken, color_make, color_white, palette_darken,
    palette_lighten, palette_main, GradDir, Palette,
};
pub use selector::Selector;
pub use style::{
    darken_filter_cb, lv_pct, props, BorderSide, ColorFilter, Style, StyleBuilder, TextDecor,
    TransitionDsc, LV_SIZE_CONTENT,
};
pub use theme::Theme;
