#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget Msgbox 2 — Settings dialog
//!
//! A non-modal message box styled as a settings dialog. Contains a title
//! with minimize and close header buttons, two sliders (brightness, speed)
//! in the content area, and Apply/Cancel buttons in an indigo-styled footer.

use oxivgl::{
    layout::FlexFlow,
    style::Selector,
    view::View,
    widgets::{Child, Label, Msgbox, Screen, Slider, WidgetError},
};

struct WidgetMsgbox2 {
    _screen: Screen,
    _mbox: Msgbox<'static>,
    _lbl_bright: Child<Label<'static>>,
    _slider_bright: Child<Slider<'static>>,
    _lbl_speed: Child<Label<'static>>,
    _slider_speed: Child<Slider<'static>>,
}

impl View for WidgetMsgbox2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        // Non-modal msgbox (parent = screen)
        let mbox = Msgbox::new(Some(&screen))?;
        mbox.size(300, 200);
        mbox.center();
        mbox.style_clip_corner(true, Selector::DEFAULT);

        // Title + header buttons
        mbox.add_title("Settings");
        mbox.add_header_button(&oxivgl::symbols::MINUS);
        mbox.add_close_button();

        // Content area — flex column with sliders
        let content = mbox.get_content();
        content.set_flex_flow(FlexFlow::Column);
        content.pad(10);

        let lbl_bright = Label::new(&*content)?;
        lbl_bright.text("Brightness");
        let lbl_bright = Child::new(lbl_bright);

        let slider_bright = Slider::new(&*content)?;
        slider_bright.width(250);
        slider_bright.set_value(70);
        let slider_bright = Child::new(slider_bright);

        let lbl_speed = Label::new(&*content)?;
        lbl_speed.text("Speed");
        let lbl_speed = Child::new(lbl_speed);

        let slider_speed = Slider::new(&*content)?;
        slider_speed.width(250);
        slider_speed.set_value(40);
        let slider_speed = Child::new(slider_speed);

        // Footer buttons
        let btn_apply = mbox.add_footer_button("Apply");
        btn_apply.set_flex_grow(1);

        let btn_cancel = mbox.add_footer_button("Cancel");
        btn_cancel.set_flex_grow(1);

        // Style footer with indigo background
        let footer = mbox.get_footer().expect("footer exists after add_footer_button");
        let indigo = oxivgl::style::palette_main(oxivgl::style::Palette::Indigo);
        footer.style_bg_color(indigo, Selector::DEFAULT);
        footer.bg_opa(255);

        Ok(Self {
            _screen: screen,
            _mbox: mbox,
            _lbl_bright: lbl_bright,
            _slider_bright: slider_bright,
            _lbl_speed: lbl_speed,
            _slider_speed: slider_speed,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(WidgetMsgbox2);
