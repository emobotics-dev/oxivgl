#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(
    target_arch = "xtensa",
    feature(impl_trait_in_assoc_type, type_alias_impl_trait)
)]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Demo Benchmark — run LVGL's benchmark demo and report the result.
//!
//! Needs a configuration directory that enables `LV_USE_DEMO_BENCHMARK` and
//! compiles the demo's C sources. `./run_benchmark.sh` selects
//! `examples/conf-benchmark` for you; see that script and the module docs of
//! `oxivgl::demo`.
//!
//! The demo takes roughly two minutes and destroys every widget on the screen
//! it runs on, so [`oxivgl::demo::benchmark`] gives it a throwaway screen and
//! restores this view's screen when it ends. That is what this example
//! demonstrates: the "Benchmark running…" label below is still alive and
//! updatable afterwards, and the summary is written into it.

#[cfg(not(demo_benchmark))]
compile_error!(
    "the demo_benchmark example needs an lv_conf.h with LV_USE_DEMO_BENCHMARK 1 \
     and the demo's C sources — build it with ./run_benchmark.sh, which selects \
     DEP_LV_CONFIG_PATH=examples/conf-benchmark"
);

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;

use oxivgl::{
    demo::Summary,
    view::{NavAction, View},
    widgets::{Align, Label, LabelLongMode, Obj, WidgetError},
};

#[derive(Default)]
struct DemoBenchmark {
    label: Option<Label<'static>>,
    /// Filled in by the end callback, which cannot borrow the view because
    /// LVGL keeps the callback alive past this `create` call.
    result: Rc<RefCell<Option<Summary>>>,
    reported: bool,
}

impl View for DemoBenchmark {
    fn create(&mut self, container: &Obj<'static>) -> Result<(), WidgetError> {
        let label = Label::new(container)?;
        label
            .set_long_mode(LabelLongMode::Wrap)
            .text("Benchmark running…")
            .width(300)
            .align(Align::TopLeft, 8, 8);

        let sink = Rc::clone(&self.result);
        match oxivgl::demo::benchmark(move |summary| {
            oxivgl_examples_common::log::info!(
                "benchmark: {} FPS, {} % CPU, {} ms/frame over {} scenes",
                summary.fps,
                summary.cpu_pct,
                summary.total_ms(),
                summary.valid_scenes
            );
            for scene in &summary.scenes {
                if scene.has_data() {
                    oxivgl_examples_common::log::info!(
                        "  {:<28} {:>3} FPS  {:>3} %  {:>3} ms ({} render + {} flush)",
                        scene.name,
                        scene.fps,
                        scene.cpu_pct,
                        scene.total_ms(),
                        scene.render_ms,
                        scene.flush_ms
                    );
                } else {
                    oxivgl_examples_common::log::info!("  {:<28} not measured", scene.name);
                }
            }
            *sink.borrow_mut() = Some(summary.clone());
        }) {
            Ok(()) => {}
            Err(e) => {
                oxivgl_examples_common::log::error!("benchmark did not start: {e}");
                label.text(&alloc::format!("Benchmark did not start: {e}"));
            }
        }

        self.label = Some(label);
        Ok(())
    }

    fn update(&mut self) -> Result<NavAction, WidgetError> {
        if self.reported {
            return Ok(NavAction::None);
        }
        let Some(summary) = self.result.borrow_mut().take() else {
            return Ok(NavAction::None);
        };
        if let Some(label) = &self.label {
            label.text(&alloc::format!(
                "Benchmark done\n{} FPS, {} % CPU\n{} ms/frame ({} render + {} flush)\nover {} scenes",
                summary.fps,
                summary.cpu_pct,
                summary.total_ms(),
                summary.render_ms,
                summary.flush_ms,
                summary.valid_scenes
            ));
        }
        self.reported = true;
        Ok(NavAction::None)
    }
}

/// LVGL heap for the run, registered in PSRAM on target.
///
/// The benchmark's widgets scene needs ~38 KiB, which does not fit the internal
/// primary this config leaves deliberately small — see `examples/conf-benchmark`.
const LVGL_POOL_BYTES: usize = 512 * 1024;

oxivgl_examples_common::example_main_psram!(DemoBenchmark::default(), LVGL_POOL_BYTES);
