#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Click — Add click event to a button
//!
//! TODO: Hardware target (fire27) has no touch screen yet — click events
//! require an input device to trigger. The GUI is fully wired; only the
//! physical input is missing.

use oxivgl::{
    view::{Event, View},
    widgets::{
        Button, LV_EVENT_CLICKED, LV_OBJ_FLAG_EVENT_BUBBLE, Label, Screen, WidgetError,
    },
};

struct EventClick {
    btn: Button<'static>,
    label: Label<'static>,
    cnt: u32,
}

impl View for EventClick {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // TODO: No touch input on fire27 hardware — click events won't fire
        // until an input device is connected.
        #[cfg(target_arch = "xtensa")]
        oxivgl_examples_common::warn!("event_click: no touch input — click events require input device");

        let btn = Button::new(&screen)?;
        btn.size(100, 50).center();
        btn.add_flag(LV_OBJ_FLAG_EVENT_BUBBLE);

        let label = Label::new(&btn)?;
        label.text("Click me!\0")?.center();

        Ok(Self { btn, label, cnt: 0 })
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == LV_EVENT_CLICKED && event.target_handle() == self.btn.handle() {
            self.cnt += 1;
            let mut buf = heapless::String::<12>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", self.cnt));
            self.label.set_text(&buf);
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(EventClick);
