#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Bubble — Demonstrate event bubbling
//!
//! TODO: Hardware target (fire27) has no touch screen yet — click events
//! require an input device to trigger. The GUI is fully wired; only the
//! physical input is missing.

use oxivgl::{
    style::{palette_main, Palette, Selector},
    view::{register_event_on, View},
    widgets::{Button, Event, EventCode, FlexFlow, Label, Obj, Screen, WidgetError},
};

struct EventBubble {
    cont: Obj<'static>,
    _buttons: heapless::Vec<Button<'static>, 30>,
    _labels: heapless::Vec<Label<'static>, 30>,
}

impl View for EventBubble {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // TODO: No touch input on fire27 hardware — click events won't fire
        // until an input device is connected.
        #[cfg(target_arch = "xtensa")]
        oxivgl_examples_common::warn!(
            "event_bubble: no touch input — click events require input device"
        );

        let cont = Obj::new(&screen)?;
        cont.size(290, 200).center();
        cont.set_flex_flow(FlexFlow::RowWrap);

        let mut buttons = heapless::Vec::<Button<'static>, 30>::new();
        let mut labels = heapless::Vec::<Label<'static>, 30>::new();

        for i in 0..30u32 {
            let btn = Button::new(&cont)?;
            btn.size(70, 50);
            btn.bubble_events();

            let label = Label::new(&btn)?;
            let mut buf = heapless::String::<4>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", i));
            label.text(&buf).center();

            let _ = buttons.push(btn);
            let _ = labels.push(label);
        }

        Ok(Self {
            cont,
            _buttons: buttons,
            _labels: labels,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.cont.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() != EventCode::CLICKED {
            return;
        }
        let target = event.target_handle();
        if target == self.cont.handle() {
            return;
        }
        event.target_style_bg_color(palette_main(Palette::Red), Selector::DEFAULT);
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(EventBubble);
