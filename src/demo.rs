// SPDX-License-Identifier: MIT OR Apache-2.0
//! LVGL's built-in benchmark demo, wrapped so it cannot damage a live UI.
//!
//! [`lv_demo_benchmark`](https://docs.lvgl.io/master/details/integration/demos/index.html)
//! is the broadest end-to-end diagnostic LVGL ships: it walks ~30 scenes that
//! exercise rectangles, borders, shadows, images, text, arcs, sub-pixel
//! rendering, masking, blending and scrolling, and reports averaged FPS, CPU,
//! render time and flush time per scene. It measures the whole pipeline —
//! draw unit, buffer strategy, DMA flush and the display driver — which is why
//! it is the first thing to run when render performance is in question. See
//! `docs/render-pipeline.md`.
//!
//! # Availability
//!
//! This module exists only when the application's `lv_conf.h` sets
//! `LV_USE_DEMO_BENCHMARK 1` (which also requires `LV_USE_DEMO_WIDGETS 1`, the
//! demo's final scene) and the application compiles the demo's C sources.
//! `oxivgl-sys` compiles LVGL's `demos/` tree whenever `lv_conf.h` sets
//! `LV_BUILD_DEMOS 1`, so enabling the demo is a configuration change and
//! nothing else — see `examples/conf-benchmark/`. It costs roughly 900 KB of
//! flash, most of it image assets: **it must never be in a production image.**
//!
//! # Memory
//!
//! The demo needs ~48 KiB of LVGL heap at peak, measured;
//! [`benchmark()`](crate::demo::benchmark) asks for 64 KiB. LVGL's own guards
//! name 128 KB (`lv_demo_benchmark.c`, a `#warning`) and 38 KB
//! (`lv_demo_widgets.c`, an `#error`). That budget may come from a runtime pool
//! — [`crate::mem::reserve_pool`] — and does not have to sit in `LV_MEM_SIZE`,
//! which on ESP32 comes out of internal DRAM and cannot be raised far beside
//! the link-asserted main-stack floor.
//!
//! Those guards test `LV_MEM_SIZE` alone, which stopped meaning "the whole
//! heap" once `lv_mem_add_pool` existed, so they refused a small primary with a
//! runtime overflow while it had the memory. `oxivgl-sys` widens both to
//! `LV_MEM_SIZE + LV_MEM_POOL_EXPAND_SIZE`, leaving single-pool builds judged
//! as before. That ceiling is still only what *may* be registered, so
//! [`benchmark()`](crate::demo::benchmark) asks the heap that exists and
//! returns
//! [`InsufficientHeap`](crate::demo::BenchmarkError::InsufficientHeap)
//! when it is short.
//!
//! # Why a wrapper is needed
//!
//! `lv_demo_benchmark()` does not create a screen of its own. Every scene is
//! built on the *active* screen, and each scene change starts with
//! `lv_obj_clean(lv_screen_active())` (`lv_demo_benchmark.c:638`). Calling the
//! C function directly from an oxivgl application therefore deletes every
//! widget the running [`View`](crate::view::View) holds a pointer to, turning
//! the next `View::update` into a use-after-free and the eventual `View` drop
//! into a double free. Saving and restoring `lv_screen_active()` does not help:
//! the pointers are already dangling by then.
//!
//! [`benchmark`](crate::demo::benchmark) instead runs the demo on a throwaway
//! screen, restores the application's screen when the run ends, and deletes the
//! throwaway together with the header label the demo leaves on the top layer.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::ffi::CStr;

use oxivgl_sys::*;

use crate::widgets::{Obj, Screen};

// ---------------------------------------------------------------------------
// Results
// ---------------------------------------------------------------------------

/// Averaged measurements for one benchmark scene.
///
/// The demo accumulates one sample per `LV_SYSMON` refresh period and divides
/// by [`measurements`](Self::measurements) at the end; these fields hold the
/// quotient, matching the numbers LVGL's own summary table shows. A scene with
/// no samples (`measurements == 0`) reports `N/A` there and zeroes here — test
/// [`has_data`](Self::has_data) before believing a value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SceneResult {
    /// Scene name as given in the demo's scene table (e.g. `"Rectangle"`).
    pub name: String,
    /// Number of samples averaged. Zero means the scene never ran.
    pub measurements: u32,
    /// Average CPU load while the scene ran, in percent.
    pub cpu_pct: u32,
    /// Average frames per second, capped by `LV_DEF_REFR_PERIOD`.
    pub fps: u32,
    /// Average time spent rendering one frame, in milliseconds.
    pub render_ms: u32,
    /// Average time spent flushing one frame (driver call plus the wait for
    /// flush-ready), in milliseconds.
    pub flush_ms: u32,
}

impl SceneResult {
    /// Whether the scene produced any samples. When `false` every averaged
    /// field is zero because there was nothing to average, not because the
    /// scene was infinitely fast.
    pub fn has_data(&self) -> bool {
        self.measurements > 0
    }

    /// Average total frame time — [`render_ms`](Self::render_ms) plus
    /// [`flush_ms`](Self::flush_ms) — in milliseconds.
    pub fn total_ms(&self) -> u32 {
        self.render_ms + self.flush_ms
    }
}

/// Result of a complete benchmark run.
///
/// The top-level figures are averages over the scenes that produced samples,
/// exactly as LVGL computes them; scenes that produced none are excluded from
/// both [`valid_scenes`](Self::valid_scenes) and the averages, but still appear
/// in [`scenes`](Self::scenes) with `measurements == 0`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Summary {
    /// Every scene in the demo's table, in run order.
    pub scenes: Vec<SceneResult>,
    /// How many scenes contributed to the averages below.
    pub valid_scenes: u32,
    /// Average CPU load across the valid scenes, in percent.
    pub cpu_pct: u32,
    /// Average frames per second across the valid scenes.
    pub fps: u32,
    /// Average render time across the valid scenes, in milliseconds.
    pub render_ms: u32,
    /// Average flush time across the valid scenes, in milliseconds.
    pub flush_ms: u32,
    /// High-water mark of LVGL's own heap over the run, in bytes.
    ///
    /// `None` unless `lv_conf.h` selects `LV_STDLIB_BUILTIN`: only the built-in
    /// TLSF allocator has a pool to introspect, so under `LV_STDLIB_CLIB` there
    /// is no figure to report rather than a zero to misread.
    ///
    /// This is LVGL's heap, not the system heap — sized by
    /// `LV_MEM_SIZE` or the pools added with
    /// [`mem::reserve_pool`](crate::mem::reserve_pool).
    pub peak_heap_bytes: Option<usize>,
}

impl Summary {
    /// Whether any scene produced samples. A run reporting `false` measured
    /// nothing; the averaged fields are zero for lack of input.
    pub fn has_data(&self) -> bool {
        self.valid_scenes > 0
    }

    /// Average total frame time — [`render_ms`](Self::render_ms) plus
    /// [`flush_ms`](Self::flush_ms) — in milliseconds.
    pub fn total_ms(&self) -> u32 {
        self.render_ms + self.flush_ms
    }
}

/// Why a benchmark run could not be started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkError {
    /// The application's `lv_conf.h` leaves `LV_USE_PERF_MONITOR` off.
    ///
    /// The demo takes its samples from the sysmon performance subject, so
    /// without the monitor every scene records zero measurements and the run
    /// reports `0 FPS over 0 scenes` — a silent zero that reads like a result.
    /// Set `LV_USE_SYSMON 1` and `LV_USE_PERF_MONITOR 1`.
    PerfMonitorDisabled,
    /// A run started earlier has not finished yet.
    AlreadyRunning,
    /// No display has been initialised, so there is no screen to protect and
    /// nothing to render on. Call [`LvglDriver::init`](crate::driver::LvglDriver::init)
    /// first.
    NoActiveScreen,
    /// The LVGL heap has less free space than the benchmark needs.
    ///
    /// Register more memory before running — on ESP32 that is normally a PSRAM
    /// region via [`crate::mem::reserve_pool`], not a larger `LV_MEM_SIZE`,
    /// which comes out of internal DRAM.
    InsufficientHeap {
        /// Free heap at the time of the call, across every registered pool.
        free: usize,
        /// What LVGL's own guards document the benchmark as needing.
        required: usize,
    },
}

impl core::fmt::Display for BenchmarkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PerfMonitorDisabled => {
                write!(f, "LV_USE_PERF_MONITOR is disabled in lv_conf.h")
            }
            Self::AlreadyRunning => write!(f, "a benchmark run is already in progress"),
            Self::InsufficientHeap { free, required } => write!(
                f,
                "LVGL heap has {free} B free, the benchmark needs {required} B"
            ),
            Self::NoActiveScreen => write!(f, "no active screen — initialise the display first"),
        }
    }
}

// ---------------------------------------------------------------------------
// Run state (single-threaded stash for the C end-callback trampoline)
// ---------------------------------------------------------------------------

/// What [`benchmark`] must give back once the demo reaches its last scene: the
/// screen that was active when the run started, the throwaway screen the demo
/// ran on, how many objects the top layer held beforehand, and the caller's
/// closure.
struct RunState {
    /// The application's screen, reloaded when the run ends. LVGL owns it.
    previous: *mut lv_obj_t,
    /// The screen the demo built its scenes on; deleted when the run ends.
    bench: Obj<'static>,
    /// Top-layer child count before the run, so the demo's header label (and
    /// with it the sysmon observer bound to that label) can be removed without
    /// touching anything the application put there.
    top_children: u32,
    /// Delivered the converted [`Summary`] once the run ends.
    on_end: Box<dyn FnOnce(&Summary)>,
}

/// SAFETY: LVGL is single-threaded. [`benchmark`] writes this cell from the
/// LVGL task and the end-callback trampoline — invoked by `lv_timer_handler`
/// on that same task — takes it back. `RunState` holds an `lv_obj_t` pointer
/// and a `!Send` closure; the cell never crosses threads.
struct StateCell(UnsafeCell<Option<RunState>>);
unsafe impl Sync for StateCell {}

static RUN: StateCell = StateCell(UnsafeCell::new(None));

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run LVGL's benchmark demo on a throwaway screen and hand `on_end` the
/// results when it finishes.
///
/// Returns as soon as the first scene is loaded — the demo advances on an LVGL
/// timer, so the caller must keep pumping the render loop
/// ([`run_app`](crate::view::run_app), [`Ui::run`](crate::view::Ui::run) or
/// [`LvglDriver::timer_handler`](crate::driver::LvglDriver::timer_handler)).
/// A full pass takes roughly two minutes. `on_end` runs on the LVGL task from
/// inside `lv_timer_handler`, after the application's screen has been restored.
///
/// The application's screen and every widget on it survive untouched: the demo
/// only ever cleans the throwaway screen. Views are not updated *visually*
/// during the run because their screen is not the active one, but their
/// [`update`](crate::view::View::update) still runs and their widgets stay
/// valid.
///
/// # Errors
///
/// See [`BenchmarkError`]. A run may be repeated: `oxivgl-sys` zeroes the
/// demo's per-scene accumulators at the top of each run, so runs do not blend
/// into one another. Only an OVERLAPPING run is refused
/// ([`BenchmarkError::AlreadyRunning`]).
///
/// # Liveness: a missing callback is a signal, not a failure mode
///
/// `on_end` fires from the demo's own scene timer once the last scene ends, so
/// it arrives **only if `lv_timer_handler` keeps running to completion**. There
/// is no timeout and no error path: if the display pipeline wedges mid-run, no
/// callback arrives at all.
///
/// That makes the run usable as a display-path soak test. A completed run logs
/// `oxivgl-benchmark: complete, <n> of <m> scenes` before invoking `on_end`, so:
///
/// * marker absent — the pipeline stopped; the run never reached its last scene
/// * marker present, `valid_scenes < scenes.len()` — it ran through, but some
///   scenes recorded no samples
///
/// A wedge is therefore distinguishable from a degraded run, and both from a
/// clean one, on a serial log alone.
///
/// # Example
///
/// ```ignore
/// oxivgl::demo::benchmark(|summary| {
///     log::info!("{} FPS over {} scenes", summary.fps, summary.valid_scenes);
///     for scene in &summary.scenes {
///         if scene.has_data() {
///             log::info!("{}: {} FPS", scene.name, scene.fps);
///         }
///     }
/// })?;
/// ```
pub fn benchmark<F>(on_end: F) -> Result<(), BenchmarkError>
where
    F: FnOnce(&Summary) + 'static,
{
    if LV_USE_PERF_MONITOR != 1 {
        return Err(BenchmarkError::PerfMonitorDisabled);
    }
    // SAFETY: single-threaded access — see `StateCell` doc.
    let slot = unsafe { &mut *RUN.0.get() };
    if slot.is_some() {
        return Err(BenchmarkError::AlreadyRunning);
    }

    // SAFETY: reads the default display's active screen. Returns NULL when no
    // display exists, which is checked below rather than assumed.
    let previous = unsafe { lv_screen_active() };
    if previous.is_null() {
        return Err(BenchmarkError::NoActiveScreen);
    }

    // Only meaningful once `lv_init` has run, which the check above established.
    if let Some(free) = free_heap_bytes()
        && free < BENCHMARK_HEAP_BYTES
    {
        return Err(BenchmarkError::InsufficientHeap {
            free,
            required: BENCHMARK_HEAP_BYTES,
        });
    }

    let top_children = layer_top().map_or(0, |top| top.get_child_count());

    // The demo builds its scenes on whatever screen is active, so give it one
    // of its own before it can touch the application's.
    let bench = Screen::create();
    Screen::load_instant(&bench);

    *slot = Some(RunState {
        previous,
        bench,
        top_children,
        on_end: Box::new(on_end),
    });

    // SAFETY: `end_trampoline` is a `'static` `extern "C"` function, so the
    // pointer LVGL stores stays valid for the process lifetime.
    unsafe { lv_demo_benchmark_set_end_cb(Some(end_trampoline)) };
    // SAFETY: the demo only touches the active screen (the throwaway just
    // loaded), the top layer, and LVGL timers. Valid after display init,
    // established by the non-null `lv_screen_active` above.
    unsafe { lv_demo_benchmark() };

    Ok(())
}

/// Called by the demo once the last scene has run. Restores the application's
/// screen, disposes of everything the demo left behind, then delivers the
/// converted summary.
unsafe extern "C" fn end_trampoline(summary: *const lv_demo_benchmark_summary_t) {
    // SAFETY: single-threaded access — see `StateCell` doc. Taking the state
    // also makes a re-entrant call (LVGL invoking the callback twice) a no-op.
    let Some(state) = (unsafe { (*RUN.0.get()).take() }) else {
        return;
    };
    let RunState {
        previous,
        bench,
        top_children,
        on_end,
    } = state;

    // Restore before dropping: the screen being deleted must not be the active
    // one. LVGL's own `auto_del` order (`lv_display.c:821`, `:829`).
    Screen::load_instant(&Obj::from_raw_non_owning(previous));
    drop(bench);

    // The demo's FPS header lives on the top layer, not on the screen, so the
    // screen swap does not remove it — and the sysmon observer is bound to it.
    if let Some(top) = layer_top() {
        while top.get_child_count() > top_children {
            top.delete_child(top.get_child_count() as i32 - 1);
        }
    }

    let converted = if summary.is_null() {
        Summary::default()
    } else {
        // SAFETY: LVGL passes a pointer to `next_scene_timer_cb`'s stack
        // `lv_demo_benchmark_summary_t`, valid for the duration of this call.
        // Its `scenes` member points at the demo's `'static` scene table.
        unsafe { summary_from_raw(&*summary, peak_heap_bytes()) }
    };

    // Fixed marker, ahead of the callback: a serial-only rig can assert on this
    // line rather than needing a host-side timer. Its absence is the signal —
    // see `benchmark`'s liveness note.
    info!(
        "oxivgl-benchmark: complete, {} of {} scenes",
        converted.valid_scenes,
        converted.scenes.len()
    );

    on_end(&converted);
}

/// Non-owning handle to the default display's top layer, or `None` before a
/// display exists.
fn layer_top() -> Option<crate::widgets::Child<Obj<'static>>> {
    // SAFETY: `lv_layer_top` reads the default display and returns NULL when
    // there is none; the null case is handled rather than assumed.
    let ptr = unsafe { lv_layer_top() };
    if ptr.is_null() {
        None
    } else {
        Some(Obj::from_raw_non_owning(ptr))
    }
}

// ---------------------------------------------------------------------------
// C summary → Rust Summary
// ---------------------------------------------------------------------------

/// Divide an accumulated total by its sample count, yielding zero rather than
/// trapping when nothing was sampled.
fn average(total: u32, count: u32) -> u32 {
    if count == 0 { 0 } else { total / count }
}

/// Convert one C scene descriptor, dividing its accumulated sums by the sample
/// count the way `lv_demo_benchmark_summary_display` does.
///
/// # Safety
/// `dsc.name` must be either NULL or a valid NUL-terminated string that
/// outlives the call. Both hold for the demo's `'static` scene table.
unsafe fn scene_result(dsc: &lv_demo_benchmark_scene_dsc_t) -> SceneResult {
    let name = if dsc.name.is_null() {
        String::new()
    } else {
        // SAFETY: non-null and NUL-terminated by the caller's contract.
        let bytes = unsafe { CStr::from_ptr(dsc.name) }.to_bytes();
        String::from_utf8_lossy(bytes).into_owned()
    };
    let cnt = dsc.measurement_cnt;
    SceneResult {
        name,
        measurements: cnt,
        cpu_pct: average(dsc.cpu_avg_usage, cnt),
        fps: average(dsc.fps_avg, cnt),
        render_ms: average(dsc.render_avg_time, cnt),
        flush_ms: average(dsc.flush_avg_time, cnt),
    }
}

/// Walk the demo's scene table, which is terminated by a sentinel entry whose
/// `create_cb` is NULL (`lv_demo_benchmark.h`), and convert every real entry.
///
/// # Safety
/// `first` must be NULL, or point at a scene array terminated by such a
/// sentinel, with every `name` NULL or NUL-terminated.
unsafe fn collect_scenes(first: *const lv_demo_benchmark_scene_dsc_t) -> Vec<SceneResult> {
    let mut out = Vec::new();
    if first.is_null() {
        return out;
    }
    let mut i = 0usize;
    loop {
        // SAFETY: every index up to and including the sentinel is in bounds by
        // the caller's contract; the loop stops at the sentinel.
        let dsc = unsafe { &*first.add(i) };
        if dsc.create_cb.is_none() {
            return out;
        }
        // SAFETY: `dsc.name` obeys the caller's contract.
        out.push(unsafe { scene_result(dsc) });
        i += 1;
    }
}

/// Convert LVGL's summary struct into the owned Rust [`Summary`], dividing the
/// totals by the valid-scene count the way LVGL's own summary table does.
///
/// # Safety
/// `raw.scenes` must obey [`collect_scenes`]'s contract.
unsafe fn summary_from_raw(
    raw: &lv_demo_benchmark_summary_t,
    peak_heap_bytes: Option<usize>,
) -> Summary {
    // SAFETY: delegated to the caller's contract.
    let scenes = unsafe { collect_scenes(raw.scenes) };
    let valid = raw.valid_scene_cnt.max(0) as u32;
    Summary {
        scenes,
        valid_scenes: valid,
        cpu_pct: average(raw.total_avg_cpu.max(0) as u32, valid),
        fps: average(raw.total_avg_fps.max(0) as u32, valid),
        render_ms: average(raw.total_avg_render_time.max(0) as u32, valid),
        flush_ms: average(raw.total_avg_flush_time.max(0) as u32, valid),
        peak_heap_bytes,
    }
}

/// LVGL's heap high-water mark, or `None` when the allocator has no pool to
/// report on.
///
/// Gated on the same cfg as [`crate::mem`]'s pool API: only `LV_STDLIB_BUILTIN`
/// keeps TLSF figures. Under `LV_STDLIB_CLIB` LVGL defers to libc `malloc`, and
/// `lv_mem_monitor` would report zeroes that read like a measurement.
///
/// Called only from the end-of-run trampoline, never from the conversion: on an
/// uninitialised pool `lv_mem_monitor` divides by a zero total and raises
/// SIGFPE, which is reachable from the unit tests.
#[cfg(lvgl_builtin_malloc)]
fn peak_heap_bytes() -> Option<usize> {
    let mut mon = core::mem::MaybeUninit::<lv_mem_monitor_t>::uninit();
    // SAFETY: `lv_mem_monitor` fills the struct it is given; valid after
    // `lv_init`, which `Ui::init` performed before any run can start.
    let mon = unsafe {
        lv_mem_monitor(mon.as_mut_ptr());
        mon.assume_init()
    };
    Some(mon.max_used)
}

#[cfg(not(lvgl_builtin_malloc))]
fn peak_heap_bytes() -> Option<usize> {
    None
}

/// ~1.3x the measured peak — 44,404 B on ESP32, 48,292 B on ESP32-S3. LVGL's
/// own 128 KB is a `#warning` recommendation, not a requirement; used as one it
/// refused boards that complete every scene.
const BENCHMARK_HEAP_BYTES: usize = 64 * 1024;

/// Free heap across every registered pool, or `None` when LVGL is not using its
/// own allocator and there is nothing to measure.
///
/// The counterpart to the compile-time guard removed from `lv_demo_widgets.c`:
/// that one read `LV_MEM_SIZE`, which cannot see a pool added by
/// `lv_mem_add_pool`; this reads what is actually free right now.
#[cfg(lvgl_builtin_malloc)]
fn free_heap_bytes() -> Option<usize> {
    let mut mon = core::mem::MaybeUninit::<lv_mem_monitor_t>::uninit();
    // SAFETY: `lv_mem_monitor` fills the struct it is given; valid after
    // `lv_init`, which the non-null `lv_screen_active` check established.
    let mon = unsafe {
        lv_mem_monitor(mon.as_mut_ptr());
        mon.assume_init()
    };
    Some(mon.free_size)
}

#[cfg(not(lvgl_builtin_malloc))]
fn free_heap_bytes() -> Option<usize> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stand-in for a scene's `create_cb`. Only its non-NULL-ness matters: the
    /// scene walk uses it to tell a real entry from the sentinel.
    unsafe extern "C" fn dummy_create_cb() {}

    fn scene(
        name: &'static CStr,
        cnt: u32,
        cpu: u32,
        fps: u32,
        render: u32,
        flush: u32,
    ) -> lv_demo_benchmark_scene_dsc_t {
        lv_demo_benchmark_scene_dsc_t {
            name: name.as_ptr(),
            create_cb: Some(dummy_create_cb),
            scene_time: 3000,
            cpu_avg_usage: cpu,
            fps_avg: fps,
            render_avg_time: render,
            flush_avg_time: flush,
            measurement_cnt: cnt,
        }
    }

    fn sentinel() -> lv_demo_benchmark_scene_dsc_t {
        lv_demo_benchmark_scene_dsc_t {
            name: c"".as_ptr(),
            create_cb: None,
            ..Default::default()
        }
    }

    #[test]
    fn average_divides_and_survives_zero() {
        assert_eq!(average(90, 3), 30);
        assert_eq!(average(0, 0), 0);
        assert_eq!(average(7, 0), 0);
        // Truncating division, matching LVGL's integer summary table.
        assert_eq!(average(7, 2), 3);
    }

    #[test]
    fn scene_result_averages_the_accumulated_sums() {
        let dsc = scene(c"Rectangle", 4, 200, 120, 40, 60);
        // SAFETY: `name` is a 'static CStr.
        let r = unsafe { scene_result(&dsc) };
        assert_eq!(r.name, "Rectangle");
        assert_eq!(r.measurements, 4);
        assert_eq!(r.cpu_pct, 50);
        assert_eq!(r.fps, 30);
        assert_eq!(r.render_ms, 10);
        assert_eq!(r.flush_ms, 15);
        assert_eq!(r.total_ms(), 25);
        assert!(r.has_data());
    }

    #[test]
    fn scene_result_reports_no_data_without_measurements() {
        let dsc = scene(c"Never ran", 0, 0, 0, 0, 0);
        // SAFETY: `name` is a 'static CStr.
        let r = unsafe { scene_result(&dsc) };
        assert!(!r.has_data());
        assert_eq!(r.fps, 0);
    }

    #[test]
    fn scene_walk_stops_at_the_sentinel() {
        let table = [
            scene(c"A", 2, 100, 60, 20, 10),
            scene(c"B", 1, 40, 25, 5, 5),
            sentinel(),
            // Deliberately past the sentinel: reaching this entry means the
            // walk ignored the terminator.
            scene(c"PAST END", 1, 999, 999, 999, 999),
        ];
        // SAFETY: sentinel-terminated array of 'static-named scenes.
        let scenes = unsafe { collect_scenes(table.as_ptr()) };
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].name, "A");
        assert_eq!(scenes[0].fps, 30);
        assert_eq!(scenes[1].name, "B");
        assert_eq!(scenes[1].fps, 25);
    }

    #[test]
    fn scene_walk_handles_an_empty_table() {
        let table = [sentinel()];
        // SAFETY: sentinel-terminated array.
        let scenes = unsafe { collect_scenes(table.as_ptr()) };
        assert!(scenes.is_empty());

        // SAFETY: NULL is an explicitly permitted input.
        let none = unsafe { collect_scenes(core::ptr::null()) };
        assert!(none.is_empty());
    }

    #[test]
    fn summary_averages_over_valid_scenes() {
        let mut table = [
            scene(c"A", 2, 100, 60, 20, 10),
            scene(c"B", 1, 40, 25, 5, 5),
            sentinel(),
        ];
        let raw = lv_demo_benchmark_summary_t {
            scenes: table.as_mut_ptr(),
            total_avg_cpu: 90,
            total_avg_fps: 55,
            total_avg_render_time: 15,
            total_avg_flush_time: 10,
            valid_scene_cnt: 2,
        };
        // SAFETY: sentinel-terminated table with 'static names.
        let s = unsafe { summary_from_raw(&raw, None) };
        assert_eq!(s.scenes.len(), 2);
        assert_eq!(s.valid_scenes, 2);
        assert_eq!(s.cpu_pct, 45);
        assert_eq!(s.fps, 27);
        assert_eq!(s.render_ms, 7);
        assert_eq!(s.flush_ms, 5);
        assert_eq!(s.total_ms(), 12);
        assert!(s.has_data());
    }

    #[test]
    fn summary_with_no_valid_scenes_reports_no_data() {
        let mut table = [scene(c"A", 0, 0, 0, 0, 0), sentinel()];
        let raw = lv_demo_benchmark_summary_t {
            scenes: table.as_mut_ptr(),
            total_avg_cpu: 0,
            total_avg_fps: 0,
            total_avg_render_time: 0,
            total_avg_flush_time: 0,
            valid_scene_cnt: 0,
        };
        // SAFETY: sentinel-terminated table with 'static names.
        let s = unsafe { summary_from_raw(&raw, None) };
        assert_eq!(s.scenes.len(), 1);
        assert!(!s.scenes[0].has_data());
        assert!(!s.has_data());
        assert_eq!(s.fps, 0);
    }

    #[test]
    fn summary_from_a_null_scene_table_is_empty() {
        let raw = lv_demo_benchmark_summary_t {
            scenes: core::ptr::null_mut(),
            total_avg_cpu: 10,
            total_avg_fps: 10,
            total_avg_render_time: 10,
            total_avg_flush_time: 10,
            valid_scene_cnt: 1,
        };
        // SAFETY: NULL is an explicitly permitted `scenes` value.
        let s = unsafe { summary_from_raw(&raw, None) };
        assert!(s.scenes.is_empty());
        assert_eq!(s.fps, 10);
    }
}
