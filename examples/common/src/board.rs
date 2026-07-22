// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unified M5Stack hardware harness for both the Fire27 (ESP32) and CoreS3
//! (ESP32-S3), built on the `m5stack-core` BSP.
//!
//! [`board_main!`] (and the `_nav` / `_psram` variants) generate the entire
//! `#[esp_rtos::main]` entry point: BSP board bring-up, the DMA display flush
//! task, the LVGL render loop, and the board's input device — the caller only
//! supplies the [`View`](oxivgl::view::View) type.
//!
//! The two boards diverge in exactly two places, both handled by the BSP:
//!
//! * **Display bring-up.** Fire27's panel has a GPIO reset pin; the CoreS3's is
//!   reset over the AW9523B expander and powered by the AXP2101 PMIC, so its
//!   path first runs `power_display_reset` over I2C. `into_display_only`
//!   (a DMA `SpiDmaBus`) hides the rest.
//! * **Input.** Fire27 has three front-panel buttons → an LVGL **keypad**
//!   indev; the CoreS3 has an FT6336U touch panel → an LVGL **pointer** indev
//!   fed by an async poll task. The board reports which via `io::input_caps()`.
//!
//! Board selection is the `fire27` / `cores3` cargo feature (both chips are
//! xtensa, so `target_arch` cannot distinguish them). Exactly one must be set.

#[cfg(not(any(feature = "fire27", feature = "cores3")))]
compile_error!(
    "an xtensa example build must select a board: pass `--features fire27` \
     or `--features cores3` (the run_*.sh scripts do this)"
);

/// Generate a complete board `main` for the given [`View`](oxivgl::view::View),
/// using [`run_app`](oxivgl::view::run_app) (single-screen).
#[macro_export]
macro_rules! board_main {
    ($view_expr:expr) => {
        $crate::board_body!($view_expr, single, psram_bytes = 0);
    };
}

/// Like [`board_main!`] but uses [`run_app_nav`](oxivgl::view::run_app_nav) for
/// multi-screen navigation.
#[macro_export]
macro_rules! board_main_nav {
    ($view_expr:expr) => {
        $crate::board_body!($view_expr, nav, psram_bytes = 0);
    };
}

/// Like [`board_main!`], but hands LVGL's heap a PSRAM region of `$bytes`
/// (via the BSP's `mem::psram_split`) before the render loop starts.
///
/// Panics rather than continuing if the pool is refused: a silently
/// unregistered pool would leave LVGL on the internal heap with no indication,
/// which is the failure this whole path exists to prevent.
#[macro_export]
macro_rules! board_main_psram {
    ($view_expr:expr, $bytes:expr) => {
        $crate::board_body!($view_expr, single, psram_bytes = $bytes);
    };
}

/// Like [`board_main_nav!`], but drives the UI as an **encoder** rather than a
/// keypad/pointer: the board's three inputs (Fire27 physical buttons, CoreS3
/// touch-strip zones — both via the BSP's unified `ButtonEvent`) feed an
/// [`EncoderState`](oxivgl::indev::EncoderState), and the loop runs via
/// [`run_app_nav_encoder`](oxivgl::view::run_app_nav_encoder) (event mode +
/// integrated wake, so input reaches LVGL with no read-timer latency).
///
/// `Short(n)` on the outer buttons is `turn(∓n)`, a center `Short` is `click`,
/// and a center `Long` is `long_press` (toggle edit mode).
#[macro_export]
macro_rules! board_main_nav_encoder {
    ($view_expr:expr) => {
        $crate::board_body!($view_expr, nav_encoder, psram_bytes = 0);
    };
}

/// Spawn a task, panicking with call-site context if the task pool is exhausted.
///
/// Replaces embassy-executor's `Spawner::must_spawn` (removed in 0.10.0). For
/// one-shot init code with the default `pool_size = 1`, exhaustion is a logic
/// bug, so panicking is correct. Works with both `Spawner` and `SendSpawner`.
#[macro_export]
#[doc(hidden)]
macro_rules! must_spawn {
    ($spawner:expr, $task:expr) => {
        $spawner.spawn(
            $task.unwrap_or_else(|e| panic!(
                concat!("spawn ", stringify!($task), ": {:?}"), e))
        )
    };
}

/// Internal: the shared board harness body. Do not call directly.
///
/// `$mode` is `single` (uses `run_app`) or `nav` (uses `run_app_nav`).
#[macro_export]
#[doc(hidden)]
macro_rules! board_body {
    ($view_expr:expr, $mode:ident, psram_bytes = $psram_bytes:expr) => {
        // Crate aliases for proc-macro attributes and direct paths.
        use $crate::esp_hal as esp_hal;
        use $crate::m5stack_core as m5stack_core;

        use embassy_executor::Spawner;
        use esp_hal::{
            dma::{DmaRxBuf, DmaTxBuf},
            dma_buffers,
            interrupt::Priority,
        };
        use esp_rtos::embassy::InterruptExecutor;
        use m5stack_core::board;
        #[cfg(feature = "fire27")]
        use m5stack_core::board::fire27::Board;
        #[cfg(feature = "cores3")]
        use m5stack_core::board::cores3::Board;
        use m5stack_core::mem::{self, HeapProfile};
        use $crate::oxivgl::display::LvglBuffers;
        use $crate::oxivgl::flush_pipeline::{DisplayOutput, UiError, flush_frame_buffer};
        use $crate::static_cell::make_static;

        // BSP provides the panic handler and the esp-idf app descriptor.
        m5stack_core::app_desc!();

        const SCREEN_W: u16 = board::SCREEN_W;
        const SCREEN_H: u16 = board::SCREEN_H;
        const LVGL_BUF_BYTES: usize =
            SCREEN_W as usize * $crate::oxivgl::display::COLOR_BUF_LINES * 2;

        // ── Display driver (wraps the BSP DMA display bus) ──────────────────

        type DisplayBus = m5stack_core::board::spi2::DisplayBus;

        struct DisplayDriver {
            bus: DisplayBus,
        }

        // SAFETY: the bus holds SPI-DMA state marked `!Send` to prevent
        // accidental cross-task sharing. On these single-core boards `flush_task`
        // owns it exclusively; no concurrent access occurs.
        unsafe impl Send for DisplayDriver {}

        impl DisplayOutput for DisplayDriver {
            async fn show_raw_data(
                &mut self, x: u16, y: u16, w: u16, h: u16, data: &[u8],
            ) -> Result<(), UiError> {
                self.bus
                    .display
                    .show_raw_data(x, y, w, h, data)
                    .await
                    .map_err(|_| UiError::Display)
            }
        }

        #[embassy_executor::task]
        #[esp_hal::ram]
        async fn flush_task(driver: DisplayDriver) -> ! {
            flush_frame_buffer(driver).await
        }

        /// One LVGL stripe of RX/TX DMA descriptors + buffers. RX is a throwaway
        /// 64 B (the panel is write-only); TX holds one flush region.
        fn dma_bufs() -> (DmaRxBuf, DmaTxBuf) {
            let (rx_buffer, rx_desc, tx_buffer, tx_desc) = dma_buffers!(64, LVGL_BUF_BYTES);
            let rx = DmaRxBuf::new(rx_desc, rx_buffer).expect("DMA rx buf");
            let tx = DmaTxBuf::new(tx_desc, tx_buffer).expect("DMA tx buf");
            (rx, tx)
        }

        // ── Input: keypad / pointer (nav, single) or encoder (nav_encoder) ──
        //
        // All input primitives for both modes are declared here; the active
        // `$mode` selects which is spawned (see `board_input_spawn!`) and which
        // indev is registered (see `board_maybe_indev!`). The others are inert,
        // hence `#[allow(dead_code)]`.

        /// Woken by the input task after it enqueues a key, so the render loop
        /// need not busy-poll. Unused on the pointer path (LVGL's read timer
        /// polls the `PointerState` directly).
        #[cfg(feature = "fire27")]
        #[allow(dead_code)]
        static __OXIVGL_HARNESS_KEYPAD: $crate::oxivgl::indev::KeypadState =
            $crate::oxivgl::indev::KeypadState::new();
        #[cfg(feature = "cores3")]
        #[allow(dead_code)]
        static __OXIVGL_HARNESS_POINTER: $crate::oxivgl::indev::PointerState =
            $crate::oxivgl::indev::PointerState::new();
        /// Encoder state for the `nav_encoder` mode, fed by the board's unified
        /// `ButtonEvent`. Every producer call fires its integrated wake, so
        /// `run_app_nav_encoder` reads with no read-timer latency.
        #[allow(dead_code)]
        static __OXIVGL_HARNESS_ENCODER: $crate::oxivgl::indev::EncoderState =
            $crate::oxivgl::indev::EncoderState::new();

        /// Fire27: map the three debounced front-panel buttons to LVGL keys.
        #[cfg(feature = "fire27")]
        #[allow(dead_code)]
        #[embassy_executor::task]
        async fn input_task(mut input: m5stack_core::io::buttons::Buttons<'static>) -> ! {
            use m5stack_core::io::buttons::ButtonId;
            use $crate::oxivgl::enums::Key;
            loop {
                let ev = input.next_event().await;
                let key = match ev.id {
                    ButtonId::Left => Key::PREV,
                    ButtonId::Center => Key::ENTER,
                    ButtonId::Right => Key::NEXT,
                };
                __OXIVGL_HARNESS_KEYPAD.send(key);
            }
        }

        /// CoreS3: poll the FT6336U (~50 Hz) and feed the pointer state.
        #[cfg(feature = "cores3")]
        #[allow(dead_code)]
        #[embassy_executor::task]
        async fn touch_poll_task(
            i2c: &'static m5stack_core::io::shared_i2c::SharedI2cBus,
        ) -> ! {
            use $crate::embassy_time::{Duration, Timer};
            loop {
                match m5stack_core::driver::ft6336u::read_touch(i2c).await {
                    Ok(Some((x, y))) => __OXIVGL_HARNESS_POINTER.touch(x, y),
                    Ok(None) => __OXIVGL_HARNESS_POINTER.release(),
                    Err(_) => __OXIVGL_HARNESS_POINTER.release(),
                }
                Timer::after(Duration::from_millis(20)).await;
            }
        }

        /// Map one BSP `ButtonEvent` onto the encoder: outer buttons turn (with
        /// the multi-tap count), center short-clicks, center long-press toggles
        /// edit mode. Shared by both boards (identical `ButtonEvent`).
        #[allow(dead_code)]
        fn __oxivgl_encoder_feed(ev: m5stack_core::io::buttons::ButtonEvent) {
            use m5stack_core::io::buttons::{ButtonAction, ButtonId};
            match (ev.id, ev.action) {
                (ButtonId::Left, ButtonAction::Short(n)) => {
                    __OXIVGL_HARNESS_ENCODER.turn(-(n as i32))
                }
                (ButtonId::Right, ButtonAction::Short(n)) => {
                    __OXIVGL_HARNESS_ENCODER.turn(n as i32)
                }
                (ButtonId::Center, ButtonAction::Short(_)) => __OXIVGL_HARNESS_ENCODER.click(),
                (ButtonId::Center, ButtonAction::Long) => __OXIVGL_HARNESS_ENCODER.long_press(),
                // Outer long-press has no encoder meaning; leave to the app.
                _ => {}
            }
        }

        /// Fire27 (`nav_encoder`): feed the three physical buttons to the encoder.
        #[cfg(feature = "fire27")]
        #[allow(dead_code)]
        #[embassy_executor::task]
        async fn encoder_input_task(mut input: m5stack_core::io::buttons::Buttons<'static>) -> ! {
            loop {
                __oxivgl_encoder_feed(input.next_event().await);
            }
        }

        /// CoreS3 (`nav_encoder`): the touch-strip zones emulate the same three
        /// buttons (BSP `TouchButtons`); feed their events to the encoder.
        #[cfg(feature = "cores3")]
        #[allow(dead_code)]
        #[embassy_executor::task]
        async fn encoder_touch_task(
            i2c: &'static m5stack_core::io::shared_i2c::SharedI2cBus,
        ) -> ! {
            use m5stack_core::io::touch_buttons::{TouchButtons, TouchButtonsConfig};
            // multi_tap_ms = 0: emit each tap immediately (no multi-tap window),
            // so the encoder feels responsive. The encoder accumulator still
            // sums rapid taps into multi-step, so no count is lost. See
            // m5stack-core#58 for the equivalent Fire27 (async-button) knob.
            let config = TouchButtonsConfig { multi_tap_ms: 0, ..TouchButtonsConfig::default() };
            let mut buttons = TouchButtons::new(i2c, config);
            loop {
                __oxivgl_encoder_feed(buttons.next_event().await);
            }
        }

        /// Thin wrapper that registers the board's LVGL indev on first `create`
        /// (after `lv_init`, before any widget), then delegates to the user view.
        /// Holds the indev so it lives for the program's duration.
        struct BoardView<V: $crate::oxivgl::view::View> {
            inner: V,
            // In `nav_encoder` mode the encoder indev is owned by
            // `run_app_nav_encoder`, so this stays `None` — hence the allow.
            #[cfg(feature = "fire27")]
            #[allow(dead_code)]
            _indev: Option<$crate::oxivgl::indev::KeypadIndev>,
            #[cfg(feature = "cores3")]
            #[allow(dead_code)]
            _indev: Option<$crate::oxivgl::indev::PointerIndev>,
        }

        impl<V: $crate::oxivgl::view::View> $crate::oxivgl::view::View for BoardView<V> {
            fn create(
                &mut self,
                container: &$crate::oxivgl::widgets::Obj<'static>,
            ) -> Result<(), $crate::oxivgl::widgets::WidgetError> {
                // Register the keypad/pointer indev (nav, single). In
                // nav_encoder mode this is a no-op; the encoder indev is created
                // and bound by run_app_nav_encoder.
                $crate::board_maybe_indev!($mode, self)?;
                self.inner.create(container)
            }

            fn update(&mut self) -> Result<$crate::oxivgl::view::NavAction, $crate::oxivgl::widgets::WidgetError> {
                self.inner.update()
            }

            fn on_event(&mut self, event: &$crate::oxivgl::event::Event) -> $crate::oxivgl::view::NavAction {
                self.inner.on_event(event)
            }

            fn register_events_on(&mut self, container: &$crate::oxivgl::widgets::Obj<'static>) {
                self.inner.register_events_on(container);
            }

            fn input_group(&self) -> Option<$crate::oxivgl::group::GroupRef> {
                self.inner.input_group()
            }

            fn will_hide(&mut self) {
                self.inner.will_hide();
            }

            fn did_show(&mut self) {
                self.inner.did_show();
            }
        }

        // ── Entry point ─────────────────────────────────────────────────────

        #[esp_rtos::main]
        async fn main(spawner: Spawner) {
            let p = board::init();
            let b = Board::split(p);

            // BSP console: Fire27 over UART0, CoreS3 over USB-Serial-JTAG CDC.
            #[cfg(feature = "fire27")]
            let _console = m5stack_core::io::console::install(
                spawner,
                m5stack_core::io::console::Config {
                    serial: Some(m5stack_core::io::console::SerialResources {
                        uart: b.uart0, tx_pin: b.uart0_tx, rx_pin: b.uart0_rx,
                    }),
                    level: $crate::log::LevelFilter::Info,
                },
            );
            #[cfg(feature = "cores3")]
            let _console = m5stack_core::io::console::install(
                spawner,
                m5stack_core::io::console::Config {
                    serial: Some(m5stack_core::io::console::SerialResources { usb: b.usb_device }),
                    level: $crate::log::LevelFilter::Info,
                },
            );

            // DRAM heap (reclaimed ROM only). PSRAM, if requested, goes to LVGL
            // via psram_split below rather than into the global allocator —
            // keeping it out is what lets oxivgl route draw buffers to internal,
            // DMA-capable RAM (the ESP32 cannot DMA from PSRAM at all).
            mem::init_heap(HeapProfile::Lvgl, None);

            if $psram_bytes > 0 {
                match mem::psram_split(b.psram, Some($psram_bytes)) {
                    Ok(split) => match $crate::oxivgl::mem::reserve_pool(split.private) {
                        Ok(()) => $crate::log::info!(
                            "LVGL pool: {} KiB PSRAM (global heap +{} KiB)",
                            $psram_bytes / 1024, split.global_free / 1024,
                        ),
                        Err(e) => panic!("PSRAM pool rejected by LVGL: {:?}", e),
                    },
                    Err(e) => panic!("psram_split failed: {:?}", e),
                }
            }

            esp_rtos::start(b.system.timer0_0, b.system.sw_int.software_interrupt0);
            $crate::log::info!("Embassy initialized");

            let (dma_rx, dma_tx) = dma_bufs();

            // Display + input bring-up — the one real per-board divergence.
            // Fire27: panel resets via GPIO; input = three buttons. CoreS3: the
            // one I2C bus resets/powers the panel (AW9523B + AXP2101) *and*
            // drives the FT6336U touch, so it is created once and shared.
            #[cfg(feature = "fire27")]
            let (dbus, input) = {
                let dbus = b.spi2.into_display_only(dma_rx, dma_tx).await.expect("display init");
                (dbus, b.buttons.into_buttons())
            };
            #[cfg(feature = "cores3")]
            let (dbus, i2c) = {
                use $crate::static_cell::StaticCell;
                use m5stack_core::io::shared_i2c::SharedI2cBus;
                static I2C_BUS: StaticCell<SharedI2cBus> = StaticCell::new();
                let i2c: &'static SharedI2cBus =
                    I2C_BUS.init(SharedI2cBus::new(b.i2c0.into_async()));
                m5stack_core::board::cores3::power_display_reset(i2c).await;
                let dbus = b.spi2.into_display_only(dma_rx, dma_tx).await.expect("display init");
                (dbus, i2c)
            };
            let driver = DisplayDriver { bus: dbus };
            $crate::log::info!("Display initialized");

            // Flush runs on a high-priority interrupt executor so it preempts
            // the LVGL render loop the moment a frame is ready.
            let int_exec =
                make_static!(InterruptExecutor::new(b.system.sw_int.software_interrupt1));
            let hi_spawner = int_exec.start(Priority::min());
            $crate::must_spawn!(hi_spawner, flush_task(driver));

            // Unify the per-board input source into one local so the spawn can
            // be dispatched on `$mode` (the token must be passed, not named
            // across the macro boundary, to keep local-variable hygiene).
            #[cfg(feature = "fire27")]
            let __input_src = input;
            #[cfg(feature = "cores3")]
            let __input_src = i2c;
            $crate::board_input_spawn!($mode, spawner, __input_src);

            static mut LVGL_BUFS: LvglBuffers<LVGL_BUF_BYTES> = LvglBuffers::new();
            // SAFETY: accessed only here, before the single-threaded LVGL render
            // loop takes exclusive ownership.
            let bufs = unsafe { &mut *core::ptr::addr_of_mut!(LVGL_BUFS) };

            let wrapper = BoardView { inner: $view_expr, _indev: None };
            $crate::board_launch!(wrapper, bufs, $mode);
        }
    };
}

/// Internal: launch the render loop for the selected mode. Do not call directly.
#[macro_export]
#[doc(hidden)]
macro_rules! board_launch {
    ($wrapper:ident, $bufs:ident, single) => {
        $crate::oxivgl::view::run_app::<BoardView<_>, LVGL_BUF_BYTES>(
            SCREEN_W.into(), SCREEN_H.into(), $bufs, $wrapper,
        ).await
    };
    ($wrapper:ident, $bufs:ident, nav) => {
        $crate::oxivgl::view::run_app_nav::<LVGL_BUF_BYTES>(
            SCREEN_W.into(), SCREEN_H.into(), $bufs, $wrapper,
        ).await
    };
    ($wrapper:ident, $bufs:ident, nav_encoder) => {
        $crate::oxivgl::view::run_app_nav_encoder::<LVGL_BUF_BYTES>(
            SCREEN_W.into(), SCREEN_H.into(), $bufs, $wrapper, &__OXIVGL_HARNESS_ENCODER,
        ).await
    };
}

/// Internal: spawn the input task for the selected mode. Do not call directly.
///
/// `$src` is the per-board input source (Fire27 `Buttons`, CoreS3 shared I2C),
/// passed as a token so local-variable hygiene is preserved across the macro
/// boundary; the referenced task fns are declared in `board_body!`.
#[macro_export]
#[doc(hidden)]
macro_rules! board_input_spawn {
    (nav_encoder, $spawner:expr, $src:expr) => {
        #[cfg(feature = "fire27")]
        $crate::must_spawn!($spawner, encoder_input_task($src));
        #[cfg(feature = "cores3")]
        $crate::must_spawn!($spawner, encoder_touch_task($src));
    };
    ($other:ident, $spawner:expr, $src:expr) => {
        #[cfg(feature = "fire27")]
        $crate::must_spawn!($spawner, input_task($src));
        #[cfg(feature = "cores3")]
        $crate::must_spawn!($spawner, touch_poll_task($src));
    };
}

/// Internal: register the keypad/pointer indev for the selected mode. Do not
/// call directly. A no-op in `nav_encoder` mode (the encoder indev is owned by
/// `run_app_nav_encoder`). Evaluates to `Result<(), WidgetError>`.
#[macro_export]
#[doc(hidden)]
macro_rules! board_maybe_indev {
    (nav_encoder, $self:expr) => {
        core::result::Result::<(), $crate::oxivgl::widgets::WidgetError>::Ok(())
    };
    ($other:ident, $self:expr) => {{
        if $self._indev.is_none() {
            #[cfg(feature = "fire27")]
            {
                $self._indev =
                    Some($crate::oxivgl::indev::KeypadIndev::new(&__OXIVGL_HARNESS_KEYPAD)?);
            }
            #[cfg(feature = "cores3")]
            {
                $self._indev =
                    Some($crate::oxivgl::indev::PointerIndev::new(&__OXIVGL_HARNESS_POINTER)?);
            }
        }
        core::result::Result::<(), $crate::oxivgl::widgets::WidgetError>::Ok(())
    }};
}
