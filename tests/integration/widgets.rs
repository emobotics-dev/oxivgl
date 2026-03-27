// SPDX-License-Identifier: MIT OR Apache-2.0
//! Widget-specific integration tests — individual widget types exercised against
//! a real (headless) LVGL instance.

use crate::common::{fresh_screen, pump};

use oxivgl::enums::{ObjFlag, ObjState, ScrollDir};
use oxivgl::layout::{FlexAlign, FlexFlow};
use oxivgl::style::{
    color_make, palette_main, Palette, Selector, TextDecor,
};
use oxivgl::widgets::{
    AnimImg, Arc, ArcLabel, ArcLabelDir, Bar, Button, Buttonmatrix,
    ButtonmatrixCtrl, ButtonmatrixMap, Calendar, CalendarDate, Chart, ChartAxis,
    ChartType, ChartUpdateMode, Checkbox, Dropdown, Image,
    Imagebutton, ImagebuttonState, Keyboard, KeyboardMode, Label, Led, Line, Menu, MenuHeaderMode,
    Msgbox, BarOrientation, Obj, Roller, RollerMode, Screen, Slider, SliderOrientation,
    Spinbox, Spinner, Spangroup, SpanMode, SpanOverflow, Switch, Table, TableCellCtrl, Tabview,
    Textarea, Tileview, ValueLabel, Win,
};

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

// ── Dropdown ─────────────────────────────────────────────────────────────────

#[test]
fn dropdown_set_symbol_static() {
    let screen = fresh_screen();
    let dd = Dropdown::new(&screen).unwrap();
    dd.set_options("A\nB\nC");
    dd.set_symbol(c"▼");
    pump();
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

// ── Tabview ───────────────────────────────────────────────────────────────────

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

#[test]
fn buttonmatrix_set_button_ctrl_all() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl_all(ButtonmatrixCtrl::CHECKABLE);
    assert!(btnm.has_button_ctrl(0, ButtonmatrixCtrl::CHECKABLE));
}

#[test]
fn buttonmatrix_set_one_checked() {
    let screen = fresh_screen();
    let btnm = Buttonmatrix::new(&screen).unwrap();
    btnm.set_button_ctrl_all(ButtonmatrixCtrl::CHECKABLE);
    btnm.set_one_checked(true);
    btnm.set_button_ctrl(0, ButtonmatrixCtrl::CHECKED);
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

// ── Text letter space ───────────────────────────────────────────────────────

#[test]
fn text_letter_space() {
    let screen = fresh_screen();
    let lbl = Label::new(&screen).unwrap();
    lbl.text("Spaced");
    lbl.style_text_letter_space(5, Selector::DEFAULT);
    pump();
}

// ── ArcLabel ─────────────────────────────────────────────────────────────────

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

#[test]
fn arclabel_direction() {
    let screen = fresh_screen();
    let al = ArcLabel::new(&screen).unwrap();
    al.set_text_static(c"CCW");
    al.set_dir(ArcLabelDir::CounterClockwise);
    pump();
}

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

// ── Screen::layer_top ───────────────────────────────────────────────────────

#[test]
fn screen_layer_top() {
    let _screen = fresh_screen();
    let top = Screen::layer_top();
    assert!(!top.handle().is_null());
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

// ── Chart new APIs ───────────────────────────────────────────────────────────

#[test]
fn chart_set_update_mode() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
}

#[test]
fn chart_add_cursor_and_set_point() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    for i in 0..10 { chart.set_next_value(&ser, i * 10); }
    let color = palette_main(Palette::Blue);
    let cursor = chart.add_cursor(color, 0x01 | 0x08);
    chart.set_cursor_point(&cursor, Some(&ser), 5);
}

#[test]
fn chart_get_pressed_point_none() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    assert!(chart.get_pressed_point().is_none());
}

#[test]
fn chart_get_series_next() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    let _ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    let first = chart.get_series_next(None);
    assert!(first.is_some());
    let second = chart.get_series_next(first.as_ref());
    assert!(second.is_none());
}

#[test]
fn chart_get_x_start_point() {
    let screen = fresh_screen();
    let chart = Chart::new(&screen).unwrap();
    chart.set_update_mode(ChartUpdateMode::Circular);
    let ser = chart.add_series(palette_main(Palette::Red), ChartAxis::PrimaryY);
    let _ = chart.get_x_start_point(&ser);
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
    let screen = fresh_screen();
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
    let screen = fresh_screen();
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

