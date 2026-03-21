#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Textarea 1 — Numeric keypad
//!
//! One-line textarea with a custom numeric button matrix keypad.
//! Pressing a digit appends it; backspace deletes; newline sends READY.

use oxivgl::{
    btnmatrix_map,
    enums::{EventCode, ObjFlag, ObjState},
    event::Event,
    view::{register_event_on, View},
    widgets::{Align, Buttonmatrix, ButtonmatrixMap, Screen, Textarea, WidgetError,
    },
};

/// LV_SYMBOL_BACKSPACE (U+F55A)
const SYMBOL_BACKSPACE: &str = "\u{F55A}";
/// LV_SYMBOL_NEW_LINE (U+F8A2)
const SYMBOL_NEW_LINE: &str = "\u{F8A2}";

static BTNM_MAP: &ButtonmatrixMap = btnmatrix_map!(
    c"1", c"2", c"3", c"\n",
    c"4", c"5", c"6", c"\n",
    c"7", c"8", c"9", c"\n",
    c"\u{F55A}", c"0", c"\u{F8A2}"
);

struct WidgetTextarea1 {
    ta: Textarea<'static>,
    btnm: Buttonmatrix<'static>,
}

impl View for WidgetTextarea1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let ta = Textarea::new(&screen)?;
        ta.set_one_line(true);
        ta.align(Align::TopMid, 0, 10);
        ta.add_state(ObjState::FOCUSED);
        ta.bubble_events();

        let btnm = Buttonmatrix::new(&screen)?;
        btnm.set_map(BTNM_MAP);
        btnm.size(200, 150);
        btnm.align(Align::BottomMid, 0, -10);
        btnm.remove_flag(ObjFlag::CLICK_FOCUSABLE); // Keep textarea focused on button clicks
        btnm.bubble_events();

        Ok(Self { ta, btnm })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.btnm.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.btnm, EventCode::VALUE_CHANGED) {
            let btn_id = self.btnm.get_selected_button();
            if let Some(txt) = self.btnm.get_button_text(btn_id) {
                if txt == SYMBOL_BACKSPACE {
                    self.ta.delete_char();
                } else if txt == SYMBOL_NEW_LINE {
                    self.ta.send_event(EventCode::READY);
                } else {
                    self.ta.add_text(txt);
                }
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetTextarea1);
