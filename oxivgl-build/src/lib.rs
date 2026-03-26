// SPDX-License-Identifier: MIT OR Apache-2.0
//! Build-time helpers for oxivgl: PNG → LVGL image conversion.

use std::path::PathBuf;

/// Build-time configuration for LVGL image asset compilation.
pub struct ImageConfig {
    /// Path to directory containing `lv_conf.h`.
    pub lv_conf_dir: PathBuf,
    /// Path to LVGL header root (directory containing `lvgl.h`).
    pub lvgl_include_dir: PathBuf,
    /// Path to `LVGLImage.py` converter script.
    pub converter: PathBuf,
}

impl ImageConfig {
    /// Create config from environment.
    ///
    /// - `lv_conf_dir` from `DEP_LV_CONFIG_PATH` env var
    /// - `lvgl_include_dir` and `converter` discovered from the
    ///   `oxivgl_sys` cargo git checkout or thirdparty fallback.
    ///
    /// # Panics
    /// If `DEP_LV_CONFIG_PATH` is not set or LVGL source tree not found.
    pub fn from_env() -> Self {
        let lv_conf_dir = PathBuf::from(
            std::env::var("DEP_LV_CONFIG_PATH")
                .expect("DEP_LV_CONFIG_PATH must be set (points to dir containing lv_conf.h)"),
        );
        let lvgl_root = find_lvgl_root();
        ImageConfig {
            lv_conf_dir,
            lvgl_include_dir: lvgl_root.join("src"),
            converter: lvgl_root.join("scripts/LVGLImage.py"),
        }
    }

    /// Convert a PNG to an LVGL C image source, compile it, and link it.
    ///
    /// - `name`: C symbol name (e.g. `"cogwheel"`). Must be a valid C identifier.
    /// - `png_path`: path to PNG file, relative to `CARGO_MANIFEST_DIR`.
    ///
    /// Color format is derived from `LV_COLOR_DEPTH` in `lv_conf.h`.
    ///
    /// # Build requirements
    /// Python 3 with `pypng` and `lz4` packages.
    ///
    /// # Panics
    /// - PNG file not found
    /// - `LVGLImage.py` exits non-zero
    /// - `cc` compilation fails
    pub fn image_asset(&self, name: &str, png_path: &str) {
        let manifest_dir =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
        let png_abs = manifest_dir.join(png_path);
        assert!(
            png_abs.exists(),
            "image asset not found: {}",
            png_abs.display()
        );

        let cf = color_format_from_conf(&self.lv_conf_dir);

        // Run LVGLImage.py
        let status = std::process::Command::new("python3")
            .arg(&self.converter)
            .args(["--ofmt", "C"])
            .args(["--cf", cf])
            .args(["--align", "1"])
            .args(["--name", name])
            .args(["-o", out_dir.to_str().unwrap()])
            .arg(&png_abs)
            .status()
            .unwrap_or_else(|e| panic!("failed to run LVGLImage.py: {e}"));
        assert!(
            status.success(),
            "LVGLImage.py failed with exit code {:?}",
            status.code()
        );

        // Compile the generated .c file
        let c_file = out_dir.join(format!("{name}.c"));
        assert!(
            c_file.exists(),
            "LVGLImage.py did not produce {}",
            c_file.display()
        );

        cc::Build::new()
            .file(&c_file)
            .define("LV_LVGL_H_INCLUDE_SIMPLE", None)
            .include(&self.lvgl_include_dir)
            .include(&self.lv_conf_dir)
            .opt_level(2)
            .compile(&format!("lvgl_img_{name}"));

        println!("cargo:rerun-if-changed={png_path}");
    }
}

/// Read `LV_COLOR_DEPTH` from `lv_conf.h` and return the matching
/// Locate the LVGL source tree.
///
/// Checks (in order): sibling `oxivgl-sys/lvgl`, cargo git checkouts
/// for `oxivgl_sys-*` or `oxivgl_sys-*`, then `thirdparty/oxivgl_sys/lvgl`.
fn find_lvgl_root() -> PathBuf {
    // Primary: cargo metadata from oxivgl-sys (links = "lv")
    if let Ok(dir) = std::env::var("DEP_LV_SRC_DIR") {
        let p = PathBuf::from(dir);
        if p.join("lv_version.h").exists() {
            return p;
        }
    }

    // Fallback: sibling oxivgl-sys crate (workspace layout)
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    // Check both manifest_dir/oxivgl-sys/lvgl (when called from oxivgl root)
    // and manifest_dir/../oxivgl-sys/lvgl (when called from a sibling crate)
    for base in [
        manifest_dir.as_path(),
        manifest_dir.parent().unwrap_or(&manifest_dir),
    ] {
        let candidate = base.join("oxivgl-sys").join("lvgl");
        if candidate.join("lv_version.h").exists() {
            return candidate;
        }
    }

    let cargo_home = std::env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cargo")
        });
    let checkouts = cargo_home.join("git/checkouts");
    if let Ok(entries) = std::fs::read_dir(&checkouts) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("oxivgl_sys-") || name.starts_with("oxivgl_sys-") {
                if let Ok(revs) = std::fs::read_dir(entry.path()) {
                    for rev in revs.flatten() {
                        let candidate = rev.path().join("lvgl");
                        if candidate.join("lv_version.h").exists() {
                            return candidate;
                        }
                    }
                }
            }
        }
    }
    // Fallback: thirdparty submodule (legacy)
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let fallback = manifest_dir.join("thirdparty/oxivgl_sys/lvgl");
    if fallback.join("lv_version.h").exists() {
        return fallback;
    }
    panic!(
        "LVGL source tree not found in oxivgl-sys/lvgl/, \
         {}/git/checkouts/{{oxivgl_sys,oxivgl_sys}}-*/*/lvgl/, \
         or thirdparty/oxivgl_sys/lvgl/",
        cargo_home.display()
    );
}

/// `LVGLImage.py` `--cf` color format string.
fn color_format_from_conf(lv_conf_dir: &std::path::Path) -> &'static str {
    let conf_path = lv_conf_dir.join("lv_conf.h");
    let contents = std::fs::read_to_string(&conf_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", conf_path.display()));

    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with("#define") && line.contains("LV_COLOR_DEPTH") {
            // e.g. "#define LV_COLOR_DEPTH 16"
            if let Some(val) = line.split_whitespace().nth(2) {
                return match val {
                    "16" => "RGB565",
                    "24" => "RGB888",
                    "32" => "ARGB8888",
                    other => panic!(
                        "unsupported LV_COLOR_DEPTH {other} in {} (expected 16, 24, or 32)",
                        conf_path.display()
                    ),
                };
            }
        }
    }
    panic!("LV_COLOR_DEPTH not found in {}", conf_path.display());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_color_depth_16() {
        let dir = std::env::temp_dir().join("oxivgl_build_test_16");
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("lv_conf.h")).unwrap();
        writeln!(f, "#define LV_COLOR_DEPTH 16").unwrap();
        assert_eq!(color_format_from_conf(&dir), "RGB565");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_color_depth_32() {
        let dir = std::env::temp_dir().join("oxivgl_build_test_32");
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("lv_conf.h")).unwrap();
        writeln!(f, "#define LV_COLOR_DEPTH 32").unwrap();
        assert_eq!(color_format_from_conf(&dir), "ARGB8888");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[should_panic(expected = "unsupported LV_COLOR_DEPTH")]
    fn parse_color_depth_unsupported() {
        let dir = std::env::temp_dir().join("oxivgl_build_test_bad");
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("lv_conf.h")).unwrap();
        writeln!(f, "#define LV_COLOR_DEPTH 8").unwrap();
        color_format_from_conf(&dir);
    }
}
