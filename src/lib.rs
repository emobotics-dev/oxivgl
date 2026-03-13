// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg_attr(target_os = "none", no_std)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(target_os = "none", feature(asm_experimental_arch))]

extern crate alloc;

mod fmt;

pub mod fonts;
pub mod lvgl;
pub mod lvgl_buffers;
pub mod view;
pub mod widgets;

