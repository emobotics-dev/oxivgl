#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 7 — Translation / i18n
//!
//! Uses LVGL's `lv_translation_*` API to switch between English, German,
//! and Spanish. Labels use translation tags and auto-update when the
//! language changes via the dropdown.
//!
//! Requires `LV_USE_TRANSLATION = 1` in `lv_conf.h`.

use oxivgl::{
    enums::EventCode,
    event::Event,
    translation::{self, StaticCStr as S},
    view::{register_event_on, View},
    widgets::{Align, Dropdown, Label, Screen, WidgetError},
};

// NULL-terminated static arrays — LVGL stores these pointers directly.
static LANGUAGES: [S; 4] = [S::from_cstr(c"en"), S::from_cstr(c"de"), S::from_cstr(c"es"), S::NULL];
static TAGS: [S; 5] = [
    S::from_cstr(c"hello"), S::from_cstr(c"welcome"),
    S::from_cstr(c"button"), S::from_cstr(c"goodbye"), S::NULL,
];
// Translations flattened: [en_hello, en_welcome, en_button, en_goodbye, de_..., es_...]
static TRANSLATIONS: [S; 12] = [
    S::from_cstr(c"Hello!"), S::from_cstr(c"Welcome to LVGL"),
    S::from_cstr(c"Press a button"), S::from_cstr(c"Goodbye"),
    S::from_cstr(c"Hallo!"), S::from_cstr(c"Willkommen bei LVGL"),
    S::from_cstr(c"Taste druecken"), S::from_cstr(c"Auf Wiedersehen"),
    S::from_cstr(c"Hola!"), S::from_cstr(c"Bienvenido a LVGL"),
    S::from_cstr(c"Pulsa un boton"), S::from_cstr(c"Adios"),
];

static LANG_CSTR: [&core::ffi::CStr; 3] = [c"en", c"de", c"es"];

struct WidgetLabel7 {
    dd: Dropdown<'static>,
    _lbl_hello: Label<'static>,
    _lbl_welcome: Label<'static>,
    _lbl_button: Label<'static>,
    _lbl_goodbye: Label<'static>,
}

impl View for WidgetLabel7 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Register translation pack and set initial language
        translation::add_static(&LANGUAGES, &TAGS, &TRANSLATIONS);
        translation::set_language(c"en");

        // Language selector dropdown
        let dd = Dropdown::new(&screen)?;
        dd.set_options("English\nDeutsch\nEspanol");
        dd.align(Align::TopMid, 0, 10);
        dd.bubble_events();

        // Labels with translation tags — auto-update on language change
        let lbl_hello = Label::new(&screen)?;
        lbl_hello.set_translation_tag("hello");
        lbl_hello.align(Align::Center, 0, -40);

        let lbl_welcome = Label::new(&screen)?;
        lbl_welcome.set_translation_tag("welcome");
        lbl_welcome.align(Align::Center, 0, -10);

        let lbl_button = Label::new(&screen)?;
        lbl_button.set_translation_tag("button");
        lbl_button.align(Align::Center, 0, 20);

        let lbl_goodbye = Label::new(&screen)?;
        lbl_goodbye.set_translation_tag("goodbye");
        lbl_goodbye.align(Align::Center, 0, 50);

        Ok(Self {
            dd,
            _lbl_hello: lbl_hello,
            _lbl_welcome: lbl_welcome,
            _lbl_button: lbl_button,
            _lbl_goodbye: lbl_goodbye,
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

oxivgl_examples_common::example_main!(WidgetLabel7);
