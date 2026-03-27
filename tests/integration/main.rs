// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests — exercise widgets against a real (headless) LVGL instance.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test integration
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

#[path = "../common/mod.rs"]
mod common;

mod anim;
mod draw;
mod event;
mod misc;
mod obj;
mod observer;
mod style;
mod timer;
mod widgets_container;
mod widgets_display;
mod widgets_input;
