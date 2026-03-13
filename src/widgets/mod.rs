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

mod arc;
mod bar;
mod button;
mod child;
mod image;
mod label;
mod led;
mod line;
mod obj;
mod palette;
mod style;
mod scale;
mod slider;
mod value_label;

pub use arc::Arc;
pub use bar::Bar;
pub use button::Button;
pub use child::{detach, Child};
pub use image::Image;
pub use label::Label;
pub use led::Led;
pub use line::Line;
pub use obj::{Align, AsLvHandle, Obj, Part, Screen, TextAlign};
pub use palette::{
    color_black, color_make, color_white, palette_darken, palette_lighten, palette_main, GradDir,
    Palette,
};
pub use style::{
    LV_STATE_PRESSED, LV_SIZE_CONTENT, BorderSide, ColorFilter, Style, TextDecor, TransitionDsc,
    anim_path_linear, darken_filter_cb, lv_pct, props,
};
pub use scale::{Scale, ScaleMode};
pub use slider::Slider;
pub use value_label::ValueLabel;

// Re-export raw event types so example callbacks don't need `lvgl_rust_sys`.
pub use lvgl_rust_sys::{lv_event_t, lv_event_code_t};
/// `LV_EVENT_VALUE_CHANGED` — fired by sliders, dropdowns, etc.
pub const LV_EVENT_VALUE_CHANGED: lv_event_code_t =
    lvgl_rust_sys::lv_event_code_t_LV_EVENT_VALUE_CHANGED;

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
    use super::{LVGL_SCALE, to_lvgl};

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
                defmt::write!(f, "Could not extend C string: {:?}", crate::fmt::Debug2Format(&ee))
            }
        }
    }
}
