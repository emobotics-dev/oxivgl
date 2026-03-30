// SPDX-License-Identifier: MIT OR Apache-2.0
//! M5Stack Fire27 hardware boilerplate macro.
//!
//! [`fire27_main!`] generates the entire `#[esp_rtos::main]` entry point
//! including SPI bus, ILI9342C display init, flush task, LVGL render loop,
//! and a keypad input device driven by the three front-panel buttons
//! (A=GPIO39, B=GPIO38, C=GPIO37, active-low with external pull-ups) —
//! the caller only supplies the [`View`] type.

/// Generate a complete Fire27 main function for the given [`oxivgl::view::View`] type.
///
/// The macro registers a LVGL `KEYPAD` input device backed by the three
/// hardware buttons on the M5Stack Fire v2.7:
///
/// | Button | GPIO | LVGL key      |
/// |--------|------|---------------|
/// | A      | 39   | `Key::PREV`   |
/// | B      | 38   | `Key::ENTER`  |
/// | C      | 37   | `Key::NEXT`   |
///
/// Call [`Group::assign_to_keyboard_indevs`] from your `View::create` to route
/// key events to a focus group.
#[macro_export]
macro_rules! fire27_main {
    ($view_expr:expr) => {
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
            gpio::{Input, InputConfig, Level, Output, OutputConfig},
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
        use $crate::oxivgl::display::LvglBuffers;
        use $crate::oxivgl::view::run_app;
        use $crate::static_cell::{StaticCell, make_static};

        esp_bootloader_esp_idf::esp_app_desc!();

        #[unsafe(no_mangle)]
        fn custom_halt() -> ! {
            loop {}
        }

        const SCREEN_W: u16 = 320;
        const SCREEN_H: u16 = 240;
        const LVGL_BUF_BYTES: usize =
            SCREEN_W as usize * oxivgl::display::COLOR_BUF_LINES * 2;

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

        // -------------------------------------------------------------------
        // Hardware button → LVGL keypad input device
        // -------------------------------------------------------------------

        /// Pending key code written by button tasks, read by the LVGL callback.
        /// 0 means no pending key. Single-core ESP32: Relaxed ordering is sufficient.
        static KEY_PENDING: core::sync::atomic::AtomicU32 =
            core::sync::atomic::AtomicU32::new(0);

        /// One embassy task per button. Loops forever: awaits a ShortPress event
        /// and stores the corresponding LVGL key code in KEY_PENDING.
        #[embassy_executor::task(pool_size = 3)]
        async fn button_task(
            pin: Input<'static>,
            key_code: u32,
        ) -> ! {
            use $crate::async_button::{Button, ButtonConfig, Mode as BtnMode};
            let config = ButtonConfig {
                mode: BtnMode::PullUp,
                ..ButtonConfig::default()
            };
            let mut btn = Button::new(pin, config);
            loop {
                use $crate::async_button::ButtonEvent;
                match btn.update().await {
                    ButtonEvent::ShortPress { .. } => {
                        // Only write if no key is pending (last press wins on overlap).
                        KEY_PENDING.store(
                            key_code,
                            core::sync::atomic::Ordering::Relaxed,
                        );
                    }
                    ButtonEvent::LongPress => {}
                }
            }
        }

        /// LVGL keypad read callback. Called by LVGL on every timer tick.
        ///
        /// # SAFETY
        /// Called from the LVGL task only (single-core ESP32). `data` is
        /// a non-null pointer provided by LVGL.
        unsafe extern "C" fn keypad_read_cb(
            _indev: *mut oxivgl_sys::lv_indev_t,
            data: *mut oxivgl_sys::lv_indev_data_t,
        ) {
            let key = KEY_PENDING.swap(0, core::sync::atomic::Ordering::Relaxed);
            // SAFETY: `data` is non-null and exclusively owned by LVGL for the
            // duration of this callback.
            unsafe {
                if key != 0 {
                    (*data).key = key;
                    (*data).state =
                        oxivgl_sys::lv_indev_state_t_LV_INDEV_STATE_PRESSED;
                } else {
                    (*data).state =
                        oxivgl_sys::lv_indev_state_t_LV_INDEV_STATE_RELEASED;
                }
            }
        }

        /// Thin wrapper that registers the LVGL keypad indev before delegating
        /// to the user-supplied [`View`] instance. This ensures the indev is
        /// created after `lv_init()` (inside `run_app`) but before the user
        /// view's `create()`.
        struct Fire27View<V: $crate::oxivgl::view::View> {
            inner: V,
            indev_registered: bool,
        }

        impl<V: $crate::oxivgl::view::View> $crate::oxivgl::view::View for Fire27View<V> {
            fn create(
                &mut self,
                container: &$crate::oxivgl::widgets::obj::Obj<'static>,
            ) -> Result<(), $crate::oxivgl::widgets::WidgetError> {
                if !self.indev_registered {
                    // SAFETY: lv_indev_create / lv_indev_set_type / lv_indev_set_read_cb
                    // are called after lv_init() (guaranteed by run_app) and before any
                    // widget is created. The indev pointer is non-null (checked).
                    // keypad_read_cb has the correct signature for a KEYPAD indev.
                    unsafe {
                        let indev = oxivgl_sys::lv_indev_create();
                        assert!(!indev.is_null(), "lv_indev_create returned NULL");
                        oxivgl_sys::lv_indev_set_type(
                            indev,
                            oxivgl_sys::lv_indev_type_t_LV_INDEV_TYPE_KEYPAD,
                        );
                        oxivgl_sys::lv_indev_set_read_cb(indev, Some(keypad_read_cb));
                    }
                    self.indev_registered = true;
                }
                self.inner.create(container)
            }

            fn update(&mut self) -> Result<(), $crate::oxivgl::widgets::WidgetError> {
                self.inner.update()
            }

            fn on_event(&mut self, event: &$crate::oxivgl::event::Event) {
                self.inner.on_event(event);
            }

            fn register_events(&mut self) {
                self.inner.register_events();
            }
        }

        // -------------------------------------------------------------------

        #[esp_rtos::main]
        async fn main(_low_prio_spawner: Spawner) {
            $crate::esp_println::logger::init_logger_from_env();

            let p =
                esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
            $crate::esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 50 * 1024);

            let tg0 = TimerGroup::new(p.TIMG0);
            let sw_int = SoftwareInterruptControl::new(p.SW_INTERRUPT);
            esp_rtos::start(tg0.timer0);
            info!("Embassy initialized");

            // Configure hardware buttons (active-low, external pull-ups on GPIO34-39).
            // GPIO34-39 on ESP32 are input-only; no internal pull-up available.
            let btn_a = Input::new(p.GPIO39, InputConfig::default()); // A — PREV
            let btn_b = Input::new(p.GPIO38, InputConfig::default()); // B — ENTER
            let btn_c = Input::new(p.GPIO37, InputConfig::default()); // C — NEXT

            // Spawn one task per button before the LVGL loop starts.
            _low_prio_spawner
                .must_spawn(button_task(btn_a, $crate::oxivgl::enums::Key::PREV.0));
            _low_prio_spawner
                .must_spawn(button_task(btn_b, $crate::oxivgl::enums::Key::ENTER.0));
            _low_prio_spawner
                .must_spawn(button_task(btn_c, $crate::oxivgl::enums::Key::NEXT.0));

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

            let wrapper = Fire27View { inner: $view_expr, indev_registered: false };
            run_app::<Fire27View<_>, LVGL_BUF_BYTES>(
                SCREEN_W.into(), SCREEN_H.into(), bufs, wrapper,
            ).await;
        }
    };
}
