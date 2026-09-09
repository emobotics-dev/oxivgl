// SPDX-License-Identifier: MIT OR Apache-2.0

#ifndef LVGL_API_H
#define LVGL_API_H

#ifdef __cplusplus
extern "C"
{
#endif

#include "lvgl.h"

/* LVGL's benchmark demo. Whether it exists at all is the application's
 * decision, taken in its own lv_conf.h (LV_USE_DEMO_BENCHMARK) — see
 * examples/conf-benchmark. The LVGL root is already an include directory and
 * lv_conf_internal.h has defaulted the flag to 0 by this point, so the include
 * simply vanishes for every application that leaves the demo off. When it is
 * on, bindgen emits the lv_demo_benchmark* declarations and build.rs turns
 * that into the `demo_benchmark` cfg that gates `oxivgl::demo`. */
#if LV_USE_DEMO_BENCHMARK
#include "demos/benchmark/lv_demo_benchmark.h"
#endif

    lv_color_t _LV_COLOR_MAKE(uint8_t r, uint8_t g, uint8_t b);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /*LVGL_API*/
