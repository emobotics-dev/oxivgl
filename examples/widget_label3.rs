#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Label 3 — Mixed LTR, RTL (Hebrew), and Chinese text
//!
//! Three labels: English (LTR), Hebrew (RTL), and Chinese, each with its own
//! font.

use oxivgl::{
    fonts,
    view::View,
    style::Selector,
    widgets::{Obj, Align, BaseDir, Label, WidgetError},
};

#[derive(Default)]
struct WidgetLabel3 {
    _ltr_label: Option<Label<'static>>,
    _rtl_label: Option<Label<'static>>,
    _cjk_label: Option<Label<'static>>,
}

impl View for WidgetLabel3 {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {

        // LTR English label
        let ltr_label = Label::new(container)?;
        ltr_label.text_long(
            "In modern terminology, a microcontroller is similar to a \
             system on a chip (SoC).",
        );
        ltr_label.font(fonts::MONTSERRAT_16);
        ltr_label.width(310);
        ltr_label.align(Align::TopLeft, 5, 5);

        // RTL Hebrew label
        let rtl_label = Label::new(container)?;
        rtl_label.text_long(
            "\u{05DE}\u{05E2}\u{05D1}\u{05D3}, \u{05D0}\u{05D5} \
             \u{05D1}\u{05E9}\u{05DE}\u{05D5} \u{05D4}\u{05DE}\u{05DC}\u{05D0} \
             \u{05D9}\u{05D7}\u{05D9}\u{05D3}\u{05EA} \u{05E2}\u{05D9}\u{05D1}\u{05D5}\u{05D3} \
             \u{05DE}\u{05E8}\u{05DB}\u{05D6}\u{05D9}\u{05EA} \
             (\u{05D1}\u{05D0}\u{05E0}\u{05D2}\u{05DC}\u{05D9}\u{05EA}: \
             CPU - Central Processing Unit).",
        );
        rtl_label.style_base_dir(BaseDir::Rtl, Selector::DEFAULT);
        rtl_label.font(fonts::DEJAVU_16_PERSIAN_HEBREW);
        rtl_label.width(310);
        rtl_label.align(Align::LeftMid, 5, 0);

        // Chinese label
        let cjk_label = Label::new(container)?;
        cjk_label.text_long(
            "\u{5D4C}\u{5165}\u{5F0F}\u{7CFB}\u{7EDF}\u{FF08}Embedded System\u{FF09}\u{FF0C}\n\
             \u{662F}\u{4E00}\u{79CD}\u{5D4C}\u{5165}\u{673A}\u{68B0}\u{6216}\u{7535}\u{6C14}\
             \u{7CFB}\u{7EDF}\u{5185}\u{90E8}\u{3001}\u{5177}\u{6709}\u{4E13}\u{4E00}\u{529F}\
             \u{80FD}\u{548C}\u{5B9E}\u{65F6}\u{8BA1}\u{7B97}\u{6027}\u{80FD}\u{7684}\u{8BA1}\
             \u{7B97}\u{673A}\u{7CFB}\u{7EDF}\u{3002}",
        );
        cjk_label.font(fonts::SOURCE_HAN_SANS_SC_16_CJK);
        cjk_label.width(310);
        cjk_label.align(Align::BottomLeft, 5, -5);

                self._ltr_label = Some(ltr_label);
        self._rtl_label = Some(rtl_label);
        self._cjk_label = Some(cjk_label);
        Ok(())
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetLabel3::default());
