#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Observer 6 — Light / dark theme switching via `bind_style`
//!
//! A panel containing 10 buttons is displayed at 90 % × 90 % of the screen,
//! centered.  Clicking any button toggles the theme subject between 0 (light)
//! and 1 (dark).
//!
//! **Approach**: base (light) styles are applied unconditionally with
//! `add_style`; dark-overlay styles are conditionally applied with
//! `bind_style` whenever the theme subject equals 1.  No runtime style
//! mutation is required — LVGL's observer mechanism handles the switch.

use core::ffi::c_void;
use core::ptr::null_mut;

use oxivgl::{
    enums::EventCode,
    event::Event,
    layout::{FlexAlign, FlexFlow},
    style::{GradDir, Selector, StyleBuilder, color_make, lv_pct},
    view::View,
    widgets::{
        Align, AsLvHandle, Button, Child, Label, Obj, Part, Screen, Subject, WidgetError,
        RADIUS_MAX,
    },
};

struct Observer6 {
    // Panel stored as a top-level widget — owns children in LVGL.
    _panel: Obj<'static>,
    // Raw handles for event matching (10 buttons).
    btn_handles: [*mut c_void; 10],

    // Styles kept alive via Rc refcount inside Style.
    _panel_base: oxivgl::style::Style,
    _panel_dark: oxivgl::style::Style,
    _scrollbar_base: oxivgl::style::Style,
    _scrollbar_dark: oxivgl::style::Style,
    _btn_base: oxivgl::style::Style,
    _btn_dark: oxivgl::style::Style,
    _btn_pressed_light: oxivgl::style::Style,
    _btn_pressed_dark: oxivgl::style::Style,

    // Subject dropped last so observer linkage outlives all widgets.
    theme_subject: Subject,
}

impl View for Observer6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // ── Theme subject (initial: 1 = dark) ──────────────────────────────
        let theme_subject = Subject::new_int(1);

        // ── Panel base style (light) ───────────────────────────────────────
        let mut b = StyleBuilder::new();
        b.radius(12)
            .bg_opa(255)
            .bg_color_hex(0xffffff)
            .shadow_width(24)
            .shadow_offset_x(4)
            .shadow_offset_y(6)
            .shadow_color(color_make(0x88, 0x88, 0x88))
            .text_color_hex(0x222222)
            .pad_all(12)
            .pad_gap(16);
        let panel_base = b.build();

        // ── Panel dark overlay ─────────────────────────────────────────────
        let mut b = StyleBuilder::new();
        b.bg_color_hex(0x040038)
            .shadow_color(color_make(0xaa, 0xaa, 0xaa))
            .text_color_hex(0xeeeeee);
        let panel_dark = b.build();

        // ── Scrollbar base style (light) ───────────────────────────────────
        let mut b = StyleBuilder::new();
        b.width(4)
            .radius(2)
            .pad_right(8)
            .pad_ver(8)
            .bg_opa(127) // ~50% of 255
            .bg_color_hex(0x888888);
        let scrollbar_base = b.build();

        // ── Scrollbar dark overlay ─────────────────────────────────────────
        let mut b = StyleBuilder::new();
        b.bg_color_hex(0xaaaaaa);
        let scrollbar_dark = b.build();

        // ── Button base style (light) ──────────────────────────────────────
        let mut b = StyleBuilder::new();
        b.radius(RADIUS_MAX as i16)
            .bg_opa(255)
            .bg_grad_dir(GradDir::Hor)
            .bg_color_hex(0x3379de)
            .bg_grad_color_hex(0xd249a5)
            .shadow_width(24)
            .shadow_offset_y(6)
            .shadow_color(color_make(0x33, 0x79, 0xde))
            .text_color_hex(0xffffff)
            .pad_left(32)
            .pad_right(32)
            .pad_top(12)
            .pad_ver(12);
        let btn_base = b.build();

        // ── Button dark overlay ────────────────────────────────────────────
        let mut b = StyleBuilder::new();
        b.bg_color_hex(0xde1382)
            .bg_grad_color_hex(0x4b0c72)
            .shadow_color(color_make(0x4b, 0x0c, 0x72));
        let btn_dark = b.build();

        // ── Button pressed style (light) — shade filter opa 70% ────────────
        let mut b = StyleBuilder::new();
        b.color_filter_shade(178); // LV_OPA_70 ≈ 178
        let btn_pressed_light = b.build();

        // ── Button pressed style (dark) — shade filter opa 30% ─────────────
        let mut b = StyleBuilder::new();
        b.color_filter_shade(76); // LV_OPA_30 ≈ 76
        let btn_pressed_dark = b.build();

        // ── Panel ──────────────────────────────────────────────────────────
        let panel = Obj::new(&screen)?;
        panel
            .remove_style_all()
            .size(lv_pct(90), lv_pct(90))
            .align(Align::Center, 0, 0)
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center);

        // Apply base (light) styles unconditionally.
        panel.add_style(&panel_base, Selector::DEFAULT);
        panel.add_style(&scrollbar_base, Part::Scrollbar);

        // Bind dark overlays when theme == 1.
        panel.bind_style(&panel_dark, Selector::DEFAULT, &theme_subject, 1);
        panel.bind_style(&scrollbar_dark, Part::Scrollbar, &theme_subject, 1);

        // ── Buttons ────────────────────────────────────────────────────────
        let mut btn_handles: [*mut c_void; 10] = [null_mut(); 10];

        for i in 0..10_usize {
            let btn = Child::new(Button::new(&panel)?);
            btn.remove_style_all();
            btn.add_style(&btn_base, Selector::DEFAULT);
            btn.add_style(&btn_pressed_light, oxivgl::enums::ObjState::PRESSED);
            btn.bind_style(&btn_dark, Selector::DEFAULT, &theme_subject, 1);
            btn.bind_style(
                &btn_pressed_dark,
                oxivgl::enums::ObjState::PRESSED,
                &theme_subject,
                1,
            );

            // Label inside the button.
            let mut lbl_text = heapless::String::<16>::new();
            core::fmt::write(&mut lbl_text, format_args!("Button {}", i + 1))?;
            let lbl = Child::new(Label::new(&*btn)?);
            lbl.text(lbl_text.as_str()).center();

            btn_handles[i] = btn.lv_handle() as *mut c_void;
        }

        // Force initial theme application (subject starts at 1 = dark).
        theme_subject.notify();

        Ok(Self {
            _panel: panel,
            btn_handles,
            _panel_base: panel_base,
            _panel_dark: panel_dark,
            _scrollbar_base: scrollbar_base,
            _scrollbar_dark: scrollbar_dark,
            _btn_base: btn_base,
            _btn_dark: btn_dark,
            _btn_pressed_light: btn_pressed_light,
            _btn_pressed_dark: btn_pressed_dark,
            theme_subject,
        })
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::CLICKED {
            return;
        }
        let target = event.target_handle() as *mut c_void;
        for &handle in self.btn_handles.iter() {
            if !handle.is_null() && target == handle {
                // Toggle between light (0) and dark (1).
                if self.theme_subject.get_int() == 0 {
                    self.theme_subject.set_int(1);
                } else {
                    self.theme_subject.set_int(0);
                }
                return;
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Observer6);
