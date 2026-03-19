// SPDX-License-Identifier: MIT OR Apache-2.0
//! Universal convenience re-exports for widget, style, and animation types.
//!
//! ```
//! use oxivgl::prelude::*;
//! ```

// View trait
pub use crate::view::View;

// Core LVGL enums
pub use crate::enums::{
    EventCode, ObjFlag, ObjState, Opa, ScrollDir, ScrollSnap, ScrollbarMode,
};

// Event system
pub use crate::event::Event;

// Timer
pub use crate::timer::Timer;

// Layout
pub use crate::layout::{
    FlexAlign, FlexFlow, GridAlign, GridCell, Layout, GRID_TEMPLATE_LAST, grid_fr,
};

// Widgets
pub use crate::widgets::{
    Align, Arc, ArcMode, AsLvHandle, Bar, BarMode, BaseDir, Button, Buttonmatrix, ButtonmatrixMap,
    Chart, ChartAxis, ChartSeries, ChartType, Checkbox, Child, DdDir, Dropdown, Image, ImageAlign,
    Keyboard, KeyboardMode, Label, LabelLongMode, Led, Line, List, Matrix, Menu, MenuHeaderMode,
    Msgbox, Obj, Part, Roller, RollerMode, Scale, ScaleBuilder, ScaleLabels, ScaleMode,
    ScaleSection, Screen, Slider, SliderMode, Switch, SwitchOrientation, Textarea, TextAlign,
    ValueLabel, WidgetError, RADIUS_MAX, SCALE_LABEL_ROTATE_KEEP_UPRIGHT,
    SCALE_LABEL_ROTATE_MATCH_TICKS,
};
pub use crate::widgets::{lv_color_t, lv_image_dsc_t, lv_point_precise_t};

// Style system
pub use crate::style::{
    color_black, color_brightness, color_darken, color_make, color_white, darken_filter_cb, lv_pct,
    palette_darken, palette_lighten, palette_main, props, BorderSide, ColorFilter, GradDir, GradDsc,
    GradExtend, Palette, Selector, Style, StyleBuilder, TextDecor, Theme, TransitionDsc,
    LV_SIZE_CONTENT,
};

// Math utilities
pub use crate::math::{bezier3, map, trigo_cos, trigo_sin, BEZIER_VAL_MAX, TRIGO_SHIFT};

// Symbol icons
pub use crate::symbols::Symbol;

// Animation system
pub use crate::anim::{
    anim_path_bounce, anim_path_ease_in, anim_path_ease_in_out, anim_path_ease_out,
    anim_path_linear, anim_path_overshoot, anim_set_arc_value, anim_set_bar_value,
    anim_set_height, anim_set_pad_column, anim_set_pad_row, anim_set_size,
    anim_set_slider_value, anim_set_width, anim_set_x, anim_set_y, Anim, AnimTimeline,
    ANIM_REPEAT_INFINITE, ANIM_TIMELINE_PROGRESS_MAX,
};
