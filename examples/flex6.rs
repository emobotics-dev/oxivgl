#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 6 — RTL base direction changes order of the items

use oxivgl::{
    view::View,
    widgets::{BaseDir, FlexFlow, Label, Obj, Screen, WidgetError, LV_SIZE_CONTENT},
};

struct Flex6 {
    _cont: Obj<'static>,
    _items: heapless::Vec<Obj<'static>, 20>,
    _labels: heapless::Vec<Label<'static>, 20>,
}

impl View for Flex6 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let cont = Obj::new(&screen)?;
        cont.set_style_base_dir(BaseDir::Rtl, 0);
        cont.size(300, 220).center();
        cont.set_flex_flow(FlexFlow::RowWrap);

        let mut items = heapless::Vec::<Obj<'static>, 20>::new();
        let mut labels = heapless::Vec::<Label<'static>, 20>::new();

        for i in 0..20u32 {
            let obj = Obj::new(&cont)?;
            obj.size(70, LV_SIZE_CONTENT);

            let label = Label::new(&obj)?;
            let mut buf = heapless::String::<4>::new();
            let _ = core::fmt::Write::write_fmt(&mut buf, format_args!("{}", i));
            label.text(&buf).center();

            let _ = items.push(obj);
            let _ = labels.push(label);
        }

        Ok(Self {
            _cont: cont,
            _items: items,
            _labels: labels,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Flex6);
