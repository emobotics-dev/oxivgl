// SPDX-License-Identifier: MIT OR Apache-2.0
use lvgl_rust_sys::*;

use super::palette::GradDir;

/// Text decoration flags. Combine with `|`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextDecor(u32);

impl TextDecor {
    /// No decoration.
    pub const NONE: Self = Self(0x00);
    /// Underline text.
    pub const UNDERLINE: Self = Self(0x01);
    /// Strikethrough text.
    pub const STRIKETHROUGH: Self = Self(0x02);
}

impl core::ops::BitOr for TextDecor {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Wraps `lv_style_transition_dsc_t`.
///
/// The `props` slice and `path_cb` must outlive this descriptor.
/// Store as a field alongside the [`Style`] that references it.
pub struct TransitionDsc {
    pub(crate) inner: lv_style_transition_dsc_t,
}

impl TransitionDsc {
    /// Create a transition descriptor.
    ///
    /// `props`: null-terminated array of `lv_style_prop_t` (use [`props`] constants).
    /// `path_cb`: animation path function (e.g. [`super::anim_path_linear`]).
    /// `time`: transition duration in ms.
    /// `delay`: delay before transition starts in ms.
    pub fn new(
        props: &'static [lv_style_prop_t],
        path_cb: Option<unsafe extern "C" fn(*const lv_anim_t) -> i32>,
        time: u32,
        delay: u32,
    ) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_style_transition_dsc_t>() };
        unsafe {
            lv_style_transition_dsc_init(
                &mut inner,
                props.as_ptr(),
                path_cb,
                time,
                delay,
                core::ptr::null_mut(),
            )
        };
        Self { inner }
    }
}

/// Commonly used style property constants (cast to `lv_style_prop_t`).
pub mod props {
    pub use lvgl_rust_sys::lv_style_prop_t;

    /// Background color property.
    pub const BG_COLOR: lv_style_prop_t =
        lvgl_rust_sys::_lv_style_id_t_LV_STYLE_BG_COLOR as lv_style_prop_t;
    /// Border color property.
    pub const BORDER_COLOR: lv_style_prop_t =
        lvgl_rust_sys::_lv_style_id_t_LV_STYLE_BORDER_COLOR as lv_style_prop_t;
    /// Border width property.
    pub const BORDER_WIDTH: lv_style_prop_t =
        lvgl_rust_sys::_lv_style_id_t_LV_STYLE_BORDER_WIDTH as lv_style_prop_t;
}

/// Bitflags for border side selection. Combine with `|` operator.
///
/// ```ignore
/// let sides = BorderSide::BOTTOM | BorderSide::RIGHT;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderSide(u32);

impl BorderSide {
    /// No border.
    pub const NONE: Self = Self(0x00);
    /// Bottom border.
    pub const BOTTOM: Self = Self(0x01);
    /// Top border.
    pub const TOP: Self = Self(0x02);
    /// Left border.
    pub const LEFT: Self = Self(0x04);
    /// Right border.
    pub const RIGHT: Self = Self(0x08);
    /// All four sides.
    pub const FULL: Self = Self(0x0F);
}

impl core::ops::BitOr for BorderSide {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Return an LVGL percentage value. Wraps `lv_pct()`.
pub fn lv_pct(v: i32) -> i32 {
    // SAFETY: lv_pct is a pure arithmetic function.
    unsafe { lvgl_rust_sys::lv_pct(v) }
}

/// Special LVGL size value: object sizes itself to fit its content.
/// Equivalent to the C macro `LV_SIZE_CONTENT`.
pub const LV_SIZE_CONTENT: i32 =
    (lvgl_rust_sys::LV_COORD_MAX | lvgl_rust_sys::LV_COORD_TYPE_SPEC) as i32;

/// Owned LVGL style. Wraps `lv_style_t`.
///
/// # Lifetime contract
/// Once passed to [`super::Obj::add_style`], this struct MUST NOT be moved or dropped
/// while any widget holds a reference to it. Store styles as fields in a `View`
/// struct that lives for the entire LVGL lifetime.
pub struct Style {
    pub(crate) inner: lv_style_t,
}

impl Style {
    /// Create a new empty style.
    pub fn new() -> Self {
        // SAFETY: lv_style_t can be zero-initialized; lv_style_init sets it up.
        let mut inner = unsafe { core::mem::zeroed::<lv_style_t>() };
        unsafe { lv_style_init(&mut inner) };
        Self { inner }
    }

    /// Create a heap-allocated style. Convenience for views that store
    /// styles as `Box<Style>` to satisfy LVGL's lifetime requirements.
    pub fn boxed() -> alloc::boxed::Box<Self> {
        alloc::boxed::Box::new(Self::new())
    }

    /// Set corner radius.
    pub fn radius(&mut self, r: i16) -> &mut Self {
        unsafe { lv_style_set_radius(&mut self.inner, r as lv_coord_t) };
        self
    }

    /// Set background opacity (0-255).
    pub fn bg_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_bg_opa(&mut self.inner, opa as lv_opa_t) };
        self
    }

    /// Set background color.
    pub fn bg_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_color(&mut self.inner, color) };
        self
    }

    /// Set background color from RGB hex.
    pub fn bg_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.bg_color(color)
    }

    /// Set background gradient end color.
    pub fn bg_grad_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_bg_grad_color(&mut self.inner, color) };
        self
    }

    /// Set background gradient direction.
    pub fn bg_grad_dir(&mut self, dir: GradDir) -> &mut Self {
        unsafe { lv_style_set_bg_grad_dir(&mut self.inner, dir as lv_grad_dir_t) };
        self
    }

    /// Set border color.
    pub fn border_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_border_color(&mut self.inner, color) };
        self
    }

    /// Set border color from RGB hex.
    pub fn border_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.border_color(color)
    }

    /// Set border opacity (0-255).
    pub fn border_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_border_opa(&mut self.inner, opa as lv_opa_t) };
        self
    }

    /// Set border width in pixels.
    pub fn border_width(&mut self, w: i16) -> &mut Self {
        unsafe { lv_style_set_border_width(&mut self.inner, w as lv_coord_t) };
        self
    }

    /// Set text color.
    pub fn text_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_text_color(&mut self.inner, color) };
        self
    }

    /// Set text color from RGB hex.
    pub fn text_color_hex(&mut self, hex: u32) -> &mut Self {
        let color = unsafe { lv_color_hex(hex) };
        self.text_color(color)
    }

    /// Apply a color filter with given opacity.
    pub fn color_filter(&mut self, filter: &ColorFilter, opa: u8) -> &mut Self {
        unsafe {
            lv_style_set_color_filter_dsc(&mut self.inner, &filter.inner);
            lv_style_set_color_filter_opa(&mut self.inner, opa as lv_opa_t);
        }
        self
    }

    /// Set style width.
    pub fn width(&mut self, w: i32) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_width(&mut self.inner, w) };
        self
    }

    /// Set style height.
    pub fn height(&mut self, h: i32) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_height(&mut self.inner, h) };
        self
    }

    /// Set style X offset.
    pub fn x(&mut self, x: i32) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_x(&mut self.inner, x) };
        self
    }

    /// Set style Y offset.
    pub fn y(&mut self, y: i32) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_y(&mut self.inner, y) };
        self
    }

    /// Set vertical padding (top + bottom).
    pub fn pad_ver(&mut self, p: i32) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_pad_ver(&mut self.inner, p) };
        self
    }

    /// Set left padding.
    pub fn pad_left(&mut self, p: i32) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_pad_left(&mut self.inner, p) };
        self
    }

    /// Set background gradient from a full gradient descriptor.
    ///
    /// For simple two-color gradients, prefer [`bg_grad_color`](Self::bg_grad_color)
    /// + [`bg_grad_dir`](Self::bg_grad_dir).
    pub fn bg_grad(&mut self, grad: &super::grad::GradDsc) -> &mut Self {
        // SAFETY: inner was initialized; grad is a valid descriptor reference.
        unsafe { lv_style_set_bg_grad(&mut self.inner, &grad.inner) };
        self
    }

    /// Set which border sides to draw.
    pub fn border_side(&mut self, side: BorderSide) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_border_side(&mut self.inner, side.0 as lv_border_side_t) };
        self
    }

    /// Set outline width.
    pub fn outline_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_outline_width(&mut self.inner, w) };
        self
    }

    /// Set outline color.
    pub fn outline_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_outline_color(&mut self.inner, color) };
        self
    }

    /// Set outline padding (gap from border).
    pub fn outline_pad(&mut self, pad: i32) -> &mut Self {
        unsafe { lv_style_set_outline_pad(&mut self.inner, pad) };
        self
    }

    /// Set shadow width.
    pub fn shadow_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_width(&mut self.inner, w) };
        self
    }

    /// Set shadow color.
    pub fn shadow_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_shadow_color(&mut self.inner, color) };
        self
    }

    /// Set arc line color.
    pub fn arc_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_arc_color(&mut self.inner, color) };
        self
    }

    /// Set arc line width.
    pub fn arc_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_arc_width(&mut self.inner, w) };
        self
    }

    /// Set padding on all sides.
    pub fn pad_all(&mut self, p: i32) -> &mut Self {
        unsafe { lv_style_set_pad_all(&mut self.inner, p) };
        self
    }

    /// Set letter spacing.
    pub fn text_letter_space(&mut self, s: i32) -> &mut Self {
        unsafe { lv_style_set_text_letter_space(&mut self.inner, s) };
        self
    }

    /// Set line spacing.
    pub fn text_line_space(&mut self, s: i32) -> &mut Self {
        unsafe { lv_style_set_text_line_space(&mut self.inner, s) };
        self
    }

    /// Set text decoration (underline, strikethrough).
    pub fn text_decor(&mut self, decor: TextDecor) -> &mut Self {
        unsafe { lv_style_set_text_decor(&mut self.inner, decor.0 as lv_text_decor_t) };
        self
    }

    /// Set line color.
    pub fn line_color(&mut self, color: lv_color_t) -> &mut Self {
        unsafe { lv_style_set_line_color(&mut self.inner, color) };
        self
    }

    /// Set line width.
    pub fn line_width(&mut self, w: i32) -> &mut Self {
        unsafe { lv_style_set_line_width(&mut self.inner, w) };
        self
    }

    /// Enable/disable rounded line endings.
    pub fn line_rounded(&mut self, rounded: bool) -> &mut Self {
        unsafe { lv_style_set_line_rounded(&mut self.inner, rounded) };
        self
    }

    /// Set transition descriptor for animated property changes.
    pub fn transition(&mut self, tr: &TransitionDsc) -> &mut Self {
        unsafe { lv_style_set_transition(&mut self.inner, &tr.inner) };
        self
    }

    /// Set shadow X offset.
    pub fn shadow_offset_x(&mut self, x: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_offset_x(&mut self.inner, x) };
        self
    }

    /// Set shadow Y offset.
    pub fn shadow_offset_y(&mut self, y: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_offset_y(&mut self.inner, y) };
        self
    }

    /// Set shadow opacity (0-255).
    pub fn shadow_opa(&mut self, opa: u8) -> &mut Self {
        unsafe { lv_style_set_shadow_opa(&mut self.inner, opa as lv_opa_t) };
        self
    }

    /// Set shadow spread (extra size).
    pub fn shadow_spread(&mut self, s: i32) -> &mut Self {
        unsafe { lv_style_set_shadow_spread(&mut self.inner, s) };
        self
    }

    /// Set flex layout flow direction.
    pub fn flex_flow(&mut self, flow: super::obj::FlexFlow) -> &mut Self {
        unsafe { lv_style_set_flex_flow(&mut self.inner, flow as lv_flex_flow_t) };
        self
    }

    /// Set flex main-axis alignment.
    pub fn flex_main_place(&mut self, align: super::obj::FlexAlign) -> &mut Self {
        unsafe { lv_style_set_flex_main_place(&mut self.inner, align as lv_flex_align_t) };
        self
    }

    /// Set layout engine (flex or grid).
    pub fn layout(&mut self, layout: super::Layout) -> &mut Self {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_set_layout(&mut self.inner, layout as u16) };
        self
    }
}

impl Drop for Style {
    fn drop(&mut self) {
        // SAFETY: inner was initialized by lv_style_init.
        unsafe { lv_style_reset(&mut self.inner) };
    }
}

/// Wraps `lv_color_filter_dsc_t` with a C callback function pointer.
pub struct ColorFilter {
    pub(crate) inner: lv_color_filter_dsc_t,
}

impl ColorFilter {
    /// `lv_color_filter_dsc_init` is not available in bindings; set field directly.
    pub fn new(
        cb: unsafe extern "C" fn(*const lv_color_filter_dsc_t, lv_color_t, lv_opa_t) -> lv_color_t,
    ) -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_color_filter_dsc_t>() };
        inner.filter_cb = Some(cb);
        Self { inner }
    }
}

/// Standard "darken" color filter callback — pass to [`ColorFilter::new`].
pub unsafe extern "C" fn darken_filter_cb(
    _dsc: *const lv_color_filter_dsc_t,
    color: lv_color_t,
    opa: lv_opa_t,
) -> lv_color_t {
    // SAFETY: lv_color_darken is a pure color computation.
    unsafe { lv_color_darken(color, opa) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- TextDecor ---------------------------------------------------------

    #[test]
    fn text_decor_values() {
        assert_eq!(TextDecor::NONE.0, 0);
        assert_eq!(TextDecor::UNDERLINE.0, 1);
        assert_eq!(TextDecor::STRIKETHROUGH.0, 2);
    }

    #[test]
    fn text_decor_bitor() {
        let combined = TextDecor::UNDERLINE | TextDecor::STRIKETHROUGH;
        assert_eq!(combined.0, 0x03);
    }

    // -- BorderSide --------------------------------------------------------

    #[test]
    fn border_side_values() {
        assert_eq!(BorderSide::NONE.0, 0x00);
        assert_eq!(BorderSide::BOTTOM.0, 0x01);
        assert_eq!(BorderSide::TOP.0, 0x02);
        assert_eq!(BorderSide::LEFT.0, 0x04);
        assert_eq!(BorderSide::RIGHT.0, 0x08);
        assert_eq!(BorderSide::FULL.0, 0x0F);
    }

    #[test]
    fn border_side_bitor() {
        let combined = BorderSide::TOP | BorderSide::BOTTOM;
        assert_eq!(combined.0, 0x03);
    }

    #[test]
    fn border_side_full_is_all_sides() {
        let all = BorderSide::BOTTOM | BorderSide::TOP | BorderSide::LEFT | BorderSide::RIGHT;
        assert_eq!(all, BorderSide::FULL);
    }

    // -- lv_pct ------------------------------------------------------------

    #[test]
    fn lv_pct_monotonic() {
        assert!(lv_pct(50) > lv_pct(0));
        assert!(lv_pct(100) > lv_pct(50));
    }

    #[test]
    fn lv_pct_difference_matches_input() {
        // lv_pct(x) = LV_PCT_BASE + x, so difference should equal input difference.
        assert_eq!(lv_pct(100) - lv_pct(0), 100);
        assert_eq!(lv_pct(50) - lv_pct(0), 50);
    }

    // -- LV_SIZE_CONTENT ---------------------------------------------------

    #[test]
    fn size_content_uses_spec_type() {
        // LV_SIZE_CONTENT = LV_COORD_MAX | LV_COORD_TYPE_SPEC
        let expected = (lvgl_rust_sys::LV_COORD_MAX | lvgl_rust_sys::LV_COORD_TYPE_SPEC) as i32;
        assert_eq!(LV_SIZE_CONTENT, expected);
    }
}
