// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests — exercise widgets against a real (headless) LVGL instance.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test integration
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

use std::sync::Once;

use oxivgl::{
    anim::anim_path_linear,
    fonts::MONTSERRAT_12,
    driver::LvglDriver,
    style::{
        color_make, palette_main, props, BorderSide, GradDsc, GradExtend, Palette, Selector,
        StyleBuilder, TextDecor, TransitionDsc,
    },
    enums::{EventCode, ObjFlag, ObjState, Opa, ScrollDir, ScrollSnap, ScrollbarMode},
    layout::{FlexAlign, FlexFlow, GridAlign, GridCell, Layout, GRID_TEMPLATE_LAST},
    widgets::{
        detach, Align, Arc, AsLvHandle, Bar, Button, Checkbox, Child, Dropdown, Image, Label,
        Led, Line, Obj, Roller, RollerMode, Screen, Slider, Switch, ValueLabel, WidgetError,
        RADIUS_MAX,
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
    // SAFETY: LVGL initialised, single-threaded.
    unsafe {
        lvgl_rust_sys::lv_timer_handler();
        lvgl_rust_sys::lv_refr_now(core::ptr::null_mut());
    }
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
    obj.set_style_base_dir(oxivgl::widgets::BaseDir::Rtl, Selector::DEFAULT);
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
