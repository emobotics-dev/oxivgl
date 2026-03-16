#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 1 — A simple row and a column layout with flexbox

use oxivgl::{
    style::{lv_pct, LV_SIZE_CONTENT},
    view::View,
    layout::FlexFlow,
    widgets::{Align, Button, Label, Obj, Screen, WidgetError},
};

struct Flex1 {
    _cont_row: Obj<'static>,
    _cont_col: Obj<'static>,
    _buttons: heapless::Vec<Button<'static>, 20>,
    _labels: heapless::Vec<Label<'static>, 20>,
}

impl View for Flex1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let cont_row = Obj::new(&screen)?;
        cont_row.size(300, 75).align(Align::TopMid, 0, 5);
        cont_row.set_flex_flow(FlexFlow::Row);

        let cont_col = Obj::new(&screen)?;
        cont_col.size(200, 150);
        cont_col.align_to(&cont_row, Align::OutBottomMid, 0, 5);
        cont_col.set_flex_flow(FlexFlow::Column);

        let mut buttons = heapless::Vec::<Button<'static>, 20>::new();
        let mut labels = heapless::Vec::<Label<'static>, 20>::new();

        for i in 0..10u32 {
            let mut buf = heapless::String::<16>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("Item: {}", i));

            // Row item
            let btn = Button::new(&cont_row)?;
            btn.size(100, lv_pct(100));
            let lbl = Label::new(&btn)?;
            lbl.text(&buf).center();
            let _ = buttons.push(btn);
            let _ = labels.push(lbl);

            // Column item
            let btn = Button::new(&cont_col)?;
            btn.size(lv_pct(100), LV_SIZE_CONTENT);
            let lbl = Label::new(&btn)?;
            lbl.text(&buf).center();
            let _ = buttons.push(btn);
            let _ = labels.push(lbl);
        }

        Ok(Self {
            _cont_row: cont_row,
            _cont_col: cont_col,
            _buttons: buttons,
            _labels: labels,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Flex1);
