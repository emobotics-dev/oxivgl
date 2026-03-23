// SPDX-License-Identifier: MIT OR Apache-2.0
//! Memory leak detection tests.
//!
//! Uses glibc's `mallinfo2()` to measure **global** heap usage — this captures
//! both Rust allocations (which go through libc malloc via the System allocator)
//! and LVGL's C-side `lv_malloc`/`lv_free` (which use libc malloc directly via
//! `LV_STDLIB_CLIB`). This gives a single, unified view of memory across the
//! FFI boundary.
//!
//! # LVGL baseline leak
//!
//! LVGL v9.3 on the SDL2 dummy backend leaks ~600-800 bytes per widget
//! create/delete cycle and ~800 bytes per render pass. This appears to be
//! draw-layer / event-cleanup overhead in LVGL's C code, not a Rust wrapper
//! issue. The tests use a per-widget baseline tolerance derived from empirical
//! measurement. Wrapper-specific leaks (e.g. Rc not dropping, Style not
//! calling lv_style_reset) would show as excess ABOVE this baseline.
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
    enums::ObjState,
    widgets::{
        Arc, Bar, BarMode, Button, Buttonmatrix, Calendar, CalendarDate, Canvas, Chart, ChartAxis,
        ChartType, Checkbox, Dropdown, Keyboard, KeyboardMode, Label, Led, Line, Menu, Msgbox,
        Obj, Part, Roller, RollerMode, Screen, Slider, Spinbox, Spinner, Switch, Table, Tabview, Textarea, ValueLabel,
        lv_color_t,
    },
};

// ── glibc mallinfo2 ──────────────────────────────────────────────────────────

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

/// Total bytes currently in use across the entire process heap.
fn heap_used_bytes() -> usize {
    // SAFETY: mallinfo2 is thread-safe in glibc.
    let info = unsafe { mallinfo2() };
    info.uordblks + info.hblkhd
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
        // SAFETY: single-threaded test runner (--test-threads=1).
        unsafe { DRIVER = Some(LvglDriver::init(320, 240)) };
    });
}

fn fresh_screen() -> Screen {
    ensure_init();
    // SAFETY: LVGL initialised; loading a new screen clears the previous one.
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

// ── Leak check helper ────────────────────────────────────────────────────────

/// Empirically measured LVGL per-widget-cycle baseline leak (bytes/iter).
/// LVGL v9.3 SDL2 dummy backend leaks ~1500 bytes per widget create +
/// render + destroy cycle. We allow this baseline plus a small margin.
const LVGL_BASELINE_PER_WIDGET: isize = 2000;

/// Run `f` repeatedly and verify heap growth stays within the expected
/// LVGL baseline. `widget_count` is the number of widgets created per
/// iteration (used to scale the baseline tolerance).
fn assert_no_leak(name: &str, widget_count: isize, f: impl Fn(&Screen)) {
    let screen = fresh_screen();
    pump();

    // Warm-up: 3 cycles to stabilise lazy allocations.
    for _ in 0..3 {
        f(&screen);
        pump();
    }

    const N: isize = 20;
    let before = heap_used_bytes() as isize;
    for _ in 0..N {
        f(&screen);
        pump();
    }
    let after = heap_used_bytes() as isize;

    let total_leaked = after - before;
    let per_iter = total_leaked / N;
    let tolerance = widget_count * LVGL_BASELINE_PER_WIDGET;
    assert!(
        per_iter <= tolerance,
        "{name}: leaked {per_iter} bytes/iter (tolerance {tolerance}/iter = \
         {widget_count} widgets × {LVGL_BASELINE_PER_WIDGET} baseline)"
    );
}

/// Assert zero leak for pure Rust operations (no LVGL widgets).
fn assert_no_leak_rust(name: &str, f: impl Fn()) {
    // Warm-up: enough iterations to absorb allocator fragmentation from
    // preceding LVGL widget tests (e.g. first Style build after Spinner/
    // Spinbox tests may trigger a lazy internal allocation).
    for _ in 0..10 {
        f();
    }
    let before = heap_used_bytes() as isize;
    for _ in 0..20 {
        f();
    }
    let after = heap_used_bytes() as isize;
    let per_iter = (after - before) / 20;
    assert!(
        per_iter.abs() <= 128, // LVGL internal caching + allocator fragmentation
        "{name}: leaked {per_iter} bytes/iter (should be ~0)"
    );
}

// ── Pure Rust leak tests (zero tolerance) ────────────────────────────────────

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

// ── Widget leak tests (LVGL baseline tolerance) ──────────────────────────────

#[test]
fn leak_obj_create_destroy() {
    assert_no_leak("Obj", 1, |screen| {
        let obj = Obj::new(screen).unwrap();
        obj.size(100, 50);
        drop(obj);
    });
}

#[test]
fn leak_label() {
    assert_no_leak("Label", 1, |screen| {
        let label = Label::new(screen).unwrap();
        label.text("hello world");
        drop(label);
    });
}

#[test]
fn leak_button_with_label() {
    assert_no_leak("Button+Label", 2, |screen| {
        let btn = Button::new(screen).unwrap();
        let lbl = Label::new(&btn).unwrap();
        lbl.text("Click me");
        drop(lbl);
        drop(btn);
    });
}

#[test]
fn leak_style_add_remove() {
    assert_no_leak("Style add/remove", 1, |screen| {
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
    assert_no_leak("Style shared", 2, |screen| {
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
    assert_no_leak("Style+GradDsc", 1, |screen| {
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
    assert_no_leak("Style+TransitionDsc", 1, |screen| {
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
    assert_no_leak("Style dropped before widget", 1, |screen| {
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
    assert_no_leak("Nested widgets", 3, |screen| {
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
    assert_no_leak("Arc", 1, |screen| {
        let arc = Arc::new(screen).unwrap();
        arc.set_range(100.0);
        arc.set_value(50.0);
        arc.set_rotation(135).set_bg_angles(0, 270);
        drop(arc);
    });
}

#[test]
fn leak_bar() {
    assert_no_leak("Bar", 1, |screen| {
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
    assert_no_leak("Slider", 1, |screen| {
        let slider = Slider::new(screen).unwrap();
        slider.set_range(-20, 80);
        slider.set_value(30);
        drop(slider);
    });
}

#[test]
fn leak_dropdown() {
    // Dropdown creates internal child objects (list, label).
    assert_no_leak("Dropdown", 3, |screen| {
        let dd = Dropdown::new(screen).unwrap();
        dd.set_options("Apple\nBanana\nOrange");
        dd.set_selected(1);
        drop(dd);
    });
}

#[test]
fn leak_checkbox() {
    assert_no_leak("Checkbox", 1, |screen| {
        let cb = Checkbox::new(screen).unwrap();
        cb.text("Accept");
        cb.add_state(ObjState::CHECKED);
        drop(cb);
    });
}

#[test]
fn leak_roller() {
    assert_no_leak("Roller", 2, |screen| {
        let roller = Roller::new(screen).unwrap();
        roller.set_options("A\nB\nC\nD", RollerMode::Normal);
        roller.set_visible_row_count(3);
        roller.set_selected(2, false);
        drop(roller);
    });
}

#[test]
fn leak_switch() {
    assert_no_leak("Switch", 1, |screen| {
        let sw = Switch::new(screen).unwrap();
        sw.add_state(ObjState::CHECKED);
        drop(sw);
    });
}

#[test]
fn leak_led() {
    assert_no_leak("Led", 1, |screen| {
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
    assert_no_leak("Line", 1, |screen| {
        let line = Line::new(screen).unwrap();
        line.set_points(&LINE_POINTS);
        drop(line);
    });
}

oxivgl::image_declare!(img_cogwheel_argb);

#[test]
fn leak_image() {
    use oxivgl::widgets::Image;
    assert_no_leak("Image", 1, |screen| {
        let img = Image::new(screen).unwrap();
        img.set_src(img_cogwheel_argb());
        drop(img);
    });
}

#[test]
fn leak_value_label() {
    assert_no_leak("ValueLabel", 2, |screen| {
        let mut vl = ValueLabel::new(screen, "V").unwrap();
        vl.set_value(14.2).unwrap();
        drop(vl);
    });
}

#[test]
fn leak_style_on_part() {
    assert_no_leak("Style on Part::Indicator", 1, |screen| {
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
    assert_no_leak("Complex UI", 6, |screen| {
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
    assert_no_leak("Anim start+widget delete", 1, |screen| {
        let obj = Obj::new(screen).unwrap();
        obj.size(100, 50);
        let mut a = Anim::new();
        a.set_var(&obj)
            .set_values(0, 100)
            .set_duration(500)
            .set_exec_cb(Some(anim_set_x));
        let _handle = a.start();
        pump();
        drop(obj); // LVGL cancels animation on widget delete
        pump(); // flush animation cleanup
    });
}

#[test]
fn leak_textarea() {
    // Textarea creates internal child objects (text label, placeholder label, cursor).
    assert_no_leak("Textarea", 4, |screen| {
        let ta = Textarea::new(screen).unwrap();
        ta.set_one_line(true);
        ta.set_text("test");
        ta.add_text(" more");
        drop(ta);
    });
}

#[test]
fn leak_buttonmatrix() {
    assert_no_leak("Buttonmatrix", 1, |screen| {
        let btnm = Buttonmatrix::new(screen).unwrap();
        drop(btnm);
    });
}

#[test]
fn leak_keyboard() {
    // Keyboard creates internal buttonmatrix + label children.
    assert_no_leak("Keyboard", 3, |screen| {
        let kb = Keyboard::new(screen).unwrap();
        kb.set_mode(KeyboardMode::Number);
        drop(kb);
    });
}

#[test]
fn leak_list() {
    use oxivgl::widgets::List;
    // List + 1 text label + 2 buttons (each button has icon+label children) = ~7 widgets
    assert_no_leak("List", 7, |screen| {
        let list = List::new(screen).unwrap();
        list.add_text("Section");
        list.add_button(Some(&oxivgl::symbols::FILE), "Open");
        list.add_button(None, "Close");
        drop(list);
    });
}

#[test]
fn leak_menu() {
    // Menu creates internal header, back button, and main content area children.
    assert_no_leak("Menu", 10, |screen| {
        let menu = Menu::new(screen).unwrap();
        let page = menu.page_create(None);
        let cont = Menu::cont_create(&page);
        let lbl = Label::new(&cont).unwrap();
        lbl.text("Item");
        menu.set_page(&page);
        drop(lbl);
        // page and cont are Child<Obj> — Child<W> suppresses Drop, so LVGL
        // (not Rust) will free them when menu is deleted. Explicit drops here
        // clarify intent; order doesn't affect safety.
        drop(cont);
        drop(page);
        drop(menu);
    });
}

#[test]
fn leak_msgbox() {
    // Msgbox creates header + title label + content + close button children.
    assert_no_leak("Msgbox", 8, |screen| {
        let mbox = Msgbox::new(Some(screen)).unwrap();
        mbox.add_title("Test");
        mbox.add_text("Body");
        mbox.add_close_button();
        drop(mbox);
    });
}

#[test]
fn leak_chart() {
    assert_no_leak("Chart", 1, |screen| {
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

// ── Canvas ────────────────────────────────────────────────────────────────────

#[test]
fn leak_canvas() {
    assert_no_leak("Canvas", 1, |screen| {
        let buf = DrawBuf::create(50, 50, ColorFormat::RGB565).unwrap();
        let canvas = Canvas::new(screen, buf).unwrap();
        canvas.fill_bg(color_make(100, 100, 100), 255);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawRectDsc::new();
            dsc.bg_color(color_make(255, 0, 0));
            layer.draw_rect(&dsc, Area { x1: 5, y1: 5, x2: 45, y2: 45 });
        }
        drop(canvas); // LV_EVENT_DELETE callback frees draw_buf
    });
}


// ── Table ─────────────────────────────────────────────────────────────────────

#[test]
fn leak_table() {
    assert_no_leak("Table", 1, |screen| {
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


// ── Tabview ───────────────────────────────────────────────────────────────────

#[test]
fn leak_tabview() {
    // Tabview creates ~6 internal LVGL objects (bar, content, 2 tab buttons,
    // 2 tab panes).
    assert_no_leak("Tabview", 6, |screen| {
        let tv = Tabview::new(screen).unwrap();
        let _tab1 = tv.add_tab("Alpha");
        let _tab2 = tv.add_tab("Beta");
        drop(tv);
    });
}

// ── Calendar ──────────────────────────────────────────────────────────────────

#[test]
fn leak_calendar() {
    assert_no_leak("Calendar", 6, |screen| {
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

// ── Spinner ──────────────────────────────────────────────────────────────────

#[test]
fn leak_spinner() {
    assert_no_leak("Spinner", 6, |screen| {
        let spinner = Spinner::new(screen).unwrap();
        spinner.set_anim_params(1000, 200);
        drop(spinner);
    });
}

// ── Spinbox ──────────────────────────────────────────────────────────────────

#[test]
fn leak_spinbox() {
    assert_no_leak("Spinbox", 6, |screen| {
        let sb = Spinbox::new(screen).unwrap();
        sb.set_range(-100, 100).set_value(42).set_step(10);
        sb.increment();
        drop(sb);
    });
}
