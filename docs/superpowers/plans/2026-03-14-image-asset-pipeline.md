# Image Asset Pipeline Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build-time PNG→LVGL image conversion pipeline via `oxivgl-build` crate + `image_declare!` macro + `Image::set_src()`.

**Architecture:** `oxivgl-build` crate provides `ImageConfig::image_asset()` called from `build.rs`. It runs `LVGLImage.py` to convert PNG→C, then `cc` to compile+link. Rust code uses `image_declare!` macro for extern declaration and `Image::set_src()` to bind at runtime.

**Tech Stack:** Rust (no_std/std), `cc` crate, Python 3 (`LVGLImage.py`, `pypng`, `lz4`), LVGL v9.3.0

**Spec:** `docs/superpowers/specs/2026-03-14-image-asset-pipeline-design.md`

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `oxivgl-build/Cargo.toml` | Create | Crate manifest — depends on `cc` |
| `oxivgl-build/src/lib.rs` | Create | `ImageConfig` struct + `image_asset()` |
| `src/widgets/mod.rs` | Modify | Re-export `lv_image_dsc_t` |
| `src/widgets/image.rs` | Modify | Add `set_src()` method |
| `src/lib.rs` | Modify | Add `image_declare!` macro |
| `build.rs` | Modify | Add `oxivgl-build` calls for example assets |
| `Cargo.toml` | Modify | Add `oxivgl-build` build-dependency |
| `examples/assets/` | Create dir | PNG sources for examples |
| `examples/image1.rs` | Create | First image example (validates pipeline) |

---

## Chunk 1: oxivgl-build crate

### Task 1: Create `oxivgl-build` crate scaffold

**Files:**
- Create: `oxivgl-build/Cargo.toml`
- Create: `oxivgl-build/src/lib.rs`

- [ ] **Step 1: Create `oxivgl-build/Cargo.toml`**

```toml
# SPDX-License-Identifier: MIT OR Apache-2.0
[package]
name = "oxivgl-build"
edition = "2021"
version = "0.1.0"
license = "MIT OR Apache-2.0"
description = "Build-time helpers for oxivgl (LVGL image asset conversion)"

[dependencies]
cc = "1"
```

Note: edition `2021` (not `2024`) — build-dependency crates should use stable edition for maximum compatibility.

- [ ] **Step 2: Create minimal `oxivgl-build/src/lib.rs`**

```rust
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
```

- [ ] **Step 3: Verify it compiles**

Run: `cd oxivgl-build && cargo check`
Expected: OK (no errors)

- [ ] **Step 4: Commit**

```bash
git add oxivgl-build/
git commit -m "feat: scaffold oxivgl-build crate"
```

---

### Task 2: Implement `ImageConfig::from_env()`

**Files:**
- Modify: `oxivgl-build/src/lib.rs`

- [ ] **Step 1: Implement `from_env()`**

```rust
impl ImageConfig {
    /// Create config from environment.
    ///
    /// - `lv_conf_dir` from `DEP_LV_CONFIG_PATH` env var
    /// - `lvgl_include_dir` and `converter` from `CARGO_MANIFEST_DIR`
    ///   (assumes oxivgl workspace layout with `thirdparty/lvgl_rust_sys/lvgl/`)
    ///
    /// # Panics
    /// If `DEP_LV_CONFIG_PATH` is not set.
    pub fn from_env() -> Self {
        let lv_conf_dir = PathBuf::from(
            std::env::var("DEP_LV_CONFIG_PATH")
                .expect("DEP_LV_CONFIG_PATH must be set (points to dir containing lv_conf.h)"),
        );
        let manifest_dir = PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
        );
        let lvgl_root = manifest_dir.join("thirdparty/lvgl_rust_sys/lvgl");
        ImageConfig {
            lv_conf_dir,
            lvgl_include_dir: lvgl_root.join("src"),
            converter: lvgl_root.join("scripts/LVGLImage.py"),
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd oxivgl-build && cargo check`
Expected: OK

- [ ] **Step 3: Commit**

```bash
git add oxivgl-build/src/lib.rs
git commit -m "feat: ImageConfig::from_env() for oxivgl workspace layout"
```

---

### Task 3: Implement `image_asset()` — color depth parsing

**Files:**
- Modify: `oxivgl-build/src/lib.rs`

- [ ] **Step 1: Add helper to read `LV_COLOR_DEPTH` from `lv_conf.h`**

```rust
/// Read `LV_COLOR_DEPTH` from `lv_conf.h` and return the matching
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
    panic!(
        "LV_COLOR_DEPTH not found in {}",
        conf_path.display()
    );
}
```

- [ ] **Step 2: Add unit test**

```rust
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
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn parse_color_depth_32() {
        let dir = std::env::temp_dir().join("oxivgl_build_test_32");
        std::fs::create_dir_all(&dir).unwrap();
        let mut f = std::fs::File::create(dir.join("lv_conf.h")).unwrap();
        writeln!(f, "#define LV_COLOR_DEPTH 32").unwrap();
        assert_eq!(color_format_from_conf(&dir), "ARGB8888");
        std::fs::remove_dir_all(&dir).unwrap();
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
```

- [ ] **Step 3: Run tests**

Run: `cd oxivgl-build && cargo test`
Expected: 3 tests pass

- [ ] **Step 4: Commit**

```bash
git add oxivgl-build/src/lib.rs
git commit -m "feat: parse LV_COLOR_DEPTH from lv_conf.h"
```

---

### Task 4: Implement `image_asset()` — full pipeline

**Files:**
- Modify: `oxivgl-build/src/lib.rs`

- [ ] **Step 1: Implement `image_asset()`**

```rust
impl ImageConfig {
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
        let manifest_dir = PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"),
        );
        let out_dir = PathBuf::from(
            std::env::var("OUT_DIR").expect("OUT_DIR not set"),
        );
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
            "LVGLImage.py did not produce {}", c_file.display()
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
```

- [ ] **Step 2: Verify it compiles**

Run: `cd oxivgl-build && cargo check`
Expected: OK

- [ ] **Step 3: Commit**

```bash
git add oxivgl-build/src/lib.rs
git commit -m "feat: ImageConfig::image_asset() — full PNG→C→.o pipeline"
```

---

## Chunk 2: oxivgl integration

### Task 5: Re-export `lv_image_dsc_t` and add `image_declare!` macro

**Files:**
- Modify: `src/widgets/mod.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Re-export `lv_image_dsc_t` from `src/widgets/mod.rs`**

In the existing re-exports block (line 86):

```rust
// Re-export raw types so callbacks don't need `lvgl_rust_sys`.
pub use lvgl_rust_sys::{lv_color_t, lv_event_t, lv_image_dsc_t, lv_point_precise_t};
```

- [ ] **Step 2: Add `image_declare!` macro to `src/lib.rs`**

Add after the module declarations:

```rust
/// Declare an LVGL image asset compiled by `oxivgl-build`.
///
/// Equivalent to LVGL's `LV_IMAGE_DECLARE`. Generates an `extern "C"` static
/// binding to the `lv_image_dsc_t` symbol produced by `LVGLImage.py`.
///
/// # Example
///
/// ```ignore
/// oxivgl::image_declare!(img_cogwheel_argb);
/// // Use: image.set_src(unsafe { &img_cogwheel_argb });
/// ```
#[macro_export]
macro_rules! image_declare {
    ($name:ident) => {
        extern "C" {
            #[allow(non_upper_case_globals)]
            static $name: $crate::widgets::lv_image_dsc_t;
        }
    };
}
```

- [ ] **Step 3: Verify it compiles**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`
Expected: OK

- [ ] **Step 4: Commit**

```bash
git add src/widgets/mod.rs src/lib.rs
git commit -m "feat: image_declare! macro + lv_image_dsc_t re-export"
```

---

### Task 6: Add `Image::set_src()`

**Files:**
- Modify: `src/widgets/image.rs`

- [ ] **Step 1: Add `set_src()` to `Image`**

Add inside the `impl<'p> Image<'p>` block, after `new()`:

```rust
    /// Set the image source from a compiled image descriptor.
    ///
    /// The descriptor is typically produced by `oxivgl-build::image_asset()`
    /// and declared via [`image_declare!`](crate::image_declare).
    ///
    /// # Example
    ///
    /// ```ignore
    /// oxivgl::image_declare!(my_icon);
    /// let img = Image::new(&screen)?;
    /// img.set_src(unsafe { &my_icon });
    /// ```
    pub fn set_src(&self, dsc: &lv_image_dsc_t) -> &Self {
        // SAFETY: handle non-null (from Image::new); dsc points to valid
        // static lv_image_dsc_t produced by LVGLImage.py + cc.
        unsafe {
            lv_image_set_src(
                self.obj.handle(),
                dsc as *const lv_image_dsc_t as *const core::ffi::c_void,
            )
        };
        self
    }
```

Add `lv_image_dsc_t` and `lv_image_set_src` to the imports from `lvgl_rust_sys::*` (already covered by the wildcard import).

- [ ] **Step 2: Verify it compiles**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`
Expected: OK

- [ ] **Step 3: Commit**

```bash
git add src/widgets/image.rs
git commit -m "feat: Image::set_src() for compiled image descriptors"
```

---

### Task 7: Wire `oxivgl-build` into oxivgl's build

**Files:**
- Modify: `Cargo.toml`
- Modify: `build.rs`
- Create: `examples/assets/` (copy PNGs)

- [ ] **Step 1: Add `oxivgl-build` as build-dependency in `Cargo.toml`**

Add to the `[build-dependencies]` section:

```toml
oxivgl-build = { path = "oxivgl-build" }
```

- [ ] **Step 2: Copy `img_cogwheel_argb.png` to `examples/assets/`**

```bash
mkdir -p examples/assets
cp thirdparty/lvgl_rust_sys/lvgl/examples/assets/img_cogwheel_argb.png examples/assets/
```

- [ ] **Step 3: Update `build.rs` to compile example image assets**

Add after the existing Xtensa block:

```rust
fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.starts_with("xtensa-") {
        cmake_lvgl();
    }

    // Image assets (all targets)
    let cfg = oxivgl_build::ImageConfig::from_env();
    cfg.image_asset("img_cogwheel_argb", "examples/assets/img_cogwheel_argb.png");
}
```

- [ ] **Step 4: Verify full build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`
Expected: OK — `LVGLImage.py` runs, `.c` compiles, links

If Python deps missing: `pip3 install pypng lz4`

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml build.rs examples/assets/img_cogwheel_argb.png
git commit -m "feat: wire oxivgl-build into build.rs with cogwheel asset"
```

---

## Chunk 3: Validation example

### Task 8: Create `image1` example

**Files:**
- Create: `examples/image1.rs`

Reference: `thirdparty/lvgl_rust_sys/lvgl/examples/widgets/image/lv_example_image_1.c`

- [ ] **Step 1: Read the original C example**

```c
void lv_example_image_1(void) {
    LV_IMAGE_DECLARE(img_cogwheel_argb);
    lv_obj_t * img1 = lv_image_create(lv_screen_active());
    lv_image_set_src(img1, &img_cogwheel_argb);
    lv_obj_align(img1, LV_ALIGN_CENTER, 0, 0);
}
```

- [ ] **Step 2: Write the Rust example**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Image 1 — Basic image display
//!
//! Centered cogwheel image.

use oxivgl::{
    view::View,
    widgets::{Image, Screen, WidgetError},
};

oxivgl::image_declare!(img_cogwheel_argb);

struct WidgetImage1 {
    _img: Image<'static>,
}

impl View for WidgetImage1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let img = Image::new(&screen)?;
        img.set_src(unsafe { &img_cogwheel_argb });
        img.center();

        Ok(Self { _img: img })
    }

    fn on_event(&mut self, _event: &oxivgl::widgets::Event) {}

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetImage1);
```

- [ ] **Step 3: Run the example on host**

Run: `./run_host.sh image1`
Expected: SDL window showing centered cogwheel image

- [ ] **Step 4: Add to screenshot runner and README**

Add `image1` to `run_screenshots.sh`. Run `./run_screenshots.sh` to capture screenshot.

Add to `examples/doc/README.md` under a new `## Widgets — Image` section:

```markdown
## Widgets — Image

### Widget Image 1 — Basic Image Display

Centered cogwheel image from compiled PNG asset.

![widget_image1](screenshots/widget_image1.png)
```

Update the TOC to include `- [Widgets — Image](#widgets--image)`.

- [ ] **Step 5: Commit**

```bash
git add examples/image1.rs examples/doc/README.md run_screenshots.sh
git add examples/doc/screenshots/image1.png
git commit -m "feat: image1 example — validates image asset pipeline"
```

---

### Task 9: Run full test suite

- [ ] **Step 1: Run all tests**

Run: `./run_tests.sh all`
Expected: All existing tests pass (no regressions)

- [ ] **Step 2: Run all screenshots**

Run: `./run_screenshots.sh`
Expected: All screenshots generated including new `image1`

- [ ] **Step 3: Check ESP32 build**

Run: `cargo +esp -Zbuild-std=alloc,core check --features esp-hal,log-04`
Expected: OK (image asset compiles for Xtensa too)

- [ ] **Step 4: Commit any fixes if needed**

---

## Unresolved

- `image_declare!` exposes a `pub static` — should it be non-pub so only the declaring module sees it? (Current matches `LV_IMAGE_DECLARE` which is file-scoped in C)
- Additional `Image` widget methods (rotation, scale, pivot, offset) — defer to later tasks
- Example images beyond `img_cogwheel_argb` — add incrementally as examples need them
