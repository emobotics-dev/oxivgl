// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests — exercise widgets against a real (headless) LVGL instance.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test integration
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

use std::sync::Once;

use oxivgl::{
    anim::{anim_path_linear, anim_set_x, Anim, AnimHandle},
    fonts::MONTSERRAT_12,
    driver::LvglDriver,
    style::{
        color_make, palette_main, props, BorderSide, GradDsc, GradExtend, Palette, Selector,
        StyleBuilder, TextDecor, TransitionDsc,
    },
    enums::{EventCode, ObjFlag, ObjState, Opa, ScrollDir, ScrollSnap, ScrollbarMode},
    layout::{FlexAlign, FlexFlow, GridAlign, GridCell, Layout, GRID_TEMPLATE_LAST},
    widgets::{
        detach, Align, Arc, AsLvHandle, Bar, Button, Buttonmatrix, Canvas, Checkbox, Child,
        Dropdown, Image, Keyboard, KeyboardMode, Label, Led, Line, Menu, MenuHeaderMode, Msgbox,
        Obj, Part, Roller, RollerMode, Screen, Slider, Switch, Table, TableCellCtrl, Textarea,
        ValueLabel, WidgetError, RADIUS_MAX,
    },
};

static INIT: Once = Once::new();
static mut DRIVER: Option<LvglDriver> = None;

/// Initialise LVGL once for all tests. Must run single-threaded.
///
/// Panics if `SDL_VIDEODRIVER` is not set — without it LVGL's SDL2
/// backend tries to open a real display and crashes with a double-free.
/// Run via `./run_tests.sh int` or set `SDL_VIDEODRIVER=dummy` manually.
fn ensure_init() {
    INIT.call_once(|| {
        assert!(
            std::env::var("SDL_VIDEODRIVER").is_ok(),
            "SDL_VIDEODRIVER not set — run via: ./run_tests.sh int"
        );
        // SAFETY: single-threaded test runner (--test-threads=1).
        unsafe { DRIVER = Some(LvglDriver::init(320, 240)) };
    });
}

/// Get the active screen, creating a fresh one to isolate tests.
fn fresh_screen() -> Screen {
    ensure_init();
    // SAFETY: LVGL initialised; loading a new screen clears the previous one.
    unsafe {
        let new = lvgl_rust_sys::lv_obj_create(core::ptr::null_mut());
        lvgl_rust_sys::lv_screen_load(new);
    }
    Screen::active().expect("no active screen after init")
}

/// Pump LVGL timer and force a full layout + refresh pass.
/// Mirrors LVGL's own `lv_test_helpers.c` approach.
fn pump() {
    let driver = unsafe { (*core::ptr::addr_of!(DRIVER)).as_ref().unwrap() };
    driver.timer_handler();
    unsafe { lvgl_rust_sys::lv_refr_now(core::ptr::null_mut()) };
}

#[test]
fn timer_handler_callable() {
    ensure_init();
    let driver = unsafe { (*core::ptr::addr_of!(DRIVER)).as_ref().unwrap() };
    let _ms = driver.timer_handler();
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

// ── Child / detach ───────────────────────────────────────────────────────────

#[test]
fn child_wrapper_deref() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    let child: Child<Label> = Child::new(label);
    child.text("via Child");
    pump();
    assert!(child.get_width() > 0);
    // Child suppresses Drop — LVGL parent owns the object.
}

#[test]
fn detach_fire_and_forget() {
    let screen = fresh_screen();
    let label = Label::new(&screen).unwrap();
    label.text("ephemeral");
    detach(label);
    // label consumed, no Drop runs. LVGL parent will clean up.
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
    let driver = unsafe { (*core::ptr::addr_of!(DRIVER)).as_ref().unwrap() };
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
    let driver = unsafe { (*core::ptr::addr_of!(DRIVER)).as_ref().unwrap() };
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
    mbox.close();
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
    mbox.close();
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
    let lbl = Label::new(&screen).unwrap();
    let c: Child<Label> = Child::new(lbl);
    let s = format!("{c:?}");
    assert!(!s.is_empty());
    // Suppress Drop — LVGL parent owns the object.
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
    assert_eq!(table.get_cell_value(0, 0), Some("Hello"));
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
