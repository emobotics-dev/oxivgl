// SPDX-License-Identifier: MIT OR Apache-2.0
//! Type-safe style selector combining [`Part`](super::obj::Part) and
//! [`ObjState`](super::ObjState) bits.
//!
//! Replaces raw `u32` in all style-related methods.
//!
//! ```ignore
//! use oxivgl::widgets::{ObjState, Selector};
//! use oxivgl::widgets::obj::Part;
//!
//! btn.add_style(&style, Selector::DEFAULT);
//! btn.add_style(&style, ObjState::PRESSED);
//! slider.add_style(&style, Part::Indicator | ObjState::PRESSED);
//! ```

/// Style selector = [`Part`](super::obj::Part) + [`ObjState`](super::ObjState)
/// bits. Pass to methods like [`Obj::add_style`](super::Obj::add_style),
/// [`Obj::radius`](super::Obj::radius), etc.
#[derive(Clone, Copy, Debug, Default)]
pub struct Selector(u32);

impl Selector {
    /// Default selector (main part, default state).
    pub const DEFAULT: Self = Self(0);

    /// Raw `u32` value for passing to LVGL FFI.
    pub fn raw(self) -> u32 {
        self.0
    }
}

impl From<super::obj::Part> for Selector {
    fn from(p: super::obj::Part) -> Self {
        Self(p as u32)
    }
}

impl From<super::ObjState> for Selector {
    fn from(s: super::ObjState) -> Self {
        Self(s.0)
    }
}

impl core::ops::BitOr<super::ObjState> for super::obj::Part {
    type Output = Selector;
    fn bitor(self, rhs: super::ObjState) -> Selector {
        Selector(self as u32 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::obj::Part;
    use crate::widgets::ObjState;

    #[test]
    fn default_is_zero() {
        assert_eq!(Selector::DEFAULT.raw(), 0);
        assert_eq!(Selector::default().raw(), 0);
    }

    #[test]
    fn from_part() {
        let s: Selector = Part::Indicator.into();
        assert_eq!(s.raw(), Part::Indicator as u32);
    }

    #[test]
    fn from_state() {
        let s: Selector = ObjState::PRESSED.into();
        assert_eq!(s.raw(), ObjState::PRESSED.0);
    }

    #[test]
    fn part_bitor_state() {
        let s = Part::Indicator | ObjState::PRESSED;
        assert_eq!(s.raw(), Part::Indicator as u32 | ObjState::PRESSED.0);
    }
}
