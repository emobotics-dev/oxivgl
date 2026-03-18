// SPDX-License-Identifier: GPL-3.0-only
use cmake::Config;

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("xtensa-") {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
        cmake_lvgl();
    }
    // On host targets, lvgl_rust_sys's own build.rs compiles LVGL.

    // Image assets (all targets)
    let cfg = oxivgl_build::ImageConfig::from_env();
    cfg.image_asset("img_cogwheel_argb", "examples/assets/img_cogwheel_argb.png");
    cfg.image_asset("img_skew_strip", "examples/assets/img_skew_strip.png");
}

fn cmake_lvgl() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lv_config_path = std::env::var("DEP_LV_CONFIG_PATH")
        .expect("DEP_LV_CONFIG_PATH must be set (points to dir containing lv_conf.h)");
    let target = std::env::var("TARGET").unwrap_or_default();
    let toolchain = if target.contains("esp32s3") { "toolchain-esp32s3.cmake" } else { "toolchain-esp32.cmake" };
    let dst = Config::new(format!("{}/thirdparty/lvgl_rust_sys/lvgl", manifest_dir))
        .define("CMAKE_TOOLCHAIN_FILE", format!("{}/src/{}", manifest_dir, toolchain))
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("CMAKE_VERBOSE_MAKEFILE", "ON")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("CONFIG_LV_USE_THORVG_INTERNAL", "OFF")
        .define("CONFIG_LV_BUILD_EXAMPLES", "OFF")
        .define("CONFIG_LV_BUILD_DEMOS", "OFF")
        .define("LV_BUILD_CONF_PATH", format!("{}/lv_conf.h", lv_config_path))
        .cflag(format!("-I{}", lv_config_path))
        .asmflag(format!("-I{}", lv_config_path))
        .cflag("-mlongcalls")
        .cflag("-Ofast")
        .cflag("-flto")
        .cflag("-ftree-vectorize")
        .cflag("-fno-strict-aliasing")
        .cflag("-fdata-sections")
        .cflag("-ffunction-sections")
        .profile("Release")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=lvgl");
}
