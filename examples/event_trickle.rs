#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Event Trickle — Demonstrate event trickle-down
//!
//! TODO: Hardware target (fire27) has no touch screen yet — press events
//! require an input device to trigger. The GUI is fully wired; only the
//! physical input is missing.

use oxivgl::{
    view::View,
    widgets::{
        color_black, color_white, FlexFlow, Label, Obj, ObjFlag, ObjState, Screen, Style,
        StyleBuilder, WidgetError,
    },
};

struct EventTrickle {
    _cont: Obj<'static>,
    _style_black: Style,
    _subconts: heapless::Vec<Obj<'static>, 9>,
    _labels: heapless::Vec<Label<'static>, 9>,
}

impl View for EventTrickle {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // TODO: No touch input on fire27 hardware — press/focus events won't
        // fire until an input device is connected.
        #[cfg(target_arch = "xtensa")]
        oxivgl_examples_common::warn!(
            "event_trickle: no touch input — press events require input device"
        );

        let mut style_black = StyleBuilder::new();
        style_black
            .text_color(color_white())
            .bg_color(color_black());
        let style_black = style_black.build();

        let cont = Obj::new(&screen)?;
        cont.size(290, 200).center();
        cont.set_flex_flow(FlexFlow::RowWrap);
        cont.add_flag(ObjFlag::EVENT_TRICKLE);
        cont.add_style(&style_black, ObjState::PRESSED);

        let mut subconts = heapless::Vec::<Obj<'static>, 9>::new();
        let mut labels = heapless::Vec::<Label<'static>, 9>::new();

        for i in 0..9u32 {
            let subcont = Obj::new(&cont)?;
            subcont.size(70, 50);
            subcont.add_style(&style_black, ObjState::FOCUSED);

            let label = Label::new(&subcont)?;
            let mut buf = heapless::String::<4>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", i));
            label.text(&buf);

            let _ = subconts.push(subcont);
            let _ = labels.push(label);
        }

        Ok(Self {
            _cont: cont,
            _style_black: style_black,
            _subconts: subconts,
            _labels: labels,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(EventTrickle);
