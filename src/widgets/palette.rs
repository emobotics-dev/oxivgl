// SPDX-License-Identifier: MIT OR Apache-2.0
use lvgl_rust_sys::*;

/// LVGL material design color palette (`lv_palette_t`).
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum Palette {
    Red = 0,
    Pink = 1,
    Purple = 2,
    DeepPurple = 3,
    Indigo = 4,
    Blue = 5,
    LightBlue = 6,
    Cyan = 7,
    Teal = 8,
    Green = 9,
    LightGreen = 10,
    Lime = 11,
    Yellow = 12,
    Amber = 13,
    Orange = 14,
    DeepOrange = 15,
    Brown = 16,
    BlueGrey = 17,
    Grey = 18,
}

/// Gradient direction (`lv_grad_dir_t`).
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum GradDir {
    None = 0,
    Ver = 1,
    Hor = 2,
    Linear = 3,
    Radial = 4,
    Conical = 5,
}

/// Returns the main (500-shade) color for a palette entry as a raw `lv_color_t`.
pub fn palette_main(p: Palette) -> lv_color_t {
    // SAFETY: lv_palette_main is a pure lookup function.
    unsafe { lv_palette_main(p as lv_palette_t) }
}

/// Returns a lightened shade of a palette color.
/// `level` is 1–5 (1 = lightest, 5 = darkest light variant).
pub fn palette_lighten(p: Palette, level: u8) -> lv_color_t {
    // SAFETY: pure lookup.
    unsafe { lv_palette_lighten(p as lv_palette_t, level) }
}

#[cfg(test)]
mod tests {
    use super::{GradDir, Palette};

    #[test]
    fn palette_discriminants() {
        assert_eq!(Palette::Red as u32, 0);
        assert_eq!(Palette::Grey as u32, 18);
    }

    #[test]
    fn grad_dir_discriminants() {
        assert_eq!(GradDir::None as u32, 0);
        assert_eq!(GradDir::Ver as u32, 1);
        assert_eq!(GradDir::Hor as u32, 2);
        assert_eq!(GradDir::Linear as u32, 3);
        assert_eq!(GradDir::Radial as u32, 4);
        assert_eq!(GradDir::Conical as u32, 5);
    }
}
