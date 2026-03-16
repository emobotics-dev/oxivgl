# LVGL Image Asset Pipeline — Design Spec

## Goal

Enable embedding arbitrary PNG images as LVGL image sources in oxivgl, compiled at build time via LVGL's `LVGLImage.py` converter.

## Architecture

Build-time pipeline: PNG → `LVGLImage.py` → `.c` → `cc` crate → linked static data.
Three components: `oxivgl-build` crate (build helper), `image_declare!` macro (extern declaration), `Image::set_src()` (runtime binding).

## Components

### 1. `oxivgl-build` crate

New crate at `oxivgl-build/`.

**Public API:**

```rust
/// Build-time configuration for image asset compilation.
pub struct ImageConfig {
    /// Path to dir containing `lv_conf.h`.
    pub lv_conf_dir: PathBuf,
    /// Path to LVGL header root (parent of `lvgl/` include dir).
    pub lvgl_include_dir: PathBuf,
    /// Path to `LVGLImage.py`.
    pub converter: PathBuf,
}

impl ImageConfig {
    /// Create config from environment variables set by lvgl_rust_sys.
    /// Uses `DEP_LV_CONFIG_PATH` for lv_conf_dir and derives
    /// lvgl_include_dir and converter path from the workspace layout.
    /// Panics with clear message if env vars are missing.
    pub fn from_env() -> Self;

    /// Convert a PNG to an LVGL C image source, compile it, and link it.
    ///
    /// - `name`: C symbol name (e.g. `"cogwheel"`), must be a valid C identifier
    /// - `path`: path to PNG file, relative to `CARGO_MANIFEST_DIR`
    ///
    /// Color format derived from `LV_COLOR_DEPTH` in `lv_conf.h`.
    /// Emits `cargo:rerun-if-changed` for the source PNG.
    ///
    /// # Build requirements
    /// Python 3 with `pypng` and `lz4` packages.
    ///
    /// # Panics
    /// - PNG file not found
    /// - `LV_COLOR_DEPTH` unsupported (must be 16, 24, or 32)
    /// - `LVGLImage.py` exits non-zero
    /// - `cc` compilation fails
    pub fn image_asset(&self, name: &str, path: &str);
}
```

**Internal steps:**

1. Resolve `path` relative to `CARGO_MANIFEST_DIR`
2. Read `LV_COLOR_DEPTH` from `lv_conf.h` → map to `--cf` flag:
   - 16 → `RGB565`
   - 24 → `RGB888`
   - 32 → `ARGB8888`
   - Other values → panic with clear error
3. Run: `python3 <converter> --ofmt C --cf <fmt> --align 1 --name <name> -o <OUT_DIR> <path>`
4. Compile `<OUT_DIR>/<name>.c` via `cc` crate with:
   - `-DLV_LVGL_H_INCLUDE_SIMPLE` (so generated `.c` uses `#include "lvgl.h"`)
   - Include path: `lvgl_include_dir` (for LVGL headers)
   - Include path: `lv_conf_dir` (for `lv_conf.h`)
5. Emit `cargo:rerun-if-changed=<path>`

Image compilation runs on **all targets** (host + Xtensa) — the generated `.c` is just `const` static data with no platform-specific code.

**Dependencies:** `cc` crate. Build-time: Python 3, `pypng`, `lz4`.

**Crate structure:**

```
oxivgl-build/
  Cargo.toml
  src/
    lib.rs          # ImageConfig, image_asset(), helpers
```

### 2. `image_declare!` macro (in oxivgl)

Rust equivalent of LVGL's `LV_IMAGE_DECLARE`. Uses re-exported type to avoid consumer dependency on `lvgl_rust_sys`:

```rust
// src/widgets/mod.rs — re-export
pub use lvgl_rust_sys::lv_image_dsc_t;

// src/widgets/image.rs or src/lib.rs
#[macro_export]
macro_rules! image_declare {
    ($name:ident) => {
        extern "C" {
            #[allow(non_upper_case_globals)]
            pub static $name: $crate::widgets::lv_image_dsc_t;
        }
    };
}
```

### 3. `Image::set_src()` (in oxivgl)

```rust
impl<'p> Image<'p> {
    /// Set the image source from a compiled image descriptor.
    pub fn set_src(&self, dsc: &lv_image_dsc_t) -> &Self {
        // SAFETY: handle non-null (from Image::new); dsc is a valid
        // lv_image_dsc_t produced by LVGLImage.py.
        unsafe {
            lv_image_set_src(self.handle(), dsc as *const _ as *const core::ffi::c_void)
        };
        self
    }
}
```

### 4. oxivgl's `build.rs` integration

oxivgl adds `oxivgl-build` as a build-dependency and calls `image_asset()` for example assets:

```rust
// build.rs
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

PNG source files for examples copied from `thirdparty/lvgl_rust_sys/lvgl/examples/assets/`.

## Usage

### Consumer build.rs

```toml
[build-dependencies]
oxivgl-build = { path = "../oxivgl-build" }
```

```rust
// build.rs
fn main() {
    let cfg = oxivgl_build::ImageConfig {
        lv_conf_dir: "conf".into(),
        lvgl_include_dir: "thirdparty/lvgl_rust_sys/lvgl/src".into(),
        converter: "thirdparty/lvgl_rust_sys/lvgl/scripts/LVGLImage.py".into(),
    };
    cfg.image_asset("logo", "assets/logo.png");
}
```

### Consumer Rust code

```rust
oxivgl::image_declare!(logo);

fn create_view() {
    let img = Image::new(&screen)?;
    img.set_src(unsafe { &logo });
}
```

The `unsafe` on `&logo` is required because it's an `extern static` — same constraint as LVGL's `LV_IMAGE_DECLARE` in C.

## Image data lifecycle

- Pixel data lives in `.rodata` (flash on ESP32, read-only memory on host)
- No RAM cost for storage; LVGL allocates temporary decode buffers internally during rendering
- No compression needed — flash storage is not a constraint

## Color format mapping

| `LV_COLOR_DEPTH` | `--cf` flag | Notes |
|---|---|---|
| 16 | `RGB565` | Default for this project |
| 24 | `RGB888` | — |
| 32 | `ARGB8888` | — |

Other values produce a build-time panic with a clear error message.
Format is read from `lv_conf.h` at build time, not hardcoded.

## Examples unlocked

With this pipeline, these previously-skipped examples become possible:

| Example | Asset | Priority |
|---|---|---|
| Style 6 — Image recolor/rotation | `img_cogwheel_argb` | High |
| Image 1 — Basic display | `img_cogwheel_argb` | High |
| Image 2 — Runtime recoloring | `img_cogwheel_argb` | Medium |
| Image 3 — Zoom + rotation anim | `img_cogwheel_argb` | Medium |
| Image 4 — Offset animation | `img_skew_strip` | Medium |
| Bar 4 — Stripe pattern | `img_skew_strip` | Medium |
| AnimImg 1 — Animated frames | `animimg001-003` | Low (needs AnimImg wrapper) |
| Scale 3 — Gauge needle | `img_hand` | Low |
| Dropdown 3 — Custom arrow | `img_caret_down` | Low |

## Constraints

- `LVGLImage.py` requires Python 3 + `pypng` + `lz4` at build time
- One `image_asset()` call per image — explicit, no globbing
- Symbol names must be valid C identifiers and valid filenames
- `LV_COLOR_DEPTH` must be 16, 24, or 32
