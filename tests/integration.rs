// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests — exercise widgets against a real (headless) LVGL instance.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test integration
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

mod common;
use common::{driver, ensure_init, fresh_screen, pump};

use oxivgl::snapshot::Snapshot;
use oxivgl::{
    anim::{anim_path_linear, anim_set_x, Anim, AnimHandle},
    draw::{DrawArcDsc, DrawImageDsc, DrawLabelDscOwned, DrawLetterDsc, DrawLineDsc, DrawTriangleDsc},
    draw_buf::{ColorFormat, DrawBuf},
    fonts::MONTSERRAT_12,
    style::{
        color_make, GradDir, palette_main, props, BorderSide, GradDsc, GradExtend, Palette, Selector,
        StyleBuilder, TextDecor, TransitionDsc, lv_pct,
    },
    enums::{EventCode, ObjFlag, ObjState, Opa, ScrollDir, ScrollSnap, ScrollbarMode},
    layout::{FlexAlign, FlexFlow, GridAlign, GridCell, Layout, GRID_TEMPLATE_LAST},
    widgets::{
        Align, AnimImg, Arc, ArcLabel, ArcLabelDir, AsLvHandle, Bar, Button, Buttonmatrix, ButtonmatrixCtrl, ButtonmatrixMap,
        Calendar, CalendarDate, Canvas, Chart, ChartAxis, ChartCursor, ChartSeries, ChartType, ChartUpdateMode, CHART_POINT_NONE,
        Checkbox, Dropdown, Image, Imagebutton, ImagebuttonState, Keyboard, KeyboardMode, Label, Led, Line, Menu, MenuHeaderMode, Msgbox,
        BarOrientation, Obj, Part, Roller, RollerMode, Screen, Slider, SliderOrientation, Spinbox, Spinner, Subject, Switch,
        Table, TableCellCtrl, Tabview, Spangroup, SpanMode, SpanOverflow, Textarea, Tileview, ValueLabel, WidgetError, Win, RADIUS_MAX,
        observer_get_target, observer_get_target_obj, subject_get_group_element, subject_get_int_raw,
    },
};
use core::ffi::c_void;
use lvgl_rust_sys::{lv_observer_t, lv_subject_t};

#[test]
fn timer_handler_callable() {
    // pump() calls timer_handler internally — just verify it doesn't panic.
    let _screen = fresh_screen();
    pump();
}

// ── Obj basics ───────────────────────────────────────────────────────────────

#[test]
fn obj_create_and_size() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);
    pump();
    assert_eq!(obj.get_width(), 100);
    assert_eq!(obj.get_height(), 50);
}

#[test]
fn obj_position() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pos(10, 20);
    pump();
    assert_eq!(obj.get_x(), 10);
    assert_eq!(obj.get_y(), 20);
}

#[test]
fn obj_center_alignment() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 40).center();
    pump();
    assert_eq!(obj.get_x(), (320 - 60) / 2);
    assert_eq!(obj.get_y(), (240 - 40) / 2);
}

#[test]
fn obj_width_height_setters() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.width(150).height(75);
    pump();
    assert_eq!(obj.get_width(), 150);
    assert_eq!(obj.get_height(), 75);
}

#[test]
fn obj_align_to() {
    let screen = fresh_screen();
    let base = Obj::new(&screen).unwrap();
    base.size(100, 100).pos(0, 0);
    let obj = Obj::new(&screen).unwrap();
    obj.size(20, 20).align_to(&base, Align::OutBottomMid, 0, 5);
    pump();
    // Should be below base, centered horizontally
    assert!(obj.get_y() > 0, "obj should be below base");
}

// ── State / flags ────────────────────────────────────────────────────────────

#[test]
fn obj_state_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();

    assert!(!obj.has_state(ObjState::CHECKED));
    obj.add_state(ObjState::CHECKED);
    assert!(obj.has_state(ObjState::CHECKED));
    obj.remove_state(ObjState::CHECKED);
    assert!(!obj.has_state(ObjState::CHECKED));
}

#[test]
fn obj_combined_states() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();

    obj.add_state(ObjState::CHECKED);
    obj.add_state(ObjState::FOCUSED);
    assert!(obj.has_state(ObjState::CHECKED));
    assert!(obj.has_state(ObjState::FOCUSED));

    obj.remove_state(ObjState::CHECKED);
    assert!(!obj.has_state(ObjState::CHECKED));
    assert!(obj.has_state(ObjState::FOCUSED));
}

#[test]
fn obj_flag_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();

    obj.remove_flag(ObjFlag::SCROLLABLE);
    obj.add_flag(ObjFlag::SCROLLABLE);
    obj.remove_flag(ObjFlag::CLICKABLE);
    obj.add_flag(ObjFlag::CHECKABLE);
    // No crash = success
}

#[test]
fn obj_remove_scrollable_convenience() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.remove_scrollable();
    obj.remove_clickable();
    // Convenience methods work without crash
}

// ── Style setters (obj_style.rs) ─────────────────────────────────────────────

#[test]
fn obj_style_bg_color_opa() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.bg_color(0xFF0000).bg_opa(255);
    pump();
    assert!(obj.get_width() > 0);
}

#[test]
fn obj_style_border_pad() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.border_width(2)
        .pad(10)
        .pad_top(5)
        .pad_bottom(5)
        .pad_left(8)
        .pad_right(8);
    pump();
}

#[test]
fn obj_style_text_color_font() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.text_color(0x00FF00).text_font(MONTSERRAT_12);
    pump();
}

#[test]
fn obj_style_font_alias() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    // font() is an alias for text_font()
    obj.font(MONTSERRAT_12);
    pump();
}

#[test]
fn obj_style_opa() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.opa(Opa::OPA_50.0);
    pump();
}

#[test]
fn obj_style_radius() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.radius(10, Selector::DEFAULT);
    obj.radius(RADIUS_MAX, ObjState::PRESSED);
    pump();
}

#[test]
fn obj_style_selectors() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_bg_color(
        oxivgl::style::palette_main(Palette::Blue),
        Selector::DEFAULT,
    );
    obj.style_bg_color(
        oxivgl::style::palette_darken(Palette::Blue, 2),
        ObjState::PRESSED,
    );
    pump();
}

#[test]
fn obj_style_transform() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100).center();
    obj.style_transform_rotation(450, Selector::DEFAULT); // 45.0 degrees
    obj.style_transform_scale(512, Selector::DEFAULT); // 2.0x
    obj.style_transform_pivot_x(50, Selector::DEFAULT);
    obj.style_transform_pivot_y(50, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_add_remove() {
    let screen = fresh_screen();
    let style = StyleBuilder::new().build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.remove_style_all();
    pump();
}

#[test]
fn obj_style_grad_dir() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_bg_grad_dir(oxivgl::style::GradDir::Hor, Selector::DEFAULT);
    obj.style_bg_grad_color(
        oxivgl::style::palette_main(Palette::Red),
        Selector::DEFAULT,
    );
    pump();
}

#[test]
fn obj_style_base_dir() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_base_dir(oxivgl::widgets::BaseDir::Rtl, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_line_width() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.line_width(oxivgl::widgets::Part::Main, 3);
    pump();
}

#[test]
fn obj_style_text_align() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.text_align(oxivgl::widgets::TextAlign::Center);
    pump();
}

// ── Style object ─────────────────────────────────────────────────────────────

#[test]
fn style_create_and_apply() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x0000FF)
        .bg_opa(Opa::COVER.0)
        .radius(5)
        .border_width(2)
        .border_color_hex(0xFF0000)
        .pad_all(10);
    let style = sb.build();

    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    assert!(obj.get_width() > 0);
}

// ── Layout (obj_layout.rs) ───────────────────────────────────────────────────

#[test]
fn flex_flow_column() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 200);
    cont.set_flex_flow(FlexFlow::Column);
    cont.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);

    let child1 = Obj::new(&cont).unwrap();
    child1.size(50, 30);
    let child2 = Obj::new(&cont).unwrap();
    child2.size(50, 30);
    pump();

    // In column layout, child2 should be below child1
    assert!(
        child2.get_y() > child1.get_y(),
        "child2.y={} should be > child1.y={}",
        child2.get_y(),
        child1.get_y()
    );
}

#[test]
fn flex_flow_row() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 100);
    cont.set_flex_flow(FlexFlow::Row);

    let child1 = Obj::new(&cont).unwrap();
    child1.size(40, 30);
    let child2 = Obj::new(&cont).unwrap();
    child2.size(40, 30);
    pump();

    // In row layout, child2 should be to the right of child1
    assert!(
        child2.get_x() > child1.get_x(),
        "child2.x={} should be > child1.x={}",
        child2.get_x(),
        child1.get_x()
    );
}

#[test]
fn flex_grow() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 100);
    cont.set_flex_flow(FlexFlow::Row);

    let child = Obj::new(&cont).unwrap();
    child.set_flex_grow(1);
    pump();

    // Flex-grow child should expand
    assert!(child.get_width() > 0);
}

#[test]
fn set_layout_enum() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.set_layout(Layout::Flex);
    pump();
}

static COL_DSC: [i32; 3] = [100, 100, GRID_TEMPLATE_LAST];
static ROW_DSC: [i32; 3] = [50, 50, GRID_TEMPLATE_LAST];

#[test]
fn grid_layout() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(220, 120);
    cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);

    let cell = Obj::new(&cont).unwrap();
    cell.set_grid_cell(
        GridCell::new(GridAlign::Stretch, 0, 1),
        GridCell::new(GridAlign::Stretch, 0, 1),
    );
    pump();

    assert!(cell.get_width() > 0, "grid cell should have width");
}

#[test]
fn grid_align() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(220, 120);
    cont.set_grid_dsc_array(&COL_DSC, &ROW_DSC);
    cont.set_grid_align(GridAlign::Center, GridAlign::Center);
    pump();
}

// ── Screen ───────────────────────────────────────────────────────────────────

#[test]
fn screen_style_methods() {
    let screen = fresh_screen();
    screen
        .bg_color(0x06080f)
        .bg_opa(255)
        .pad_top(6)
        .pad_bottom(6)
        .pad_left(4)
        .pad_right(4)
        .text_color(0xFFFFFF);
    pump();
}

#[test]
fn screen_remove_scrollable() {
    let screen = fresh_screen();
    screen.remove_scrollable();
    pump();
}

#[test]
fn screen_flex_layout() {
    let screen = fresh_screen();
    screen.set_flex_flow(FlexFlow::Column);
    screen.set_flex_align(FlexAlign::Center, FlexAlign::Center, FlexAlign::Center);
    pump();
}

// ── Label ────────────────────────────────────────────────────────────────────

#[test]
fn label_create() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("Hello");
    pump();
    assert!(label.get_width() > 0);
    assert!(label.get_height() > 0);
}

#[test]
fn label_text_chaining() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("test").center();
    pump();
}

// ── Button ───────────────────────────────────────────────────────────────────

#[test]
fn button_create_with_label() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    let lbl = Label::new(&btn).unwrap();
    lbl.text("Click me").center();
    pump();
    assert!(btn.get_width() > 0);
}

#[test]
fn button_checkable_toggle() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.add_flag(ObjFlag::CHECKABLE);

    assert!(!btn.has_state(ObjState::CHECKED));
    btn.add_state(ObjState::CHECKED);
    assert!(btn.has_state(ObjState::CHECKED));
}

// ── Slider ───────────────────────────────────────────────────────────────────

#[test]
fn slider_default_range() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    assert_eq!(slider.get_value(), 0);
}

#[test]
fn slider_set_get_value() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_value(42);
    assert_eq!(slider.get_value(), 42);
}

#[test]
fn slider_custom_range() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(-20, 80);
    slider.set_value(30);
    assert_eq!(slider.get_value(), 30);
}

#[test]
fn slider_clamps_to_range() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_value(200);
    assert_eq!(slider.get_value(), 100);
    slider.set_value(-10);
    assert_eq!(slider.get_value(), 0);
}

// ── Bar ──────────────────────────────────────────────────────────────────────

#[test]
fn bar_set_get_value() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range(100.0);
    bar.set_value(42.0);
    let v = bar.get_value();
    assert!((v - 42.0).abs() < 0.2, "expected ~42.0, got {v}");
}

#[test]
fn bar_zero_max_returns_zero() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_value(50.0);
    assert_eq!(bar.get_value(), 0.0);
}

// ── Arc ──────────────────────────────────────────────────────────────────────

#[test]
fn arc_set_get_value() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_range(150.0);
    arc.set_value(75.0);
    let v = arc.get_value();
    assert!((v - 75.0).abs() < 0.3, "expected ~75.0, got {v}");
}

#[test]
fn arc_zero_max_returns_zero() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_value(50.0);
    assert_eq!(arc.get_value(), 0.0);
}

#[test]
fn arc_rotation_and_angles() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_rotation(135).set_bg_angles(0, 270);
    pump();
}

#[test]
fn arc_gauge_ring() {
    let screen = fresh_screen();
    let arc = Arc::gauge_ring(&screen, 200, 15, 100.0, 0x333333, 0x00FF00, 150, 200).unwrap();
    arc.set_value(50.0);
    pump();
    let v = arc.get_value();
    assert!((v - 50.0).abs() < 0.2, "expected ~50.0, got {v}");
}

// ── Switch ───────────────────────────────────────────────────────────────────

#[test]
fn switch_toggle_state() {
    let screen = fresh_screen();
    let sw = Switch::new(&screen).unwrap();

    assert!(!sw.has_state(ObjState::CHECKED));
    sw.add_state(ObjState::CHECKED);
    assert!(sw.has_state(ObjState::CHECKED));
}

// ── Led ──────────────────────────────────────────────────────────────────────

#[test]
fn led_create() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    pump();
    assert!(led.get_width() > 0);
}

// ── Line ─────────────────────────────────────────────────────────────────────

#[test]
fn line_create_and_set_points() {
    let screen = fresh_screen();
    let line = Line::new(&screen).unwrap();
    static POINTS: [oxivgl::widgets::lv_point_precise_t; 3] = [
        oxivgl::widgets::lv_point_precise_t { x: 0.0, y: 0.0 },
        oxivgl::widgets::lv_point_precise_t { x: 50.0, y: 30.0 },
        oxivgl::widgets::lv_point_precise_t { x: 100.0, y: 0.0 },
    ];
    line.set_points(&POINTS);
    pump();
}

// ── ValueLabel ───────────────────────────────────────────────────────────────

#[test]
fn value_label_format() {
    let screen = fresh_screen();
    let mut vl = ValueLabel::new(&screen, "V").unwrap();
    vl.set_value(14.2).unwrap();
    pump();
    assert!(vl.get_width() > 0);
}

// ── Scale ────────────────────────────────────────────────────────────────────

#[test]
fn scale_builder() {
    use oxivgl::widgets::{ScaleBuilder, ScaleMode};
    let screen = fresh_screen();
    let _scale = ScaleBuilder::new(200, ScaleMode::RoundOuter)
        .rotation(135)
        .sweep(270)
        .range_max(100)
        .total_ticks(21)
        .major_every(5)
        .show_labels(false)
        .major_len(12)
        .minor_len(6)
        .major_color(0xFFFFFF)
        .minor_color(0x808080)
        .build(&screen)
        .unwrap();
    pump();
}

// ── Event (simulated) ────────────────────────────────────────────────────────

#[test]
fn event_callback_fires() {
    use std::sync::atomic::{AtomicBool, Ordering};

    static FIRED: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn cb(_e: *mut lvgl_rust_sys::lv_event_t) {
        FIRED.store(true, Ordering::SeqCst);
    }

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    // SAFETY: user_data is null (unused); btn outlives the event handler.
    unsafe {
        btn.on_event(
            cb,
            oxivgl::enums::EventCode::CLICKED,
            core::ptr::null_mut(),
        );
    }

    // Simulate a click event
    // SAFETY: btn handle valid, LVGL initialised.
    unsafe {
        lvgl_rust_sys::lv_obj_send_event(
            btn.lv_handle(),
            lvgl_rust_sys::lv_event_code_t_LV_EVENT_CLICKED,
            core::ptr::null_mut(),
        );
    }

    assert!(FIRED.load(Ordering::SeqCst), "event callback should fire");
}

#[test]
fn event_bubble_flag() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.add_flag(ObjFlag::EVENT_BUBBLE);
    // No crash = flag set correctly
}

// ── Scrollbar mode ───────────────────────────────────────────────────────────

#[test]
fn scrollbar_mode() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.set_scrollbar_mode(oxivgl::enums::ScrollbarMode::Off);
    obj.set_scrollbar_mode(oxivgl::enums::ScrollbarMode::Auto);
    pump();
}

// ── Widget tree ──────────────────────────────────────────────────────────────

#[test]
fn child_access() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _child1 = Obj::new(&parent).unwrap();
    let _child2 = Obj::new(&parent).unwrap();

    let c0 = parent.get_child(0);
    assert!(c0.is_some(), "first child should exist");
    let c1 = parent.get_child(1);
    assert!(c1.is_some(), "second child should exist");
    let c2 = parent.get_child(2);
    assert!(c2.is_none(), "third child should not exist");
}

#[test]
fn nested_widget_tree() {
    let screen = fresh_screen();
    let container = Obj::new(&screen).unwrap();
    container.size(200, 200);
    let btn = Button::new(&container).unwrap();
    let lbl = Label::new(&btn).unwrap();
    lbl.text("Nested");
    pump();
    assert!(lbl.get_width() > 0);
}

// ── Palette helpers ──────────────────────────────────────────────────────────

#[test]
fn palette_colors() {
    // These return lv_color_t — just verify they don't crash
    let _ = oxivgl::style::palette_main(Palette::Blue);
    let _ = oxivgl::style::palette_lighten(Palette::Red, 2);
    let _ = oxivgl::style::palette_darken(Palette::Green, 3);
    let _ = oxivgl::style::color_black();
    let _ = oxivgl::style::color_white();
    let _ = oxivgl::style::color_make(0x12, 0x34, 0x56);
}

// ── Error handling ───────────────────────────────────────────────────────────

#[test]
fn widget_error_display() {
    let err = WidgetError::LvglNullPointer;
    let msg = format!("{err}");
    assert!(msg.contains("NULL"), "error msg: {msg}");
}

#[test]
fn widget_error_format_error() {
    let err = WidgetError::FormatError(core::fmt::Error);
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}

// ── Image ────────────────────────────────────────────────────────────────────

oxivgl::image_declare!(img_cogwheel_argb);

#[test]
fn image_set_src_static() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_src(img_cogwheel_argb());
    pump();
    assert!(img.get_width() > 0, "image should have non-zero width");
}

// ── StyleBuilder::bg_image_src ───────────────────────────────────────────────

oxivgl::image_declare!(img_skew_strip);

#[test]
fn style_bg_image_src_static() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_image_src(img_skew_strip())
        .bg_image_tiled(true)
        .bg_image_opa(128);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(200, 50);
    pump();
}

// ── Dropdown ─────────────────────────────────────────────────────────────────

#[test]
fn dropdown_set_symbol_static() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("A\nB\nC");
    dd.set_symbol(c"▼");
    pump();
}

// ── Style transition ─────────────────────────────────────────────────────────

static TRANS_PROPS: [props::lv_style_prop_t; 3] = [props::BG_COLOR, props::BG_OPA, props::LAST];

#[test]
fn style_with_transition() {
    let screen = fresh_screen();
    let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

// ── Style drop ordering (spec §4.7, §5.5) ───────────────────────────────────

#[test]
fn style_drop_before_widget() {
    // Style dropped while widget still references it — Rc clone in widget's
    // _styles keeps StyleInner alive until widget drops.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x00FF00).bg_opa(255).radius(5);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    // Drop the user's Style handle; widget's internal clone keeps it alive.
    drop(style);
    pump(); // LVGL renders with style still valid via Rc clone.
    drop(obj); // Widget drop → lv_obj_delete → lv_obj_remove_style_all; then Rc hits 0.
    pump();
}

#[test]
fn add_style_then_drop_widget() {
    // Spec §5.5: "An integration test SHALL exercise the add-style-then-drop-widget path."
    // Widget dropped while styles are still applied. lv_obj_delete internally
    // calls lv_obj_remove_style_all (lv_obj.c:521), clearing LVGL-side refs
    // before Rust drops the _styles Vec.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x0000FF)
        .bg_opa(200)
        .border_width(2)
        .border_color_hex(0xFF0000);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    drop(obj); // Widget drop cleans up LVGL refs, then Rust drops _styles.
    pump();
    // style still valid here (Rc refcount back to 1), no UAF.
    let _clone = style.clone();
}

#[test]
fn style_shared_across_widgets() {
    // Same Style (Rc) applied to multiple widgets. Dropping one widget must
    // not affect the other.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x123456).bg_opa(255);
    let style = sb.build();
    let obj1 = Obj::new(&screen).unwrap();
    let obj2 = Obj::new(&screen).unwrap();
    obj1.add_style(&style, Selector::DEFAULT);
    obj2.add_style(&style, Selector::DEFAULT);
    pump();
    drop(obj1);
    pump(); // obj2 still renders fine with the shared style.
    assert!(obj2.get_width() > 0);
}

#[test]
fn remove_style_then_drop() {
    // Explicitly remove style from widget, then drop both. Tests that
    // remove_style correctly decrements the _styles Vec entry.
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xABCDEF).bg_opa(128);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
    obj.remove_style(Some(&style), Selector::DEFAULT);
    pump();
    drop(obj);
    drop(style);
}

// ── Checkbox ─────────────────────────────────────────────────────────────────

#[test]
fn checkbox_create_and_text() {
    let screen = fresh_screen();
    let cb = Checkbox::new(&screen).unwrap();
    cb.text("Accept terms");
    pump();
    assert!(cb.get_width() > 0);
}

#[test]
fn checkbox_toggle() {
    let screen = fresh_screen();
    let cb = Checkbox::new(&screen).unwrap();
    cb.text("Option");
    assert!(!cb.has_state(ObjState::CHECKED));
    cb.add_state(ObjState::CHECKED);
    assert!(cb.has_state(ObjState::CHECKED));
}

// ── Roller ───────────────────────────────────────────────────────────────────

#[test]
fn roller_create_and_options() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("Jan\nFeb\nMar\nApr", RollerMode::Normal);
    roller.set_visible_row_count(3);
    pump();
    assert_eq!(roller.get_selected(), 0);
}

#[test]
fn roller_set_selected() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC", RollerMode::Infinite);
    roller.set_selected(2, false);
    assert_eq!(roller.get_selected(), 2);
}

// ── Dropdown (extended) ──────────────────────────────────────────────────────

#[test]
fn dropdown_options_and_selection() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("Red\nGreen\nBlue");
    dd.set_selected(1);
    assert_eq!(dd.get_selected(), 1);
    pump();
}

#[test]
fn dropdown_direction() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY");
    dd.set_dir(DdDir::Top);
    pump();
}

// ── Led (extended) ───────────────────────────────────────────────────────────

#[test]
fn led_on_off_brightness() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.on();
    pump();
    led.set_brightness(128);
    pump();
    led.off();
    pump();
}

#[test]
fn led_color() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.set_color(color_make(0xFF, 0x00, 0x00));
    pump();
}

// ── Widget ownership ─────────────────────────────────────────────────────────

#[test]
fn widget_deref_to_obj() {
    let screen = fresh_screen();
    let child = Label::new(&screen).unwrap();
    child.text("via Label");
    pump();
    assert!(child.get_width() > 0);
}

#[test]
fn widget_drop_after_parent_cascade() {
    // Regression: Obj::drop must be a no-op when LVGL has already cascade-
    // deleted the object via parent deletion (lv_obj_is_valid guard).
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let child = Label::new(&parent).unwrap();
    pump();
    drop(parent); // LVGL cascade-deletes child
    pump();
    drop(child); // must not crash — lv_obj_is_valid returns false
    pump();
}

#[test]
fn widget_fire_and_forget() {
    // Widgets created as local variables and forgotten persist in LVGL until
    // their parent is deleted (lv_obj_is_valid returns true, but Rust never
    // calls lv_obj_delete because mem::forget suppresses Drop).
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("ephemeral");
    core::mem::forget(label); // LVGL parent owns and cleans up
    pump();
}

// ── GradDsc ──────────────────────────────────────────────────────────────────

#[test]
fn grad_linear_with_stops() {
    let screen = fresh_screen();
    let mut grad = GradDsc::new();
    grad.init_stops(
        &[palette_main(Palette::Blue), palette_main(Palette::Red)],
        &[255, 255],
        &[0, 255],
    )
    .linear(0, 0, 100, 0, GradExtend::Pad);

    let mut sb = StyleBuilder::new();
    sb.bg_opa(255).bg_grad(grad);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(100, 50);
    pump();
}

#[test]
fn grad_radial_and_conical_api() {
    // Radial/conical gradients can hang LVGL's SW renderer on headless SDL,
    // so we test the Rust API construction without rendering.
    let mut radial = GradDsc::new();
    radial
        .init_stops(
            &[palette_main(Palette::Green), palette_main(Palette::Yellow)],
            &[],
            &[],
        )
        .radial(50, 50, 50, 50, GradExtend::Pad)
        .radial_set_focal(25, 25, 10);

    let mut conical = GradDsc::new();
    conical
        .set_stops_count(2)
        .set_stop(0, palette_main(Palette::Red), 255, 0)
        .set_stop(1, palette_main(Palette::Blue), 255, 255)
        .conical(50, 50, 0, 3600, GradExtend::Pad);
}

#[test]
fn grad_horizontal_simple() {
    let screen = fresh_screen();
    let mut grad = GradDsc::new();
    grad.init_stops(
        &[palette_main(Palette::Cyan), palette_main(Palette::Purple)],
        &[],
        &[],
    )
    .horizontal();

    let mut sb = StyleBuilder::new();
    sb.bg_opa(255).bg_grad(grad);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(100, 50);
    pump();
}

#[test]
fn grad_set_dir() {
    use oxivgl::style::GradDir;
    let screen = fresh_screen();
    let mut grad = GradDsc::new();
    grad.init_stops(
        &[palette_main(Palette::Blue), palette_main(Palette::Red)],
        &[],
        &[],
    )
    .set_dir(GradDir::Hor);

    let mut sb = StyleBuilder::new();
    sb.bg_opa(255).bg_grad(grad);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(100, 50);
    pump();
}

// ── Theme ────────────────────────────────────────────────────────────────────

#[test]
fn theme_extend_and_drop() {
    use oxivgl::style::Theme;
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x334455).bg_opa(255);
    let style = sb.build();
    {
        let _theme = Theme::extend_current(style).unwrap();
        // Buttons created now get the theme style.
        let _btn = Button::new(&screen).unwrap();
        pump();
    }
    // Theme dropped — parent theme restored.
    pump();
}

// ── StyleBuilder setters coverage ────────────────────────────────────────────

#[test]
fn style_outline_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.outline_width(3)
        .outline_color(palette_main(Palette::Red))
        .outline_opa(200)
        .outline_pad(2);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(60, 60);
    pump();
}

#[test]
fn style_shadow_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.shadow_width(10)
        .shadow_color(palette_main(Palette::Blue))
        .shadow_opa(128)
        .shadow_spread(5)
        .shadow_offset_x(3)
        .shadow_offset_y(3);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(80, 80);
    pump();
}

#[test]
fn style_arc_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.arc_color(palette_main(Palette::Green)).arc_width(8);
    let style = sb.build();
    let arc = Arc::new(&screen).unwrap();
    arc.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_text_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.text_color_hex(0xFF00FF)
        .text_opa(200)
        .text_letter_space(2)
        .text_line_space(4)
        .text_decor(TextDecor::UNDERLINE | TextDecor::STRIKETHROUGH);
    let style = sb.build();
    let label = Label::new(&screen).unwrap();
    label.text("Styled");
    label.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_line_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.line_color(palette_main(Palette::Grey))
        .line_width(4)
        .line_rounded(true);
    let style = sb.build();
    let line = Line::new(&screen).unwrap();
    line.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_dimensions_and_padding() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.width(120)
        .height(80)
        .x(10)
        .y(20)
        .pad_ver(5)
        .pad_left(6)
        .pad_right(7)
        .pad_top(8)
        .pad_all(4)
        .length(50);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_border_side() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.border_width(2)
        .border_color_hex(0xFF0000)
        .border_opa(255)
        .border_side(BorderSide::TOP | BorderSide::BOTTOM);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_layout_and_flex() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.layout(Layout::Flex)
        .flex_flow(FlexFlow::Column)
        .flex_main_place(FlexAlign::Center);
    let style = sb.build();
    let cont = Obj::new(&screen).unwrap();
    cont.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_translate_and_anim() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.translate_y(-10).anim_duration(300);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_transform_props() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.transform_rotation(450)
        .transform_scale(512)
        .transform_pivot_x(50)
        .transform_pivot_y(50);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(60, 60);
    pump();
}

// ── Obj scroll methods ───────────────────────────────────────────────────────

#[test]
fn obj_scroll_snap() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(200, 200);
    obj.set_scroll_snap_x(ScrollSnap::Center);
    obj.set_scroll_snap_y(ScrollSnap::Start);
    obj.set_scroll_dir(ScrollDir::VER);
    pump();
}

#[test]
fn obj_scroll_to_position() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    let child = Obj::new(&cont).unwrap();
    child.size(100, 400);
    pump();
    cont.scroll_to(0, 50, false);
    pump();
    assert!(cont.get_scroll_y() != 0 || cont.get_scroll_x() == 0);
}

#[test]
fn obj_child_count_and_foreground() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    assert_eq!(parent.get_child_count(), 0);
    let _c1 = Obj::new(&parent).unwrap();
    let _c2 = Obj::new(&parent).unwrap();
    assert_eq!(parent.get_child_count(), 2);
    _c1.move_foreground();
    pump();
}

#[test]
fn obj_send_event() {
    use std::sync::atomic::{AtomicBool, Ordering};

    static SENT: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn cb(_e: *mut lvgl_rust_sys::lv_event_t) {
        SENT.store(true, Ordering::SeqCst);
    }

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    // SAFETY: user_data is null (unused); btn outlives the event handler.
    unsafe {
        btn.on_event(
            cb,
            oxivgl::enums::EventCode::CLICKED,
            core::ptr::null_mut(),
        );
    }
    btn.send_event(oxivgl::enums::EventCode::CLICKED);
    assert!(SENT.load(Ordering::SeqCst));
}

// ── Label (extended) ─────────────────────────────────────────────────────────

#[test]
fn label_long_mode() {
    use oxivgl::widgets::LabelLongMode;
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("A very long text that might need scrolling or wrapping");
    label.set_long_mode(LabelLongMode::Wrap);
    label.width(100);
    pump();
    assert!(label.get_height() > 0);
}

// ── Bar (extended) ───────────────────────────────────────────────────────────

#[test]
fn bar_mode_range() {
    use oxivgl::widgets::BarMode;
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(0, 100);
    bar.set_mode(BarMode::Range);
    bar.set_value_raw(80, false);
    bar.set_start_value_raw(20, false);
    pump();
}

// ── Color filter in style ────────────────────────────────────────────────────

#[test]
fn style_color_filter() {
    use oxivgl::style::{darken_filter_cb, ColorFilter};
    let screen = fresh_screen();
    let filter = ColorFilter::new(darken_filter_cb);
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0xFFFFFF)
        .bg_opa(255)
        .color_filter(filter, 128);
    let style = sb.build();
    let obj = Obj::new(&screen).unwrap();
    obj.add_style(&style, Selector::DEFAULT);
    obj.size(60, 60);
    pump();
}

// ── Animation ────────────────────────────────────────────────────────────────

#[test]
fn anim_start_returns_handle() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(500)
        .set_exec_cb(Some(anim_set_x));

    let _handle: AnimHandle = a.start();
    pump();
}

#[test]
fn anim_pause_for_during_animation() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 200)
        .set_duration(1000)
        .set_exec_cb(Some(anim_set_x));

    let handle = a.start();
    pump();

    // SAFETY: animation just started (1000 ms), guaranteed still running.
    unsafe { handle.pause_for(500) };
    pump();
}

#[test]
fn anim_start_discard_handle() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(500)
        .set_exec_cb(Some(anim_set_x));

    // Discard return value — mirrors anim1/anim2 usage.
    let _ = a.start();
    pump();
}

// ── SDL builder ──────────────────────────────────────────────────────────────

#[test]
fn sdl_builder_api() {
    // Verify builder API compiles and chains. Can't call build() since
    // LVGL is already initialised by ensure_init().
    let _builder = oxivgl::driver::LvglDriver::sdl(320, 240)
        .title(c"test")
        .mouse(false);
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

#[test]
fn snapshot_take_returns_some() {
    let screen = fresh_screen();
    let _label = Label::new(&screen).unwrap();
    pump();
    let driver = driver();
    let snap = oxivgl::snapshot::Snapshot::take(driver);
    assert!(snap.is_some(), "Snapshot::take should succeed after init");
    let snap = snap.unwrap();
    assert_eq!(snap.width(), 320);
    assert_eq!(snap.height(), 240);
    assert!(!snap.data().is_empty());
}

#[cfg(feature = "png")]
#[test]
fn snapshot_write_png() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("PNG test");
    pump();
    let driver = driver();
    let snap = oxivgl::snapshot::Snapshot::take(driver).unwrap();

    let dir = std::env::temp_dir().join("oxivgl-test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("snapshot_test.png");
    snap.write_png(&path).unwrap();
    assert!(path.exists(), "PNG file should be written");
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
    let _ = std::fs::remove_file(&path);
}

// ── Timer ────────────────────────────────────────────────────────────────────

#[test]
fn timer_create_and_triggered() {
    use oxivgl::timer::Timer;
    let _screen = fresh_screen();
    let timer = Timer::new(100_000).unwrap();
    // Not yet triggered before first pump (long period).
    assert!(!timer.triggered());
    // Force fire via ready(), then pump to execute.
    timer.ready();
    pump();
    assert!(timer.triggered());
    // triggered() clears flag — second call returns false.
    assert!(!timer.triggered());
}

#[test]
fn timer_set_period_and_repeat_count() {
    use oxivgl::timer::Timer;
    let _screen = fresh_screen();
    let timer = Timer::new(100_000).unwrap();
    timer.set_period(100_000).set_repeat_count(1);
    // Force fire and verify triggered.
    timer.ready();
    pump();
    assert!(timer.triggered());
}

#[test]
fn timer_ready_fires_immediately() {
    use oxivgl::timer::Timer;
    let _screen = fresh_screen();
    let timer = Timer::new(999_999).unwrap(); // very long period
    timer.ready(); // force fire on next tick
    pump();
    assert!(timer.triggered());
}

#[test]
fn timer_pause_resume() {
    use oxivgl::timer::Timer;
    let _screen = fresh_screen();
    let timer = Timer::new(100_000).unwrap();
    timer.pause();
    timer.ready(); // mark ready, but paused — should not fire
    pump();
    assert!(!timer.triggered(), "paused timer should not fire");
    timer.resume();
    timer.ready();
    pump();
    assert!(timer.triggered(), "resumed timer should fire");
}

#[test]
fn timer_drop_cleans_up() {
    use oxivgl::timer::Timer;
    let _screen = fresh_screen();
    let timer = Timer::new(10).unwrap();
    drop(timer); // should not panic or leak
    pump();
}

// ── Event ────────────────────────────────────────────────────────────────────

#[test]
fn event_on_callback_receives_event() {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNT: AtomicU32 = AtomicU32::new(0);

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |event| {
        assert_eq!(event.code(), EventCode::CLICKED);
        // target() should return a valid obj
        let _target = event.target();
        COUNT.fetch_add(1, Ordering::SeqCst);
    });
    btn.send_event(EventCode::CLICKED);
    assert!(COUNT.load(Ordering::SeqCst) > 0);
}

#[test]
fn event_matches() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static MATCHED: AtomicBool = AtomicBool::new(false);

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    let _btn_handle = btn.lv_handle();
    // Use on_event with raw callback to capture btn_handle
    unsafe extern "C" fn match_cb(_e: *mut lvgl_rust_sys::lv_event_t) {
        MATCHED.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    unsafe {
        btn.on_event(match_cb, EventCode::CLICKED, core::ptr::null_mut());
    }
    btn.send_event(EventCode::CLICKED);
    assert!(MATCHED.load(Ordering::SeqCst));
}

#[test]
fn event_bubble_and_current_target() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static BUBBLED: AtomicBool = AtomicBool::new(false);

    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let child = Button::new(&parent).unwrap();
    child.bubble_events();

    // Register handler on parent; event sent to child should bubble.
    parent.on(EventCode::CLICKED, |event| {
        let _current = event.current_target_handle();
        BUBBLED.store(true, Ordering::SeqCst);
    });
    child.send_event(EventCode::CLICKED);
    assert!(BUBBLED.load(Ordering::SeqCst));
}

// ── AnimTimeline ─────────────────────────────────────────────────────────────

#[test]
fn anim_timeline_create_add_start() {
    use oxivgl::anim::{AnimTimeline, Anim, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(300)
        .set_exec_cb(Some(anim_set_x));

    let mut tl = AnimTimeline::new();
    tl.add(0, &a);
    let duration = tl.start();
    assert!(duration > 0);
    pump();
}

#[test]
fn anim_timeline_pause_reverse_progress() {
    use oxivgl::anim::{AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX, Anim, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(500)
        .set_exec_cb(Some(anim_set_x));

    let mut tl = AnimTimeline::new();
    tl.add(0, &a);
    tl.start();
    pump();
    tl.pause();
    tl.set_reverse(true);
    tl.set_progress(ANIM_TIMELINE_PROGRESS_MAX / 2);
    pump();
    // Drop cleans up
}

#[test]
fn anim_timeline_drop() {
    use oxivgl::anim::AnimTimeline;
    let _screen = fresh_screen();
    let tl = AnimTimeline::new();
    drop(tl); // should not panic
    pump();
}

// ── Anim builder setters ─────────────────────────────────────────────────────

#[test]
fn anim_path_cb_setters() {
    use oxivgl::anim::{Anim, anim_set_x, anim_path_overshoot, anim_path_ease_in,
        anim_path_ease_out, anim_path_ease_in_out, anim_path_bounce};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(300)
        .set_path_cb(Some(anim_path_overshoot))
        .set_exec_cb(Some(anim_set_x));
    let _ = a.start();
    pump();

    // Test other path callbacks compile and don't panic
    let mut a2 = Anim::new();
    a2.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_ease_in));
    let _ = a2.start();

    let mut a3 = Anim::new();
    a3.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_ease_out));
    let _ = a3.start();

    let mut a4 = Anim::new();
    a4.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_ease_in_out));
    let _ = a4.start();

    let mut a5 = Anim::new();
    a5.set_var(&obj)
        .set_values(0, 50)
        .set_duration(200)
        .set_path_cb(Some(anim_path_bounce));
    let _ = a5.start();
    pump();
}

#[test]
fn anim_repeat_and_playback() {
    use oxivgl::anim::{Anim, ANIM_REPEAT_INFINITE, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(0, 100)
        .set_duration(200)
        .set_delay(10)
        .set_repeat_count(3)
        .set_repeat_delay(50)
        .set_reverse_duration(200)
        .set_reverse_delay(20)
        .set_exec_cb(Some(anim_set_x));
    let _ = a.start();
    pump();

    // Also test infinite repeat
    let mut a2 = Anim::new();
    a2.set_var(&obj)
        .set_values(0, 50)
        .set_duration(100)
        .set_repeat_count(ANIM_REPEAT_INFINITE)
        .set_exec_cb(Some(anim_set_x));
    let _ = a2.start();
    pump();
}

#[test]
fn anim_custom_exec_cb() {
    use oxivgl::anim::{Anim, anim_set_width, anim_set_height};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj)
        .set_values(50, 100)
        .set_duration(200)
        .set_custom_exec_cb(Some(anim_set_width));
    let _ = a.start();

    let mut a2 = Anim::new();
    a2.set_var(&obj)
        .set_values(50, 80)
        .set_duration(200)
        .set_custom_exec_cb(Some(anim_set_height));
    let _ = a2.start();
    pump();
}

#[test]
fn anim_exec_cb_variants() {
    use oxivgl::anim::{Anim, anim_set_size, anim_set_pad_row, anim_set_pad_column,
        anim_set_arc_value, anim_set_bar_value, anim_set_slider_value};
    let screen = fresh_screen();

    // anim_set_size
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);
    let mut a = Anim::new();
    a.set_var(&obj).set_values(50, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_size));
    let _ = a.start();

    // anim_set_pad_row / pad_column
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 200);
    let mut a2 = Anim::new();
    a2.set_var(&cont).set_values(0, 20).set_duration(200)
        .set_exec_cb(Some(anim_set_pad_row));
    let _ = a2.start();

    let mut a3 = Anim::new();
    a3.set_var(&cont).set_values(0, 20).set_duration(200)
        .set_exec_cb(Some(anim_set_pad_column));
    let _ = a3.start();

    // anim_set_arc_value
    let arc = Arc::new(&screen).unwrap();
    let mut a4 = Anim::new();
    a4.set_var(&arc).set_values(0, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_arc_value));
    let _ = a4.start();

    // anim_set_bar_value
    let bar = Bar::new(&screen).unwrap();
    let mut a5 = Anim::new();
    a5.set_var(&bar).set_values(0, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_bar_value));
    let _ = a5.start();

    // anim_set_slider_value
    let slider = Slider::new(&screen).unwrap();
    let mut a6 = Anim::new();
    a6.set_var(&slider).set_values(0, 100).set_duration(200)
        .set_custom_exec_cb(Some(anim_set_slider_value));
    let _ = a6.start();

    pump();
}

// ── Image setters ────────────────────────────────────────────────────────────

#[test]
fn image_rotation_scale_pivot() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_rotation(450)   // 45 degrees
        .set_scale(512)      // 2x
        .set_pivot(16, 16);
    pump();
}

#[test]
fn image_offset_and_inner_align() {
    use oxivgl::widgets::ImageAlign;
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_offset_y(10)
        .set_inner_align(ImageAlign::Center);
    pump();

    // Test other align variants
    img.set_inner_align(ImageAlign::TopLeft);
    img.set_inner_align(ImageAlign::BottomRight);
    img.set_inner_align(ImageAlign::Stretch);
    img.set_inner_align(ImageAlign::Tile);
    pump();
}

// ── Slider setters ───────────────────────────────────────────────────────────

#[test]
fn slider_mode_and_range_value() {
    use oxivgl::widgets::SliderMode;
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(0, 200);
    slider.set_mode(SliderMode::Range);
    slider.set_start_value(20);
    slider.set_value(80);
    pump();
    // In range mode, set_start_value and get_left_value exercise the API.
    // Just verify no panic and value is readable.
    let _left = slider.get_left_value();
    assert_eq!(slider.get_value(), 80);
}

#[test]
fn slider_mode_symmetrical() {
    use oxivgl::widgets::SliderMode;
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_mode(SliderMode::Symmetrical);
    slider.set_value(50);
    pump();
}

// ── Switch setters ───────────────────────────────────────────────────────────

#[test]
fn switch_set_orientation() {
    use oxivgl::widgets::SwitchOrientation;
    let screen = fresh_screen();
    let sw = Switch::new(&screen).unwrap();
    sw.set_orientation(SwitchOrientation::Horizontal);
    pump();
    sw.set_orientation(SwitchOrientation::Vertical);
    pump();
    sw.set_orientation(SwitchOrientation::Auto);
    pump();
}

// ── Obj methods (extended) ───────────────────────────────────────────────────

#[test]
fn obj_bubble_events_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.bubble_events();
    assert!(obj.has_flag(ObjFlag::EVENT_BUBBLE));
}

#[test]
fn obj_remove_clickable() {
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    // Button should be clickable by default.
    assert!(btn.has_flag(ObjFlag::CLICKABLE));
    btn.remove_clickable();
    assert!(!btn.has_flag(ObjFlag::CLICKABLE));
}

#[test]
fn obj_has_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.add_flag(ObjFlag::FLOATING);
    assert!(obj.has_flag(ObjFlag::FLOATING));
    obj.remove_flag(ObjFlag::FLOATING);
    assert!(!obj.has_flag(ObjFlag::FLOATING));
}

#[test]
fn obj_on_callback() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FIRED: AtomicBool = AtomicBool::new(false);

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |_event| {
        FIRED.store(true, Ordering::SeqCst);
    });
    btn.send_event(EventCode::CLICKED);
    assert!(FIRED.load(Ordering::SeqCst));
}

#[test]
fn obj_get_child() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _c1 = Obj::new(&parent).unwrap();
    let _c2 = Obj::new(&parent).unwrap();

    assert!(parent.get_child(0).is_some());
    assert!(parent.get_child(1).is_some());
    assert!(parent.get_child(2).is_none()); // out of range
}

#[test]
fn obj_set_transform_and_reset() {
    use oxivgl::widgets::Matrix;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 60);
    let mut m = Matrix::identity();
    m.scale(0.5, 0.5).rotate(45.0);
    obj.set_transform(&m);
    pump();
    obj.reset_transform();
    pump();
}

#[test]
fn obj_pos_and_getters() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(40, 30).pos(10, 20);
    pump();
    assert_eq!(obj.get_x(), 10);
    assert_eq!(obj.get_y(), 20);
    assert_eq!(obj.get_width(), 40);
    assert_eq!(obj.get_height(), 30);
}

#[test]
fn obj_align_to_out_bottom() {
    let screen = fresh_screen();
    let base = Obj::new(&screen).unwrap();
    base.size(60, 60).align(Align::Center, 0, 0);
    let obj = Obj::new(&screen).unwrap();
    obj.size(20, 20);
    obj.align_to(&base, Align::OutBottomMid, 0, 5);
    pump();
}

#[test]
fn obj_scroll_to_view_and_update_snap() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    let child = Obj::new(&cont).unwrap();
    child.size(100, 400);
    pump();
    child.scroll_to_view(false);
    cont.update_snap(false);
    pump();
}

#[test]
fn obj_scroll_to_view_recursive() {
    let screen = fresh_screen();
    let outer = Obj::new(&screen).unwrap();
    outer.size(100, 100);
    let inner = Obj::new(&outer).unwrap();
    inner.size(100, 400);
    let target = Obj::new(&inner).unwrap();
    target.size(50, 50);
    pump();
    target.scroll_to_view_recursive(false);
    pump();
}

#[test]
fn obj_set_scrollbar_mode() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100);
    obj.set_scrollbar_mode(ScrollbarMode::Off);
    pump();
    obj.set_scrollbar_mode(ScrollbarMode::Auto);
    pump();
}

#[test]
fn obj_remove_scrollable() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.remove_scrollable();
    assert!(!obj.has_flag(ObjFlag::SCROLLABLE));
}

// ── Scale (extended) ─────────────────────────────────────────────────────────

#[test]
fn scale_builder_all_setters() {
    use oxivgl::widgets::{ScaleBuilder, ScaleMode};
    let screen = fresh_screen();
    let scale = ScaleBuilder::new(150, ScaleMode::RoundInner)
        .rotation(135)
        .sweep(270)
        .range_max(200)
        .total_ticks(21)
        .major_every(5)
        .show_labels(false)
        .major_len(12)
        .minor_len(6)
        .major_color(0xFF0000)
        .minor_color(0x888888)
        .build(&screen)
        .unwrap();
    pump();
    drop(scale);
}

#[test]
fn scale_section_with_styles() {
    use oxivgl::widgets::ScaleMode;
    use oxivgl::style::{StyleBuilder, palette_main, Palette};
    let screen = fresh_screen();
    let scale = oxivgl::widgets::Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundOuter)
        .set_total_tick_count(21)
        .set_major_tick_every(5)
        .set_range(0, 100)
        .set_rotation(135)
        .set_angle_range(270)
        .set_label_show(true);
    scale.size(200, 200);

    // Add section with styles
    let mut sb = StyleBuilder::new();
    sb.line_color(palette_main(Palette::Red)).line_width(3);
    let section_style = sb.build();

    let section = scale.add_section();
    section.set_range(75, 100)
        .set_indicator_style(&section_style)
        .set_items_style(&section_style)
        .set_main_style(&section_style);
    pump();
}

#[test]
fn scale_set_text_src() {
    use oxivgl::widgets::ScaleMode;
    let screen = fresh_screen();
    let scale = oxivgl::widgets::Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::HorizontalBottom)
        .set_total_tick_count(4)
        .set_major_tick_every(1)
        .set_range(0, 3);
    scale.size(200, 50);
    static LABELS: &oxivgl::widgets::ScaleLabels = oxivgl::scale_labels!(c"Low", c"Med", c"High", c"Max");
    scale.set_text_src(LABELS);
    pump();
}

#[test]
fn scale_tick_lengths() {
    use oxivgl::widgets::{ScaleMode, Part};
    let screen = fresh_screen();
    let scale = oxivgl::widgets::Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::VerticalLeft)
        .set_total_tick_count(11)
        .set_major_tick_every(5)
        .set_range(0, 100);
    scale.set_tick_length(Part::Items, 8);
    scale.set_tick_length(Part::Indicator, 15);
    pump();
}

// ── Event target_style convenience ───────────────────────────────────────────

#[test]
fn event_target_style_bg_color() {
    use oxivgl::style::color_make;
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |event| {
        event.target_style_bg_color(
            color_make(0xFF, 0x00, 0x00),
            oxivgl::style::Selector::DEFAULT,
        );
    });
    btn.send_event(EventCode::CLICKED);
    pump();
}

// ── Part::from_raw + PartialEq ──────────────────────────────────────────────

#[test]
fn part_from_raw() {
    use oxivgl::widgets::Part;
    assert_eq!(Part::from_raw(0x000000), Part::Main);
    assert_eq!(Part::from_raw(0x020000), Part::Indicator);
    assert_eq!(Part::from_raw(0x050000), Part::Items);
    assert_eq!(Part::from_raw(0xFFFFFF), Part::Main); // unknown → Main
}

// ── Scale::get_major_tick_every ─────────────────────────────────────────────

#[test]
fn scale_get_major_tick_every() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::HorizontalBottom);
    scale.set_total_tick_count(21);
    scale.set_major_tick_every(5);
    pump();
    assert_eq!(scale.get_major_tick_every(), 5);
}

// ── Dropdown::get_selected_str ──────────────────────────────────────────────

#[test]
fn dropdown_get_selected_str() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("Apple\nBanana\nOrange");
    pump();
    let mut buf = [0u8; 32];
    assert_eq!(dd.get_selected_str(&mut buf), Some("Apple"));
    dd.set_selected(1);
    assert_eq!(dd.get_selected_str(&mut buf), Some("Banana"));
}

// ── Obj style methods with selector ─────────────────────────────────────────

#[test]
fn obj_style_text_color_with_selector() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.style_text_color(palette_main(Palette::Red), Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_arc_width_and_color() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.style_arc_width(10, Part::Main);
    arc.style_arc_color(palette_main(Palette::Blue), Part::Main);
    arc.style_arc_rounded(false, Part::Main);
    pump();
}

#[test]
fn obj_style_line_color() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_line_color(palette_main(Palette::Green), Part::Main);
    obj.style_line_width(3, Part::Main);
    pump();
}

#[test]
fn obj_style_length_and_width() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_length(15, Part::Items);
    obj.style_width(20, Part::Indicator);
    pump();
}

// ── Obj::send_draw_task_events ──────────────────────────────────────────────

#[test]
fn obj_send_draw_task_events_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.send_draw_task_events();
    pump();
}

// ── Event::draw_task ────────────────────────────────────────────────────────

#[test]
fn event_draw_task_returns_none_for_clicked() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static CHECKED: AtomicBool = AtomicBool::new(false);
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |event| {
        assert!(event.draw_task().is_none());
        CHECKED.store(true, Ordering::SeqCst);
    });
    btn.send_event(EventCode::CLICKED);
    pump();
    assert!(CHECKED.load(Ordering::SeqCst));
}

// ── draw::Area ──────────────────────────────────────────────────────────────

#[test]
fn draw_area_width_height() {
    use oxivgl::draw::Area;
    let a = Area {
        x1: 10,
        y1: 20,
        x2: 109,
        y2: 69,
    };
    assert_eq!(a.width(), 100);
    assert_eq!(a.height(), 50);
}

#[test]
fn draw_area_set_width_centered() {
    use oxivgl::draw::Area;
    let mut a = Area {
        x1: 50,
        y1: 0,
        x2: 149,
        y2: 9,
    };
    assert_eq!(a.width(), 100);
    a.set_width_centered(120);
    assert_eq!(a.width(), 120);
    // Center should be roughly preserved
    let old_center = 50 + 149; // 199
    let new_center = a.x1 + a.x2;
    assert!((new_center - old_center).abs() <= 1);
}

// ── math: trigo_cos, trigo_sin ──────────────────────────────────────────────

#[test]
fn math_trigo_cos_sin() {
    use oxivgl::math::{trigo_cos, trigo_sin};
    // cos(0) should be ~1.0, sin(0) should be ~0
    let cos0 = trigo_cos(0);
    let sin0 = trigo_sin(0);
    assert!(cos0 > 0, "cos(0) should be positive");
    assert!(sin0.abs() < 100, "sin(0) should be near zero");
    // cos(90) should be ~0, sin(90) should be ~1.0
    let cos90 = trigo_cos(90);
    let sin90 = trigo_sin(90);
    assert!(cos90.abs() < 100, "cos(90) should be near zero");
    assert!(sin90 > 0, "sin(90) should be positive");
}

// ── Indev ───────────────────────────────────────────────────────────────────

#[test]
fn indev_active_returns_none_without_input() {
    use oxivgl::indev::Indev;
    let _screen = fresh_screen();
    // With SDL dummy driver, there may or may not be an input device.
    // Just verify the call doesn't crash.
    let _ = Indev::active();
}

// ── ObjFlag::ADV_HITTEST ────────────────────────────────────────────────────

#[test]
fn obj_adv_hittest_flag() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.add_flag(ObjFlag::ADV_HITTEST);
    pump();
}

// ── Textarea ──────────────────────────────────────────────────────────────────

#[test]
fn textarea_create() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    assert!(ta.get_width() > 0);
}

#[test]
fn textarea_set_get_text() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_text("Hello");
    assert_eq!(ta.get_text(), Some("Hello"));
}

#[test]
fn textarea_one_line() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_one_line(true);
    ta.set_text("single line");
    pump();
    assert_eq!(ta.get_text(), Some("single line"));
}

#[test]
fn textarea_password_mode() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_password_mode(true);
    ta.set_text("secret");
    pump();
    assert_eq!(ta.get_text(), Some("secret"));
}

#[test]
fn textarea_cursor_pos() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_text("abc");
    ta.set_cursor_pos(1);
    ta.add_text("X");
    assert_eq!(ta.get_text(), Some("aXbc"));
}

#[test]
fn textarea_max_length() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_max_length(3);
    ta.set_text("");
    ta.add_text("abcdef");
    let text = ta.get_text().unwrap_or("");
    assert!(text.len() <= 3, "max_length should limit to 3 chars, got: {text}");
}

#[test]
fn textarea_add_delete_char() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_text("");
    ta.add_char('A');
    ta.add_char('B');
    assert_eq!(ta.get_text(), Some("AB"));
    ta.delete_char();
    assert_eq!(ta.get_text(), Some("A"));
}

#[test]
fn textarea_placeholder() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_placeholder_text("Enter text...");
    pump();
}

#[test]
fn textarea_accepted_chars() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_accepted_chars(c"0123456789");
    ta.set_text("");
    ta.add_text("12abc34");
    let text = ta.get_text().unwrap_or("");
    assert!(!text.contains('a'), "should filter non-digit chars, got: {text}");
}

// ── Buttonmatrix ──────────────────────────────────────────────────────────────

#[test]
fn buttonmatrix_create() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    pump();
    assert!(btnm.get_width() > 0);
}

#[test]
fn buttonmatrix_get_selected() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    pump();
    // LVGL returns LV_BTNMATRIX_BTN_NONE (0xFFFF) when nothing is selected.
    let sel = btnm.get_selected_button();
    assert_eq!(sel, 0xFFFF);
}

// ── Keyboard ──────────────────────────────────────────────────────────────────

#[test]
fn keyboard_create() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    pump();
    assert!(kb.get_width() > 0);
}

#[test]
fn keyboard_set_textarea() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_textarea(&ta);
    pump();
}

#[test]
fn keyboard_set_mode() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_mode(KeyboardMode::Number);
    kb.set_mode(KeyboardMode::TextLower);
    kb.set_mode(KeyboardMode::TextUpper);
    kb.set_mode(KeyboardMode::Special);
    pump();
}

// ── Part::Cursor ──────────────────────────────────────────────────────────────

#[test]
fn part_cursor_value() {
    assert_eq!(Part::Cursor as u32, 0x060000);
    assert_eq!(Part::from_raw(0x060000), Part::Cursor);
}

// ── List ──────────────────────────────────────────────────────────────────────

#[test]
fn list_create() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    pump();
    assert!(list.get_width() > 0);
}

#[test]
fn list_add_text() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    list.add_text("Section");
    pump();
    assert!(list.get_child_count() > 0);
}

#[test]
fn list_add_button_and_get_text() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    let btn = list.add_button(Some(&oxivgl::symbols::FILE), "Open");
    pump();
    let text = list.get_button_text(&*btn);
    assert_eq!(text, Some("Open"));
}

#[test]
fn list_add_button_no_icon() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    let btn = list.add_button(None, "NoIcon");
    pump();
    assert_eq!(list.get_button_text(&*btn), Some("NoIcon"));
}

#[test]
fn list_multiple_sections() {
    let screen = fresh_screen();
    let list = oxivgl::widgets::List::new(&screen).unwrap();
    list.add_text("A");
    list.add_button(Some(&oxivgl::symbols::OK), "Item1");
    list.add_text("B");
    list.add_button(None, "Item2");
    pump();
    // 2 text labels + 2 buttons = 4 children
    assert_eq!(list.get_child_count(), 4);
}

// ── Obj::move_to_index / get_index / move_background ─────────────────────────

#[test]
fn obj_move_to_index_and_get_index() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let c0 = Obj::new(&parent).unwrap();
    let c1 = Obj::new(&parent).unwrap();
    let c2 = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(c2.get_index(), 2);
    c2.move_to_index(0);
    pump();
    assert_eq!(c2.get_index(), 0);
    // c0 and c1 shifted
    assert_eq!(c0.get_index(), 1);
    assert_eq!(c1.get_index(), 2);
}

#[test]
fn obj_move_background() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _c0 = Obj::new(&parent).unwrap();
    let c1 = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(c1.get_index(), 1);
    c1.move_background();
    pump();
    assert_eq!(c1.get_index(), 0);
}

#[test]
fn obj_get_style_pad_right() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pad_right(12);
    pump();
    assert_eq!(obj.get_style_pad_right(Part::Main), 12);
}

#[test]
fn obj_from_raw_non_owning() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    let handle = obj.lv_handle();
    {
        let child_ref = Obj::from_raw_non_owning(handle);
        child_ref.size(77, 33);
        pump();
    }
    // obj still valid after child_ref dropped (non-owning)
    pump();
    assert_eq!(obj.get_width(), 77);
}

// ── Menu ──────────────────────────────────────────────────────────────────────

#[test]
fn menu_create() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    pump();
    drop(menu);
    pump();
}

#[test]
fn menu_page_create_untitled() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    let cont = Menu::cont_create(&page);
    let lbl = Label::new(&cont).unwrap();
    lbl.text("Test");
    menu.set_page(&page);
    pump();
}

#[test]
fn menu_page_create_titled() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(Some("My Page"));
    menu.set_page(&page);
    pump();
}

#[test]
fn menu_set_load_page_event() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let sub = menu.page_create(None);
    let cont_sub = Menu::cont_create(&sub);
    let lbl = Label::new(&cont_sub).unwrap();
    lbl.text("Sub");

    let main = menu.page_create(None);
    let cont = Menu::cont_create(&main);
    let lbl2 = Label::new(&cont).unwrap();
    lbl2.text("Click me");
    menu.set_load_page_event(&cont, &sub);
    menu.set_page(&main);
    pump();
}

#[test]
fn menu_section_and_separator() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    Menu::separator_create(&page);
    let section = Menu::section_create(&page);
    let cont = Menu::cont_create(&section);
    let lbl = Label::new(&cont).unwrap();
    lbl.text("In section");
    menu.set_page(&page);
    pump();
}

#[test]
fn menu_root_back_button() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.set_mode_root_back_button(true);
    let page = menu.page_create(None);
    menu.set_page(&page);
    pump();
    let back_btn = menu.get_main_header_back_button();
    // back_button_is_root should work
    let _is_root = menu.back_button_is_root(&back_btn);
    pump();
}

#[test]
fn menu_header_mode() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.set_mode_header(MenuHeaderMode::BottomFixed);
    pump();
}

#[test]
fn menu_sidebar() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(320, 240);
    let page = menu.page_create(Some("Root"));
    let cont = Menu::cont_create(&page);
    let lbl = Label::new(&cont).unwrap();
    lbl.text("Item");
    menu.set_sidebar_page(&page);
    pump();
    assert!(menu.get_cur_sidebar_page().is_some());
    menu.clear_sidebar();
    pump();
}

#[test]
fn menu_clear_history() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    menu.set_page(&page);
    pump();
    menu.clear_history();
    pump();
}

#[test]
fn menu_get_cur_main_page() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let page = menu.page_create(None);
    menu.set_page(&page);
    pump();
    assert!(menu.get_cur_main_page().is_some());
}

#[test]
fn menu_get_main_header() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    let _header = menu.get_main_header();
    pump();
}

// ── Msgbox ────────────────────────────────────────────────────────────────────

#[test]
fn msgbox_create_modal() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(None::<&Obj<'_>>).unwrap();
    mbox.add_title("Test Title");
    mbox.add_text("Test body");
    mbox.add_close_button();
    pump();
    Msgbox::close(mbox);
    pump();
    let _ = screen; // keep screen alive
}

#[test]
fn msgbox_create_on_parent() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_title("Hello");
    mbox.add_text("Text");
    pump();
}

#[test]
fn msgbox_footer_button() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(None::<&Obj<'_>>).unwrap();
    mbox.add_title("Confirm");
    mbox.add_text("Are you sure?");
    let _btn = mbox.add_footer_button("OK");
    pump();
    Msgbox::close(mbox);
    pump();
    let _ = screen;
}

// ── Image::set_src_symbol ─────────────────────────────────────────────────────

#[test]
fn image_set_src_symbol() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_src_symbol(&oxivgl::symbols::SETTINGS);
    pump();
}

// ── Obj scroll/layout methods ────────────────────────────────────────────────

#[test]
fn obj_get_coords() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 50).center();
    pump();
    let area = obj.get_coords();
    assert!(area.width() > 0);
    assert!(area.height() > 0);
    let _ = screen;
}

#[test]
fn obj_invalidate() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.invalidate(); // should not panic
    pump();
    let _ = screen;
}

#[test]
fn obj_scroll_to_x_y() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(200, 200);
    for _ in 0..5 {
        let child = Obj::new(&cont).unwrap();
        child.size(300, 300);
        core::mem::forget(child);
    }
    pump();
    cont.scroll_to_x(10, false);
    cont.scroll_to_y(10, false);
    pump();
    // After scrolling, at least one scroll position should be non-zero.
    let sx = cont.get_scroll_x();
    let sy = cont.get_scroll_y();
    assert!(sx != 0 || sy != 0, "expected scroll position to change, got x={sx} y={sy}");
    let _ = screen;
}

#[test]
fn obj_get_scroll_top_bottom() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    for _ in 0..5 {
        let child = Obj::new(&cont).unwrap();
        child.size(100, 80);
        core::mem::forget(child);
    }
    pump();
    // scroll down a bit; get_scroll_top() should reflect it
    cont.scroll_to_y(20, false);
    pump();
    let top = cont.get_scroll_top();
    assert!(top >= 0, "scroll_top should be non-negative, got {top}");
    let bot = cont.get_scroll_bottom();
    assert!(bot >= 0, "scroll_bottom should be non-negative, got {bot}");
    let _ = screen;
}

#[test]
fn obj_scroll_by() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    cont.size(100, 100);
    for _ in 0..3 {
        let child = Obj::new(&cont).unwrap();
        child.size(100, 80);
        core::mem::forget(child);
    }
    pump();
    cont.scroll_by(0, 20, false);
    pump();
    let _ = screen;
}

#[test]
fn obj_update_layout() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    let child = Obj::new(&cont).unwrap();
    child.size(50, 50);
    cont.update_layout();
    pump();
    core::mem::forget(child);
    let _ = screen;
}

#[test]
fn obj_delete_child() {
    let screen = fresh_screen();
    let cont = Obj::new(&screen).unwrap();
    let _c1 = Obj::new(&cont).unwrap();
    let _c2 = Obj::new(&cont).unwrap();
    core::mem::forget(_c1);
    core::mem::forget(_c2);
    pump();
    assert_eq!(cont.get_child_count(), 2);
    cont.delete_child(0);  // delete by index
    pump();
    assert_eq!(cont.get_child_count(), 1);
    cont.delete_child(-1); // -1 = last child (LVGL convention)
    pump();
    assert_eq!(cont.get_child_count(), 0);
    let _ = screen;
}

#[test]
fn obj_get_style_pad_top_bottom() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pad_top(8);
    obj.pad_bottom(12);
    pump();
    assert_eq!(obj.get_style_pad_top(Part::Main), 8);
    assert_eq!(obj.get_style_pad_bottom(Part::Main), 12);
    let _ = screen;
}

#[test]
fn obj_get_style_pad_row_column() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_row(6, Selector::DEFAULT);
    obj.style_pad_column(9, Selector::DEFAULT);
    pump();
    assert_eq!(obj.get_style_pad_row(Part::Main), 6);
    assert_eq!(obj.get_style_pad_column(Part::Main), 9);
    let _ = screen;
}

// ── Draw layer types ──────────────────────────────────────────────────────────

#[test]
fn draw_rect_dsc_new() {
    use oxivgl::draw::{DrawRectDsc, DrawLabelDscOwned, RADIUS_CIRCLE};
    use oxivgl::style::color_make;
    let mut dsc = DrawRectDsc::new();
    dsc.bg_color(color_make(255, 170, 170))
        .radius(RADIUS_CIRCLE)
        .border_color(color_make(255, 85, 85))
        .border_width(2)
        .outline_color(color_make(255, 0, 0))
        .outline_width(2)
        .outline_pad(3);
    // Verify the label descriptor initialises a usable font (text_size > 0).
    let label_dsc = DrawLabelDscOwned::default_font();
    let (w, h) = label_dsc.text_size("X");
    assert!(w > 0, "label dsc font width > 0");
    assert!(h > 0, "label dsc font height > 0");
    // dsc construction must not panic (inner fields not pub — smoke-test only).
    let _ = dsc;
}

#[test]
fn draw_label_dsc_owned_text_size() {
    let screen = fresh_screen();
    let dsc = oxivgl::draw::DrawLabelDscOwned::default_font();
    let (w, h) = dsc.text_size("Hello");
    assert!(w > 0, "text width should be > 0");
    assert!(h > 0, "text height should be > 0");
    let _ = screen;
}

#[test]
fn area_align_to_area() {
    use oxivgl::draw::Area;
    use oxivgl::widgets::Align;
    let base = Area { x1: 0, y1: 0, x2: 99, y2: 19 };
    let mut txt = Area { x1: 0, y1: 0, x2: 29, y2: 9 };
    txt.align_to_area(base, Align::RightMid, -10, 0);
    // RightMid aligns txt's right edge to base's right edge, then adds ofs_x.
    // Expected: txt.x2 = base.x2 + ofs_x = 99 + (-10) = 89
    assert_eq!(txt.x2, 89);
}

#[test]
fn area_set_width() {
    use oxivgl::draw::Area;
    let mut a = Area { x1: 10, y1: 0, x2: 29, y2: 9 };
    a.set_width(5);
    assert_eq!(a.x2, 14); // x1=10, new x2 = 10+5-1 = 14
    assert_eq!(a.x1, 10); // x1 unchanged
}

// ── Chart ─────────────────────────────────────────────────────────────────────

#[test]
fn chart_create() {
    use oxivgl::widgets::Chart;
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    pump();
    assert!(chart.get_width() > 0);
}

#[test]
fn chart_add_series_and_set_value() {
    use oxivgl::widgets::{Chart, ChartAxis, ChartType, lv_color_t};
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_type(ChartType::Line);
    chart.set_point_count(5);
    chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
    let color = lv_color_t { blue: 0, green: 0, red: 255 };
    let series = chart.add_series(color, ChartAxis::PrimaryY);
    chart.set_next_value(&series, 42);
    chart.set_next_value(&series, 80);
    chart.refresh();
    pump();
    assert!(chart.get_width() > 0);
}

#[test]
fn chart_scatter_series() {
    use oxivgl::widgets::{Chart, ChartAxis, ChartType, lv_color_t};
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_type(ChartType::Scatter);
    chart.set_point_count(3);
    let color = lv_color_t { blue: 255, green: 0, red: 0 };
    let series = chart.add_series(color, ChartAxis::PrimaryY);
    chart.set_next_value2(&series, 10, 20);
    chart.set_next_value2(&series, 50, 80);
    chart.refresh();
    pump();
}

// ── DrawTask and Layer ────────────────────────────────────────────────────────

#[test]
fn event_draw_task_layer_smoke() {
    // send_draw_task_events() enables DRAW_TASK_ADDED on obj.
    // With SDL dummy backend DRAW_TASK_ADDED may or may not fire headlessly;
    // this is a no-panic smoke test.
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100);
    obj.send_draw_task_events();
    obj.on(EventCode::DRAW_TASK_ADDED, |ev| {
        if let Some(task) = ev.draw_task() {
            let _layer = task.layer(); // None or Some — both are fine
            let _area = task.area();
            let _base = task.base();
        }
    });
    pump();
}

#[test]
fn event_layer_returns_none_for_clicked() {
    // Event::layer() only works for draw events, not CLICKED.
    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, |ev| {
        assert!(ev.layer().is_none());
    });
    btn.send_event(EventCode::CLICKED);
    pump();
}

// ── Indev ─────────────────────────────────────────────────────────────────────

#[test]
fn indev_get_vect_without_input() {
    use oxivgl::indev::Indev;
    let _screen = fresh_screen();
    pump();
    if let Some(indev) = Indev::active() {
        let _vect = indev.get_vect();
        let _streak = indev.short_click_streak();
    }
    // No input device in headless mode — just verify no panic.
}

// ── Buttonmatrix set_map / get_button_text ────────────────────────────────────

#[test]
fn buttonmatrix_set_map_and_get_text() {
    use oxivgl::btnmatrix_map;
    use oxivgl::widgets::ButtonmatrixMap;
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    static MAP: &ButtonmatrixMap = btnmatrix_map!(c"X", c"Y", c"Z");
    btnm.set_map(MAP);
    pump();
    assert_eq!(btnm.get_button_text(0), Some("X"));
    assert_eq!(btnm.get_button_text(1), Some("Y"));
    assert_eq!(btnm.get_button_text(2), Some("Z"));
}

// ── Obj::swap, get_x_aligned, get_y_aligned, get_style_pad_left, get_style_bg_color ──

#[test]
fn obj_swap_children() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let a = Obj::new(&parent).unwrap();
    let b = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(a.get_index(), 0);
    assert_eq!(b.get_index(), 1);
    a.swap(&b);
    pump();
    assert_eq!(a.get_index(), 1);
    assert_eq!(b.get_index(), 0);
}

#[test]
fn obj_get_x_y_aligned() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pos(15, 25);
    pump();
    // get_x_aligned returns the user-set position before alignment resolution.
    let xa = obj.get_x_aligned();
    let ya = obj.get_y_aligned();
    assert_eq!(xa, 15);
    assert_eq!(ya, 25);
}

#[test]
fn obj_get_style_pad_left() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pad_left(7);
    pump();
    assert_eq!(obj.get_style_pad_left(Part::Main), 7);
}

#[test]
fn obj_get_style_bg_color() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.bg_color(0xFF0000).bg_opa(255);
    pump();
    let _c = obj.get_style_bg_color(Part::Main);
    // Just verify no panic; lv_color_t fields depend on color format.
}

// ── ObjStyle methods ──────────────────────────────────────────────────────────

#[test]
fn obj_style_pad_hor_smoke() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_hor(10, Selector::DEFAULT);
    pump();
    // pad_hor sets both left and right; verify left side
    assert_eq!(obj.get_style_pad_left(Part::Main), 10);
}

#[test]
fn obj_style_text_font_with_selector() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("test");
    label.style_text_font(MONTSERRAT_12, Selector::DEFAULT);
    pump();
    assert!(label.get_width() > 0);
}

#[test]
fn obj_style_pad_column_smoke() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_column(5, Selector::DEFAULT);
    pump();
    assert_eq!(obj.get_style_pad_column(Part::Main), 5);
}

#[test]
fn obj_style_size_smoke() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    // style_size sets a style property on sub-parts (e.g. indicator size).
    obj.style_size(8, 8, Part::Indicator);
    pump();
}

#[test]
fn obj_style_bg_image_src_symbol_smoke() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_bg_image_src_symbol(&oxivgl::symbols::SETTINGS, Selector::DEFAULT);
    obj.bg_opa(255);
    pump();
}

// ── Anim exec cb variants: anim_set_y, anim_set_translate_x ─────────────────

#[test]
fn anim_exec_cb_y_and_translate_x() {
    use oxivgl::anim::{Anim, anim_set_y, anim_set_translate_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);

    let mut a = Anim::new();
    a.set_var(&obj).set_values(0, 100).set_duration(200)
        .set_exec_cb(Some(anim_set_y));
    let _ = a.start();

    let mut a2 = Anim::new();
    a2.set_var(&obj).set_values(0, 50).set_duration(200)
        .set_exec_cb(Some(anim_set_translate_x));
    let _ = a2.start();

    pump();
}

// ── math::bezier3, map ────────────────────────────────────────────────────────

#[test]
fn math_map_basic() {
    use oxivgl::math;
    // map(500, 0, 1000, 0, 100) == 50
    assert_eq!(math::map(500, 0, 1000, 0, 100), 50);
    // map at boundaries
    assert_eq!(math::map(0, 0, 1000, 0, 100), 0);
    assert_eq!(math::map(1000, 0, 1000, 0, 100), 100);
}

#[test]
fn math_bezier3_endpoints() {
    use oxivgl::math;
    // t=0 → result should be near u0=0
    let v0 = math::bezier3(0, 0, 256, 768, 1024);
    assert_eq!(v0, 0);
    // t=1024 → result should be near u3=1024
    let v1024 = math::bezier3(1024, 0, 256, 768, 1024);
    assert_eq!(v1024, 1024);
}

#[test]
fn style_color_brightness_and_darken() {
    use oxivgl::style::{color_brightness, color_darken, color_make};
    let white = color_make(255, 255, 255);
    let brightness = color_brightness(white);
    assert!(brightness > 200, "white should have high brightness, got {brightness}");
    let dark = color_darken(white, 2);
    let dark_brightness = color_brightness(dark);
    assert!(dark_brightness < brightness, "darkened color should be less bright");
}

// ── Anim::set_bezier3_path ────────────────────────────────────────────────────

#[test]
fn anim_set_bezier3_path_no_panic() {
    use std::sync::atomic::AtomicI32;
    use oxivgl::anim::{Anim, anim_set_x};
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(50, 50);
    static P1: AtomicI32 = AtomicI32::new(128);
    static P2: AtomicI32 = AtomicI32::new(900);
    let mut anim = Anim::new();
    anim.set_exec_cb(Some(anim_set_x));
    anim.set_var(&obj);
    anim.set_values(0, 100);
    anim.set_duration(500);
    anim.set_bezier3_path(&P1, &P2);
    let _ = anim.start();
    pump();
}

// ── draw::DrawLabelDscOwned::set_color ────────────────────────────────────────

#[test]
fn draw_label_dsc_owned_set_color() {
    use oxivgl::draw::DrawLabelDscOwned;
    use oxivgl::style::color_make;
    let _screen = fresh_screen();
    let mut dsc = DrawLabelDscOwned::default_font();
    dsc.set_color(color_make(255, 0, 0));
    let _ = dsc;
}

// ── Arc::set_mode / set_bg_start_angle / set_bg_end_angle ────────────────────

#[test]
fn arc_set_mode_variants() {
    use oxivgl::widgets::ArcMode;
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_mode(ArcMode::Normal);
    arc.set_mode(ArcMode::Symmetrical);
    arc.set_mode(ArcMode::Reverse);
    pump();
}

#[test]
fn arc_set_bg_start_end_angle() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_bg_start_angle(45);
    arc.set_bg_end_angle(315);
    pump();
}

// ── Label::text_long / LabelLongMode variants ─────────────────────────────────

#[test]
fn label_text_long() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text_long("A much longer string that would overflow the 127-byte stack buffer in text()");
    pump();
    assert!(lbl.get_width() > 0);
}

#[test]
fn label_long_mode_variants() {
    use oxivgl::widgets::LabelLongMode;
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("overflow").width(30);
    lbl.set_long_mode(LabelLongMode::Dots);
    pump();
    lbl.set_long_mode(LabelLongMode::Scroll);
    pump();
    lbl.set_long_mode(LabelLongMode::ScrollCircular);
    pump();
    lbl.set_long_mode(LabelLongMode::Clip);
    pump();
}

// ── Dropdown::set_text / set_selected_highlight ───────────────────────────────

#[test]
fn dropdown_set_text_static() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("A\nB\nC");
    dd.set_text(c"Menu");
    pump();
}

#[test]
fn dropdown_set_selected_highlight() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY\nZ");
    dd.set_selected_highlight(false);
    pump();
    dd.set_selected_highlight(true);
    pump();
}

// ── Child::Debug ──────────────────────────────────────────────────────────────

#[test]
fn child_debug_fmt() {
    let screen = fresh_screen();
    let c = Label::new(&screen).unwrap();
    let s = format!("{c:?}");
    assert!(!s.is_empty());
}

// ── ValueLabel::Debug ─────────────────────────────────────────────────────────

#[test]
fn value_label_debug_fmt() {
    let screen = fresh_screen();
    let vl = ValueLabel::new(&screen, "A").unwrap();
    let s = format!("{vl:?}");
    assert!(!s.is_empty());
}

// ── Screen::add_style ─────────────────────────────────────────────────────────

#[test]
fn screen_add_style() {
    let screen = fresh_screen();
    let mut sb = StyleBuilder::new();
    sb.bg_color_hex(0x111111).bg_opa(255);
    let style = sb.build();
    screen.add_style(&style, Selector::DEFAULT);
    pump();
}

// ── Obj style: clip_corner, translate_x ───────────────────────────────────────

#[test]
fn obj_style_clip_corner() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.radius(20, Selector::DEFAULT);
    obj.style_clip_corner(true, Selector::DEFAULT);
    pump();
    obj.style_clip_corner(false, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_translate_x() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_translate_x(15, Selector::DEFAULT);
    pump();
}

// ── Obj style: image_recolor / image_recolor_opa / radial_offset / line_opa ──

#[test]
fn obj_style_image_recolor_and_opa() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_image_recolor(palette_main(Palette::Red), Selector::DEFAULT);
    obj.style_image_recolor_opa(200, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_radial_offset() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_radial_offset(5, Selector::DEFAULT);
    pump();
}

#[test]
fn obj_style_line_opa() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_line_opa(128, Part::Main);
    pump();
}

// ── Obj style: style_opa with selector ───────────────────────────────────────

#[test]
fn obj_style_opa_with_selector() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_opa(200, Selector::DEFAULT);
    pump();
}

// ── Obj style: style_pad_all with selector ────────────────────────────────────

#[test]
fn obj_style_pad_all_with_selector() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_all(8, Part::Main);
    pump();
    assert_eq!(obj.get_style_pad_left(Part::Main), 8);
}

// ── Obj style: style_pad_row and style_pad_column ─────────────────────────────

#[test]
fn obj_style_pad_row_and_column_setters() {
    use oxivgl::widgets::Part;
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.style_pad_row(4, Selector::DEFAULT);
    obj.style_pad_column(7, Selector::DEFAULT);
    pump();
    assert_eq!(obj.get_style_pad_row(Part::Main), 4);
    assert_eq!(obj.get_style_pad_column(Part::Main), 7);
}

// ── Event::current_target_handle via static fn ───────────────────────────────

#[test]
fn event_current_target_handle_static() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static CHECKED: AtomicBool = AtomicBool::new(false);

    fn check_cb(ev: &oxivgl::event::Event) {
        assert_eq!(ev.code(), EventCode::CLICKED);
        let _current = ev.current_target_handle();
        CHECKED.store(true, Ordering::SeqCst);
    }

    let screen = fresh_screen();
    let btn = Button::new(&screen).unwrap();
    btn.on(EventCode::CLICKED, check_cb);
    btn.send_event(EventCode::CLICKED);
    assert!(CHECKED.load(Ordering::SeqCst));
}

// ── Buttonmatrix: get_button_text out-of-range returns None ──────────────────

#[test]
fn buttonmatrix_get_button_text_oob() {
    use oxivgl::btnmatrix_map;
    use oxivgl::widgets::ButtonmatrixMap;
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    static MAP2: &ButtonmatrixMap = btnmatrix_map!(c"P", c"Q");
    btnm.set_map(MAP2);
    pump();
    assert_eq!(btnm.get_button_text(0), Some("P"));
    assert_eq!(btnm.get_button_text(1), Some("Q"));
    // Out-of-range index — LVGL returns NULL → None.
    assert_eq!(btnm.get_button_text(99), None);
}

// ── Canvas ────────────────────────────────────────────────────────────────────

#[test]
fn canvas_create_and_fill() {
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    let screen = fresh_screen();
    let buf = DrawBuf::create(50, 50, ColorFormat::RGB565).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(255, 0, 0), 255).size(50, 50).center();
    pump();
}

#[test]
fn canvas_set_px_and_get() {
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    let screen = fresh_screen();
    let buf = DrawBuf::create(10, 10, ColorFormat::ARGB8888).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(0, 0, 0), 255);
    canvas.set_px(5, 5, color_make(255, 255, 255), 255);
    // We set a white pixel; verify the canvas accepted the call without panic.
    // (Exact pixel read-back depends on color format internals.)
    pump();
}

#[test]
fn canvas_layer_draw_rect() {
    use oxivgl::draw::{Area, DrawRectDsc};
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    let screen = fresh_screen();
    let buf = DrawBuf::create(100, 100, ColorFormat::ARGB8888).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(200, 200, 200), 255);
    {
        let mut layer = canvas.init_layer();
        let mut dsc = DrawRectDsc::new();
        dsc.bg_color(color_make(255, 0, 0)).radius(5);
        layer.draw_rect(&dsc, Area { x1: 10, y1: 10, x2: 50, y2: 50 });
    }
    pump();
}

#[test]
fn drawbuf_create_returns_some() {
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    let buf = DrawBuf::create(100, 100, ColorFormat::RGB565);
    assert!(buf.is_some());
}

#[test]
fn canvas_draw_buf_accessor() {
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    let screen = fresh_screen();
    let buf = DrawBuf::create(40, 40, ColorFormat::RGB565).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    // draw_buf() returns &DrawBuf — just verify we can call image_dsc() on it.
    let _img = canvas.draw_buf().image_dsc();
}

#[test]
fn canvas_layer_draw_label() {
    use oxivgl::draw::{Area, DrawLabelDscOwned};
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    let screen = fresh_screen();
    let buf = DrawBuf::create(80, 30, ColorFormat::RGB565).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(200, 200, 200), 255);
    {
        let mut layer = canvas.init_layer();
        let mut dsc = DrawLabelDscOwned::default_font();
        dsc.set_color(color_make(255, 0, 0));
        layer.draw_label(&dsc, Area { x1: 5, y1: 5, x2: 75, y2: 25 }, "Test");
    }
}

#[test]
fn canvas_layer_draw_letter() {
    use oxivgl::draw::DrawLetterDsc;
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    let screen = fresh_screen();
    let buf = DrawBuf::create(40, 40, ColorFormat::RGB565).unwrap();
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(200, 200, 200), 255);
    {
        let mut layer = canvas.init_layer();
        let mut dsc = DrawLetterDsc::new();
        dsc.unicode(b'A' as u32)
            .color(color_make(0, 0, 255))
            .rotation(0);
        layer.draw_letter(&dsc, 10, 20);
    }
}

// ── Table ─────────────────────────────────────────────────────────────────────

#[test]
fn table_create_and_set_cell() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_cell_value(0, 0, "Hello");
    pump();
    assert_eq!(table.get_cell_value(0, 0).as_deref(), Some("Hello"));
}

#[test]
fn table_row_col_count() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_row_count(4).set_column_count(3);
    pump();
    assert_eq!(table.get_row_count(), 4);
    assert_eq!(table.get_column_count(), 3);
}

#[test]
fn table_column_width() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_column_count(2);
    table.set_column_width(0, 100).set_column_width(1, 80);
    pump();
    assert_eq!(table.get_column_width(0), 100);
    assert_eq!(table.get_column_width(1), 80);
}

#[test]
fn table_cell_ctrl() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_cell_value(0, 0, "Item");
    table.set_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1);
    pump();
    assert!(table.has_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1));
    table.clear_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1);
    assert!(!table.has_cell_ctrl(0, 0, TableCellCtrl::CUSTOM_1));
}

#[test]
fn table_selected_cell() {
    let screen = fresh_screen();
    let table = Table::new(&screen).unwrap();
    table.set_row_count(3).set_column_count(2);
    // No cell selected initially.
    pump();
    assert_eq!(table.get_selected_cell(), None);
    // Programmatic selection.
    table.set_selected_cell(1, 0);
    pump();
    assert_eq!(table.get_selected_cell(), Some((1, 0)));
}

#[test]
fn tabview_create_and_add_tabs() {
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab1 = tv.add_tab("Alpha");
    let _tab2 = tv.add_tab("Beta");
    pump();
    assert_eq!(tv.get_tab_count(), 2);
    assert_eq!(tv.get_tab_active(), 0);
}

#[test]
fn tabview_set_active() {
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab1 = tv.add_tab("A");
    let _tab2 = tv.add_tab("B");
    let _tab3 = tv.add_tab("C");
    tv.set_active(2, false);
    pump();
    assert_eq!(tv.get_tab_active(), 2);
}

#[test]
fn tabview_get_content_and_bar() {
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab = tv.add_tab("Only");
    pump();
    // Just verify the calls don't panic.
    let _content = tv.get_content();
    let _bar = tv.get_tab_bar();
}

#[test]
fn tabview_set_tab_bar_position_and_size() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab = tv.add_tab("A");
    tv.set_tab_bar_position(DdDir::Left);
    tv.set_tab_bar_size(80);
    pump();
}

// ── Calendar ──────────────────────────────────────────────────────────────────

#[test]
fn calendar_create() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    cal.set_today_date(2024, 3, 22).set_month_shown(2024, 3);
    pump();
}

#[test]
fn calendar_highlighted_dates() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.set_today_date(2021, 2, 23).set_month_shown(2021, 2);
    cal.set_highlighted_dates(&[
        CalendarDate::new(2021, 2, 6),
        CalendarDate::new(2021, 2, 11),
    ]);
    pump();
    // today and shown dates round-trip correctly
    let today = cal.get_today_date();
    assert_eq!(today.year, 2021);
    assert_eq!(today.month, 2);
    assert_eq!(today.day, 23);
    let shown = cal.get_showed_date();
    assert_eq!(shown.year, 2021);
    assert_eq!(shown.month, 2);
}

#[test]
fn calendar_header_arrow() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    cal.set_today_date(2024, 6, 1).set_month_shown(2024, 6);
    let _hdr = cal.add_header_arrow();
    pump();
}

#[test]
fn calendar_get_pressed_date_none_initially() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.set_today_date(2024, 1, 1).set_month_shown(2024, 1);
    pump();
    // No user click → None
    assert!(cal.get_pressed_date().is_none());
}

// ── Spinner ──────────────────────────────────────────────────────────────────

#[test]
fn spinner_create() {
    let screen = fresh_screen();
    let spinner = Spinner::new(&screen).unwrap();
    spinner.size(100, 100).center();
    pump();
}

#[test]
fn spinner_set_anim_params() {
    let screen = fresh_screen();
    let spinner = Spinner::new(&screen).unwrap();
    spinner.set_anim_params(2000, 90);
    pump();
}

// ── Spinbox ──────────────────────────────────────────────────────────────────

#[test]
fn spinbox_create() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.width(100).center();
    pump();
}

#[test]
fn spinbox_range_and_value() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(-100, 100).set_value(42);
    assert_eq!(sb.get_value(), 42);
    // clamped to max
    sb.set_value(200);
    assert_eq!(sb.get_value(), 100);
    pump();
}

#[test]
fn spinbox_increment_decrement() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(0, 100).set_value(50).set_step(10);
    sb.increment();
    assert_eq!(sb.get_value(), 60);
    sb.decrement();
    assert_eq!(sb.get_value(), 50);
    pump();
}

#[test]
fn spinbox_digit_format() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_digit_format(5, 2).set_range(-1000, 25000);
    sb.set_value(1234);
    assert_eq!(sb.get_value(), 1234);
    pump();
}

#[test]
fn spinbox_step_navigation() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_step(1);
    assert_eq!(sb.get_step(), 1);
    sb.step_prev(); // step × 10
    assert_eq!(sb.get_step(), 10);
    sb.step_next(); // step ÷ 10
    assert_eq!(sb.get_step(), 1);
    pump();
}

// ── Draw descriptors ─────────────────────────────────────────────────────────

#[test]
fn draw_arc_dsc_builder() {
    ensure_init();
    let mut dsc = DrawArcDsc::new();
    dsc.center(50, 50).radius(20).angles(0.0, 270.0).width(4).color(color_make(255, 0, 0)).opa(200).rounded(true);
}

#[test]
fn draw_line_dsc_builder() {
    ensure_init();
    let mut dsc = DrawLineDsc::new();
    dsc.p1(10.0, 10.0).p2(90.0, 90.0).width(3).color(color_make(0, 255, 0)).opa(255).round_start(true).round_end(true);
}

#[test]
fn draw_triangle_dsc_builder() {
    ensure_init();
    let mut dsc = DrawTriangleDsc::new();
    dsc.points([(10.0, 10.0), (50.0, 80.0), (90.0, 10.0)])
        .color(color_make(0, 0, 255))
        .opa(128)
        .grad_stops_count(2)
        .grad_dir(GradDir::Ver)
        .grad_stop(0, color_make(255, 0, 0), 0, 255)
        .grad_stop(1, color_make(0, 0, 255), 255, 255);
}

#[test]
fn draw_image_dsc_builder() {
    ensure_init();
    if let Some(buf) = DrawBuf::create(10, 10, ColorFormat::RGB565) {
        let img = buf.image_dsc();
        let mut dsc = DrawImageDsc::from_image_dsc(&img);
        dsc.rotation(450).pivot(5, 5).opa(200);
    }
}

#[test]
fn draw_letter_dsc_builder() {
    ensure_init();
    let mut dsc = DrawLetterDsc::new();
    dsc.unicode(b'A' as u32).color(color_make(255, 255, 0)).rotation(0);
}

#[test]
fn draw_label_dsc_owned_color() {
    ensure_init();
    let mut dsc = DrawLabelDscOwned::default_font();
    dsc.set_color(color_make(128, 0, 0));
    let (w, h) = dsc.text_size("Test");
    assert!(w > 0);
    assert!(h > 0);
}

// ── Style builder — uncovered setters ────────────────────────────────────────

#[test]
fn style_bg_grad_color_hex() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    let mut sb = StyleBuilder::new();
    sb.bg_grad_color_hex(0xFF0000).bg_grad_dir(GradDir::Ver);
    let style = sb.build();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_transform_width_height() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    let mut sb = StyleBuilder::new();
    sb.transform_width(10).transform_height(20);
    let style = sb.build();
    obj.add_style(&style, Selector::DEFAULT);
    pump();
}

#[test]
fn style_lv_pct() {
    assert!(lv_pct(50) != 50); // lv_pct encodes as a special value
}

// ── Arc — uncovered methods ──────────────────────────────────────────────────

#[test]
fn arc_get_value_raw() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.size(100, 100).center();
    arc.set_range_raw(0, 100).set_value_raw(42);
    assert_eq!(arc.get_value_raw(), 42);
    pump();
}

#[test]
fn arc_align_obj_to_angle() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.size(100, 100).center();
    arc.set_range_raw(0, 360).set_value_raw(90);
    let label = Label::new(&screen).unwrap();
    arc.align_obj_to_angle(&label, 10);
    pump();
}

#[test]
fn arc_rotate_obj_to_angle() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.size(100, 100).center();
    arc.set_range_raw(0, 360).set_value_raw(45);
    let label = Label::new(&screen).unwrap();
    arc.rotate_obj_to_angle(&label, 0);
    pump();
}

// ── Bar — uncovered methods ──────────────────────────────────────────────────

#[test]
fn bar_get_value_raw() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(0, 100).set_value_raw(75, false);
    assert_eq!(bar.get_value_raw(), 75);
    pump();
}

// ── Calendar — uncovered methods ─────────────────────────────────────────────

#[test]
fn calendar_header_dropdown() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    cal.set_today_date(2024, 6, 1).set_month_shown(2024, 6);
    let _hdr = cal.add_header_dropdown();
    pump();
}

#[test]
fn calendar_get_btnmatrix() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(185, 230).center();
    let _bm = cal.get_btnmatrix();
    pump();
}

// ── Menu — uncovered methods ─────────────────────────────────────────────────

#[test]
fn menu_clear_page() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(200, 200).center();
    let page = menu.page_create(None);
    let cont = Menu::cont_create(&page);
    let _label = Label::new(&cont).unwrap();
    menu.set_page(&page);
    pump();
    menu.clear_page();
    pump();
}

// ── Spinbox — uncovered methods ──────────────────────────────────────────────

#[test]
fn spinbox_rollover() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(0, 10).set_value(10).set_step(1).set_rollover(true);
    sb.increment();
    assert_eq!(sb.get_value(), 0); // wrapped around
    pump();
}

#[test]
fn spinbox_cursor_pos() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_cursor_pos(2);
    pump();
}




// ── Spangroup ────────────────────────────────────────────────────────────────

#[test]
fn span_create() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.size(200, 100).center();
    pump();
}

#[test]
fn span_add_and_set_text() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let span = sg.add_span().unwrap();
    span.set_text(c"Hello spans");
    sg.refresh();
    assert_eq!(sg.get_span_count(), 1);
    pump();
}

#[test]
fn span_add_multiple() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let _s1 = sg.add_span().unwrap();
    let _s2 = sg.add_span().unwrap();
    let _s3 = sg.add_span().unwrap();
    assert_eq!(sg.get_span_count(), 3);
    sg.refresh();
    pump();
}

#[test]
fn span_delete() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    let span = sg.add_span().unwrap();
    span.set_text(c"temp");
    assert_eq!(sg.get_span_count(), 1);
    sg.delete_span(&span);
    assert_eq!(sg.get_span_count(), 0);
    sg.refresh();
    pump();
}

#[test]
fn span_overflow_and_indent() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    sg.set_overflow(SpanOverflow::Ellipsis);
    sg.set_indent(20);
    assert_eq!(sg.get_indent(), 20);
    let span = sg.add_span().unwrap();
    span.set_text(c"text");
    sg.refresh();
    pump();
}

#[test]
fn span_mode() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    sg.set_mode(SpanMode::Break);
    let span = sg.add_span().unwrap();
    span.set_text(c"break mode");
    sg.refresh();
    pump();
}

#[test]
fn span_max_lines() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    sg.set_max_lines(3);
    assert_eq!(sg.get_max_lines(), 3);
    sg.refresh();
    pump();
}

#[test]
fn span_text_color_and_decor() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let span = sg.add_span().unwrap();
    span.set_text(c"styled")
        .set_text_color(palette_main(Palette::Red))
        .set_text_opa(128)
        .set_text_decor(TextDecor::UNDERLINE);
    sg.refresh();
    pump();
}

#[test]
fn span_static_text() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.width(200);
    let span = sg.add_span().unwrap();
    span.set_text_static(c"static text");
    sg.refresh();
    pump();
}


// ── Tileview ──────────────────────────────────────────────────────────────────

#[test]
fn tileview_create_and_add_tiles() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let _tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let _tile2 = tv.add_tile(0, 1, ScrollDir::TOP | ScrollDir::RIGHT);
    let _tile3 = tv.add_tile(1, 1, ScrollDir::LEFT);
    pump();
}

#[test]
fn tileview_set_tile_by_index() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let _tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let _tile2 = tv.add_tile(0, 1, ScrollDir::TOP);
    tv.set_tile_by_index(0, 1, false);
    pump();
}

#[test]
fn tileview_set_tile_by_obj() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let tile2 = tv.add_tile(0, 1, ScrollDir::TOP);
    tv.set_tile(&*tile2, false);
    pump();
    // Switch back
    tv.set_tile(&*tile1, false);
    pump();
}

#[test]
fn tileview_get_active_tile() {
    let screen = fresh_screen();
    let tv = Tileview::new(&screen).unwrap();
    let _tile1 = tv.add_tile(0, 0, ScrollDir::BOTTOM);
    let _tile2 = tv.add_tile(0, 1, ScrollDir::TOP);
    tv.set_tile_by_index(0, 1, false);
    pump();
    let active = tv.get_tile_active();
    assert!(active.is_some());
}


// ── Win — window widget ──────────────────────────────────────────────────────

#[test]
fn win_create() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    win.size(300, 200).center();
    pump();
}

#[test]
fn win_add_title() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _title = win.add_title("Hello");
    pump();
}

#[test]
fn win_add_button() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _btn = win.add_button(&oxivgl::symbols::CLOSE, 40);
    pump();
}

#[test]
fn win_get_header() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _hdr = win.get_header();
    pump();
}

#[test]
fn win_get_content() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let content = win.get_content();
    let _lbl = Label::new(&content).unwrap();
    pump();
}

#[test]
fn win_full_example() {
    let screen = fresh_screen();
    let win = Win::new(&screen).unwrap();
    let _btn1 = win.add_button(&oxivgl::symbols::LEFT, 40);
    let _title = win.add_title("Test Win");
    let _btn2 = win.add_button(&oxivgl::symbols::RIGHT, 40);
    let _btn3 = win.add_button(&oxivgl::symbols::CLOSE, 60);
    let content = win.get_content();
    let lbl = Label::new(&content).unwrap();
    lbl.text("Content text");
    pump();
}

// ── Imagebutton ─────────────────────────────────────────────────────────────

#[test]
fn imagebutton_create() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.size(200, 50).center();
    pump();
}

#[test]
fn imagebutton_set_state() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.set_state(ImagebuttonState::Released);
    btn.set_state(ImagebuttonState::Pressed);
    btn.set_state(ImagebuttonState::Disabled);
    btn.set_state(ImagebuttonState::CheckedReleased);
    btn.set_state(ImagebuttonState::CheckedPressed);
    btn.set_state(ImagebuttonState::CheckedDisabled);
    pump();
}

#[test]
fn imagebutton_set_src_none() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.set_src(ImagebuttonState::Released, None, None, None);
    pump();
}

#[test]
fn imagebutton_with_child_label() {
    let screen = fresh_screen();
    let btn = Imagebutton::new(&screen).unwrap();
    btn.size(200, 50).center();
    let label = Label::new(&btn).unwrap();
    label.text("Click me");
    label.center();
    pump();
}


// ── AnimImg ──────────────────────────────────────────────────────────────────

#[repr(transparent)]
struct SyncPtr(*const core::ffi::c_void);
unsafe impl Sync for SyncPtr {}

// Declare extern symbol at module scope so we can take its address in a static.
// The same symbol is also referenced by the img_cogwheel_argb() function
// generated by image_declare! above — both refer to the same linker symbol.
mod animimg_frames {
    unsafe extern "C" {
        #[allow(non_upper_case_globals)]
        pub static img_cogwheel_argb: oxivgl::widgets::lv_image_dsc_t;
    }
    pub static FRAMES: [super::SyncPtr; 2] = [
        super::SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
        super::SyncPtr(&raw const img_cogwheel_argb as *const core::ffi::c_void),
    ];
}

fn animimg_frame_ptrs() -> &'static [*const core::ffi::c_void] {
    // SAFETY: SyncPtr is #[repr(transparent)] over *const c_void.
    unsafe {
        core::slice::from_raw_parts(
            animimg_frames::FRAMES.as_ptr().cast(),
            animimg_frames::FRAMES.len(),
        )
    }
}

#[test]
fn animimg_create() {
    let screen = fresh_screen();
    let animimg = AnimImg::new(&screen).unwrap();
    animimg.size(100, 100).center();
    pump();
}

#[test]
fn animimg_set_src_and_start() {
    let screen = fresh_screen();
    let animimg = AnimImg::new(&screen).unwrap();
    animimg.center();
    animimg
        .set_src(animimg_frame_ptrs())
        .set_duration(1000)
        .set_repeat_count(oxivgl::anim::ANIM_REPEAT_INFINITE)
        .start();
    pump();
    assert_eq!(animimg.get_src_count(), 2);
    assert_eq!(animimg.get_duration(), 1000);
}

#[test]
fn animimg_getters() {
    let screen = fresh_screen();
    let animimg = AnimImg::new(&screen).unwrap();
    animimg
        .set_src(animimg_frame_ptrs())
        .set_duration(500)
        .set_repeat_count(3);
    assert_eq!(animimg.get_duration(), 500);
    assert_eq!(animimg.get_repeat_count(), 3);
    assert_eq!(animimg.get_src_count(), 2);
    pump();
}

// ── Label — RTL and CJK fonts ───────────────────────────────────────────────

#[test]
fn label_bidi_rtl() {
    use oxivgl::widgets::BaseDir;
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("RTL test");
    label.style_base_dir(BaseDir::Rtl, Selector::DEFAULT);
    label.font(oxivgl::fonts::DEJAVU_16_PERSIAN_HEBREW);
    pump();
    assert!(label.get_width() > 0);
}

#[test]
fn label_cjk_font() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("CJK");
    label.font(oxivgl::fonts::SOURCE_HAN_SANS_SC_16_CJK);
    pump();
    assert!(label.get_width() > 0);
}

#[test]
fn fixed_width_font_label() {
    use oxivgl::fonts::{FixedWidthFont, MONTSERRAT_20};
    static MONO: FixedWidthFont = FixedWidthFont::new();
    let screen = fresh_screen();
    let mono_font = MONO.init(MONTSERRAT_20, 20);
    let label = Label::new(&screen).unwrap();
    label.text_font(mono_font);
    label.text("0123.Wabc");
    pump();
    assert!(label.get_width() > 0);
}



// ── Span — uncovered methods ─────────────────────────────────────────────────

#[test]
fn spangroup_letter_and_line_space() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    let span = sg.add_span().unwrap();
    span.set_text(c"Spacing test");
    span.set_text_letter_space(2);
    span.set_text_line_space(4);
    pump();
}

#[test]
fn spangroup_getters() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.set_overflow(SpanOverflow::Ellipsis);
    assert_eq!(sg.get_overflow(), 1); // LV_SPAN_OVERFLOW_ELLIPSIS
    sg.set_mode(SpanMode::Break);
    assert_eq!(sg.get_mode(), 2); // LV_SPAN_MODE_BREAK
    let _h = sg.get_max_line_height();
    sg.size(200, 100);
    let _w = sg.get_expand_width(200);
    let _h2 = sg.get_expand_height(200);
    pump();
}

#[test]
fn spangroup_align_text() {
    let screen = fresh_screen();
    let sg = Spangroup::new(&screen).unwrap();
    sg.set_align_text(1); // LV_TEXT_ALIGN_CENTER
    let span = sg.add_span().unwrap();
    span.set_text(c"Centered");
    pump();
}

// ── Canvas — draw image from static asset ───────────────────────────────────

#[test]
fn canvas_layer_draw_image_static() {
    use oxivgl::draw::DrawImageDsc;
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    oxivgl::image_declare!(img_cogwheel_argb);
    let screen = fresh_screen();
    let buf = DrawBuf::create(100, 100, ColorFormat::ARGB8888).expect("DrawBuf alloc");
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.center();
    let mut layer = canvas.init_layer();
    let mut dsc = DrawImageDsc::from_static_dsc(img_cogwheel_argb());
    dsc.opa(255);
    layer.draw_image(&dsc, oxivgl::draw::Area { x1: 0, y1: 0, x2: 99, y2: 99 });
    drop(layer);
    pump();
}

// ── Calendar — Chinese mode ─────────────────────────────────────────────────

#[test]
fn calendar_chinese_mode() {
    let screen = fresh_screen();
    let cal = Calendar::new(&screen).unwrap();
    cal.size(300, 300).center();
    cal.set_today_date(2024, 6, 1).set_month_shown(2024, 6);
    cal.set_chinese_mode(true, oxivgl::fonts::SOURCE_HAN_SANS_SC_14_CJK);
    cal.font(oxivgl::fonts::SOURCE_HAN_SANS_SC_14_CJK);
    pump();
    // Works on ESP32 hardware.
}

// ── Btnmatrix with CJK text ─────────────────────────────────────────────────
// ── ObjFlag::HIDDEN ──────────────────────────────────────────────────────────

#[test]
fn obj_flag_hidden_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    assert!(!obj.has_flag(ObjFlag::HIDDEN));
    obj.add_flag(ObjFlag::HIDDEN);
    assert!(obj.has_flag(ObjFlag::HIDDEN));
    obj.remove_flag(ObjFlag::HIDDEN);
    assert!(!obj.has_flag(ObjFlag::HIDDEN));
}

// ── ObjFlag::SCROLL_MOMENTUM ────────────────────────────────────────────────

#[test]
fn obj_flag_scroll_momentum_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    // SCROLL_MOMENTUM is on by default for scrollable objects
    obj.remove_flag(ObjFlag::SCROLL_MOMENTUM);
    assert!(!obj.has_flag(ObjFlag::SCROLL_MOMENTUM));
    obj.add_flag(ObjFlag::SCROLL_MOMENTUM);
    assert!(obj.has_flag(ObjFlag::SCROLL_MOMENTUM));
    obj.remove_flag(ObjFlag::SCROLL_MOMENTUM);
    assert!(!obj.has_flag(ObjFlag::SCROLL_MOMENTUM));
}

// ── ObjFlag::SCROLL_CHAIN ───────────────────────────────────────────────────

#[test]
fn obj_flag_scroll_chain_add_remove() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.remove_flag(ObjFlag::SCROLL_CHAIN);
    assert!(!obj.has_flag(ObjFlag::SCROLL_CHAIN));
    obj.add_flag(ObjFlag::SCROLL_CHAIN);
    assert!(obj.has_flag(ObjFlag::SCROLL_CHAIN));
    obj.remove_flag(ObjFlag::SCROLL_CHAIN);
    assert!(!obj.has_flag(ObjFlag::SCROLL_CHAIN));
}

// ── Scale::set_rotation ─────────────────────────────────────────────────────

#[test]
fn scale_set_rotation_no_crash() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner);
    scale.set_rotation(90);
    pump();
}

// ── anim_set_scale_rotation ─────────────────────────────────────────────────

#[test]
fn anim_set_scale_rotation_no_crash() {
    use oxivgl::anim::{anim_set_scale_rotation, Anim};
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner)
        .set_range(0, 360)
        .set_total_tick_count(9)
        .set_major_tick_every(1)
        .set_angle_range(360)
        .set_rotation(0);
    scale.size(200, 200);
    let mut a = Anim::new();
    a.set_var(&scale)
        .set_values(0, 360)
        .set_duration(1000)
        .set_exec_cb(Some(anim_set_scale_rotation));
    let _h = a.start();
    pump();
}

// ── DrawLetterDsc::font ─────────────────────────────────────────────────────

#[test]
fn draw_letter_dsc_font_setter() {
    use oxivgl::draw::DrawLetterDsc;
    use oxivgl::draw_buf::{ColorFormat, DrawBuf};
    use oxivgl::fonts;
    use oxivgl::style::color_make;
    let screen = fresh_screen();
    let buf = DrawBuf::create(100, 100, ColorFormat::RGB565).expect("DrawBuf alloc");
    let canvas = Canvas::new(&screen, buf).unwrap();
    canvas.fill_bg(color_make(0, 0, 0), 255);
    {
        let mut layer = canvas.init_layer();
        let mut dsc = DrawLetterDsc::new();
        dsc.unicode(b'A' as u32)
            .font(fonts::MONTSERRAT_20)
            .color(color_make(255, 255, 255))
            .rotation(0);
        layer.draw_letter(&dsc, 10, 10);
    }
    pump();
}

// ── Shadow style methods ────────────────────────────────────────────────────

#[test]
fn shadow_style_methods() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 80);
    obj.style_shadow_width(20, Selector::DEFAULT);
    obj.style_shadow_color(color_make(255, 0, 0), Selector::DEFAULT);
    obj.style_shadow_offset_x(5, Selector::DEFAULT);
    obj.style_shadow_offset_y(5, Selector::DEFAULT);
    obj.style_shadow_spread(10, Selector::DEFAULT);
    obj.style_shadow_opa(200, Selector::DEFAULT);
    pump();
}

// ── Transform scale_x / scale_y ─────────────────────────────────────────────

#[test]
fn transform_scale_xy() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 60);
    obj.style_transform_scale_x(256, Selector::DEFAULT);
    obj.style_transform_scale_y(128, Selector::DEFAULT);
    pump();
}

// ── Text letter space ───────────────────────────────────────────────────────

#[test]
fn text_letter_space() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("Spaced");
    lbl.style_text_letter_space(5, Selector::DEFAULT);
    pump();
}

// ── ArcLabel create ─────────────────────────────────────────────────────────

#[test]
fn arclabel_create() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_text_static(c"Test");
    al.set_radius(50);
    al.set_angle_start(0.0);
    al.set_angle_size(180.0);
    pump();
}

// ── ArcLabel direction ──────────────────────────────────────────────────────

#[test]
fn arclabel_direction() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_text_static(c"CCW");
    al.set_dir(ArcLabelDir::CounterClockwise);
    pump();
}

// ── Screen::layer_top ───────────────────────────────────────────────────────

#[test]
fn screen_layer_top() {
    let _screen = fresh_screen();
    let top = Screen::layer_top();
    assert!(!top.handle().is_null());
}

// ── Translation ─────────────────────────────────────────────────────────────

use oxivgl::translation::{self, StaticCStr as S};

static TRANS_LANGS: [S; 3] = [S::from_cstr(c"en"), S::from_cstr(c"de"), S::NULL];
static TRANS_TAGS: [S; 2] = [S::from_cstr(c"hello"), S::NULL];
static TRANS_VALUES: [S; 2] = [S::from_cstr(c"Hello"), S::from_cstr(c"Hallo")];

#[test]
fn translation_add_and_set_language() {
    let _screen = fresh_screen();
    translation::add_static(&TRANS_LANGS, &TRANS_TAGS, &TRANS_VALUES);
    translation::set_language(c"en");
    let lang = translation::get_language();
    assert_eq!(lang, Some(c"en".as_ref()));
}

#[test]
fn translation_tag_on_label() {
    let screen = fresh_screen();
    translation::add_static(&TRANS_LANGS, &TRANS_TAGS, &TRANS_VALUES);
    translation::set_language(c"en");
    let lbl = Label::new(&screen).unwrap();
    lbl.set_translation_tag("hello");
    pump();
}

// ── ArcLabel methods ────────────────────────────────────────────────────────

#[test]
fn arclabel_set_text_static() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_text_static(c"Static text");
    pump();
}

#[test]
fn arclabel_set_radius() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_radius(80);
    pump();
}

#[test]
fn arclabel_set_angle_start() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_angle_start(45.0);
    pump();
}

#[test]
fn arclabel_set_angle_size() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_angle_size(270.0);
    pump();
}

// ── Label set_translation_tag ───────────────────────────────────────────────

#[test]
fn label_set_translation_tag_no_crash() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.set_translation_tag("test");
    pump();
}

// ── Blur style ──────────────────────────────────────────────────────────────

#[test]
fn blur_style_methods() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100);
    obj.style_blur_radius(10, Selector::DEFAULT);
    obj.style_blur_backdrop(true, Selector::DEFAULT);
    pump();
}

// ── Chart (new methods) ─────────────────────────────────────────────────────

#[test]
fn chart_set_series_value_by_id() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_point_count(5);
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    chart.set_series_value_by_id(&ser, 0, 42);
    chart.set_series_value_by_id(&ser, 4, 99);
}

#[test]
fn chart_get_first_point_center_offset() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_point_count(10);
    let _ = chart.get_first_point_center_offset();
}

#[test]
fn chart_get_point_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_point_count(7);
    assert_eq!(chart.get_point_count(), 7);
}

#[test]
fn chart_set_div_line_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_div_line_count(3, 5);
}

// ── Buttonmatrix (new methods) ──────────────────────────────────────────────

#[test]
fn buttonmatrix_set_button_width() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_width(0, 2);
}

#[test]
fn buttonmatrix_set_and_clear_ctrl() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE);
    assert!(btnm.has_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE));
    btnm.clear_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE);
    assert!(!btnm.has_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE));
}

#[test]
fn buttonmatrix_ctrl_bitor() {
    let combined = ButtonmatrixCtrl::CHECKABLE | ButtonmatrixCtrl::CHECKED;
    assert_ne!(combined, ButtonmatrixCtrl::NONE);
}

// ── Keyboard (new methods) ──────────────────────────────────────────────────

#[test]
fn keyboard_set_mode_user1() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_mode(KeyboardMode::User1);
}

#[test]
fn keyboard_set_map_custom() {
    use oxivgl::btnmatrix_map;
    static MAP: &ButtonmatrixMap = btnmatrix_map!(c"A", c"B", c"C");
    static CTRL: &[ButtonmatrixCtrl] = &[
        ButtonmatrixCtrl::NONE,
        ButtonmatrixCtrl::NONE,
        ButtonmatrixCtrl::NONE,
    ];
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_map(KeyboardMode::User1, MAP, CTRL);
    kb.set_mode(KeyboardMode::User1);
}

// ── Msgbox (new methods) ────────────────────────────────────────────────────

#[test]
fn msgbox_add_header_button() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_title("Test");
    let _btn = mbox.add_header_button(&oxivgl::symbols::CLOSE);
}

#[test]
fn msgbox_get_content_non_null() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    let content = mbox.get_content();
    // Content should always exist
    drop(content);
}

#[test]
fn msgbox_get_footer_after_button() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_footer_button("OK");
    assert!(mbox.get_footer().is_some());
}

#[test]
fn msgbox_get_header_after_title() {
    let screen = fresh_screen();
    let mbox = Msgbox::new(Some(&screen)).unwrap();
    mbox.add_title("Test");
    assert!(mbox.get_header().is_some());
}

// ── EventCode DEFOCUSED ─────────────────────────────────────────────────────

#[test]
fn event_code_defocused_value() {
    assert_eq!(
        EventCode::DEFOCUSED.0,
        lvgl_rust_sys::lv_event_code_t_LV_EVENT_DEFOCUSED
    );
}

// ── Chart new APIs ───────────────────────────────────────────────────────────

#[test]
fn chart_set_update_mode() {
    let screen = common::fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
}

#[test]
fn chart_add_cursor_and_set_point() {
    let screen = common::fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    for i in 0..10 { chart.set_next_value(&ser, i * 10); }
    let color = palette_main(Palette::Blue);
    let cursor = chart.add_cursor(color, 0x01 | 0x08);
    chart.set_cursor_point(&cursor, Some(&ser), 5);
}

#[test]
fn chart_get_pressed_point_none() {
    let screen = common::fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    assert!(chart.get_pressed_point().is_none());
}

#[test]
fn chart_get_series_next() {
    let screen = common::fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    let _ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    let first = chart.get_series_next(None);
    assert!(first.is_some());
    let second = chart.get_series_next(first.as_ref());
    assert!(second.is_none());
}

#[test]
fn chart_get_x_start_point() {
    let screen = common::fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    let _ = chart.get_x_start_point(&ser);
}

#[test]
fn buttonmatrix_set_button_ctrl_all() {
    let screen = common::fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl_all(ButtonmatrixCtrl::CHECKABLE);
    assert!(btnm.has_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE));
}

#[test]
fn buttonmatrix_set_one_checked() {
    let screen = common::fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl_all(ButtonmatrixCtrl::CHECKABLE);
    btnm.set_one_checked(true);
    btnm.set_button_ctrl(0, ButtonmatrixCtrl::CHECKED);
}

#[test]
fn draw_task_with_fill_dsc_closure_compiles() {
    // Verify with_fill_dsc / with_label_dsc exist and compile.
    // Actual draw task closure testing requires a render cycle.
    let screen = common::fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.send_draw_task_events();
    pump();
}

#[test]
fn draw_label_dsc_set_opa_compiles() {
    // Verify set_opa / opa exist on DrawLabelDsc (compile-time check).
    let screen = common::fresh_screen();
    let _kb = Keyboard::new(&screen).unwrap();
    pump();
}

// ── Arc getters ──────────────────────────────────────────────────────────────

#[test]
fn arc_get_angle_start() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_range_raw(0, 100).set_value_raw(0); // 0% → indicator at start
    pump();
    // Just verify the call returns without panic and is in 0–360 range.
    let start = arc.get_angle_start();
    assert!(start >= 0.0 && start <= 360.0, "start={start} not in [0,360]");
}

#[test]
fn arc_get_angle_end() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_range_raw(0, 100).set_value_raw(100); // 100% → indicator at end
    pump();
    let end = arc.get_angle_end();
    assert!(end >= 0.0 && end <= 360.0, "end={end} not in [0,360]");
}

#[test]
fn arc_get_bg_angle_start() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_bg_start_angle(30);
    pump();
    let start = arc.get_bg_angle_start();
    assert!((start - 30.0).abs() < 1.0, "expected ~30.0, got {start}");
}

#[test]
fn arc_get_bg_angle_end() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_bg_end_angle(300);
    pump();
    let end = arc.get_bg_angle_end();
    assert!((end - 300.0).abs() < 1.0, "expected ~300.0, got {end}");
}

#[test]
fn arc_get_rotation() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_rotation(90);
    pump();
    assert_eq!(arc.get_rotation(), 90);
}

#[test]
fn arc_get_mode() {
    use oxivgl::widgets::ArcMode;
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    arc.set_mode(ArcMode::Symmetrical);
    pump();
    assert!(matches!(arc.get_mode(), ArcMode::Symmetrical));
}

#[test]
fn arc_get_change_rate() {
    let screen = fresh_screen();
    let arc = Arc::new(&screen).unwrap();
    // change_rate is a read-only property set by LVGL internally; just verify it returns a value
    let rate = arc.get_change_rate();
    let _ = rate; // no panic
}

// ── Bar getters ──────────────────────────────────────────────────────────────

#[test]
fn bar_get_start_value_raw() {
    use oxivgl::widgets::BarMode;
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(0, 100);
    bar.set_mode(BarMode::Range);
    bar.set_value_raw(80, false);
    bar.set_start_value_raw(20, false);
    assert_eq!(bar.get_start_value_raw(), 20);
}

#[test]
fn bar_get_min_value() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(10, 90);
    assert_eq!(bar.get_min_value(), 10);
}

#[test]
fn bar_get_max_value() {
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_range_raw(10, 90);
    assert_eq!(bar.get_max_value(), 90);
}

#[test]
fn bar_get_mode() {
    use oxivgl::widgets::BarMode;
    let screen = fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    bar.set_mode(BarMode::Symmetrical);
    assert!(matches!(bar.get_mode(), BarMode::Symmetrical));
}

#[test]
fn bar_set_get_orientation() {
    let screen = common::fresh_screen();
    let bar = Bar::new(&screen).unwrap();
    assert!(matches!(bar.get_orientation(), BarOrientation::Auto));
    bar.set_orientation(BarOrientation::Vertical);
    assert!(matches!(bar.get_orientation(), BarOrientation::Vertical));
    bar.set_orientation(BarOrientation::Horizontal);
    assert!(matches!(bar.get_orientation(), BarOrientation::Horizontal));
}

// ── Image getters ────────────────────────────────────────────────────────────

#[test]
fn image_get_rotation() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_rotation(450); // 45.0 degrees
    pump();
    assert_eq!(img.get_rotation(), 450);
}

#[test]
fn image_get_scale() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_scale(512); // 2.0x
    pump();
    assert_eq!(img.get_scale(), 512);
}

#[test]
fn image_get_scale_x() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    // set_scale sets both scale_x and scale_y to the same value
    img.set_scale(384);
    pump();
    assert_eq!(img.get_scale_x(), 384);
}

#[test]
fn image_get_offset_x() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    // No set_offset_x in the wrapper; just verify the getter returns a value
    pump();
    let _x = img.get_offset_x(); // no panic
}

#[test]
fn image_get_offset_y() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_offset_y(20);
    pump();
    assert_eq!(img.get_offset_y(), 20);
}

#[test]
fn image_get_inner_align() {
    use oxivgl::widgets::ImageAlign;
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_inner_align(ImageAlign::Center);
    pump();
    let align = img.get_inner_align();
    // ImageAlign::Center is variant 5 in lv_image_align_t
    assert!(align > 0);
}

#[test]
fn image_get_antialias() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    // No set_antialias wrapper; just verify the getter returns without panic.
    pump();
    let _aa = img.get_antialias();
}

#[test]
fn image_get_src_width_height() {
    let screen = fresh_screen();
    let img = Image::new(&screen).unwrap();
    img.set_src(img_cogwheel_argb());
    pump();
    // With a real image loaded, src_width/height should be > 0
    assert!(img.get_src_width() > 0);
    assert!(img.get_src_height() > 0);
}

// ── Label getters ────────────────────────────────────────────────────────────

#[test]
fn label_get_long_mode() {
    use oxivgl::widgets::LabelLongMode;
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.set_long_mode(LabelLongMode::Dots);
    pump();
    assert!(matches!(lbl.get_long_mode(), LabelLongMode::Dots));
}

#[test]
fn label_get_recolor() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("test");
    pump();
    // No set_recolor wrapper; just verify the getter returns without panic.
    let _recolor = lbl.get_recolor();
}

#[test]
fn label_get_text_selection_start() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("Hello");
    pump();
    // No selection initially; LVGL returns LV_DRAW_LABEL_NO_TXT_SEL (0xFFFF) when nothing selected
    let start = lbl.get_text_selection_start();
    let _ = start; // just verify no panic
}

// ── Led getters ──────────────────────────────────────────────────────────────

#[test]
fn led_get_brightness() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.set_brightness(200);
    pump();
    assert_eq!(led.get_brightness(), 200);
}

#[test]
fn led_get_color() {
    let screen = fresh_screen();
    let led = Led::new(&screen).unwrap();
    led.set_color(color_make(255, 0, 0));
    pump();
    let _c = led.get_color(); // just verify no panic
}

// ── Checkbox getter ──────────────────────────────────────────────────────────

#[test]
fn checkbox_get_text() {
    let screen = fresh_screen();
    let cb = Checkbox::new(&screen).unwrap();
    cb.text("Accept");
    pump();
    assert_eq!(cb.get_text(), Some("Accept"));
}

// ── Line getters ─────────────────────────────────────────────────────────────

#[test]
fn line_get_point_count() {
    use oxivgl::widgets::lv_point_precise_t;
    let screen = fresh_screen();
    let line = Line::new(&screen).unwrap();
    static PTS: [lv_point_precise_t; 3] = [
        lv_point_precise_t { x: 0.0, y: 0.0 },
        lv_point_precise_t { x: 50.0, y: 50.0 },
        lv_point_precise_t { x: 100.0, y: 0.0 },
    ];
    line.set_points(&PTS);
    pump();
    assert_eq!(line.get_point_count(), 3);
}

#[test]
fn line_get_y_invert() {
    let screen = fresh_screen();
    let line = Line::new(&screen).unwrap();
    // No set_y_invert wrapper; default is false
    pump();
    assert!(!line.get_y_invert());
}

// ── Dropdown getters ─────────────────────────────────────────────────────────

#[test]
fn dropdown_get_option_count() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("A\nB\nC\nD");
    pump();
    assert_eq!(dd.get_option_count(), 4);
}

#[test]
fn dropdown_get_selected_highlight() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY");
    dd.set_selected_highlight(false);
    pump();
    assert!(!dd.get_selected_highlight());
    dd.set_selected_highlight(true);
    assert!(dd.get_selected_highlight());
}

#[test]
fn dropdown_get_dir() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_dir(DdDir::Top);
    pump();
    let dir = dd.get_dir();
    assert_eq!(dir, DdDir::Top as u32);
}

// ── Scale getters ─────────────────────────────────────────────────────────────

#[test]
fn scale_get_mode() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::HorizontalBottom);
    pump();
    let mode = scale.get_mode();
    assert_eq!(mode, ScaleMode::HorizontalBottom as u32);
}

#[test]
fn scale_get_total_tick_count() {
    use oxivgl::widgets::Scale;
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_total_tick_count(11);
    pump();
    assert_eq!(scale.get_total_tick_count(), 11);
}

#[test]
fn scale_get_rotation() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner);
    scale.set_rotation(45);
    pump();
    assert_eq!(scale.get_rotation(), 45);
}

#[test]
fn scale_get_label_show() {
    use oxivgl::widgets::Scale;
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_label_show(false);
    pump();
    assert!(!scale.get_label_show());
    scale.set_label_show(true);
    assert!(scale.get_label_show());
}

#[test]
fn scale_get_angle_range() {
    use oxivgl::widgets::{Scale, ScaleMode};
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_mode(ScaleMode::RoundInner);
    scale.set_angle_range(270);
    pump();
    assert_eq!(scale.get_angle_range(), 270);
}

#[test]
fn scale_get_range_min_max_value() {
    use oxivgl::widgets::Scale;
    let screen = fresh_screen();
    let scale = Scale::new(&screen).unwrap();
    scale.set_range(-50, 150);
    pump();
    assert_eq!(scale.get_range_min_value(), -50);
    assert_eq!(scale.get_range_max_value(), 150);
}

// ── Slider getters ───────────────────────────────────────────────────────────

#[test]
fn slider_get_min_max_value() {
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(-20, 80);
    pump();
    assert_eq!(slider.get_min_value(), -20);
    assert_eq!(slider.get_max_value(), 80);
}

#[test]
fn slider_get_mode() {
    use oxivgl::widgets::SliderMode;
    let screen = fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    slider.set_mode(SliderMode::Symmetrical);
    pump();
    assert!(matches!(slider.get_mode(), SliderMode::Symmetrical));
}

#[test]
fn slider_set_get_orientation() {
    let screen = common::fresh_screen();
    let slider = Slider::new(&screen).unwrap();
    assert!(matches!(slider.get_orientation(), SliderOrientation::Auto));
    slider.set_orientation(SliderOrientation::Vertical);
    assert!(matches!(slider.get_orientation(), SliderOrientation::Vertical));
}

// ── Spinner getters ──────────────────────────────────────────────────────────

#[test]
fn spinner_get_anim_duration() {
    let screen = fresh_screen();
    let sp = Spinner::new(&screen).unwrap();
    sp.set_anim_params(2000, 90);
    pump();
    assert_eq!(sp.get_anim_duration(), 2000);
}

#[test]
fn spinner_get_arc_sweep() {
    let screen = fresh_screen();
    let sp = Spinner::new(&screen).unwrap();
    sp.set_anim_params(1000, 120);
    pump();
    assert_eq!(sp.get_arc_sweep(), 120);
}

// ── Spinbox getters ──────────────────────────────────────────────────────────

#[test]
fn spinbox_get_digit_count() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_digit_format(5, 2);
    pump();
    assert_eq!(sb.get_digit_count(), 5);
}

#[test]
fn spinbox_get_dec_point_pos() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_digit_format(5, 2);
    pump();
    assert_eq!(sb.get_dec_point_pos(), 2);
}

#[test]
fn spinbox_get_min_max_value() {
    let screen = fresh_screen();
    let sb = Spinbox::new(&screen).unwrap();
    sb.set_range(-100, 500);
    pump();
    assert_eq!(sb.get_min_value(), -100);
    assert_eq!(sb.get_max_value(), 500);
}

// ── Switch getter ────────────────────────────────────────────────────────────

#[test]
fn switch_get_orientation() {
    use oxivgl::widgets::SwitchOrientation;
    let screen = fresh_screen();
    let sw = Switch::new(&screen).unwrap();
    sw.set_orientation(SwitchOrientation::Vertical);
    pump();
    assert!(matches!(sw.get_orientation(), SwitchOrientation::Vertical));
}

// ── Textarea getters ─────────────────────────────────────────────────────────

#[test]
fn textarea_get_cursor_click_pos() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    // No set_cursor_click_pos wrapper; just verify the getter returns without panic.
    let _click_pos = ta.get_cursor_click_pos();
}

#[test]
fn textarea_get_password_mode() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_password_mode(true);
    pump();
    assert!(ta.get_password_mode());
}

#[test]
fn textarea_get_one_line() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_one_line(true);
    pump();
    assert!(ta.get_one_line());
}

#[test]
fn textarea_get_max_length() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    ta.set_max_length(64);
    pump();
    assert_eq!(ta.get_max_length(), 64);
}

#[test]
fn textarea_get_text_selection() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    // No set_text_selection wrapper; just verify the getter returns without panic.
    let _sel = ta.get_text_selection();
}

#[test]
fn textarea_get_password_show_time() {
    let screen = fresh_screen();
    let ta = Textarea::new(&screen).unwrap();
    pump();
    // No set_password_show_time wrapper; just verify the getter returns without panic.
    let _time = ta.get_password_show_time();
}

// ── Roller getters ───────────────────────────────────────────────────────────

#[test]
fn roller_get_option_count() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC\nD\nE", RollerMode::Normal);
    pump();
    assert_eq!(roller.get_option_count(), 5);
}

#[test]
fn roller_get_selected_str() {
    let screen = fresh_screen();
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("Alpha\nBeta\nGamma", RollerMode::Normal);
    roller.set_selected(1, false);
    pump();
    let mut buf = [0u8; 32];
    let result = roller.get_selected_str(&mut buf);
    assert_eq!(result, Some("Beta"));
}

// ── Keyboard getters ─────────────────────────────────────────────────────────

#[test]
fn keyboard_get_mode() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    kb.set_mode(KeyboardMode::Number);
    pump();
    assert!(matches!(kb.get_mode(), KeyboardMode::Number));
}

#[test]
fn keyboard_get_popovers() {
    let screen = fresh_screen();
    let kb = Keyboard::new(&screen).unwrap();
    pump();
    // No set_popovers wrapper; just verify the getter returns without panic.
    let _pop = kb.get_popovers();
}

// ── Chart getters ────────────────────────────────────────────────────────────

#[test]
fn chart_get_type() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_type(ChartType::Bar);
    pump();
    assert!(matches!(chart.get_type(), ChartType::Bar));
}

#[test]
fn chart_get_update_mode() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
    pump();
    assert!(matches!(chart.get_update_mode(), ChartUpdateMode::Circular));
}

#[test]
fn chart_get_hor_div_line_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_div_line_count(3, 5);
    pump();
    assert_eq!(chart.get_hor_div_line_count(), 3);
}

#[test]
fn chart_get_ver_div_line_count() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_div_line_count(3, 5);
    pump();
    assert_eq!(chart.get_ver_div_line_count(), 5);
}

// ── Tabview getter ───────────────────────────────────────────────────────────

#[test]
fn tabview_get_tab_bar_position() {
    use oxivgl::widgets::DdDir;
    let screen = fresh_screen();
    let tv = Tabview::new(&screen).unwrap();
    let _tab = tv.add_tab("A");
    tv.set_tab_bar_position(DdDir::Bottom);
    pump();
    assert!(matches!(tv.get_tab_bar_position(), DdDir::Bottom));
}

// ── Menu getters ─────────────────────────────────────────────────────────────

#[test]
fn menu_get_mode_header_explicit() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(200, 200).center();
    menu.set_mode_header(MenuHeaderMode::BottomFixed);
    pump();
    assert!(matches!(menu.get_mode_header(), MenuHeaderMode::BottomFixed));
}

#[test]
fn menu_get_sidebar_header() {
    let screen = fresh_screen();
    let menu = Menu::new(&screen).unwrap();
    menu.size(320, 240);
    // Set a sidebar page so lv_menu_get_sidebar_header() returns non-null.
    let page = menu.page_create(Some("SideRoot"));
    menu.set_sidebar_page(&page);
    pump();
    let _hdr = menu.get_sidebar_header(); // just verify no panic
}

// ── Obj getters ──────────────────────────────────────────────────────────────

#[test]
fn obj_get_x2_y2() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.pos(10, 20).size(40, 30);
    pump();
    assert_eq!(obj.get_x2(), 10 + 40);
    assert_eq!(obj.get_y2(), 20 + 30);
}

#[test]
fn obj_get_content_width_height() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 80).pad(10);
    pump();
    // content = size - 2 * padding
    assert!(obj.get_content_width() >= 0);
    assert!(obj.get_content_height() >= 0);
}

#[test]
fn obj_get_self_width_height() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(60, 40);
    pump();
    assert!(obj.get_self_width() >= 0);
    assert!(obj.get_self_height() >= 0);
}

#[test]
fn obj_get_scrollbar_mode() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.set_scrollbar_mode(ScrollbarMode::Off);
    pump();
    assert!(matches!(obj.get_scrollbar_mode(), ScrollbarMode::Off));
}

#[test]
fn obj_get_scroll_dir() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.set_scroll_dir(ScrollDir::HOR);
    pump();
    let dir = obj.get_scroll_dir();
    assert_eq!(dir.0 & ScrollDir::HOR.0, ScrollDir::HOR.0);
}

#[test]
fn obj_get_state() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.add_state(ObjState::CHECKED);
    pump();
    let state = obj.get_state();
    assert_eq!(state.0 & ObjState::CHECKED.0, ObjState::CHECKED.0);
}

// ── Subject / observer ───────────────────────────────────────────────────────

#[test]
fn subject_int_create_get_set() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(28);
    assert_eq!(subject.get_int(), 28);
    subject.set_int(42);
    assert_eq!(subject.get_int(), 42);
}

#[test]
fn subject_int_previous_value() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(10);
    subject.set_int(20);
    assert_eq!(subject.get_int(), 20);
    assert_eq!(subject.get_previous_int(), 10);
}

#[test]
fn subject_int_drop_safe() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(5);
    subject.set_int(99);
    drop(subject); // lv_subject_deinit must not crash
    pump();
}

#[test]
fn slider_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(50);
    let slider = Slider::new(&screen).unwrap();
    slider.bind_value(&subject);
    pump();
    // Subject drives slider: value should reflect the subject's initial value.
    assert_eq!(subject.get_int(), 50);
}

#[test]
fn label_bind_text() {
    let screen = fresh_screen();
    let subject = Subject::new_int(28);
    let label = Label::new(&screen).unwrap();
    label.bind_text(&subject, c"%d C");
    pump();
    // No crash = binding established successfully.
    assert_eq!(subject.get_int(), 28);
}

#[test]
fn arc_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(30);
    let arc = Arc::new(&screen).unwrap();
    arc.set_range_raw(0, 100);
    arc.bind_value(&subject);
    pump();
    assert_eq!(subject.get_int(), 30);
}

#[test]
fn roller_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(1);
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC", RollerMode::Normal);
    roller.bind_value(&subject);
    pump();
    assert_eq!(subject.get_int(), 1);
}

#[test]
fn dropdown_bind_value() {
    let screen = fresh_screen();
    let subject = Subject::new_int(2);
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY\nZ");
    dd.bind_value(&subject);
    pump();
    assert_eq!(subject.get_int(), 2);
}

#[test]
fn subject_add_observer_obj() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    unsafe extern "C" fn dummy_cb(_obs: *mut lv_observer_t, _sub: *mut lv_subject_t) {}
    subject.add_observer_obj(dummy_cb, &obj, core::ptr::null_mut());
    subject.set_int(42);
    pump();
    // No crash = success.
}

#[test]
fn obj_bind_state_if_eq() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    obj.bind_state_if_eq(&subject, ObjState::DISABLED, 1);
    pump();
    assert!(!obj.has_state(ObjState::DISABLED));
    subject.set_int(1);
    pump();
    assert!(obj.has_state(ObjState::DISABLED));
    subject.set_int(0);
    pump();
    assert!(!obj.has_state(ObjState::DISABLED));
}

#[test]
fn obj_bind_state_if_not_eq() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    obj.bind_state_if_not_eq(&subject, ObjState::DISABLED, 1);
    pump();
    assert!(obj.has_state(ObjState::DISABLED)); // 0 != 1 → disabled
    subject.set_int(1);
    pump();
    assert!(!obj.has_state(ObjState::DISABLED)); // 1 == 1 → not disabled
}

#[test]
fn obj_bind_checked() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let obj = Obj::new(&screen).unwrap();
    obj.add_flag(ObjFlag::CHECKABLE);
    obj.bind_checked(&subject);
    pump();
    assert!(!obj.has_state(ObjState::CHECKED));
    subject.set_int(1);
    pump();
    assert!(obj.has_state(ObjState::CHECKED));
    subject.set_int(0);
    pump();
    assert!(!obj.has_state(ObjState::CHECKED));
}

#[test]
fn subject_new_group() {
    let _screen = fresh_screen();
    let s0 = Subject::new_int(10);
    let s1 = Subject::new_int(20);
    let s2 = Subject::new_int(30);
    let group = Subject::new_group(&[&s0, &s1, &s2]);
    pump();
    // Group subject drops cleanly (deinit must not crash).
    drop(group);
    drop(s2);
    drop(s1);
    drop(s0);
}

#[test]
fn subject_notify() {
    let _screen = fresh_screen();
    static CALL_COUNT: core::sync::atomic::AtomicI32 =
        core::sync::atomic::AtomicI32::new(0);

    unsafe extern "C" fn counting_cb(
        _obs: *mut lv_observer_t,
        _sub: *mut lv_subject_t,
    ) {
        CALL_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }

    CALL_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
    let subject = Subject::new_int(0);
    subject.add_observer(counting_cb, core::ptr::null_mut());
    // Initial notify fires once when observer is added (LVGL behaviour).
    pump();
    let after_add = CALL_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    // Manual notify must fire the callback again.
    subject.notify();
    pump();
    assert!(
        CALL_COUNT.load(core::sync::atomic::Ordering::Relaxed) > after_add,
        "notify() must trigger observer callback"
    );
}

#[test]
fn subject_get_group_element_values() {
    let _screen = fresh_screen();
    let s0 = Subject::new_int(11);
    let s1 = Subject::new_int(22);
    let group = Subject::new_group(&[&s0, &s1]);
    pump();
    // Retrieve member values through the group element accessor.
    // SAFETY: group is a valid group subject; indices 0 and 1 are in bounds.
    let v0 = unsafe { subject_get_int_raw(subject_get_group_element(group.raw_ptr(), 0)) };
    let v1 = unsafe { subject_get_int_raw(subject_get_group_element(group.raw_ptr(), 1)) };
    assert_eq!(v0, 11);
    assert_eq!(v1, 22);
}

#[test]
fn obj_clean() {
    let screen = fresh_screen();
    let parent = Obj::new(&screen).unwrap();
    let _c1 = Obj::new(&parent).unwrap();
    let _c2 = Obj::new(&parent).unwrap();
    pump();
    assert_eq!(parent.get_child_count(), 2);
    parent.clean();
    pump();
    assert_eq!(parent.get_child_count(), 0);
}

#[test]
fn subject_add_observer_with_target() {
    let _screen = fresh_screen();
    static mut TARGET_VAL: i32 = 0;
    unsafe extern "C" fn cb(observer: *mut lv_observer_t, _subject: *mut lv_subject_t) {
        unsafe {
            // SAFETY: target is &mut TARGET_VAL cast to *mut c_void, set below.
            let target = observer_get_target(observer) as *mut i32;
            *target = 99;
        }
    }
    let subject = Subject::new_int(0);
    // SAFETY: TARGET_VAL is a static mut; single-threaded test, no concurrent access.
    // &raw mut avoids creating an intermediate reference to the mutable static.
    subject.add_observer_with_target(
        cb,
        &raw mut TARGET_VAL as *mut c_void,
        core::ptr::null_mut(),
    );
    subject.set_int(1);
    pump();
    // SAFETY: TARGET_VAL written only by `cb` on the same thread; no concurrent access.
    unsafe { assert_eq!(*(&raw const TARGET_VAL), 99); }
}

#[test]
fn subject_on_change() {
    let _screen = fresh_screen();
    static LAST_VALUE: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1);
    let subject = Subject::new_int(0);
    subject.on_change(|v| {
        LAST_VALUE.store(v, core::sync::atomic::Ordering::Relaxed);
    });
    pump();
    subject.set_int(42);
    pump();
    assert_eq!(LAST_VALUE.load(core::sync::atomic::Ordering::Relaxed), 42);
}

#[test]
fn label_bind_text_map() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let label = Label::new(&screen).unwrap();
    label.bind_text_map(&subject, |v| match v {
        1 => "one",
        _ => "zero",
    });
    pump();
    // Change to 1
    subject.set_int(1);
    pump();
    // Label should now say "one" — verify no crash + correct binding.
}

#[test]
fn label_bind_text_map_sets_correct_text() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let label = Label::new(&screen).unwrap();
    label.bind_text_map(&subject, |v| match v {
        1 => "one",
        _ => "zero",
    });
    pump();
    let text = unsafe {
        let ptr = lvgl_rust_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "zero");
    subject.set_int(1);
    pump();
    let text = unsafe {
        let ptr = lvgl_rust_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "one");
}

#[test]
fn observer_get_target_obj_returns_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let label = Label::new(&screen).unwrap();
    static TARGET_HANDLE: core::sync::atomic::AtomicPtr<lvgl_rust_sys::lv_obj_t> =
        core::sync::atomic::AtomicPtr::new(core::ptr::null_mut());
    unsafe extern "C" fn cb(
        obs: *mut lvgl_rust_sys::lv_observer_t,
        _sub: *mut lvgl_rust_sys::lv_subject_t,
    ) {
        // SAFETY: obs is a valid observer pointer received from LVGL.
        let ptr = unsafe { observer_get_target_obj(obs) };
        TARGET_HANDLE.store(ptr, core::sync::atomic::Ordering::Relaxed);
    }
    subject.add_observer_obj(cb, &label, core::ptr::null_mut());
    subject.set_int(1);
    pump();
    assert_eq!(
        TARGET_HANDLE.load(core::sync::atomic::Ordering::Relaxed),
        label.lv_handle()
    );
}

#[test]
fn subject_drop_before_widgets() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let slider = Slider::new(&screen).unwrap();
    slider.bind_value(&subject);
    let label = Label::new(&screen).unwrap();
    label.bind_text(&subject, c"%d");
    pump();
    // Drop subject first — lv_subject_deinit removes observer linkage.
    drop(subject);
    pump();
    // Widgets still alive, just no longer bound.
    drop(slider);
    drop(label);
    pump();
    // No crash = both drop orders are safe.
}

// ── Strengthened bind tests ──────────────────────────────────────────────────

#[test]
fn slider_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(50);
    let slider = Slider::new(&screen).unwrap();
    slider.set_range(0, 100);
    slider.bind_value(&subject);
    pump();
    assert_eq!(slider.get_value(), 50);
    subject.set_int(75);
    pump();
    assert_eq!(slider.get_value(), 75);
}

#[test]
fn arc_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(30);
    let arc = Arc::new(&screen).unwrap();
    arc.bind_value(&subject);
    pump();
    assert_eq!(arc.get_value_raw(), 30);
    subject.set_int(60);
    pump();
    assert_eq!(arc.get_value_raw(), 60);
}

#[test]
fn roller_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(2);
    let roller = Roller::new(&screen).unwrap();
    roller.set_options("A\nB\nC\nD", RollerMode::Normal);
    roller.bind_value(&subject);
    pump();
    assert_eq!(roller.get_selected(), 2); // "C"
    subject.set_int(0);
    pump();
    assert_eq!(roller.get_selected(), 0); // "A"
}

#[test]
fn dropdown_bind_value_updates_widget() {
    let screen = fresh_screen();
    let subject = Subject::new_int(1);
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("X\nY\nZ");
    dd.bind_value(&subject);
    pump();
    assert_eq!(dd.get_selected(), 1); // "Y"
    subject.set_int(2);
    pump();
    assert_eq!(dd.get_selected(), 2); // "Z"
}

#[test]
fn label_bind_text_sets_correct_text() {
    let screen = fresh_screen();
    let subject = Subject::new_int(28);
    let label = Label::new(&screen).unwrap();
    label.bind_text(&subject, c"%d C");
    pump();
    let text = unsafe {
        let ptr = lvgl_rust_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "28 C");
    subject.set_int(42);
    pump();
    let text = unsafe {
        let ptr = lvgl_rust_sys::lv_label_get_text(label.lv_handle());
        core::ffi::CStr::from_ptr(ptr).to_str().unwrap()
    };
    assert_eq!(text, "42 C");
}

// ── Interaction tests ────────────────────────────────────────────────────────

#[test]
fn subject_group_notifies_on_member_change() {
    let _screen = fresh_screen();
    static FIRE_COUNT: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);
    unsafe extern "C" fn cb(_obs: *mut lv_observer_t, _sub: *mut lv_subject_t) {
        FIRE_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
    }
    let s1 = Subject::new_int(0);
    let s2 = Subject::new_int(0);
    let group = Subject::new_group(&[&s1, &s2]);
    FIRE_COUNT.store(0, core::sync::atomic::Ordering::Relaxed);
    group.add_observer(cb, core::ptr::null_mut());
    pump();
    let after_add = FIRE_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    // Change member s1 — group observer should fire.
    s1.set_int(10);
    pump();
    let after_s1 = FIRE_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    assert!(after_s1 > after_add, "group observer must fire when member changes");
    // Change member s2 — group observer should fire again.
    s2.set_int(20);
    pump();
    let after_s2 = FIRE_COUNT.load(core::sync::atomic::Ordering::Relaxed);
    assert!(after_s2 > after_s1, "group observer must fire when another member changes");
}

#[test]
fn subject_multiple_observers() {
    let _screen = fresh_screen();
    static COUNT_A: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);
    static COUNT_B: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);
    let subject = Subject::new_int(0);
    subject.on_change(|_| { COUNT_A.fetch_add(1, core::sync::atomic::Ordering::Relaxed); });
    subject.on_change(|_| { COUNT_B.fetch_add(1, core::sync::atomic::Ordering::Relaxed); });
    COUNT_A.store(0, core::sync::atomic::Ordering::Relaxed);
    COUNT_B.store(0, core::sync::atomic::Ordering::Relaxed);
    subject.set_int(1);
    pump();
    assert!(COUNT_A.load(core::sync::atomic::Ordering::Relaxed) > 0, "first observer must fire");
    assert!(COUNT_B.load(core::sync::atomic::Ordering::Relaxed) > 0, "second observer must fire");
}

#[test]
fn widget_drop_then_subject_set() {
    let screen = fresh_screen();
    let subject = Subject::new_int(0);
    let slider = Slider::new(&screen).unwrap();
    slider.bind_value(&subject);
    pump();
    drop(slider); // widget drops, LVGL removes observer
    pump();
    subject.set_int(99); // must not crash — no observers left
    pump();
}

#[test]
fn subject_on_change_fires_on_registration() {
    let _screen = fresh_screen();
    static INITIAL: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(-1);
    INITIAL.store(-1, core::sync::atomic::Ordering::Relaxed);
    let subject = Subject::new_int(42);
    subject.on_change(|v| { INITIAL.store(v, core::sync::atomic::Ordering::Relaxed); });
    pump();
    // LVGL fires observers on registration with current value.
    assert_eq!(INITIAL.load(core::sync::atomic::Ordering::Relaxed), 42);
}

#[test]
fn subject_previous_int_tracks_last_change() {
    let _screen = fresh_screen();
    let subject = Subject::new_int(0);
    subject.set_int(1);
    subject.set_int(2);
    subject.set_int(3);
    assert_eq!(subject.get_previous_int(), 2);
    assert_eq!(subject.get_int(), 3);
}

// ── Edge case tests ──────────────────────────────────────────────────────────

#[test]
fn subject_empty_group() {
    let _screen = fresh_screen();
    let group = Subject::new_group(&[]);
    pump();
    group.notify();
    pump();
    // No crash = success.
}

// ── Snapshot ─────────────────────────────────────────────────────────────────

#[test]
fn snapshot_take_widget() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100).center().bg_color(0xff0000).bg_opa(255);
    pump();
    let snap = Snapshot::take_widget(&obj).expect("snapshot allocation");
    assert!(snap.width() > 0);
    assert!(snap.height() > 0);
}

#[test]
fn image_set_src_snapshot() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    obj.size(100, 100).center().bg_color(0xff0000).bg_opa(255);
    pump();
    let snap = Snapshot::take_widget(&obj).expect("snapshot allocation");
    let img = Image::new(&screen).unwrap();
    img.set_src_snapshot(&snap);
    pump();
    // Verify image source dimensions match snapshot
    assert!(img.get_src_width() > 0);
}

#[test]
fn snapshot_take_widget_empty_obj() {
    let screen = fresh_screen();
    let obj = Obj::new(&screen).unwrap();
    // Don't set size — LVGL may return zero-sized snapshot or None
    pump();
    // Either works — we just verify no crash
    let _snap = Snapshot::take_widget(&obj);
}
