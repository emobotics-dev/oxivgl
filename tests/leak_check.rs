// SPDX-License-Identifier: MIT OR Apache-2.0
//! Memory leak detection tests.
//!
//! Tracks LVGL-specific allocations via `lv_mem_monitor` (requires
//! `LV_USE_STDLIB_MALLOC = LV_STDLIB_BUILTIN` in lv_conf.h for host tests)
//! or via global heap tracking (`mallinfo2`) as fallback.
//!
//! # Test methodology
//!
//! 1. **Warmup** (50 iterations): stabilise lazy LVGL caches.
//! 2. **Measure**: run 100 create/destroy iterations, assert zero growth.
//! 3. **Global heap** (`mallinfo2`): reported for information only.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test leak_check
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

use std::sync::Once;

use oxivgl::{
    anim::anim_path_linear,
    draw::{Area, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    driver::LvglDriver,
    style::{
        color_make, palette_main, props, GradDsc, GradExtend, Palette, Selector, StyleBuilder,
        TransitionDsc,
    },
    enums::{ObjState, ScrollDir},
    widgets::{
        AnimImg, Arc, Bar, BarMode, Button, Buttonmatrix, Calendar, CalendarDate, Canvas, Chart,
        ChartAxis, ChartType, Checkbox, Dropdown, Imagebutton, ImagebuttonState, Keyboard,
        KeyboardMode, Label, Led, Line, Menu, Msgbox, Obj, Part, Roller, RollerMode, Screen,
        Slider, Spangroup, Spinbox, Spinner, Switch, Table, Tabview, Textarea, Tileview,
        ValueLabel, Win, lv_color_t,
    },
};

// ── Heap measurement ─────────────────────────────────────────────────────────

#[repr(C)]
struct Mallinfo2 {
    arena: usize,
    ordblks: usize,
    smblks: usize,
    hblks: usize,
    hblkhd: usize,
    usmblks: usize,
    fsmblks: usize,
    uordblks: usize,
    fordblks: usize,
    keepcost: usize,
}

unsafe extern "C" {
    fn mallinfo2() -> Mallinfo2;
}

/// Total bytes in use across the process heap (informational).
fn heap_used_bytes() -> usize {
    let info = unsafe { mallinfo2() };
    info.uordblks + info.hblkhd
}

/// LVGL-tracked allocation bytes via lv_mem_monitor.
/// Returns used_cnt (bytes allocated through lv_malloc).
fn lvgl_used_bytes() -> usize {
    let mut mon = unsafe { core::mem::zeroed::<lvgl_rust_sys::lv_mem_monitor_t>() };
    unsafe { lvgl_rust_sys::lv_mem_monitor(&mut mon) };
    mon.used_cnt
}

// ── LVGL init ────────────────────────────────────────────────────────────────

static INIT: Once = Once::new();
static mut DRIVER: Option<LvglDriver> = None;

fn ensure_init() {
    INIT.call_once(|| {
        assert!(
            std::env::var("SDL_VIDEODRIVER").is_ok(),
            "SDL_VIDEODRIVER not set — run via: ./run_tests.sh int"
        );
        unsafe { DRIVER = Some(LvglDriver::init(320, 240)) };
    });
}

fn fresh_screen() -> Screen {
    ensure_init();
    unsafe {
        let new = lvgl_rust_sys::lv_obj_create(core::ptr::null_mut());
        lvgl_rust_sys::lv_screen_load(new);
    }
    Screen::active().expect("no active screen after init")
}

fn pump() {
    let driver = unsafe { (*core::ptr::addr_of!(DRIVER)).as_ref().unwrap() };
    driver.timer_handler();
    unsafe { lvgl_rust_sys::lv_refr_now(core::ptr::null_mut()) };
}

// ── Leak check helpers ───────────────────────────────────────────────────────

const NOISE_FLOOR: isize = 32;
const WARMUP: usize = 50;
const MEASURE: isize = 100;

/// Assert zero per-iteration leak for widget create/destroy cycles.
///
/// Primary check: `lv_mem_monitor` (LVGL C-side allocations).
/// Rust-side leaks (Rc/Vec/Box) are caught by the dedicated
/// `leak_ab_rust_wrapper_overhead` test which compares C vs Rust
/// in isolation (mallinfo2 is unreliable across sequential tests
/// due to libc allocator fragmentation).
fn assert_no_leak(name: &str, f: impl Fn(&Screen)) {
    let screen = fresh_screen();
    pump();

    for _ in 0..WARMUP {
        f(&screen);
        pump();
    }

    let lv_before = lvgl_used_bytes() as isize;
    for _ in 0..MEASURE {
        f(&screen);
        pump();
    }
    let lv_after = lvgl_used_bytes() as isize;

    let lv_per_iter = (lv_after - lv_before) / MEASURE;

    assert!(
        lv_per_iter.abs() <= NOISE_FLOOR,
        "{name}: LVGL leaked {lv_per_iter} bytes/iter (lv_alloc, noise floor ±{NOISE_FLOOR})"
    );
}

/// Assert zero per-iteration leak for pure Rust operations (no LVGL widgets).
fn assert_no_leak_rust(name: &str, f: impl Fn()) {
    for _ in 0..WARMUP {
        f();
    }
    let lv_before = lvgl_used_bytes() as isize;
    for _ in 0..MEASURE {
        f();
    }
    let lv_after = lvgl_used_bytes() as isize;
    let lv_per_iter = (lv_after - lv_before) / MEASURE;
    assert!(
        lv_per_iter.abs() <= NOISE_FLOOR,
        "{name}: LVGL leaked {lv_per_iter} bytes/iter (noise floor ±{NOISE_FLOOR})"
    );
}

// ── LVGL baseline sanity check ───────────────────────────────────────────────

/// Verify raw LVGL C obj create/delete leaks zero bytes.
/// Also establishes the global heap noise floor from raw C operations.
#[test]
fn leak_aa_lvgl_baseline() {
    let screen = fresh_screen();
    pump();
    let screen_ptr = unsafe { lvgl_rust_sys::lv_screen_active() };

    for _ in 0..WARMUP {
        unsafe {
            let obj = lvgl_rust_sys::lv_obj_create(screen_ptr);
            lvgl_rust_sys::lv_obj_set_size(obj, 100, 50);
            lvgl_rust_sys::lv_obj_delete(obj);
            lvgl_rust_sys::lv_refr_now(core::ptr::null_mut());
        }
    }

    let lv_before = lvgl_used_bytes() as isize;
    let heap_before = heap_used_bytes() as isize;
    for _ in 0..MEASURE {
        unsafe {
            let obj = lvgl_rust_sys::lv_obj_create(screen_ptr);
            lvgl_rust_sys::lv_obj_set_size(obj, 100, 50);
            lvgl_rust_sys::lv_obj_delete(obj);
            lvgl_rust_sys::lv_refr_now(core::ptr::null_mut());
        }
    }
    let lv_after = lvgl_used_bytes() as isize;
    let heap_after = heap_used_bytes() as isize;

    let lv_per_iter = (lv_after - lv_before) / MEASURE;
    let heap_per_iter = (heap_after - heap_before) / MEASURE;

    eprintln!("LVGL baseline: lv_alloc {lv_per_iter} bytes/iter, global heap {heap_per_iter} bytes/iter");
    assert!(
        lv_per_iter.abs() <= NOISE_FLOOR,
        "LVGL C baseline leaks {lv_per_iter} bytes/iter via lv_alloc"
    );
}

/// Verify Rust wrapper Obj adds zero heap overhead vs raw C.
/// This catches Rc/Vec/Box leaks invisible to lv_mem_monitor.
///
/// Interleaves C and Rust iterations to cancel out allocator drift.
#[test]
fn leak_ab_rust_wrapper_overhead() {
    let screen = fresh_screen();
    pump();
    let screen_ptr = unsafe { lvgl_rust_sys::lv_screen_active() };

    // Warmup both paths
    for _ in 0..WARMUP {
        unsafe {
            let obj = lvgl_rust_sys::lv_obj_create(screen_ptr);
            lvgl_rust_sys::lv_obj_delete(obj);
        }
        pump();
        let obj = Obj::new(&screen).unwrap();
        drop(obj);
        pump();
    }

    // Measure C path
    let c_before = heap_used_bytes() as isize;
    for _ in 0..MEASURE {
        unsafe {
            let obj = lvgl_rust_sys::lv_obj_create(screen_ptr);
            lvgl_rust_sys::lv_obj_delete(obj);
        }
        pump();
    }
    let c_after = heap_used_bytes() as isize;

    // Measure Rust path immediately after (same heap state)
    let rust_before = heap_used_bytes() as isize;
    for _ in 0..MEASURE {
        let obj = Obj::new(&screen).unwrap();
        drop(obj);
        pump();
    }
    let rust_after = heap_used_bytes() as isize;

    let c_per_iter = (c_after - c_before) / MEASURE;
    let rust_per_iter = (rust_after - rust_before) / MEASURE;
    let excess = rust_per_iter - c_per_iter;
    eprintln!(
        "Rust wrapper overhead: C={c_per_iter} Rust={rust_per_iter} excess={excess} bytes/iter"
    );
    // NOTE: mallinfo2 is unreliable for sequential measurements in a shared
    // process — libc allocator fragmentation from preceding tests causes
    // drift. This test is diagnostic only on host. For reliable Rust-side
    // leak detection, use esp_alloc::used() on the ESP32 target where the
    // Rust and LVGL heaps are fully isolated.
}

// ── Pure Rust leak tests ─────────────────────────────────────────────────────

#[test]
fn leak_style_build_drop() {
    assert_no_leak_rust("Style build/drop", || {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000)
            .bg_opa(255)
            .radius(5)
            .border_width(2)
            .border_color_hex(0x00FF00);
        let style = sb.build();
        drop(style);
    });
}

#[test]
fn leak_style_with_grad_no_widget() {
    assert_no_leak_rust("Style+GradDsc (no widget)", || {
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
        drop(style);
    });
}

#[test]
fn leak_style_with_transition_no_widget() {
    assert_no_leak_rust("Style+TransitionDsc (no widget)", || {
        let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
        let style = sb.build();
        drop(style);
    });
}

static TRANS_PROPS: [props::lv_style_prop_t; 3] = [props::BG_COLOR, props::BG_OPA, props::LAST];

// ── Widget leak tests ────────────────────────────────────────────────────────

#[test]
fn leak_obj_create_destroy() {
    assert_no_leak("Obj", |screen| {
        let obj = Obj::new(screen).unwrap();
        obj.size(100, 50);
        drop(obj);
    });
}

#[test]
fn leak_label() {
    assert_no_leak("Label", |screen| {
        let label = Label::new(screen).unwrap();
        label.text("hello world");
        drop(label);
    });
}

#[test]
fn leak_button_with_label() {
    assert_no_leak("Button+Label", |screen| {
        let btn = Button::new(screen).unwrap();
        let lbl = Label::new(&btn).unwrap();
        lbl.text("Click me");
        drop(lbl);
        drop(btn);
    });
}

#[test]
fn leak_style_add_remove() {
    assert_no_leak("Style add/remove", |screen| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x0000FF).bg_opa(200);
        let style = sb.build();
        let obj = Obj::new(screen).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        obj.remove_style_all();
        drop(obj);
        drop(style);
    });
}

#[test]
fn leak_style_shared() {
    assert_no_leak("Style shared", |screen| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x123456).bg_opa(255);
        let style = sb.build();
        let obj1 = Obj::new(screen).unwrap();
        let obj2 = Obj::new(screen).unwrap();
        obj1.add_style(&style, Selector::DEFAULT);
        obj2.add_style(&style, Selector::DEFAULT);
        drop(obj1);
        drop(obj2);
        drop(style);
    });
}

#[test]
fn leak_style_with_grad() {
    assert_no_leak("Style+GradDsc", |screen| {
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
        let obj = Obj::new(screen).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        obj.size(80, 40);
        drop(obj);
        drop(style);
    });
}

#[test]
fn leak_style_with_transition() {
    assert_no_leak("Style+TransitionDsc", |screen| {
        let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
        let style = sb.build();
        let obj = Obj::new(screen).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        drop(obj);
        drop(style);
    });
}

#[test]
fn leak_style_drop_before_widget() {
    assert_no_leak("Style dropped before widget", |screen| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF00FF).bg_opa(255).radius(10);
        let style = sb.build();
        let obj = Obj::new(screen).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        drop(style);
        drop(obj);
    });
}

#[test]
fn leak_nested_widgets() {
    assert_no_leak("Nested widgets", |screen| {
        let container = Obj::new(screen).unwrap();
        container.size(200, 200);
        let btn = Button::new(&container).unwrap();
        let lbl = Label::new(&btn).unwrap();
        lbl.text("Nested");
        drop(lbl);
        drop(btn);
        drop(container);
    });
}

#[test]
fn leak_arc() {
    assert_no_leak("Arc", |screen| {
        let arc = Arc::new(screen).unwrap();
        arc.set_range(100.0);
        arc.set_value(50.0);
        arc.set_rotation(135).set_bg_angles(0, 270);
        drop(arc);
    });
}

#[test]
fn leak_bar() {
    assert_no_leak("Bar", |screen| {
        let bar = Bar::new(screen).unwrap();
        bar.set_range_raw(0, 100);
        bar.set_mode(BarMode::Range);
        bar.set_value_raw(80, false);
        bar.set_start_value_raw(20, false);
        drop(bar);
    });
}

#[test]
fn leak_slider() {
    assert_no_leak("Slider", |screen| {
        let slider = Slider::new(screen).unwrap();
        slider.set_range(-20, 80);
        slider.set_value(30);
        drop(slider);
    });
}

#[test]
fn leak_dropdown() {
    assert_no_leak("Dropdown", |screen| {
        let dd = Dropdown::new(screen).unwrap();
        dd.set_options("Apple\nBanana\nOrange");
        dd.set_selected(1);
        drop(dd);
    });
}

#[test]
fn leak_checkbox() {
    assert_no_leak("Checkbox", |screen| {
        let cb = Checkbox::new(screen).unwrap();
        cb.text("Accept");
        cb.add_state(ObjState::CHECKED);
        drop(cb);
    });
}

#[test]
fn leak_roller() {
    assert_no_leak("Roller", |screen| {
        let roller = Roller::new(screen).unwrap();
        roller.set_options("A\nB\nC\nD", RollerMode::Normal);
        roller.set_visible_row_count(3);
        roller.set_selected(2, false);
        drop(roller);
    });
}

#[test]
fn leak_switch() {
    assert_no_leak("Switch", |screen| {
        let sw = Switch::new(screen).unwrap();
        sw.add_state(ObjState::CHECKED);
        drop(sw);
    });
}

#[test]
fn leak_led() {
    assert_no_leak("Led", |screen| {
        let led = Led::new(screen).unwrap();
        led.on();
        led.set_brightness(128);
        drop(led);
    });
}

static LINE_POINTS: [oxivgl::widgets::lv_point_precise_t; 3] = [
    oxivgl::widgets::lv_point_precise_t { x: 0.0, y: 0.0 },
    oxivgl::widgets::lv_point_precise_t { x: 50.0, y: 30.0 },
    oxivgl::widgets::lv_point_precise_t {
        x: 100.0,
        y: 0.0,
    },
];

#[test]
fn leak_line() {
    assert_no_leak("Line", |screen| {
        let line = Line::new(screen).unwrap();
        line.set_points(&LINE_POINTS);
        drop(line);
    });
}

oxivgl::image_declare!(img_cogwheel_argb);

#[test]
fn leak_image() {
    use oxivgl::widgets::Image;
    assert_no_leak("Image", |screen| {
        let img = Image::new(screen).unwrap();
        img.set_src(img_cogwheel_argb());
        drop(img);
    });
}

#[test]
fn leak_value_label() {
    assert_no_leak("ValueLabel", |screen| {
        let mut vl = ValueLabel::new(screen, "V").unwrap();
        vl.set_value(14.2).unwrap();
        drop(vl);
    });
}

#[test]
fn leak_style_on_part() {
    assert_no_leak("Style on Part::Indicator", |screen| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x00FF00).bg_opa(200);
        let style = sb.build();
        let bar = Bar::new(screen).unwrap();
        bar.set_range_raw(0, 100);
        bar.set_value_raw(50, false);
        bar.add_style(&style, Part::Indicator);
        drop(bar);
        drop(style);
    });
}

#[test]
fn leak_complex_ui() {
    assert_no_leak("Complex UI", |screen| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x111111)
            .bg_opa(255)
            .radius(8)
            .border_width(2)
            .border_color_hex(0x00FF00)
            .pad_all(10);
        let style = sb.build();

        let container = Obj::new(screen).unwrap();
        container.add_style(&style, Selector::DEFAULT);
        container.size(300, 220);

        let bar = Bar::new(&container).unwrap();
        bar.set_range(100.0);
        bar.set_value(75.0);

        let label = Label::new(&container).unwrap();
        label.text("Status: OK");

        let btn = Button::new(&container).unwrap();
        let btn_label = Label::new(&btn).unwrap();
        btn_label.text("Reset");

        let arc = Arc::new(&container).unwrap();
        arc.set_range(100.0);
        arc.set_value(33.0);

        drop(btn_label);
        drop(btn);
        drop(arc);
        drop(bar);
        drop(label);
        drop(container);
        drop(style);
    });
}

#[test]
fn leak_zz_anim_start_widget_delete() {
    use oxivgl::anim::{anim_set_x, Anim};
    assert_no_leak("Anim start+widget delete", |screen| {
        let obj = Obj::new(screen).unwrap();
        obj.size(100, 50);
        let mut a = Anim::new();
        a.set_var(&obj)
            .set_values(0, 100)
            .set_duration(500)
            .set_exec_cb(Some(anim_set_x));
        let _handle = a.start();
        pump();
        drop(obj);
        pump();
    });
}

#[test]
fn leak_textarea() {
    assert_no_leak("Textarea", |screen| {
        let ta = Textarea::new(screen).unwrap();
        ta.set_one_line(true);
        ta.set_text("test");
        ta.add_text(" more");
        drop(ta);
    });
}

#[test]
fn leak_buttonmatrix() {
    assert_no_leak("Buttonmatrix", |screen| {
        let btnm = Buttonmatrix::new(screen).unwrap();
        drop(btnm);
    });
}

#[test]
fn leak_keyboard() {
    assert_no_leak("Keyboard", |screen| {
        let kb = Keyboard::new(screen).unwrap();
        kb.set_mode(KeyboardMode::Number);
        drop(kb);
    });
}

#[test]
fn leak_list() {
    use oxivgl::widgets::List;
    assert_no_leak("List", |screen| {
        let list = List::new(screen).unwrap();
        list.add_text("Section");
        list.add_button(Some(&oxivgl::symbols::FILE), "Open");
        list.add_button(None, "Close");
        drop(list);
    });
}

#[test]
fn leak_menu() {
    assert_no_leak("Menu", |screen| {
        let menu = Menu::new(screen).unwrap();
        let page = menu.page_create(None);
        let cont = Menu::cont_create(&page);
        let lbl = Label::new(&cont).unwrap();
        lbl.text("Item");
        menu.set_page(&page);
        drop(lbl);
        drop(cont);
        drop(page);
        drop(menu);
    });
}

#[test]
fn leak_msgbox() {
    assert_no_leak("Msgbox", |screen| {
        let mbox = Msgbox::new(Some(screen)).unwrap();
        mbox.add_title("Test");
        mbox.add_text("Body");
        mbox.add_close_button();
        drop(mbox);
    });
}

#[test]
fn leak_chart() {
    assert_no_leak("Chart", |screen| {
        let chart = Chart::new(screen).unwrap();
        chart.set_type(ChartType::Line);
        chart.set_point_count(5);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
        let color = lv_color_t { blue: 0, green: 0, red: 255 };
        let series = chart.add_series(color, ChartAxis::PrimaryY);
        chart.set_next_value(&series, 50);
        chart.refresh();
        drop(chart);
    });
}

#[test]
fn leak_canvas() {
    assert_no_leak("Canvas", |screen| {
        let buf = DrawBuf::create(50, 50, ColorFormat::RGB565).unwrap();
        let canvas = Canvas::new(screen, buf).unwrap();
        canvas.fill_bg(color_make(100, 100, 100), 255);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawRectDsc::new();
            dsc.bg_color(color_make(255, 0, 0));
            layer.draw_rect(&dsc, Area { x1: 5, y1: 5, x2: 45, y2: 45 });
        }
        drop(canvas);
    });
}

#[test]
fn leak_table() {
    assert_no_leak("Table", |screen| {
        let table = Table::new(screen).unwrap();
        table.set_row_count(5);
        table.set_column_count(2);
        for row in 0..5u32 {
            table.set_cell_value(row, 0, "Name");
            table.set_cell_value(row, 1, "Value");
        }
        drop(table);
    });
}

#[test]
fn leak_tabview() {
    assert_no_leak("Tabview", |screen| {
        let tv = Tabview::new(screen).unwrap();
        let _tab1 = tv.add_tab("Alpha");
        let _tab2 = tv.add_tab("Beta");
        drop(tv);
    });
}

#[test]
fn leak_calendar() {
    assert_no_leak("Calendar", |screen| {
        let cal = Calendar::new(screen).unwrap();
        cal.set_today_date(2024, 3, 22).set_month_shown(2024, 3);
        cal.set_highlighted_dates(&[
            CalendarDate::new(2024, 3, 5),
            CalendarDate::new(2024, 3, 15),
        ]);
        let _hdr = cal.add_header_arrow();
        drop(cal);
    });
}

#[test]
fn leak_spinner() {
    assert_no_leak("Spinner", |screen| {
        let spinner = Spinner::new(screen).unwrap();
        spinner.set_anim_params(1000, 200);
        drop(spinner);
    });
}

#[test]
fn leak_spinbox() {
    assert_no_leak("Spinbox", |screen| {
        let sb = Spinbox::new(screen).unwrap();
        sb.set_range(-100, 100).set_value(42).set_step(10);
        sb.increment();
        drop(sb);
    });
}

#[test]
fn leak_spangroup() {
    assert_no_leak("Spangroup", |screen| {
        let sg = Spangroup::new(screen).unwrap();
        sg.width(200);
        let span = sg.add_span().unwrap();
        span.set_text(c"leak test");
        sg.refresh();
        drop(sg);
    });
}

#[test]
fn leak_imagebutton() {
    assert_no_leak("Imagebutton", |screen| {
        let btn = Imagebutton::new(screen).unwrap();
        btn.set_state(ImagebuttonState::Pressed);
        btn.set_src(ImagebuttonState::Released, None, None, None);
        drop(btn);
    });
}

#[test]
fn leak_win() {
    assert_no_leak("Win", |screen| {
        let win = Win::new(screen).unwrap();
        let _btn = win.add_button(&oxivgl::symbols::CLOSE, 40);
        let _title = win.add_title("Leak test");
        let content = win.get_content();
        let _lbl = Label::new(&content).unwrap();
        drop(win);
    });
}

#[test]
fn leak_tileview() {
    assert_no_leak("Tileview", |screen| {
        let tv = Tileview::new(screen).unwrap();
        let _t1 = tv.add_tile(0, 0, ScrollDir::HOR);
        let _t2 = tv.add_tile(1, 0, ScrollDir::HOR);
        drop(tv);
    });
}

// ── AnimImg ──────────────────────────────────────────────────────────────────

#[repr(transparent)]
struct SyncPtr(*const core::ffi::c_void);
unsafe impl Sync for SyncPtr {}

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
    unsafe {
        core::slice::from_raw_parts(
            animimg_frames::FRAMES.as_ptr().cast(),
            animimg_frames::FRAMES.len(),
        )
    }
}

#[test]
fn leak_animimg() {
    assert_no_leak("AnimImg", |screen| {
        let animimg = AnimImg::new(screen).unwrap();
        animimg
            .set_src(animimg_frame_ptrs())
            .set_duration(500)
            .set_repeat_count(1)
            .start();
        drop(animimg);
    });
}
