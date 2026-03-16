// SPDX-License-Identifier: MIT OR Apache-2.0
//! Universal convenience re-exports for widget, style, and animation types.
//!
//! ```ignore
//! use oxivgl::prelude::*;
//! ```

// View trait
pub use crate::view::View;

// Widgets
pub use crate::widgets::{
    Align, Arc, AsLvHandle, Bar, BaseDir, Button, Checkbox, Child, DdDir, Dropdown, Event,
    EventCode, FlexAlign, FlexFlow, GridAlign, GridCell, Image, Label, LabelLongMode, Layout, Led,
    Line, Obj, ObjFlag, ObjState, Opa, Part, Roller, Scale, Screen, ScrollDir, ScrollSnap,
    ScrollbarMode, Slider, Switch, TextAlign, ValueLabel, WidgetError, GRID_TEMPLATE_LAST,
    RADIUS_MAX,
};
pub use crate::widgets::{grid_fr, lv_color_t, lv_image_dsc_t, lv_point_precise_t};

// Style system
pub use crate::style::{
    darken_filter_cb, lv_pct, props, BorderSide, ColorFilter, GradDir, GradDsc, GradExtend,
    Palette, Selector, Style, StyleBuilder, TextDecor, Theme, TransitionDsc, LV_SIZE_CONTENT,
};
pub use crate::style::{
    color_black, color_make, color_white, palette_darken, palette_lighten, palette_main,
};

// Animation system
pub use crate::anim::{
    anim_path_bounce, anim_path_ease_in, anim_path_ease_in_out, anim_path_ease_out,
    anim_path_linear, anim_path_overshoot, anim_set_arc_value, anim_set_bar_value,
    anim_set_height, anim_set_pad_column, anim_set_pad_row, anim_set_size,
    anim_set_slider_value, anim_set_width, anim_set_x, Anim, AnimTimeline,
    ANIM_REPEAT_INFINITE, ANIM_TIMELINE_PROGRESS_MAX,
};
