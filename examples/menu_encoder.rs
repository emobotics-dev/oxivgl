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
//! The four on-screen buttons at the bottom feed an [`EncoderIndev`]:
//!
//! - `−` / `+` — [`EncoderState::turn`]: in **navigate** mode they move the
//!   focus highlight between sliders; in **edit** mode they change the focused
//!   slider's value. Same buttons, two meanings — LVGL decides which.
//! - `OK` — [`EncoderState::click`]: a short click *enters* edit on the focused
//!   slider (its knob starts responding to `−`/`+`).
//! - `HOLD` — [`EncoderState::long_press`]: a long press *toggles* edit mode,
//!   the canonical (and, in a multi-item group, only) way to *leave* edit.
//!
//! This mirrors a **three-button embedded board** (e.g. an M5Stack Fire's front
//! buttons, or a CoreS3 touch strip): the input producer stays context-free —
//! it emits turn / click / long-press without knowing whether the UI is
//! navigating or editing. On hardware the producer is a button task calling the
//! same [`EncoderState`] methods, and
//! [`run_app_nav_encoder`](oxivgl::view::run_app_nav_encoder) reads it with no
//! polling latency; here the producer is the on-screen buttons.
//!
//! Demonstrates:
//! - [`EncoderState`] + [`EncoderIndev`] — an encoder device fed by the app
//! - The navigate ↔ edit toggle across the *same* three inputs
//! - [`View::input_group`] routing the encoder to the sliders' [`Group`]
//! - Multi-step turns and a directly-routed decoded long press (issue #127)

use oxivgl::{
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    group::Group,
    indev::{EncoderIndev, EncoderState},
    layout::{FlexAlign, FlexFlow},
    style::{Palette, StyleBuilder, color_make, palette_lighten, palette_main},
    view::{NavAction, View},
    widgets::{Align, Button, Label, Obj, Part, Slider, WidgetError},
};

/// Shared encoder state, written by the on-screen buttons and read by LVGL.
///
/// `static` so it outlives the [`EncoderIndev`] (LVGL stores a pointer to it).
static ENC: EncoderState = EncoderState::new();

/// Slider entries (label, initial value).
const ITEMS: [(&str, i32); 3] = [("Volume", 60), ("Brightness", 40), ("Contrast", 75)];

#[derive(Default)]
struct MenuView {
    // Layout containers + labels kept alive: an owned widget wrapper deletes its
    // LVGL object on drop, so anything not stored would vanish.
    _menu_col: Option<Obj<'static>>,
    _nav_row: Option<Obj<'static>>,
    _title: Option<Label<'static>>,
    _item_labels: heapless::Vec<Label<'static>, 4>,
    _nav_labels: heapless::Vec<Label<'static>, 4>,

    // Focusable editable sliders, in the same order as `ITEMS`.
    sliders: heapless::Vec<Slider<'static>, 4>,

    // On-screen encoder buttons (touch → encoder input). Not in the focus group.
    btn_minus: Option<Button<'static>>,
    btn_ok: Option<Button<'static>>,
    btn_ok_long: Option<Button<'static>>,
    btn_plus: Option<Button<'static>>,

    status: Option<Label<'static>>,

    // The focus group (exposed via input_group) and the encoder device that
    // drives it. Both survive for the life of the view.
    group: Option<Group>,
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

        // ── On-screen encoder row: `-`  OK  HOLD  `+` ────────────────────
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

        let (minus, minus_lbl) = make_key_button(&nav_row, "-", &key_style, &key_text_style)?;
        let (ok, ok_lbl) = make_key_button(&nav_row, "OK", &key_style, &key_text_style)?;
        let (ok_long, ok_long_lbl) = make_key_button(&nav_row, "HOLD", &key_style, &key_text_style)?;
        let (plus, plus_lbl) = make_key_button(&nav_row, "+", &key_style, &key_text_style)?;
        self.btn_minus = Some(minus);
        self.btn_ok = Some(ok);
        self.btn_ok_long = Some(ok_long);
        self.btn_plus = Some(plus);
        let _ = self._nav_labels.push(minus_lbl);
        let _ = self._nav_labels.push(ok_lbl);
        let _ = self._nav_labels.push(ok_long_lbl);
        let _ = self._nav_labels.push(plus_lbl);

        // ── Status line ──────────────────────────────────────────────────
        let mut _b = StyleBuilder::new();
        _b.text_color(palette_lighten(Palette::Green, 1));
        let status_style = _b.build();

        let status = Label::new(container)?;
        status
            .text("OK edits, -/+ move or adjust, HOLD exits edit")
            .add_style(&status_style, Part::Main);
        self.status = Some(status);

        // The encoder device, fed by ENC. Created here (after lv_init) so the
        // navigator can bind input_group() to it. The first group member is
        // focused automatically when added above.
        self.encoder = Some(EncoderIndev::new(&ENC)?);

        self._menu_col = Some(menu_col);
        self._nav_row = Some(nav_row);
        self.group = Some(group);
        Ok(())
    }

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

/// Build one on-screen encoder button (`−`, `OK`, `OK··`, `+`) with a centred
/// label.
fn make_key_button(
    parent: &Obj<'static>,
    text: &str,
    style: &oxivgl::style::Style,
    label_style: &oxivgl::style::Style,
) -> Result<(Button<'static>, Label<'static>), WidgetError> {
    let btn = Button::new(parent)?;
    btn.size(64, 36);
    btn.add_style(style, Part::Main)
        // Not added to the focus group — these are touch controls, not items.
        .add_flag(ObjFlag::EVENT_BUBBLE);
    let label = Label::new(&btn)?;
    label
        .text(text)
        .add_style(label_style, Part::Main)
        .align(Align::Center, 0, 0);
    // Returned so the caller can keep it alive — dropping it deletes the label.
    Ok((btn, label))
}

// ── Entry point ──────────────────────────────────────────────────────────────

oxivgl_examples_common::example_main_nav!(MenuView::default());
