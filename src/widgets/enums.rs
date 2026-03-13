// SPDX-License-Identifier: MIT OR Apache-2.0
//! Type-safe wrappers for LVGL constants (event codes, object flags, states,
//! scrollbar modes, layout types).
//!
//! Newtype structs are used for open-ended value sets (events, flags, states)
//! so that unknown LVGL values pass through safely. Proper enums are used for
//! small, exhaustive sets (scrollbar mode, layout).

/// LVGL event code. Newtype around `u32` so that unknown codes propagate
/// without UB while known codes get ergonomic named constants.
///
/// ```ignore
/// match event.code() {
///     EventCode::CLICKED => { /* … */ }
///     EventCode::PRESSED => { /* … */ }
///     _ => {}
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventCode(pub u32);

impl EventCode {
    /// Receive all event types.
    pub const ALL: Self = Self(0);
    /// Finger/pointer pressed down.
    pub const PRESSED: Self = Self(1);
    /// Long press detected.
    pub const LONG_PRESSED: Self = Self(8);
    /// Long press repeated.
    pub const LONG_PRESSED_REPEAT: Self = Self(9);
    /// Short click (press + release).
    pub const CLICKED: Self = Self(10);
    /// Value changed (sliders, switches, etc.).
    pub const VALUE_CHANGED: Self = Self(35);
}

/// LVGL object flag. Combine with `|` for multi-flag operations.
///
/// ```ignore
/// obj.add_flag(ObjFlag::CHECKABLE | ObjFlag::EVENT_BUBBLE);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjFlag(pub u32);

impl ObjFlag {
    /// Object receives click events.
    pub const CLICKABLE: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_CLICKABLE);
    /// Widget can be toggled between checked/unchecked.
    pub const CHECKABLE: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_CHECKABLE);
    /// Object can be scrolled.
    pub const SCROLLABLE: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_SCROLLABLE);
    /// Object is excluded from layout calculations.
    pub const IGNORE_LAYOUT: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_IGNORE_LAYOUT);
    /// Events bubble up to parent.
    pub const EVENT_BUBBLE: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE);
    /// Events trickle down to children.
    pub const EVENT_TRICKLE: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_TRICKLE);
}

impl core::ops::BitOr for ObjFlag {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// LVGL object state. Combine with `|` for multi-state operations.
/// Also usable as style selectors: `obj.add_style(&s, ObjState::PRESSED.0)`.
///
/// ```ignore
/// obj.add_state(ObjState::CHECKED);
/// obj.add_style(&style, ObjState::PRESSED.0);
/// obj.add_style(&style, Part::Indicator as u32 | ObjState::PRESSED.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjState(pub u32);

impl ObjState {
    /// Normal/default state.
    pub const DEFAULT: Self = Self(lvgl_rust_sys::lv_state_t_LV_STATE_DEFAULT);
    /// Toggled / checked.
    pub const CHECKED: Self = Self(lvgl_rust_sys::lv_state_t_LV_STATE_CHECKED);
    /// Focused (e.g. via encoder or keyboard).
    pub const FOCUSED: Self = Self(lvgl_rust_sys::lv_state_t_LV_STATE_FOCUSED);
    /// Currently pressed.
    pub const PRESSED: Self = Self(lvgl_rust_sys::lv_state_t_LV_STATE_PRESSED);
}

impl core::ops::BitOr for ObjState {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// LVGL opacity level (0–255).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opa(pub u8);

impl Opa {
    /// Fully transparent.
    pub const TRANSP: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_TRANSP as u8);
    /// 10% opaque.
    pub const OPA_10: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_10 as u8);
    /// 20% opaque.
    pub const OPA_20: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_20 as u8);
    /// 30% opaque.
    pub const OPA_30: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_30 as u8);
    /// 40% opaque.
    pub const OPA_40: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_40 as u8);
    /// 50% opaque.
    pub const OPA_50: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_50 as u8);
    /// 60% opaque.
    pub const OPA_60: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_60 as u8);
    /// 70% opaque.
    pub const OPA_70: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_70 as u8);
    /// 80% opaque.
    pub const OPA_80: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_80 as u8);
    /// 90% opaque.
    pub const OPA_90: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_90 as u8);
    /// Fully opaque.
    pub const COVER: Self = Self(lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_COVER as u8);
}

/// LVGL scrollbar display mode.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarMode {
    /// Never show scrollbars.
    Off = lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF,
    /// Always show scrollbars.
    On = lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_ON,
    /// Show while scrolling, hide after.
    Active = lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_ACTIVE,
    /// Show when content overflows.
    Auto = lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_AUTO,
}

/// LVGL layout engine type.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    /// Flexbox layout.
    Flex = lvgl_rust_sys::lv_layout_t_LV_LAYOUT_FLEX,
    /// Grid layout.
    Grid = lvgl_rust_sys::lv_layout_t_LV_LAYOUT_GRID,
}
