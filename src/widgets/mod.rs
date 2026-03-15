use heapless::c_string::ExtendError;
use thiserror_no_std::Error;

/// Internal LVGL integer scale: physical values are mapped to `0..LVGL_SCALE`
/// for arc/bar ranges.
pub(crate) const LVGL_SCALE: i32 = 1000;

/// Map a physical value `v` in `0..max` to LVGL's integer range
/// `0..LVGL_SCALE`. Returns 0 if `max` is 0 to avoid division by zero.
pub(crate) fn to_lvgl(v: f32, max: f32) -> i32 {
    if max == 0.0 {
        return 0;
    }
    (((v / max) * LVGL_SCALE as f32) as i32).clamp(0, LVGL_SCALE)
}

mod anim;
mod anim_timeline;
mod arc;
mod bar;
mod button;
mod checkbox;
mod child;
mod enums;
mod dropdown;
pub(crate) mod event;
mod grad;
mod grid;
mod image;
mod label;
mod led;
mod line;
mod obj;
mod obj_layout;
mod obj_style;
mod palette;
pub mod prelude;
mod roller;
mod scale;
mod screen;
mod selector;
mod slider;
mod style;
mod switch;
mod theme;
mod value_label;

pub use anim::{
    anim_path_bounce, anim_path_ease_in, anim_path_ease_in_out, anim_path_ease_out,
    anim_path_linear, anim_path_overshoot, anim_set_arc_value, anim_set_bar_value,
    anim_set_height, anim_set_pad_column, anim_set_pad_row, anim_set_size,
    anim_set_slider_value, anim_set_width, anim_set_x, Anim, ANIM_REPEAT_INFINITE,
};
pub use anim_timeline::{AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX};
pub use arc::Arc;
pub use bar::Bar;
pub use button::Button;
pub use checkbox::Checkbox;
pub use child::{detach, Child};
pub use dropdown::{DdDir, Dropdown};
pub use enums::{
    BarMode, EventCode, Layout, ObjFlag, ObjState, Opa, ScrollDir, ScrollSnap, ScrollbarMode,
};
pub use event::Event;
pub use grad::{GradDsc, GradExtend};
pub use grid::GridCell;
pub use image::Image;
pub use label::{Label, LabelLongMode};
pub use led::Led;
pub use line::Line;
pub use obj::{
    Align, AsLvHandle, BaseDir, FlexAlign, FlexFlow, GridAlign, Matrix, Obj, Part, TextAlign,
};
pub use palette::{
    color_black, color_make, color_white, palette_darken, palette_lighten, palette_main, GradDir,
    Palette,
};
pub use scale::{Scale, ScaleBuilder, ScaleMode};
pub use roller::{Roller, RollerMode};
pub use screen::Screen;
pub use selector::Selector;
pub use slider::Slider;
pub use style::{
    darken_filter_cb, lv_pct, props, BorderSide, ColorFilter, Style, TextDecor, TransitionDsc,
    LV_SIZE_CONTENT,
};
pub use switch::Switch;
pub use theme::Theme;
pub use value_label::ValueLabel;

// Re-export raw types so callbacks don't need `lvgl_rust_sys`.
pub use lvgl_rust_sys::{lv_color_t, lv_event_t, lv_image_dsc_t, lv_point_precise_t};

// Grid helpers
/// Maximum corner radius — creates a pill/capsule shape.
/// Equivalent to LVGL's `LV_RADIUS_CIRCLE` (0x7FFF).
pub const RADIUS_MAX: i32 = 0x7FFF;

/// Sentinel value marking the end of a grid template descriptor array.
pub const GRID_TEMPLATE_LAST: i32 = lvgl_rust_sys::LV_COORD_MAX as i32;

/// Return a fractional grid unit. Equivalent to `LV_GRID_FR(x)`.
pub const fn grid_fr(x: i32) -> i32 {
    lvgl_rust_sys::LV_COORD_MAX as i32 - 100 + x
}

/// Errors returned by widget constructors and setters.
#[derive(Error, Debug)]
pub enum WidgetError {
    /// `core::fmt::write` failed (e.g. buffer too small).
    #[error(transparent)]
    FormatError(#[from] core::fmt::Error),

    /// LVGL returned a NULL pointer (e.g. out of memory).
    #[error("LVGL: got NULL pointer")]
    LvglNullPointer,

    /// `heapless::CString` extend failed (text too long for buffer).
    #[error("CString error")]
    ExtendError(#[from] ExtendError),
}

#[cfg(test)]
mod tests {
    use super::{to_lvgl, LVGL_SCALE};

    #[test]
    fn to_lvgl_zero_value() {
        assert_eq!(to_lvgl(0.0, 100.0), 0);
    }

    #[test]
    fn to_lvgl_half() {
        assert_eq!(to_lvgl(50.0, 100.0), LVGL_SCALE / 2);
    }

    #[test]
    fn to_lvgl_full() {
        assert_eq!(to_lvgl(100.0, 100.0), LVGL_SCALE);
    }

    #[test]
    fn to_lvgl_zero_max_returns_zero() {
        assert_eq!(to_lvgl(42.0, 0.0), 0);
    }

    #[test]
    fn to_lvgl_over_range_clamped() {
        assert_eq!(to_lvgl(150.0, 100.0), LVGL_SCALE);
    }

    #[test]
    fn to_lvgl_negative_clamped() {
        assert_eq!(to_lvgl(-10.0, 100.0), 0);
    }

    // -- Grid helpers ------------------------------------------------------

    #[test]
    fn grid_template_last_is_coord_max() {
        assert_eq!(
            super::GRID_TEMPLATE_LAST,
            lvgl_rust_sys::LV_COORD_MAX as i32
        );
    }

    #[test]
    fn grid_fr_1() {
        assert_eq!(super::grid_fr(1), lvgl_rust_sys::LV_COORD_MAX as i32 - 99);
    }

    #[test]
    fn grid_fr_monotonic() {
        assert!(super::grid_fr(2) > super::grid_fr(1));
        assert!(super::grid_fr(3) > super::grid_fr(2));
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for WidgetError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            WidgetError::FormatError(fe) => {
                defmt::write!(f, "FormatError: {:?}", crate::fmt::Debug2Format(&fe))
            }
            WidgetError::LvglNullPointer => defmt::write!(f, "Got NULL pointer from LVGL"),
            WidgetError::ExtendError(ee) => {
                defmt::write!(
                    f,
                    "Could not extend C string: {:?}",
                    crate::fmt::Debug2Format(&ee)
                )
            }
        }
    }
}
