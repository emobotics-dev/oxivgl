#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Menu Encoder — three-input focus + in-place editing via an LVGL ENCODER
//!
//! A settings panel of editable sliders driven by the LVGL **encoder** model:
//! one interaction set (turn−, turn+, press) drives *both* focus navigation and
//! in-place value editing, because **LVGL owns the navigate ↔ edit toggle**.
//!
//! - **turn−/turn+** — in *navigate* mode move the focus highlight between
//!   sliders; in *edit* mode change the focused slider's value. Same inputs,
//!   two meanings — LVGL decides which.
//! - **short press** — enters edit on the focused slider (its knob starts
//!   responding to turns).
//! - **long press** — toggles edit mode, the canonical (and, in a multi-item
//!   group, only) way to *leave* edit.
//!
//! **On hardware** the three inputs are the board's buttons — the M5Stack
//! Fire's physical buttons or the CoreS3's touch-strip zones — mapped by the
//! harness to the encoder: outer buttons `turn(∓count)`, center short = `click`
//! (enter edit), center long = `long_press` (leave edit). The producer stays
//! context-free; [`run_app_nav_encoder`](oxivgl::view::run_app_nav_encoder)
//! reads it with no polling latency.
//!
//! **On host** (no hardware buttons) the four on-screen buttons stand in for the
//! board inputs, driving the same [`EncoderState`].
//!
//! Demonstrates:
//! - [`EncoderState`] driving focus navigation *and* in-place editing
//! - The navigate ↔ edit toggle across the *same* three inputs
//! - [`View::input_group`] routing the encoder to the sliders' [`Group`]
//! - A directly-routed decoded long press toggling edit mode (issue #127)

use oxivgl::{
    enums::{ObjFlag, ObjState},
    group::Group,
    layout::{FlexAlign, FlexFlow},
    style::{Palette, StyleBuilder, color_make, palette_lighten, palette_main},
    view::{NavAction, View},
    widgets::{Label, Obj, Part, Slider, WidgetError},
};

// On-screen controls stand in for the board's buttons on host only; on target
// the harness feeds the encoder from the real board buttons.
#[cfg(not(target_arch = "xtensa"))]
use oxivgl::{
    enums::EventCode,
    event::Event,
    indev::{EncoderIndev, EncoderState},
    widgets::{Align, Button},
};

/// Host-only shared encoder state, written by the on-screen buttons and read by
/// LVGL. `static` so it outlives the [`EncoderIndev`].
#[cfg(not(target_arch = "xtensa"))]
static ENC: EncoderState = EncoderState::new();

/// Slider entries (label, initial value).
const ITEMS: [(&str, i32); 3] = [("Volume", 60), ("Brightness", 40), ("Contrast", 75)];

#[derive(Default)]
struct MenuView {
    // Layout containers + labels kept alive: an owned widget wrapper deletes its
    // LVGL object on drop, so anything not stored would vanish.
    _menu_col: Option<Obj<'static>>,
    _title: Option<Label<'static>>,
    _item_labels: heapless::Vec<Label<'static>, 4>,

    // Focusable editable sliders, in the same order as `ITEMS`.
    sliders: heapless::Vec<Slider<'static>, 4>,

    status: Option<Label<'static>>,

    // The focus group exposed via input_group. On target the harness binds the
    // encoder to it; on host the device below drives it.
    group: Option<Group>,

    // Last edit-mode state, so the status line only redraws on a change (a
    // press enters edit, a long press leaves it).
    was_editing: bool,

    // Host-only on-screen controls + encoder device (touch → encoder input).
    #[cfg(not(target_arch = "xtensa"))]
    _nav_row: Option<Obj<'static>>,
    #[cfg(not(target_arch = "xtensa"))]
    _nav_labels: heapless::Vec<Label<'static>, 4>,
    #[cfg(not(target_arch = "xtensa"))]
    btn_minus: Option<Button<'static>>,
    #[cfg(not(target_arch = "xtensa"))]
    btn_ok: Option<Button<'static>>,
    #[cfg(not(target_arch = "xtensa"))]
    btn_ok_long: Option<Button<'static>>,
    #[cfg(not(target_arch = "xtensa"))]
    btn_plus: Option<Button<'static>>,
    #[cfg(not(target_arch = "xtensa"))]
    encoder: Option<EncoderIndev>,
}

impl View for MenuView {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let mut _b = StyleBuilder::new();
        _b.bg_color(color_make(0x12, 0x12, 0x20))
            .bg_opa(255)
            .pad_all(6)
            .pad_gap(4);
        let bg_style = _b.build();

        container
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Start, FlexAlign::Center, FlexAlign::Center)
            .add_style(&bg_style, Part::Main);

        // ── Title ────────────────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Blue, 2));
        let title_style = _b.build();

        let title = Label::new(container)?;
        title.text("Settings (encoder)").add_style(&title_style, Part::Main);
        self._title = Some(title);

        // ── Slider column (focusable, editable items) ────────────────────
        let mut _b = StyleBuilder::new();
        _b.bg_color(color_make(0x1c, 0x1c, 0x30))
            .bg_opa(255)
            .radius(8)
            .pad_all(8)
            .pad_gap(10);
        let panel_style = _b.build();

        let menu_col = Obj::new(container)?;
        menu_col.size(288, 150);
        menu_col
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center)
            .add_style(&panel_style, Part::Main)
            .remove_flag(ObjFlag::SCROLLABLE);

        // Focus highlight applied to the focused slider's indicator (LVGL sets
        // ObjState::FOCUSED on the encoder's current item).
        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_main(Palette::Blue));
        let focus_style = _b.build();

        let mut _b = StyleBuilder::new();
        _b.text_color(color_make(0xe8, 0xe8, 0xf2));
        let label_text_style = _b.build();

        let group = Group::new()?;

        for (name, init) in ITEMS {
            let label = Label::new(&menu_col)?;
            label.text(name).add_style(&label_text_style, Part::Main);

            let slider = Slider::new(&menu_col)?;
            slider.size(240, 12);
            slider.set_range(0, 100);
            slider.set_value(init);
            // Highlight the knob when this slider is the focused group member.
            slider
                .add_style(&focus_style, ObjState::FOCUSED)
                .add_flag(ObjFlag::EVENT_BUBBLE);

            // Add to the focus group so the encoder navigates between sliders,
            // and (in edit mode) turns adjust the focused one.
            group.add_obj(&slider);
            let _ = self.sliders.push(slider);
            let _ = self._item_labels.push(label);
        }

        // ── Status line ──────────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Green, 1));
        let status_style = _b.build();

        let status = Label::new(container)?;
        status
            .text("NAVIGATE  -  press OK to edit a slider")
            .add_style(&status_style, Part::Main);
        self.status = Some(status);

        // Host-only on-screen controls + encoder device.
        #[cfg(not(target_arch = "xtensa"))]
        self.create_host_controls(container)?;

        self._menu_col = Some(menu_col);
        self.group = Some(group);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        // Reflect the encoder's navigate ↔ edit mode in the status line so a
        // press (enter edit) and long press (leave edit) are visibly confirmed.
        // LVGL owns the toggle; we just read it via `Group::is_editing`.
        if let Some(group) = &self.group {
            let editing = group.is_editing();
            if editing != self.was_editing {
                self.was_editing = editing;
                if let Some(status) = &self.status {
                    status.text(if editing {
                        "EDIT MODE  -  turn to adjust, HOLD OK to exit"
                    } else {
                        "NAVIGATE  -  press OK to edit a slider"
                    });
                }
            }
        }
        Ok(NavAction::None)
    }

    #[cfg(not(target_arch = "xtensa"))]
    fn on_event(&mut self, event: &Event) -> NavAction {
        // Each on-screen button is a discrete, one-shot encoder input: one tap
        // (CLICKED) = one turn step / one click / one long press — matching a
        // debounced hardware button that emits finished events, not held edges.
        if let Some(b) = &self.btn_minus {
            if event.matches(b, EventCode::CLICKED) {
                ENC.turn(-1);
            }
        }
        if let Some(b) = &self.btn_plus {
            if event.matches(b, EventCode::CLICKED) {
                ENC.turn(1);
            }
        }
        if let Some(b) = &self.btn_ok {
            if event.matches(b, EventCode::CLICKED) {
                ENC.click();
            }
        }
        if let Some(b) = &self.btn_ok_long {
            if event.matches(b, EventCode::CLICKED) {
                ENC.long_press();
            }
        }

        NavAction::None
    }

    fn input_group(&self) -> Option<oxivgl::group::GroupRef> {
        self.group.as_ref().map(|g| g.as_ref())
    }
}

#[cfg(not(target_arch = "xtensa"))]
impl MenuView {
    /// Build the host on-screen encoder row (`-`, `OK`, `HOLD`, `+`) and the
    /// encoder device they feed. On target these inputs come from the board's
    /// physical buttons via the harness, so this is compiled out.
    fn create_host_controls(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let nav_row = Obj::new(container)?;
        nav_row.size(300, 38);
        nav_row
            .set_flex_flow(FlexFlow::Row)
            .set_flex_align(FlexAlign::SpaceEvenly, FlexAlign::Center, FlexAlign::Center)
            .remove_flag(ObjFlag::SCROLLABLE);

        let mut _b = StyleBuilder::new();
        _b.bg_color(palette_main(Palette::Indigo)).radius(8);
        let key_style = _b.build();

        let mut _b = StyleBuilder::new();
        _b.text_color(color_make(0xff, 0xff, 0xff));
        let key_text_style = _b.build();

        for (text, slot) in [
            ("-", 0usize),
            ("OK", 1),
            ("HOLD", 2),
            ("+", 3),
        ] {
            let btn = Button::new(&nav_row)?;
            btn.size(64, 36);
            btn.add_style(&key_style, Part::Main).add_flag(ObjFlag::EVENT_BUBBLE);
            let label = Label::new(&btn)?;
            label
                .text(text)
                .add_style(&key_text_style, Part::Main)
                .align(Align::Center, 0, 0);
            let _ = self._nav_labels.push(label);
            match slot {
                0 => self.btn_minus = Some(btn),
                1 => self.btn_ok = Some(btn),
                2 => self.btn_ok_long = Some(btn),
                _ => self.btn_plus = Some(btn),
            }
        }

        // The encoder device, fed by ENC. On target this role is filled by the
        // harness + run_app_nav_encoder instead.
        self.encoder = Some(EncoderIndev::new(&ENC)?);
        self._nav_row = Some(nav_row);
        Ok(())
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

oxivgl_examples_common::example_main_nav_encoder!(MenuView::default());
