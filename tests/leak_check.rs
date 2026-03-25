// SPDX-License-Identifier: MIT OR Apache-2.0
//! Memory leak detection tests.
//!
//! Each test forks a child process with a fresh LVGL instance and counting
//! global allocator. This gives perfect isolation — no cross-test cache
//! contamination, no warmup needed.
//!
//! The counting allocator tracks ALL malloc/free calls (Rust + LVGL C).
//! After N create/destroy iterations, net allocation must be zero.
//!
//! Run with: `SDL_VIDEODRIVER=dummy cargo +nightly test --test leak_check
//!   --target x86_64-unknown-linux-gnu -- --test-threads=1`

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};

// ── Counting allocator ───────────────────────────────────────────────────────

/// Counting allocator — only tracks when TRACKING_ENABLED is true.
/// This allows the child process to enable tracking only during the
/// measurement window, excluding test harness overhead.
struct CountingAlloc;
static ALLOC_BALANCE: AtomicIsize = AtomicIsize::new(0);
static TRACKING_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() && TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOC_BALANCE.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOC_BALANCE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) };
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() && TRACKING_ENABLED.load(Ordering::Relaxed) {
            ALLOC_BALANCE.fetch_add(new_size as isize - layout.size() as isize, Ordering::Relaxed);
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

fn total_alloc_bytes() -> isize {
    ALLOC_BALANCE.load(Ordering::Relaxed)
}

fn start_tracking() {
    ALLOC_BALANCE.store(0, Ordering::Relaxed);
    TRACKING_ENABLED.store(true, Ordering::Relaxed);
}

fn stop_tracking() {
    TRACKING_ENABLED.store(false, Ordering::Relaxed);
}

// ── Forked test runner ───────────────────────────────────────────────────────

const MEASURE: isize = 100;
const NOISE_FLOOR: isize = 32;

/// Fork a child process, run `test_fn` in a fresh LVGL instance.
/// Uses libc fork() directly — child inherits the counting allocator
/// but gets a completely fresh LVGL via LvglDriver::init.
fn run_isolated(name: &str, test_fn: fn() -> isize) {
    use std::io::Read;
    use std::os::unix::io::FromRawFd;

    unsafe extern "C" {
        fn pipe(fds: *mut i32) -> i32;
        fn fork() -> i32;
        fn _exit(status: i32) -> !;
        fn close(fd: i32) -> i32;
        fn write(fd: i32, buf: *const u8, count: usize) -> isize;
        fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    }

    let mut fds = [0i32; 2];
    assert_eq!(unsafe { pipe(fds.as_mut_ptr()) }, 0);
    let (read_fd, write_fd) = (fds[0], fds[1]);

    let pid = unsafe { fork() };
    assert!(pid >= 0, "fork failed");

    if pid == 0 {
        // ── Child: fresh LVGL, run test, write result, exit ──
        unsafe { close(read_fd) };

        // Silence child output to avoid confusing the test harness
        unsafe extern "C" { fn open(path: *const u8, flags: i32) -> i32; fn dup2(old: i32, new: i32) -> i32; }
        let devnull = unsafe { open(b"/dev/null\0".as_ptr(), 2) }; // O_RDWR=2
        if devnull >= 0 {
            unsafe { dup2(devnull, 1); dup2(devnull, 2); close(devnull); }
        }

        // Init LVGL with tracking disabled — don't count init allocations
        let _driver = oxivgl::driver::LvglDriver::init(320, 240);

        let per_iter = test_fn();

        let bytes = per_iter.to_le_bytes();
        unsafe { write(write_fd, bytes.as_ptr(), bytes.len()) };
        unsafe { close(write_fd) };
        unsafe { _exit(0) };
    }

    // ── Parent: read result, assert ──
    unsafe { close(write_fd) };

    let mut status = 0i32;
    unsafe { waitpid(pid, &mut status, 0) };
    // Check WIFEXITED && WEXITSTATUS == 0
    let exited_normally = (status & 0x7f) == 0;
    let exit_code = (status >> 8) & 0xff;
    assert!(
        exited_normally && exit_code == 0,
        "{name}: child crashed or failed (raw status {status:#x})"
    );

    let mut buf = [0u8; 8];
    let mut file = unsafe { std::fs::File::from_raw_fd(read_fd) };
    file.read_exact(&mut buf).expect("failed to read from child");
    let per_iter = isize::from_le_bytes(buf);

    assert!(
        per_iter.abs() <= NOISE_FLOOR,
        "{name}: leaked {per_iter} bytes/iter (noise floor ±{NOISE_FLOOR})"
    );
}

// ── Test helpers (run inside child) ──────────────────────────────────────────

fn screen() -> oxivgl::widgets::Screen {
    oxivgl::widgets::Screen::active().expect("no active screen")
}

fn pump_child() {
    unsafe { lvgl_rust_sys::lv_refr_now(core::ptr::null_mut()) };
}

/// Measure per-iteration leak for a widget test closure.
fn measure_widget(f: impl Fn(&oxivgl::widgets::Screen)) -> isize {
    let screen = screen();
    pump_child();
    start_tracking();
    let before = total_alloc_bytes();
    for _ in 0..MEASURE {
        f(&screen);
        pump_child();
    }
    let after = total_alloc_bytes();
    stop_tracking();
    (after - before) / MEASURE
}

/// Measure per-iteration leak for a pure Rust closure (no widgets).
fn measure_rust(f: impl Fn()) -> isize {
    start_tracking();
    let before = total_alloc_bytes();
    for _ in 0..MEASURE {
        f();
    }
    let after = total_alloc_bytes();
    stop_tracking();
    (after - before) / MEASURE
}

// ── Imports for test closures ────────────────────────────────────────────────

use oxivgl::{
    anim::anim_path_linear,
    draw::{Area, DrawRectDsc},
    draw_buf::{ColorFormat, DrawBuf},
    style::{
        color_make, palette_main, props, GradDsc, GradExtend, Palette, Selector, StyleBuilder,
        TransitionDsc,
    },
    enums::{ObjState, ScrollDir},
    widgets::{
        AnimImg, Arc, Bar, BarMode, Button, Buttonmatrix, Calendar, CalendarDate, Canvas, Chart,
        ChartAxis, ChartType, Checkbox, Dropdown, Imagebutton, ImagebuttonState, Keyboard,
        KeyboardMode, Label, Led, Line, Menu, Msgbox, Obj, Part, Roller, RollerMode, Slider,
        Spangroup, Spinbox, Spinner, Switch, Table, Tabview, Textarea, Tileview, ValueLabel, Win,
        lv_color_t,
    },
};

// ── LVGL C baseline ──────────────────────────────────────────────────────────

#[test]
fn leak_aa_lvgl_baseline() {
    run_isolated("LVGL C baseline", || {
        let screen_ptr = unsafe { lvgl_rust_sys::lv_screen_active() };
        start_tracking();
        let before = total_alloc_bytes();
        for _ in 0..MEASURE {
            unsafe {
                let obj = lvgl_rust_sys::lv_obj_create(screen_ptr);
                lvgl_rust_sys::lv_obj_set_size(obj, 100, 50);
                lvgl_rust_sys::lv_obj_delete(obj);
                lvgl_rust_sys::lv_refr_now(core::ptr::null_mut());
            }
        }
        let after = total_alloc_bytes();
        stop_tracking();
        (after - before) / MEASURE
    });
}

// ── Pure Rust leak tests ─────────────────────────────────────────────────────

static TRANS_PROPS: [props::lv_style_prop_t; 3] = [props::BG_COLOR, props::BG_OPA, props::LAST];

#[test]
fn leak_style_build_drop() {
    run_isolated("Style build/drop", || {
        measure_rust(|| {
            let mut sb = StyleBuilder::new();
            sb.bg_color_hex(0xFF0000).bg_opa(255).radius(5).border_width(2).border_color_hex(0x00FF00);
            drop(sb.build());
        })
    });
}

#[test]
fn leak_style_with_grad_no_widget() {
    run_isolated("Style+GradDsc (no widget)", || {
        measure_rust(|| {
            let mut grad = GradDsc::new();
            grad.init_stops(
                &[palette_main(Palette::Blue), palette_main(Palette::Red)],
                &[255, 255], &[0, 255],
            ).linear(0, 0, 100, 0, GradExtend::Pad);
            let mut sb = StyleBuilder::new();
            sb.bg_opa(255).bg_grad(grad);
            drop(sb.build());
        })
    });
}

#[test]
fn leak_style_with_transition_no_widget() {
    run_isolated("Style+TransitionDsc (no widget)", || {
        measure_rust(|| {
            let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
            let mut sb = StyleBuilder::new();
            sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
            drop(sb.build());
        })
    });
}

// ── Widget leak tests ────────────────────────────────────────────────────────

#[test]
fn leak_obj_create_destroy() {
    run_isolated("Obj", || measure_widget(|s| { drop(Obj::new(s).unwrap()); }));
}

#[test]
fn leak_label() {
    run_isolated("Label", || measure_widget(|s| {
        let l = Label::new(s).unwrap(); l.text("hello world"); drop(l);
    }));
}

#[test]
fn leak_button_with_label() {
    run_isolated("Button+Label", || measure_widget(|s| {
        let btn = Button::new(s).unwrap();
        let lbl = Label::new(&btn).unwrap();
        lbl.text("Click me"); drop(lbl); drop(btn);
    }));
}

#[test]
fn leak_style_add_remove() {
    run_isolated("Style add/remove", || measure_widget(|s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x0000FF).bg_opa(200);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        obj.remove_style_all(); drop(obj); drop(style);
    }));
}

#[test]
fn leak_style_shared() {
    run_isolated("Style shared", || measure_widget(|s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x123456).bg_opa(255);
        let style = sb.build();
        let o1 = Obj::new(s).unwrap(); let o2 = Obj::new(s).unwrap();
        o1.add_style(&style, Selector::DEFAULT);
        o2.add_style(&style, Selector::DEFAULT);
        drop(o1); drop(o2); drop(style);
    }));
}

#[test]
fn leak_style_with_grad() {
    run_isolated("Style+GradDsc", || measure_widget(|s| {
        let mut grad = GradDsc::new();
        grad.init_stops(
            &[palette_main(Palette::Blue), palette_main(Palette::Red)],
            &[255, 255], &[0, 255],
        ).linear(0, 0, 100, 0, GradExtend::Pad);
        let mut sb = StyleBuilder::new();
        sb.bg_opa(255).bg_grad(grad);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT); obj.size(80, 40);
        drop(obj); drop(style);
    }));
}

#[test]
fn leak_style_with_transition() {
    run_isolated("Style+TransitionDsc", || measure_widget(|s| {
        let trans = TransitionDsc::new(&TRANS_PROPS, Some(anim_path_linear), 200, 0);
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF0000).bg_opa(255).transition(trans);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        drop(obj); drop(style);
    }));
}

#[test]
fn leak_style_drop_before_widget() {
    run_isolated("Style dropped before widget", || measure_widget(|s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0xFF00FF).bg_opa(255).radius(10);
        let style = sb.build();
        let obj = Obj::new(s).unwrap();
        obj.add_style(&style, Selector::DEFAULT);
        drop(style); drop(obj);
    }));
}

#[test]
fn leak_nested_widgets() {
    run_isolated("Nested widgets", || measure_widget(|s| {
        let c = Obj::new(s).unwrap(); c.size(200, 200);
        let btn = Button::new(&c).unwrap();
        let lbl = Label::new(&btn).unwrap();
        lbl.text("Nested"); drop(lbl); drop(btn); drop(c);
    }));
}

#[test]
fn leak_arc() {
    run_isolated("Arc", || measure_widget(|s| {
        let arc = Arc::new(s).unwrap();
        arc.set_range(100.0); arc.set_value(50.0);
        arc.set_rotation(135).set_bg_angles(0, 270);
        drop(arc);
    }));
}

#[test]
fn leak_bar() {
    run_isolated("Bar", || measure_widget(|s| {
        let bar = Bar::new(s).unwrap();
        bar.set_range_raw(0, 100); bar.set_mode(BarMode::Range);
        bar.set_value_raw(80, false); bar.set_start_value_raw(20, false);
        drop(bar);
    }));
}

#[test]
fn leak_slider() {
    run_isolated("Slider", || measure_widget(|s| {
        let sl = Slider::new(s).unwrap();
        sl.set_range(-20, 80); sl.set_value(30); drop(sl);
    }));
}

#[test]
fn leak_dropdown() {
    run_isolated("Dropdown", || measure_widget(|s| {
        let dd = Dropdown::new(s).unwrap();
        dd.set_options("Apple\nBanana\nOrange"); dd.set_selected(1); drop(dd);
    }));
}

#[test]
fn leak_checkbox() {
    run_isolated("Checkbox", || measure_widget(|s| {
        let cb = Checkbox::new(s).unwrap();
        cb.text("Accept"); cb.add_state(ObjState::CHECKED); drop(cb);
    }));
}

#[test]
fn leak_roller() {
    run_isolated("Roller", || measure_widget(|s| {
        let r = Roller::new(s).unwrap();
        r.set_options("A\nB\nC\nD", RollerMode::Normal);
        r.set_visible_row_count(3); r.set_selected(2, false); drop(r);
    }));
}

#[test]
fn leak_switch() {
    run_isolated("Switch", || measure_widget(|s| {
        let sw = Switch::new(s).unwrap();
        sw.add_state(ObjState::CHECKED); drop(sw);
    }));
}

#[test]
fn leak_led() {
    run_isolated("Led", || measure_widget(|s| {
        let led = Led::new(s).unwrap();
        led.on(); led.set_brightness(128); drop(led);
    }));
}

static LINE_POINTS: [oxivgl::widgets::lv_point_precise_t; 3] = [
    oxivgl::widgets::lv_point_precise_t { x: 0.0, y: 0.0 },
    oxivgl::widgets::lv_point_precise_t { x: 50.0, y: 30.0 },
    oxivgl::widgets::lv_point_precise_t { x: 100.0, y: 0.0 },
];

#[test]
fn leak_line() {
    run_isolated("Line", || measure_widget(|s| {
        let l = Line::new(s).unwrap(); l.set_points(&LINE_POINTS); drop(l);
    }));
}

oxivgl::image_declare!(img_cogwheel_argb);

#[test]
fn leak_image() {
    run_isolated("Image", || measure_widget(|s| {
        use oxivgl::widgets::Image;
        let img = Image::new(s).unwrap();
        img.set_src(img_cogwheel_argb()); drop(img);
    }));
}

#[test]
fn leak_value_label() {
    run_isolated("ValueLabel", || measure_widget(|s| {
        let mut vl = ValueLabel::new(s, "V").unwrap();
        vl.set_value(14.2).unwrap(); drop(vl);
    }));
}

#[test]
fn leak_style_on_part() {
    run_isolated("Style on Part::Indicator", || measure_widget(|s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x00FF00).bg_opa(200);
        let style = sb.build();
        let bar = Bar::new(s).unwrap();
        bar.set_range_raw(0, 100); bar.set_value_raw(50, false);
        bar.add_style(&style, Part::Indicator);
        drop(bar); drop(style);
    }));
}

#[test]
fn leak_complex_ui() {
    run_isolated("Complex UI", || measure_widget(|s| {
        let mut sb = StyleBuilder::new();
        sb.bg_color_hex(0x111111).bg_opa(255).radius(8)
            .border_width(2).border_color_hex(0x00FF00).pad_all(10);
        let style = sb.build();
        let c = Obj::new(s).unwrap();
        c.add_style(&style, Selector::DEFAULT); c.size(300, 220);
        let bar = Bar::new(&c).unwrap(); bar.set_range(100.0); bar.set_value(75.0);
        let lbl = Label::new(&c).unwrap(); lbl.text("Status: OK");
        let btn = Button::new(&c).unwrap();
        let bl = Label::new(&btn).unwrap(); bl.text("Reset");
        let arc = Arc::new(&c).unwrap(); arc.set_range(100.0); arc.set_value(33.0);
        drop(bl); drop(btn); drop(arc); drop(bar); drop(lbl); drop(c); drop(style);
    }));
}

#[test]
fn leak_zz_anim_start_widget_delete() {
    run_isolated("Anim start+widget delete", || measure_widget(|s| {
        use oxivgl::anim::{anim_set_x, Anim};
        let obj = Obj::new(s).unwrap(); obj.size(100, 50);
        let mut a = Anim::new();
        a.set_var(&obj).set_values(0, 100).set_duration(500)
            .set_exec_cb(Some(anim_set_x));
        let _h = a.start();
        pump_child();
        drop(obj);
        pump_child();
    }));
}

#[test]
fn leak_textarea() {
    run_isolated("Textarea", || measure_widget(|s| {
        let ta = Textarea::new(s).unwrap();
        ta.set_one_line(true); ta.set_text("test"); ta.add_text(" more"); drop(ta);
    }));
}

#[test]
fn leak_buttonmatrix() {
    run_isolated("Buttonmatrix", || measure_widget(|s| {
        drop(Buttonmatrix::new(s).unwrap());
    }));
}

#[test]
fn leak_keyboard() {
    run_isolated("Keyboard", || measure_widget(|s| {
        let kb = Keyboard::new(s).unwrap();
        kb.set_mode(KeyboardMode::Number); drop(kb);
    }));
}

#[test]
fn leak_list() {
    run_isolated("List", || measure_widget(|s| {
        use oxivgl::widgets::List;
        let list = List::new(s).unwrap();
        list.add_text("Section");
        list.add_button(Some(&oxivgl::symbols::FILE), "Open");
        list.add_button(None, "Close");
        drop(list);
    }));
}

#[test]
fn leak_menu() {
    run_isolated("Menu", || measure_widget(|s| {
        let menu = Menu::new(s).unwrap();
        let page = menu.page_create(None);
        let cont = Menu::cont_create(&page);
        let lbl = Label::new(&cont).unwrap(); lbl.text("Item");
        menu.set_page(&page);
        drop(lbl); drop(cont); drop(page); drop(menu);
    }));
}

#[test]
fn leak_msgbox() {
    run_isolated("Msgbox", || measure_widget(|s| {
        let mbox = Msgbox::new(Some(s)).unwrap();
        mbox.add_title("Test"); mbox.add_text("Body"); mbox.add_close_button();
        drop(mbox);
    }));
}

#[test]
fn leak_chart() {
    run_isolated("Chart", || measure_widget(|s| {
        let chart = Chart::new(s).unwrap();
        chart.set_type(ChartType::Line); chart.set_point_count(5);
        chart.set_axis_range(ChartAxis::PrimaryY, 0, 100);
        let color = lv_color_t { blue: 0, green: 0, red: 255 };
        let series = chart.add_series(color, ChartAxis::PrimaryY);
        chart.set_next_value(&series, 50); chart.refresh(); drop(chart);
    }));
}

#[test]
fn leak_canvas() {
    run_isolated("Canvas", || measure_widget(|s| {
        let buf = DrawBuf::create(50, 50, ColorFormat::RGB565).unwrap();
        let canvas = Canvas::new(s, buf).unwrap();
        canvas.fill_bg(color_make(100, 100, 100), 255);
        {
            let mut layer = canvas.init_layer();
            let mut dsc = DrawRectDsc::new();
            dsc.bg_color(color_make(255, 0, 0));
            layer.draw_rect(&dsc, Area { x1: 5, y1: 5, x2: 45, y2: 45 });
        }
        drop(canvas);
    }));
}

#[test]
fn leak_table() {
    run_isolated("Table", || measure_widget(|s| {
        let t = Table::new(s).unwrap();
        t.set_row_count(5); t.set_column_count(2);
        for row in 0..5u32 { t.set_cell_value(row, 0, "Name"); t.set_cell_value(row, 1, "Val"); }
        drop(t);
    }));
}

#[test]
fn leak_tabview() {
    run_isolated("Tabview", || measure_widget(|s| {
        let tv = Tabview::new(s).unwrap();
        let _t1 = tv.add_tab("Alpha"); let _t2 = tv.add_tab("Beta"); drop(tv);
    }));
}

#[test]
fn leak_calendar() {
    run_isolated("Calendar", || measure_widget(|s| {
        let cal = Calendar::new(s).unwrap();
        cal.set_today_date(2024, 3, 22).set_month_shown(2024, 3);
        cal.set_highlighted_dates(&[CalendarDate::new(2024, 3, 5), CalendarDate::new(2024, 3, 15)]);
        let _hdr = cal.add_header_arrow(); drop(cal);
    }));
}

#[test]
fn leak_spinner() {
    run_isolated("Spinner", || measure_widget(|s| {
        let sp = Spinner::new(s).unwrap(); sp.set_anim_params(1000, 200); drop(sp);
    }));
}

#[test]
fn leak_spinbox() {
    run_isolated("Spinbox", || measure_widget(|s| {
        let sb = Spinbox::new(s).unwrap();
        sb.set_range(-100, 100).set_value(42).set_step(10); sb.increment(); drop(sb);
    }));
}

#[test]
fn leak_spangroup() {
    run_isolated("Spangroup", || measure_widget(|s| {
        let sg = Spangroup::new(s).unwrap(); sg.width(200);
        let span = sg.add_span().unwrap(); span.set_text(c"leak test");
        sg.refresh(); drop(sg);
    }));
}

#[test]
fn leak_imagebutton() {
    run_isolated("Imagebutton", || measure_widget(|s| {
        let btn = Imagebutton::new(s).unwrap();
        btn.set_state(ImagebuttonState::Pressed);
        btn.set_src(ImagebuttonState::Released, None, None, None); drop(btn);
    }));
}

#[test]
fn leak_win() {
    run_isolated("Win", || measure_widget(|s| {
        let win = Win::new(s).unwrap();
        let _b = win.add_button(&oxivgl::symbols::CLOSE, 40);
        let _t = win.add_title("Leak test");
        let c = win.get_content();
        let _l = Label::new(&c).unwrap();
        drop(win);
    }));
}

#[test]
fn leak_tileview() {
    run_isolated("Tileview", || measure_widget(|s| {
        let tv = Tileview::new(s).unwrap();
        let _t1 = tv.add_tile(0, 0, ScrollDir::HOR);
        let _t2 = tv.add_tile(1, 0, ScrollDir::HOR);
        drop(tv);
    }));
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
    run_isolated("AnimImg", || measure_widget(|s| {
        let a = AnimImg::new(s).unwrap();
        a.set_src(animimg_frame_ptrs()).set_duration(500).set_repeat_count(1).start();
        drop(a);
    }));
}
