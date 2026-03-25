// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("xtensa-") {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
    }
    // All targets: lvgl_rust_sys's build.rs (cc crate) compiles LVGL
    // (including lv_calendar_chinese.c) via recursive add_c_files().

    // Image assets (all targets)
    let cfg = oxivgl_build::ImageConfig::from_env();
    cfg.image_asset("img_cogwheel_argb", "examples/assets/img_cogwheel_argb.png");
    cfg.image_asset("img_skew_strip", "examples/assets/img_skew_strip.png");
    cfg.image_asset("img_star", "examples/assets/img_star.png");
}
