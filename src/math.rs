// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL math utility wrappers.

use lvgl_rust_sys::*;

/// Cubic Bezier calculation with 4 control points.
///
/// `t` is the time parameter in `[0..1024]`.
/// `u0` must be 0, `u3` must be 1024 (fixed endpoints).
/// `u1` and `u2` are the control values in `[0..1024]`.
/// Returns a value in `[0..1024]`.
pub fn bezier3(t: i32, u0: i32, u1: u32, u2: i32, u3: i32) -> i32 {
    // SAFETY: lv_bezier3 is a pure math function with no side effects.
    unsafe { lv_bezier3(t, u0, u1, u2, u3) }
}

/// Linear mapping from input range to output range.
///
/// Maps `x` from `[min_in..max_in]` to `[min_out..max_out]`.
pub fn map(x: i32, min_in: i32, max_in: i32, min_out: i32, max_out: i32) -> i32 {
    // SAFETY: lv_map is a pure math function with no side effects.
    unsafe { lv_map(x, min_in, max_in, min_out, max_out) }
}

/// Maximum value for Bezier control points and time parameter (1024).
pub const BEZIER_VAL_MAX: i32 = 1024;
