#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grid 1 — A simple grid

use oxivgl::{
    view::View,
    widgets::{Button, GridAlign, Label, Screen, WidgetError, GRID_TEMPLATE_LAST},
};

static COL_DSC: [i32; 4] = [70, 70, 70, GRID_TEMPLATE_LAST];
static ROW_DSC: [i32; 4] = [50, 50, 50, GRID_TEMPLATE_LAST];

struct Grid1 {
    _cont: oxivgl::widgets::Obj<'static>,
    _buttons: heapless::Vec<Button<'static>, 9>,
    _labels: heapless::Vec<Label<'static>, 9>,
}

impl View for Grid1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let cont = oxivgl::widgets::Obj::new(&screen)?;
        cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);
        cont.size(300, 220).center();

        let mut buttons = heapless::Vec::<Button<'static>, 9>::new();
        let mut labels = heapless::Vec::<Label<'static>, 9>::new();

        for i in 0..9u32 {
            let col = (i % 3) as i32;
            let row = (i / 3) as i32;

            let btn = Button::new(&cont)?;
            btn.set_grid_cell(
                GridAlign::Stretch,
                col,
                1,
                GridAlign::Stretch,
                row,
                1,
            );

            let label = Label::new(&btn)?;
            let mut buf = heapless::String::<12>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("c{}, r{}\0", col, row));
            label.text(&buf)?.center();

            let _ = buttons.push(btn);
            let _ = labels.push(label);
        }

        Ok(Self {
            _cont: cont,
            _buttons: buttons,
            _labels: labels,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Grid1);
