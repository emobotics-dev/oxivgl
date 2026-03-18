// SPDX-License-Identifier: MIT OR Apache-2.0
//! Type-safe wrappers for LVGL constants (event codes, object flags, states,
//! scrollbar modes, opacity, scroll direction).
//!
//! Newtype structs are used for open-ended value sets (events, flags, states)
//! so that unknown LVGL values pass through safely. Proper enums are used for
//! small, exhaustive sets (scrollbar mode).

/// LVGL event code. Newtype around `u32` so that unknown codes propagate
/// without UB while known codes get ergonomic named constants.
///
/// ```
/// use oxivgl::enums::EventCode;
///
/// fn handle(code: EventCode) {
///     match code {
///         EventCode::CLICKED => { /* … */ }
///         EventCode::PRESSED => { /* … */ }
///         _ => {}
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventCode(pub u32);

impl EventCode {
    /// Receive all event types.
    pub const ALL: Self = Self(0);
    /// Finger/pointer pressed down.
    pub const PRESSED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_PRESSED);
    /// Widget is being pressed (sent continuously while pressing).
    pub const PRESSING: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_PRESSING);
    /// Short click (press + release, not sent if scrolled).
    pub const SHORT_CLICKED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_SHORT_CLICKED);
    /// First short click within small distance and short time.
    pub const SINGLE_CLICKED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_SINGLE_CLICKED);
    /// Second short click within small distance and short time.
    pub const DOUBLE_CLICKED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_DOUBLE_CLICKED);
    /// Third short click within small distance and short time.
    pub const TRIPLE_CLICKED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_TRIPLE_CLICKED);
    /// Long press detected.
    pub const LONG_PRESSED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED);
    /// Long press repeated.
    pub const LONG_PRESSED_REPEAT: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_LONG_PRESSED_REPEAT);
    /// Short click (press + release). Alias for compatibility.
    pub const CLICKED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_CLICKED);
    /// Value changed (sliders, switches, etc.).
    pub const VALUE_CHANGED: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_VALUE_CHANGED);
    /// Object is being scrolled.
    pub const SCROLL: Self = Self(lvgl_rust_sys::lv_event_code_t_LV_EVENT_SCROLL);
}

/// LVGL object flag. Combine with `|` for multi-flag operations.
///
/// ```
/// use oxivgl::enums::ObjFlag;
///
/// let _ = ObjFlag::CHECKABLE | ObjFlag::EVENT_BUBBLE;
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
    /// Elastic (bounce-back) scrolling.
    pub const SCROLL_ELASTIC: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_SCROLL_ELASTIC);
    /// Scroll only one snap-child at a time.
    pub const SCROLL_ONE: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_SCROLL_ONE);
    /// Child is a snap target for its parent's scroll snap.
    pub const SNAPPABLE: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_SNAPPABLE);
    /// Floating position — not affected by scroll or layout.
    pub const FLOATING: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_FLOATING);
    /// Keep pressed state when leaving the widget.
    pub const PRESS_LOCK: Self = Self(lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_PRESS_LOCK);
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
/// ```no_run
/// use oxivgl::enums::ObjState;
/// use oxivgl::widgets::{Part, Screen};
/// use oxivgl::style::{Selector, StyleBuilder};
///
/// let screen = Screen::active().unwrap();
/// let obj = oxivgl::widgets::Obj::new(&screen).unwrap();
/// let style = StyleBuilder::new().build();
/// obj.add_state(ObjState::CHECKED);
/// obj.add_style(&style, ObjState::PRESSED);
/// obj.add_style(&style, Part::Indicator | ObjState::PRESSED);
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
    /// Disabled (greyed out, not interactable).
    pub const DISABLED: Self = Self(lvgl_rust_sys::lv_state_t_LV_STATE_DISABLED);
    /// Currently being scrolled.
    pub const SCROLLED: Self = Self(lvgl_rust_sys::lv_state_t_LV_STATE_SCROLLED);
    /// Wildcard — matches any state.
    pub const ANY: Self = Self(lvgl_rust_sys::lv_state_t_LV_STATE_ANY);
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

/// LVGL scroll snap alignment.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollSnap {
    /// No snap alignment.
    None = lvgl_rust_sys::lv_scroll_snap_t_LV_SCROLL_SNAP_NONE,
    /// Snap to start (left/top).
    Start = lvgl_rust_sys::lv_scroll_snap_t_LV_SCROLL_SNAP_START,
    /// Snap to end (right/bottom).
    End = lvgl_rust_sys::lv_scroll_snap_t_LV_SCROLL_SNAP_END,
    /// Snap to center.
    Center = lvgl_rust_sys::lv_scroll_snap_t_LV_SCROLL_SNAP_CENTER,
}

/// LVGL scroll direction flags. Combine with `|`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollDir(pub u32);

impl ScrollDir {
    /// No direction.
    pub const NONE: Self = Self(lvgl_rust_sys::lv_dir_t_LV_DIR_NONE);
    /// Horizontal (left + right).
    pub const HOR: Self = Self(lvgl_rust_sys::lv_dir_t_LV_DIR_HOR);
    /// Vertical (top + bottom).
    pub const VER: Self = Self(lvgl_rust_sys::lv_dir_t_LV_DIR_VER);
    /// All directions.
    pub const ALL: Self = Self(lvgl_rust_sys::lv_dir_t_LV_DIR_ALL);
}

impl core::ops::BitOr for ScrollDir {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- EventCode ---------------------------------------------------------

    #[test]
    fn event_code_known_values() {
        assert_eq!(EventCode::ALL.0, 0);
        assert_eq!(EventCode::PRESSED.0, 1);
        assert_eq!(EventCode::LONG_PRESSED.0, 8);
        assert_eq!(EventCode::LONG_PRESSED_REPEAT.0, 9);
        assert_eq!(EventCode::CLICKED.0, 10);
        assert_eq!(EventCode::VALUE_CHANGED.0, 35);
    }

    #[test]
    fn event_code_equality() {
        assert_eq!(EventCode::CLICKED, EventCode::CLICKED);
        assert_ne!(EventCode::CLICKED, EventCode::PRESSED);
    }

    #[test]
    fn event_code_unknown_value_roundtrips() {
        let custom = EventCode(999);
        assert_eq!(custom.0, 999);
        assert_ne!(custom, EventCode::ALL);
    }

    // -- ObjFlag -----------------------------------------------------------

    #[test]
    fn obj_flag_values_match_bindings() {
        assert_eq!(
            ObjFlag::CLICKABLE.0,
            lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_CLICKABLE
        );
        assert_eq!(
            ObjFlag::CHECKABLE.0,
            lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_CHECKABLE
        );
        assert_eq!(
            ObjFlag::SCROLLABLE.0,
            lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_SCROLLABLE
        );
        assert_eq!(
            ObjFlag::IGNORE_LAYOUT.0,
            lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_IGNORE_LAYOUT
        );
        assert_eq!(
            ObjFlag::EVENT_BUBBLE.0,
            lvgl_rust_sys::lv_obj_flag_t_LV_OBJ_FLAG_EVENT_BUBBLE
        );
    }

    #[test]
    fn obj_flag_bitor_combines_bits() {
        let combined = ObjFlag::CLICKABLE | ObjFlag::CHECKABLE;
        assert_eq!(combined.0, ObjFlag::CLICKABLE.0 | ObjFlag::CHECKABLE.0);
    }

    #[test]
    fn obj_flag_bitor_idempotent() {
        let flag = ObjFlag::SCROLLABLE | ObjFlag::SCROLLABLE;
        assert_eq!(flag, ObjFlag::SCROLLABLE);
    }

    // -- ObjState ----------------------------------------------------------

    #[test]
    fn obj_state_values_match_bindings() {
        assert_eq!(
            ObjState::DEFAULT.0,
            lvgl_rust_sys::lv_state_t_LV_STATE_DEFAULT
        );
        assert_eq!(
            ObjState::CHECKED.0,
            lvgl_rust_sys::lv_state_t_LV_STATE_CHECKED
        );
        assert_eq!(
            ObjState::FOCUSED.0,
            lvgl_rust_sys::lv_state_t_LV_STATE_FOCUSED
        );
        assert_eq!(
            ObjState::PRESSED.0,
            lvgl_rust_sys::lv_state_t_LV_STATE_PRESSED
        );
    }

    #[test]
    fn obj_state_pressed_is_not_0x20() {
        // Regression: was hardcoded as 0x20 (32), correct value is 0x80 (128).
        assert_ne!(ObjState::PRESSED.0, 0x20);
        assert_eq!(
            ObjState::PRESSED.0,
            lvgl_rust_sys::lv_state_t_LV_STATE_PRESSED
        );
    }

    #[test]
    fn obj_state_bitor_combines() {
        let combined = ObjState::CHECKED | ObjState::PRESSED;
        assert_eq!(combined.0, ObjState::CHECKED.0 | ObjState::PRESSED.0);
    }

    #[test]
    fn obj_state_default_is_zero() {
        assert_eq!(ObjState::DEFAULT.0, 0);
    }

    // -- ScrollbarMode -----------------------------------------------------

    #[test]
    fn scrollbar_mode_discriminants() {
        assert_eq!(
            ScrollbarMode::Off as u32,
            lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF
        );
        assert_eq!(
            ScrollbarMode::On as u32,
            lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_ON
        );
        assert_eq!(
            ScrollbarMode::Active as u32,
            lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_ACTIVE
        );
        assert_eq!(
            ScrollbarMode::Auto as u32,
            lvgl_rust_sys::lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_AUTO
        );
    }

    // -- Opa ---------------------------------------------------------------

    #[test]
    fn opa_values_match_bindings() {
        assert_eq!(
            Opa::TRANSP.0,
            lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_TRANSP as u8
        );
        assert_eq!(
            Opa::OPA_20.0,
            lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_20 as u8
        );
        assert_eq!(
            Opa::OPA_50.0,
            lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_50 as u8
        );
        assert_eq!(
            Opa::COVER.0,
            lvgl_rust_sys::_lv_opacity_level_t_LV_OPA_COVER as u8
        );
    }

    #[test]
    fn opa_transp_is_zero() {
        assert_eq!(Opa::TRANSP.0, 0);
    }

    #[test]
    fn opa_cover_is_255() {
        assert_eq!(Opa::COVER.0, 255);
    }

    #[test]
    fn opa_monotonic() {
        assert!(Opa::OPA_10.0 < Opa::OPA_20.0);
        assert!(Opa::OPA_50.0 < Opa::OPA_90.0);
        assert!(Opa::OPA_90.0 < Opa::COVER.0);
    }
}
