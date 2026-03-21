#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Calendar 1 — Month view with highlighted dates and arrow header
//!
//! Displays February 2021 with three highlighted dates. An arrow header
//! allows navigating between months. Clicking a day fires a VALUE_CHANGED
//! event; the label below shows the selected date.

use oxivgl::{
    enums::EventCode,
    event::Event,
    view::View,
    widgets::{Align, Calendar, CalendarDate, Label, Screen, WidgetError},
};

struct Calendar1 {
    cal: Calendar<'static>,
    label: Label<'static>,
}

impl View for Calendar1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let cal = Calendar::new(&screen)?;
        cal.size(185, 230).align(Align::Center, 0, 27);
        cal.set_today_date(2021, 2, 23)
            .set_month_shown(2021, 2)
            .set_highlighted_dates(&[
                CalendarDate::new(2021, 2, 6),
                CalendarDate::new(2021, 2, 11),
                CalendarDate::new(2022, 2, 22),
            ]);
        cal.add_header_arrow();
        cal.bubble_events();

        let label = Label::new(&screen)?;
        label.text("Click a day").align(Align::Center, 0, -90);

        Ok(Self { cal, label })
    }

    fn on_event(&mut self, event: &Event) {
        if event.matches(&self.cal, EventCode::VALUE_CHANGED) {
            if let Some(date) = self.cal.get_pressed_date() {
                let mut buf = heapless::String::<24>::new();
                let _ = core::fmt::Write::write_fmt(
                    &mut buf,
                    format_args!("{:02}/{:02}/{}", date.day, date.month, date.year),
                );
                self.label.text(&buf);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Calendar1);
