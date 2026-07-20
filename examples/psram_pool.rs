#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! PSRAM Pool — LVGL's heap in external RAM.
//!
//! On target the harness maps PSRAM and hands 512 KiB of it to
//! [`oxivgl::mem::reserve_pool`] before the render loop starts, so LVGL
//! allocates its objects and styles from external RAM instead of the scarce
//! internal heap. Draw buffers stay in internal, DMA-capable RAM — the ESP32
//! cannot DMA from PSRAM at all.
//!
//! The screen just reports that it is running; the interesting part is the boot
//! log, which prints the pool's base and size. On host there is no PSRAM and the
//! example runs on LVGL's internal pool unchanged.

use oxivgl::{
    style::{Selector, Style},
    view::{NavAction, View},
    widgets::{Align, Label, Obj, WidgetError},
};

/// Size of the PSRAM region handed to LVGL. Must not exceed TLSF's largest
/// indexable block, which LVGL fixes at compile time from
/// `LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE` (see `oxivgl::mem`).
const PSRAM_POOL_BYTES: usize = 512 * 1024;

#[derive(Default)]
struct PsramPool {
    _panel: Option<Obj<'static>>,
    _title: Option<Label<'static>>,
    _note: Option<Label<'static>>,
}

impl View for PsramPool {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let bg = Style::new(|s| {
            s.bg_color_hex(0x102027).bg_opa(255).text_color_hex(0xffffff);
        });
        container.add_style(&bg, Selector::DEFAULT);

        // Partial opacity on a container with children forces LVGL to render
        // the subtree into an intermediate *layer* buffer and composite it.
        // That is what puts the draw-buffer allocator on the hot path here —
        // without it a flat screen allocates no draw buffers at all, and the
        // guard that keeps them out of PSRAM would never run on target.
        let panel = Obj::new(container)?;
        let panel_style = Style::new(|s| {
            s.bg_color_hex(0x1d3b47).bg_opa(255).opa(200).text_color_hex(0xffffff);
        });
        panel.add_style(&panel_style, Selector::DEFAULT);
        panel.size(280, 140).align(Align::Center, 0, 0);

        let title = Label::new(&panel)?;
        title.text("LVGL heap in PSRAM").align(Align::Center, 0, -20);

        let note = Label::new(&panel)?;
        note.text("draw buffers stay internal").align(Align::Center, 0, 10);

        self._title = Some(title);
        self._note = Some(note);
        self._panel = Some(panel);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        Ok(NavAction::None)
    }
}

oxivgl_examples_common::example_main_psram!(PsramPool::default(), PSRAM_POOL_BYTES);
