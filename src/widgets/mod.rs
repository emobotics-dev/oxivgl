// SPDX-License-Identifier: MIT OR Apache-2.0
//! Type-safe LVGL widget wrappers and supporting types.

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
mod buttonmatrix;
mod canvas;
mod chart;
mod checkbox;
mod child;
mod dropdown;
mod image;
mod keyboard;
mod label;
mod led;
mod line;
mod list;
mod menu;
mod msgbox;
mod obj;
mod obj_layout;
mod obj_style;
mod roller;
mod scale;
mod screen;
mod slider;
mod switch;
mod table;
mod textarea;
mod value_label;

pub use arc::{Arc, ArcMode};
pub use bar::{Bar, BarMode};
pub use button::Button;
pub use buttonmatrix::{Buttonmatrix, ButtonmatrixMap};
pub use canvas::{Canvas, CanvasLayer};
pub use chart::{Chart, ChartAxis, ChartSeries, ChartType};
pub use checkbox::Checkbox;
pub use child::{Child, detach};
pub use dropdown::{DdDir, Dropdown};
pub use image::{Image, ImageAlign};
pub use keyboard::{Keyboard, KeyboardMode};
pub use label::{Label, LabelLongMode};
pub use led::Led;
pub use line::Line;
pub use list::List;
// Re-export raw FFI types used in public widget APIs.
pub use lvgl_rust_sys::{lv_color_t, lv_image_dsc_t, lv_point_precise_t};
pub use menu::{Menu, MenuHeaderMode};
pub use msgbox::Msgbox;
pub use obj::{Align, AsLvHandle, BaseDir, Matrix, Obj, Part, TextAlign};
pub use roller::{Roller, RollerMode};
pub use scale::{
    SCALE_LABEL_ROTATE_KEEP_UPRIGHT, SCALE_LABEL_ROTATE_MATCH_TICKS, Scale, ScaleBuilder, ScaleLabels, ScaleMode,
    ScaleSection,
};
pub use screen::Screen;
pub use slider::{Slider, SliderMode};
pub use switch::{Switch, SwitchOrientation};
pub use table::{Table, TableCellCtrl};
pub use textarea::Textarea;
pub use value_label::ValueLabel;

/// Maximum corner radius — creates a pill/capsule shape.
/// Equivalent to LVGL's `LV_RADIUS_CIRCLE` (0x7FFF).
pub const RADIUS_MAX: i32 = 0x7FFF;

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
