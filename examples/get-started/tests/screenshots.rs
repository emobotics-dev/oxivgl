// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration test: renders each getting-started example, captures PPM screenshots.

use std::io::Write as _;
use std::path::PathBuf;

use lvgl_rust_sys::*;
use oxivgl::{
    lvgl::LvglDriver,
    widgets::{
        Align, AsLvHandle, Button, ColorFilter, GradDir, Label, Palette, Screen, Slider, Style,
        darken_filter_cb, palette_lighten, palette_main,
    },
};

const W: i32 = 320;
const H: i32 = 240;

fn write_ppm(path: &std::path::Path, w: u32, h: u32, data: &[u8]) -> std::io::Result<()> {
    let stride = w as usize * 2;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(f, "P6\n{w} {h}\n255\n")?;
    for row in 0..h as usize {
        for col in 0..w as usize {
            let off = row * stride + col * 2;
            let p = u16::from_le_bytes([data[off], data[off + 1]]);
            let r = ((p >> 11) & 0x1F) as u8;
            let g = ((p >> 5) & 0x3F) as u8;
            let b = (p & 0x1F) as u8;
            f.write_all(&[(r << 3) | (r >> 2), (g << 2) | (g >> 4), (b << 3) | (b >> 2)])?;
        }
    }
    Ok(())
}

fn pump(n: u32) {
    for _ in 0..n {
        unsafe { lv_timer_handler() };
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn capture(name: &str) {
    let draw_buf = unsafe {
        lv_snapshot_take(lv_screen_active(), lv_color_format_t_LV_COLOR_FORMAT_RGB565)
    };
    assert!(!draw_buf.is_null(), "lv_snapshot_take returned NULL");

    let buf = unsafe { &*draw_buf };
    let w = buf.header.w();
    let h = buf.header.h();
    let data = unsafe { std::slice::from_raw_parts(buf.data, buf.data_size as usize) };

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("screenshots");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}.ppm"));
    write_ppm(&path, w, h, data).expect("PPM write failed");
    println!("Screenshot: {}", path.display());

    unsafe { lv_draw_buf_destroy(draw_buf) };
}

#[test]
fn screenshots_all() {
    let _ = env_logger::try_init();
    let _driver = LvglDriver::init(W, H);

    // ── Ex1: Hello World ────────────────────────────────────────────────────────
    {
        let screen = Screen::active().expect("active screen");
        screen.bg_color(0x003a57).bg_opa(255);
        screen.text_color(0xffffff);

        let _label = Label::new(&screen).expect("label");
        _label.text("Hello world\0").expect("text").align(Align::Center, 0, 0);

        pump(10);
        capture("ex1");
        // _label dropped → lv_obj_delete; screen has no Drop.
    }

    // ── Ex2: Button ─────────────────────────────────────────────────────────────
    unsafe { lv_screen_load(lv_obj_create(core::ptr::null_mut())) };
    {
        let screen = Screen::active().expect("active screen");

        let _btn = Button::new(&screen).expect("button");
        _btn.pos(10, 10).size(120, 50);

        let _label = Label::new(&_btn).expect("label");
        _label.text("Button\0").expect("text").center();

        pump(10);
        capture("ex2");
    }

    // ── Ex3: Custom styles ───────────────────────────────────────────────────────
    unsafe { lv_screen_load(lv_obj_create(core::ptr::null_mut())) };
    {
        const LV_OPA_COVER: u8 = 255;
        const LV_OPA_20: u8 = 51;

        // Declare color_filter before styles so it outlives them on drop.
        let color_filter = ColorFilter::new(darken_filter_cb);

        let mut style_btn = Style::new();
        style_btn
            .radius(10)
            .bg_opa(LV_OPA_COVER)
            .bg_color(palette_lighten(Palette::Grey, 3))
            .bg_grad_color(palette_main(Palette::Grey))
            .bg_grad_dir(GradDir::Ver)
            .border_color_hex(0x000000)
            .border_opa(LV_OPA_20)
            .border_width(2)
            .text_color_hex(0x000000);

        let mut style_pressed = Style::new();
        style_pressed.color_filter(&color_filter, LV_OPA_20);

        let mut style_red = Style::new();
        style_red
            .bg_color(palette_main(Palette::Red))
            .bg_grad_color(palette_lighten(Palette::Red, 3));

        let screen = Screen::active().expect("active screen");

        let btn1 = Button::new(&screen).expect("btn1");
        btn1.remove_style_all().pos(10, 10).size(120, 50);
        btn1.add_style(&style_btn, 0);
        btn1.add_style(&style_pressed, 0x0020); // LV_STATE_PRESSED

        let _lbl1 = Label::new(&btn1).expect("lbl1");
        _lbl1.text("Button\0").expect("text").center();

        let btn2 = Button::new(&screen).expect("btn2");
        btn2.remove_style_all().pos(10, 80).size(120, 50);
        btn2.add_style(&style_btn, 0);
        btn2.add_style(&style_red, 0);
        btn2.add_style(&style_pressed, 0x0020); // LV_STATE_PRESSED
        unsafe { lv_obj_set_style_radius(btn2.lv_handle(), 0x7fff, 0) };

        let _lbl2 = Label::new(&btn2).expect("lbl2");
        _lbl2.text("Button 2\0").expect("text").center();

        pump(10);
        capture("ex3");
        // Drop order (reverse declaration): _lbl2, btn2, _lbl1, btn1, style_red,
        // style_pressed, style_btn, color_filter — widgets before styles, correct.
    }

    // ── Ex4: Slider ──────────────────────────────────────────────────────────────
    unsafe { lv_screen_load(lv_obj_create(core::ptr::null_mut())) };
    {
        use core::ffi::c_void;
        use core::fmt::Write as _;

        unsafe extern "C" fn slider_event_cb(e: *mut lv_event_t) {
            // SAFETY: e is a valid lv_event_t pointer; user_data is the label ptr.
            let (slider, label, val) = unsafe {
                let slider = lv_event_get_target_obj(e);
                let label = lv_event_get_user_data(e) as *mut lv_obj_t;
                let val = lv_slider_get_value(slider);
                (slider, label, val)
            };
            let mut s = heapless::String::<12>::new();
            let _ = write!(s, "{}\0", val);
            unsafe {
                lv_label_set_text(label, s.as_ptr() as *const core::ffi::c_char);
                lv_obj_align_to(label, slider, Align::OutTopMid as u32, 0, -15);
            }
        }

        let screen = Screen::active().expect("active screen");

        let slider = Slider::new(&screen).expect("slider");
        slider.width(200).center();

        let label = Label::new(&screen).expect("label");
        label.text("0\0").expect("text");
        label.align_to(&slider, Align::OutTopMid, 0, -15);

        slider.on_event(
            slider_event_cb,
            lv_event_code_t_LV_EVENT_VALUE_CHANGED,
            label.lv_handle() as *mut c_void,
        );

        pump(10);
        capture("ex4");
    }
}
