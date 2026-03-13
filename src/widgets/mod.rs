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
mod child;
mod grad;
mod image;
mod label;
mod led;
mod line;
mod obj;
mod palette;
mod scale;
mod slider;
mod style;
mod switch;
mod value_label;

pub use anim::{
    anim_path_bounce, anim_path_ease_in, anim_path_ease_in_out, anim_path_ease_out,
    anim_path_linear, anim_path_overshoot, anim_set_height, anim_set_pad_column, anim_set_pad_row,
    anim_set_size, anim_set_slider_value, anim_set_width, anim_set_x, Anim, ANIM_REPEAT_INFINITE,
};
pub use anim_timeline::{AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX};
pub use arc::Arc;
pub use bar::Bar;
pub use button::Button;
pub use child::{detach, Child};
pub use grad::{GradDsc, GradExtend};
pub use image::Image;
pub use label::Label;
pub use led::Led;
pub use line::Line;
pub use obj::{
    Align, AsLvHandle, BaseDir, FlexAlign, FlexFlow, GridAlign, Obj, Part, Screen, TextAlign,
};
pub use palette::{
    color_black, color_make, color_white, palette_darken, palette_lighten, palette_main, GradDir,
    Palette,
};
pub use scale::{Scale, ScaleMode};
pub use slider::Slider;
pub use style::{
    darken_filter_cb, lv_pct, props, BorderSide, ColorFilter, Style, TextDecor, TransitionDsc,
    LV_SIZE_CONTENT, LV_STATE_PRESSED,
};
pub use switch::Switch;
pub use value_label::ValueLabel;

// Re-export raw event types so example callbacks don't need `lvgl_rust_sys`.
pub use lvgl_rust_sys::{lv_color_t, lv_event_code_t, lv_event_t, lv_point_precise_t};
/// `LV_EVENT_VALUE_CHANGED` — fired by sliders, dropdowns, etc.
pub const LV_EVENT_VALUE_CHANGED: lv_event_code_t =
    lvgl_rust_sys::lv_event_code_t_LV_EVENT_VALUE_CHANGED;
/// `LV_EVENT_CLICKED`
pub const LV_EVENT_CLICKED: lv_event_code_t = lvgl_rust_sys::lv_event_code_t_LV_EVENT_CLICKED;
/// `LV_EVENT_ALL` — receive all event types.
pub const LV_EVENT_ALL: lv_event_code_t = lvgl_rust_sys::lv_event_code_t_LV_EVENT_ALL;
/// `LV_EVENT_PRESSED`
pub const LV_EVENT_PRESSED: lv_event_code_t = lvgl_rust_sys::lv_event_code_t_LV_EVENT_PRESSED;
/// `LV_EVENT_LONG_PRESSED`
pub const LV_EVENT_LONG_PRESSED: lv_event_code_t =
    lvgl_rust_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED;
/// `LV_EVENT_LONG_PRESSED_REPEAT`
pub const LV_EVENT_LONG_PRESSED_REPEAT: lv_event_code_t =
    lvgl_rust_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT;

// Object flags
pub const LV_OBJ_FLAG_CHECKABLE: lvgl_rust_sys::lv_obj_flag_t =
    lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_CHECKABLE;
pub const LV_OBJ_FLAG_IGNORE_LAYOUT: lvgl_rust_sys::lv_obj_flag_t =
    lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_IGNORE_LAYOUT;
pub const LV_OBJ_FLAG_EVENT_BUBBLE: lvgl_rust_sys::lv_obj_flag_t =
    lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE;
pub const LV_OBJ_FLAG_EVENT_TRICKLE: lvgl_rust_sys::lv_obj_flag_t =
    lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_TRICKLE;

// Object states
pub const LV_STATE_CHECKED: lvgl_rust_sys::lv_state_t = lvgl_rust_sys::lv_state_t_LV_STATE_CHECKED;
pub const LV_STATE_FOCUSED: lvgl_rust_sys::lv_state_t = lvgl_rust_sys::lv_state_t_LV_STATE_FOCUSED;

// Scrollbar modes
pub const LV_SCROLLBAR_MODE_OFF: lvgl_rust_sys::lv_scrollbar_mode_t =
    lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF;

// Layout types
pub const LV_LAYOUT_FLEX: u32 = lvgl_rust_sys::lv_layout_t_LV_LAYOUT_FLEX;
pub const LV_LAYOUT_GRID: u32 = lvgl_rust_sys::lv_layout_t_LV_LAYOUT_GRID;

// Grid helpers
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
