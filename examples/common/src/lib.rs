// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared infrastructure for oxivgl examples.

#![cfg_attr(target_arch = "xtensa", no_std)]
mod fmt;

#[cfg(not(target_arch = "xtensa"))]
pub mod host;

#[cfg(target_arch = "xtensa")]
pub mod fire27;

// Re-exports used by macros in downstream crates.
#[cfg(not(target_arch = "xtensa"))]
pub use env_logger;
pub use oxivgl;

#[cfg(target_arch = "xtensa")]
pub use embassy_embedded_hal;
#[cfg(target_arch = "xtensa")]
pub use embassy_executor;
#[cfg(target_arch = "xtensa")]
pub use embassy_sync;
#[cfg(target_arch = "xtensa")]
pub use embassy_time;
#[cfg(target_arch = "xtensa")]
pub use esp_alloc;
#[cfg(target_arch = "xtensa")]
pub use esp_backtrace;
#[cfg(target_arch = "xtensa")]
pub use esp_bootloader_esp_idf;
#[cfg(target_arch = "xtensa")]
pub use esp_hal;
#[cfg(target_arch = "xtensa")]
pub use esp_println;
#[cfg(target_arch = "xtensa")]
pub use esp_rtos;
#[cfg(target_arch = "xtensa")]
pub use esp_sync;
#[cfg(target_arch = "xtensa")]
pub use lcd_async;
pub use log;
#[cfg(target_arch = "xtensa")]
pub use static_cell;
#[cfg(target_arch = "xtensa")]
pub use async_button;
#[cfg(target_arch = "xtensa")]
pub use oxivgl_sys;

/// Generate a `main` function for the given [`oxivgl::view::View`] instance,
/// selecting the correct backend at compile time.
#[macro_export]
macro_rules! example_main {
    ($view_expr:expr) => {
        #[cfg(target_arch = "xtensa")]
        $crate::fire27_main!($view_expr);

        #[cfg(not(target_arch = "xtensa"))]
        $crate::host_main!($view_expr);
    };
}

/// Generate a `main` function for a **multi-screen navigation** example.
///
/// Like [`example_main!`] but uses a [`Navigator`] to process
/// [`NavAction`] values, enabling push/pop/replace/modal transitions.
#[macro_export]
macro_rules! example_main_nav {
    ($view_expr:expr) => {
        #[cfg(target_arch = "xtensa")]
        $crate::fire27_main_nav!($view_expr);

        #[cfg(not(target_arch = "xtensa"))]
        $crate::host_main_nav!($view_expr);
    };
}

/// Generate a `main` that puts LVGL's heap in PSRAM on target.
///
/// Identical to [`example_main!`] on host, which has no PSRAM — the example
/// still builds, runs and screenshots there, it simply uses LVGL's internal
/// pool.
#[macro_export]
macro_rules! example_main_psram {
    ($view_expr:expr, $bytes:expr) => {
        #[cfg(target_arch = "xtensa")]
        $crate::fire27_main_psram!($view_expr, $bytes);

        // Host has no PSRAM, so `$bytes` goes unused there. Bind it to a
        // discarded const so the example's size constant is still "used" and
        // does not trip the crate's deny-dead-code policy.
        #[cfg(not(target_arch = "xtensa"))]
        const _: usize = $bytes;
        #[cfg(not(target_arch = "xtensa"))]
        $crate::host_main!($view_expr);
    };
}
