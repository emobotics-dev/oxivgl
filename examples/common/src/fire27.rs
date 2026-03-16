// SPDX-License-Identifier: MIT OR Apache-2.0
//! M5Stack Fire27 hardware boilerplate macro.
//!
//! [`fire27_main!`] generates the entire `#[esp_rtos::main]` entry point
//! including SPI bus, ILI9342C display init, flush task, and LVGL render
//! loop — the caller only supplies the [`View`] type.

/// Generate a complete Fire27 main function for the given [`oxivgl::view::View`] type.
#[macro_export]
macro_rules! fire27_main {
    ($View:ty) => {
        // Crate aliases for proc-macro attributes (#[embassy_executor::task], etc.)
        use $crate::esp_hal as esp_hal;


        use $crate::embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
        use embassy_executor::Spawner;
        use $crate::embassy_sync::mutex::Mutex;
        use $crate::embassy_time::Delay;
        use $crate::esp_backtrace as _;
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
        use $crate::esp_println as _;
        use esp_rtos::embassy::InterruptExecutor;
        use $crate::esp_sync::RawMutex;
        use $crate::lcd_async::{
            Builder, Display,
            interface::SpiInterface,
            models::ILI9342CRgb565,
            options::{ColorInversion, ColorOrder},
        };
        use $crate::log::info;
        use $crate::oxivgl::flush_pipeline::{DisplayOutput, UiError, flush_frame_buffer};
        use $crate::oxivgl::lvgl_buffers::LvglBuffers;
        use $crate::oxivgl::view::run_lvgl;
        use $crate::static_cell::{StaticCell, make_static};

        esp_bootloader_esp_idf::esp_app_desc!();

        #[unsafe(no_mangle)]
        fn custom_halt() -> ! {
            loop {}
        }

        const SCREEN_W: u16 = 320;
        const SCREEN_H: u16 = 240;
        const LVGL_BUF_BYTES: usize =
            SCREEN_W as usize * oxivgl::lvgl_buffers::COLOR_BUF_LINES * 2;

        type SpiBusType = Spi<'static, Async>;
        type SpiDeviceType =
            SpiDeviceWithConfig<'static, RawMutex, SpiBusType, Output<'static>>;
        type DisplayInterface = SpiInterface<SpiDeviceType, Output<'static>>;
        type LcdDisplay = Display<DisplayInterface, ILI9342CRgb565, Output<'static>>;

        static SPI_BUS: StaticCell<Mutex<RawMutex, SpiBusType>> = StaticCell::new();

        struct DisplayDriver {
            _bl: Output<'static>,
            display: LcdDisplay,
        }

        // SAFETY: DisplayDriver contains Spi<Async> which uses PhantomData<*const ()>
        // to prevent accidental cross-thread sharing. On single-core ESP32,
        // `flush_task` owns this exclusively; no concurrent access occurs.
        unsafe impl Send for DisplayDriver {}

        impl DisplayOutput for DisplayDriver {
            async fn show_raw_data(
                &mut self,
                x: u16,
                y: u16,
                w: u16,
                h: u16,
                data: &[u8],
            ) -> Result<(), UiError> {
                self.display
                    .show_raw_data(x, y, w, h, data)
                    .await
                    .map_err(|_| UiError::Display)
            }
        }

        #[embassy_executor::task]
        #[ram]
        async fn flush_task(driver: DisplayDriver) -> ! {
            flush_frame_buffer(driver).await
        }

        #[esp_rtos::main]
        async fn main(_low_prio_spawner: Spawner) {
            $crate::esp_println::logger::init_logger_from_env();

            let p =
                esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
            $crate::esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 50 * 1024);

            let tg0 = TimerGroup::new(p.TIMG0);
            let sw_int = SoftwareInterruptControl::new(p.SW_INTERRUPT);
            esp_rtos::start(tg0.timer0, sw_int.software_interrupt0);
            info!("Embassy initialized");

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
            let display_cs =
                Output::new(p.GPIO14, Level::High, OutputConfig::default());
            let spi_device =
                SpiDeviceWithConfig::new(shared_bus, display_cs, spi_config);

            let mut bl = Output::new(p.GPIO32, Level::Low, OutputConfig::default());
            let dc = Output::new(p.GPIO27, Level::Low, OutputConfig::default());
            let rst = Output::new(p.GPIO33, Level::Low, OutputConfig::default());

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

            let int_exec =
                make_static!(InterruptExecutor::new(sw_int.software_interrupt1));
            let hi_spawner = int_exec.start(Priority::min());
            hi_spawner.must_spawn(flush_task(driver));

            static mut LVGL_BUFS: LvglBuffers<LVGL_BUF_BYTES> = LvglBuffers::new();
            // SAFETY: LVGL_BUFS is only accessed here, before the single-threaded
            // LVGL render loop takes exclusive ownership.
            let bufs = unsafe { &mut *core::ptr::addr_of_mut!(LVGL_BUFS) };

            run_lvgl::<$View, LVGL_BUF_BYTES>(SCREEN_W.into(), SCREEN_H.into(), bufs)
                .await;
        }
    };
}
