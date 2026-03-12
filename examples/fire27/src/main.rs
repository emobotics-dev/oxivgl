// SPDX-License-Identifier: BSD-3-Clause
//! oxivgl standalone demo — M5Stack Fire27 (ESP32), LVGL over ILI9342C.
//!
//! Architecture:
//!   - `flush_task` runs on a high-priority interrupt executor (sw_int1)
//!     so it can pre-empt the LVGL task and complete SPI transfers.
//!   - LVGL render loop runs on the low-priority main (thread-mode) executor.
//!   - SAFETY: DisplayDriver holds Spi<Async> which is !Send due to PhantomData<*const ()>.
//!     We use `unsafe impl Send` because the driver is owned exclusively by
//!     `flush_task` on a single-core ESP32; there is no concurrent access.
#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_sync::mutex::Mutex;
use embassy_time::Delay;
use esp_backtrace as _;
use esp_hal::{
    Async,
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    interrupt::Priority,
    interrupt::software::SoftwareInterruptControl,
    ram,
    spi::{Mode, master::{Config as SpiConfig, Spi}},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println as _;
use esp_rtos::embassy::InterruptExecutor;
use esp_sync::RawMutex;
use lcd_async::{
    Builder, Display,
    interface::SpiInterface,
    models::ILI9342CRgb565,
    options::{ColorInversion, ColorOrder},
};
use log::info;
use oxivgl::{
    lvgl_buffers::{DisplayOutput, LvglBuffers, UiError, flush_frame_buffer},
    view::{View, run_lvgl},
    widgets::{Align, Bar, Label, Screen, WidgetError},
};
use static_cell::{StaticCell, make_static};

esp_bootloader_esp_idf::esp_app_desc!();

// Required by esp-backtrace with custom-halt feature
#[unsafe(no_mangle)]
fn custom_halt() -> ! {
    loop {}
}

// ── Display constants ────────────────────────────────────────────────────────

const SCREEN_W: u16 = 320;
const SCREEN_H: u16 = 240;
const LVGL_BUF_BYTES: usize = SCREEN_W as usize * oxivgl::lvgl_buffers::COLOR_BUF_LINES * 2;

// ── Display driver type aliases ──────────────────────────────────────────────

type SpiBusType = Spi<'static, Async>;
type SpiDeviceType = SpiDeviceWithConfig<'static, RawMutex, SpiBusType, Output<'static>>;
type DisplayInterface = SpiInterface<SpiDeviceType, Output<'static>>;
type LcdDisplay = Display<DisplayInterface, ILI9342CRgb565, Output<'static>>;

static SPI_BUS: StaticCell<Mutex<RawMutex, SpiBusType>> = StaticCell::new();

// ── DisplayDriver wraps lcd_async Display ────────────────────────────────────

struct DisplayDriver {
    _bl: Output<'static>,
    display: LcdDisplay,
}

// SAFETY: DisplayDriver contains Spi<Async> which uses PhantomData<*const ()> to
// prevent accidental cross-thread sharing. On single-core ESP32, `flush_task` owns
// this exclusively; no other task or ISR accesses it concurrently. The `!Send`
// marker is a conservative default for Async drivers, not a soundness requirement
// for single-core bare-metal contexts.
unsafe impl Send for DisplayDriver {}

impl DisplayOutput for DisplayDriver {
    async fn show_raw_data(&mut self, x: u16, y: u16, w: u16, h: u16, data: &[u8]) -> Result<(), UiError> {
        self.display.show_raw_data(x, y, w, h, data).await.map_err(|_| UiError::Display)
    }
}

// ── Flush task (interrupt executor, high priority) ───────────────────────────

#[embassy_executor::task]
#[ram]
async fn flush_task(driver: DisplayDriver) -> ! {
    flush_frame_buffer(driver).await
}

// ── DemoView ─────────────────────────────────────────────────────────────────

struct DemoView {
    // Widget handles must be kept alive for the lifetime of the view.
    _title: Label<'static>,
    _sub: Label<'static>,
    _status: Label<'static>,
    _bar: Bar<'static>,
}

impl View for DemoView {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;
        screen.bg_color(0x0a0a1a).bg_opa(255).remove_scrollable();

        let title = Label::new(&screen)?;
        title.text("oxivgl demo")?.align(Align::TopMid, 0, 10).text_color(0x00d0ff);

        let sub = Label::new(&screen)?;
        sub.text("M5Stack Fire27")?.align(Align::Center, 0, -20);

        let status = Label::new(&screen)?;
        status.text("LVGL ready")?.align(Align::Center, 0, 20).text_color(0x00e060);

        // Progress bar: set range first, then value, then position via Deref<Obj>
        let mut bar = Bar::new(&screen)?;
        bar.set_range(100.0);
        bar.set_value(75.0);
        bar.align(Align::BottomMid, 0, -20).size(200, 20);

        Ok(Self { _title: title, _sub: sub, _status: status, _bar: bar })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

#[esp_rtos::main]
async fn main(_low_prio_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let p = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 50*1024);

    let tg0 = TimerGroup::new(p.TIMG0);
    let sw_int = SoftwareInterruptControl::new(p.SW_INTERRUPT);
    esp_rtos::start(tg0.timer0, sw_int.software_interrupt0);
    info!("Embassy initialized");

    // ── SPI2 bus ──────────────────────────────────────────────────────────────
    let spi_config = SpiConfig::default()
        .with_frequency(Rate::from_khz(40_000))
        .with_mode(Mode::_0);

    let spi_bus = Spi::new(p.SPI2, spi_config.clone())
        .expect("SPI2 init failed")
        .with_sck(p.GPIO18)
        .with_mosi(p.GPIO23)
        .with_miso(p.GPIO19)
        .into_async();

    let shared_bus = SPI_BUS.init(Mutex::new(spi_bus));
    let display_cs = Output::new(p.GPIO14, Level::High, OutputConfig::default());
    let spi_device = SpiDeviceWithConfig::new(shared_bus, display_cs, spi_config);

    // ── ILI9342C display init ─────────────────────────────────────────────────
    let mut bl  = Output::new(p.GPIO32, Level::Low,  OutputConfig::default());
    let dc  = Output::new(p.GPIO27, Level::Low,  OutputConfig::default());
    let rst = Output::new(p.GPIO33, Level::Low,  OutputConfig::default());

    let di = SpiInterface::new(spi_device, dc);
    let mut delay = Delay;
    let display = Builder::new(ILI9342CRgb565, di)
        .invert_colors(ColorInversion::Inverted)
        .color_order(ColorOrder::Bgr)
        .display_size(SCREEN_W, SCREEN_H)
        .reset_pin(rst)
        .init(&mut delay)
        .await
        .expect("Display init failed");

    bl.set_high();
    info!("Display initialized, backlight on");

    let driver = DisplayDriver { _bl: bl, display };

    // ── Interrupt executor for flush_task (high priority) ─────────────────────
    let int_exec = make_static!(InterruptExecutor::new(sw_int.software_interrupt1));
    let hi_spawner = int_exec.start(Priority::min());
    hi_spawner.must_spawn(flush_task(driver));

    // ── LVGL buffers & render loop ─────────────────────────────────────────────
    static mut LVGL_BUFS: LvglBuffers<LVGL_BUF_BYTES> = LvglBuffers::new();
    let bufs = unsafe { &mut *core::ptr::addr_of_mut!(LVGL_BUFS) };

    run_lvgl::<DemoView, LVGL_BUF_BYTES>(SCREEN_W.into(), SCREEN_H.into(), bufs).await;
}
