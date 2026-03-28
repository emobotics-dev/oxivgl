#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Translation 2 — Language switching via dropdown with auto-updating labels
//!
//! Registers a static translation pack with animal names in English, German,
//! and Spanish. A dropdown lets the user pick the language; labels bound to
//! translation tags update automatically when the language changes.
//!
//! Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.

use oxivgl::{
    enums::EventCode,
    event::Event,
    layout::{FlexAlign, FlexFlow},
    translation::{self, StaticCStr as S},
    view::{register_event_on, View},
    widgets::{Dropdown, Label, Screen, WidgetError},
};

// Language tag strings used for index-based language switching.
static LANG_CSTR: [&core::ffi::CStr; 3] = [c"English", c"Deutsch", c"Espanol"];

// NULL-terminated language and tag arrays — LVGL stores these pointers directly.
static LANGUAGES: [S; 4] = [
    S::from_cstr(c"English"),
    S::from_cstr(c"Deutsch"),
    S::from_cstr(c"Espanol"),
    S::NULL,
];
static TAGS: [S; 5] = [
    S::from_cstr(c"tiger"),
    S::from_cstr(c"lion"),
    S::from_cstr(c"rabbit"),
    S::from_cstr(c"elephant"),
    S::NULL,
];
// Translations flattened row-major: [en_tiger, de_tiger, es_tiger, en_lion, ...]
static TRANSLATIONS: [S; 12] = [
    S::from_cstr(c"The Tiger"),
    S::from_cstr(c"Der Tiger"),
    S::from_cstr(c"El Tigre"),
    S::from_cstr(c"The Lion"),
    S::from_cstr(c"Der Loewe"),
    S::from_cstr(c"El Leon"),
    S::from_cstr(c"The Rabbit"),
    S::from_cstr(c"Das Kaninchen"),
    S::from_cstr(c"El Conejo"),
    S::from_cstr(c"The Elephant"),
    S::from_cstr(c"Der Elefant"),
    S::from_cstr(c"El Elefante"),
];

struct Translation2 {
    dd: Dropdown<'static>,
    _lbl_tiger: Label<'static>,
    _lbl_lion: Label<'static>,
    _lbl_rabbit: Label<'static>,
    _lbl_elephant: Label<'static>,
}

impl View for Translation2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Arrange screen items in a centered column.
        screen
            .set_flex_flow(FlexFlow::Column)
            .set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);

        // Register static translation pack.
        translation::add_static(&LANGUAGES, &TAGS, &TRANSLATIONS);

        // Language selector dropdown.
        let dd = Dropdown::new(&screen)?;
        dd.set_options("English\nDeutsch\nEspanol");
        dd.bubble_events();

        // Labels with translation tags — auto-update on language change.
        let lbl_tiger = Label::new(&screen)?;
        lbl_tiger.set_translation_tag("tiger");

        let lbl_lion = Label::new(&screen)?;
        lbl_lion.set_translation_tag("lion");

        let lbl_rabbit = Label::new(&screen)?;
        lbl_rabbit.set_translation_tag("rabbit");

        let lbl_elephant = Label::new(&screen)?;
        lbl_elephant.set_translation_tag("elephant");

        // Set initial language to English.
        translation::set_language(c"English");

        Ok(Self {
            dd,
            _lbl_tiger: lbl_tiger,
            _lbl_lion: lbl_lion,
            _lbl_rabbit: lbl_rabbit,
            _lbl_elephant: lbl_elephant,
        })
    }

    fn register_events(&mut self) {
        register_event_on(self, self.dd.handle());
    }

    fn on_event(&mut self, event: &Event) {
        if event.code() == EventCode::VALUE_CHANGED {
            let idx = self.dd.get_selected() as usize;
            if idx < LANG_CSTR.len() {
                translation::set_language(LANG_CSTR[idx]);
            }
        }
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Translation2);
