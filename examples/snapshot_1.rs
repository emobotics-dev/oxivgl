#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Snapshot 1 — Capture and display a rotated, scaled widget snapshot
//!
//! Simplified: interactive snapshot retake on press/release omitted —
//! event user_data passing not yet wrapped. Star images replaced with
//! colored squares (compiled star asset not available).

use oxivgl::{
    layout::{FlexAlign, FlexFlow},
    snapshot::Snapshot,
    view::View,
    widgets::{Image, Obj, Part, Screen, WidgetError},
};

struct Snapshot1 {
    _snapshot_img: Image<'static>,
    _container: Obj<'static>,
    _item0: Obj<'static>,
    _item1: Obj<'static>,
    _item2: Obj<'static>,
    _item3: Obj<'static>,
    // snapshot declared LAST → dropped last, satisfying LVGL's
    // pointer-lifetime requirement (spec §3.1).
    _snapshot: Snapshot,
}

/// Colors for the four squares inside the container.
const ITEM_COLORS: [u32; 4] = [0xe74c3c, 0x2ecc71, 0x3498db, 0xf39c12];

impl View for Snapshot1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.bg_color(0xadd8e6).bg_opa(255);

        // Image widget that will display the snapshot (source set later).
        let snapshot_img = Image::new(&screen)?;
        snapshot_img.center();

        // Container: 180×180, centered, flex row-wrap, radius 50.
        let container = Obj::new(&screen)?;
        container
            .size(180, 180)
            .center()
            .radius(50, Part::Main)
            .bg_color(0x303030)
            .bg_opa(255);
        container.set_flex_flow(FlexFlow::RowWrap);
        container.set_flex_align(FlexAlign::SpaceEvenly, FlexAlign::Center, FlexAlign::Center);
        container.pad(5);

        // Four colored squares inside the container.
        let item0 = Obj::new(&container)?;
        item0.size(50, 50).bg_color(ITEM_COLORS[0]).bg_opa(255).border_width(0);
        let item1 = Obj::new(&container)?;
        item1.size(50, 50).bg_color(ITEM_COLORS[1]).bg_opa(255).border_width(0);
        let item2 = Obj::new(&container)?;
        item2.size(50, 50).bg_color(ITEM_COLORS[2]).bg_opa(255).border_width(0);
        let item3 = Obj::new(&container)?;
        item3.size(50, 50).bg_color(ITEM_COLORS[3]).bg_opa(255).border_width(0);

        // Take snapshot of the container (ARGB8888).
        let snapshot = Snapshot::take_widget(&container).ok_or(WidgetError::LvglNullPointer)?;

        // Configure the snapshot image: 0.5× scale (128), 30° rotation (300).
        snapshot_img.set_src_snapshot(&snapshot);
        snapshot_img.set_scale(128);
        snapshot_img.set_rotation(300);
        snapshot_img.center();

        Ok(Self {
            _snapshot_img: snapshot_img,
            _container: container,
            _item0: item0,
            _item1: item1,
            _item2: item2,
            _item3: item3,
            _snapshot: snapshot,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Snapshot1);
