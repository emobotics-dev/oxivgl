// SPDX-License-Identifier: GPL-3.0-only
use cmake::Config;
use std::path::{Path, PathBuf};

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("xtensa-") {
        println!("cargo:rustc-link-arg=-Tlinkall.x");
        cmake_lvgl();
    } else {
        // On host targets, lvgl_rust_sys's own build.rs compiles C files.
        // We additionally compile ThorVG C++ sources (required by Lottie widget).
        compile_thorvg_host();
    }

    // Image assets (all targets)
    let cfg = oxivgl_build::ImageConfig::from_env();
    cfg.image_asset("img_cogwheel_argb", "examples/assets/img_cogwheel_argb.png");
    cfg.image_asset("img_skew_strip", "examples/assets/img_skew_strip.png");
}

fn compile_thorvg_host() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lv_config_path = std::env::var("DEP_LV_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&manifest_dir).join("conf"));
    let lvgl_src = PathBuf::from(&manifest_dir).join("thirdparty/lvgl_rust_sys/lvgl/src");
    let vendor = PathBuf::from(&manifest_dir).join("thirdparty/lvgl_rust_sys/vendor");

    if !lvgl_src.exists() {
        return; // submodule not initialised
    }

    let mut cfg = cc::Build::new();
    cfg.cpp(true).warnings(false);
    add_cpp_files(&mut cfg, &lvgl_src);
    cfg.define("LV_CONF_INCLUDE_SIMPLE", Some("1"))
        .include(&lvgl_src)
        .include(&vendor)
        .include(&lv_config_path);
    cfg.compile("lvgl_thorvg");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
}

fn add_cpp_files(build: &mut cc::Build, path: &Path) {
    if !path.exists() {
        return;
    }
    for e in path.read_dir().unwrap() {
        let e = e.unwrap();
        let p = e.path();
        if e.file_type().unwrap().is_dir() {
            add_cpp_files(build, &p);
        } else if p.extension().and_then(|s| s.to_str()) == Some("cpp") {
            build.file(&p);
        }
    }
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
        // Disable ThorVG-dependent features for embedded — they require C++ ThorVG
        // which is not compiled for xtensa. lv_conf.h enables them for host; these
        // cflags override the macros so the embedded C files don't call tvg_* symbols.
        .cflag("-DLV_USE_THORVG_INTERNAL=0")
        .cflag("-DLV_USE_VECTOR_GRAPHIC=0")
        .cflag("-DLV_USE_LOTTIE=0")
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
