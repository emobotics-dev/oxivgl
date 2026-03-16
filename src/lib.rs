// SPDX-License-Identifier: MIT OR Apache-2.0
//! Safe Rust bindings for LVGL on embedded and host targets.
#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", feature(type_alias_impl_trait))]
#![cfg_attr(target_os = "none", feature(asm_experimental_arch))]

extern crate alloc;

mod fmt;

/// Built-in LVGL font handles.
pub mod fonts;
/// LVGL driver initialization (tick source, log bridge).
pub mod lvgl;
/// ESP32 flush pipeline: async DMA transfer between LVGL and the display driver.
#[cfg(feature = "esp-hal")]
pub mod flush_pipeline;
/// DMA-aligned render buffers and embedded display initialisation.
pub mod lvgl_buffers;
/// Animation descriptors, path functions, and timeline management.
pub mod anim;
/// Style system: builders, selectors, themes, gradients, and color palettes.
pub mod style;
/// Universal convenience re-exports (`use oxivgl::prelude::*`).
pub mod prelude;
/// View trait and LVGL render loop.
pub mod view;
/// Type-safe LVGL widget wrappers.
pub mod widgets;

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
        unsafe extern "C" {
            #[allow(non_upper_case_globals)]
            static $name: $crate::widgets::lv_image_dsc_t;
        }
    };
}
