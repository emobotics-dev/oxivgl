#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Flex 3 — Demonstrate flex grow

use oxivgl::{
    view::View,
    widgets::{FlexFlow, Obj, Screen, WidgetError},
};

struct Flex3 {
    _cont: Obj<'static>,
    _items: [Obj<'static>; 4],
}

impl View for Flex3 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let cont = Obj::new(&screen)?;
        cont.size(300, 220).center();
        cont.set_flex_flow(FlexFlow::Row);

        let obj0 = Obj::new(&cont)?;
        obj0.size(40, 40);

        let obj1 = Obj::new(&cont)?;
        obj1.height(40);
        obj1.set_flex_grow(1);

        let obj2 = Obj::new(&cont)?;
        obj2.height(40);
        obj2.set_flex_grow(2);

        let obj3 = Obj::new(&cont)?;
        obj3.size(40, 40);

        Ok(Self {
            _cont: cont,
            _items: [obj0, obj1, obj2, obj3],
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Flex3);
