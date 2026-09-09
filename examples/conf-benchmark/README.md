# `conf-benchmark` — LVGL configuration with the benchmark demo enabled

An alternative to `examples/conf`, selected by `./run_benchmark.sh` through
`DEP_LV_CONFIG_PATH`. It exists because LVGL's benchmark demo cannot be turned
on from Rust: `LV_USE_DEMO_BENCHMARK` is an `lv_conf.h` define owned by the
application, and the demo's C sources are wrapped in `#if LV_USE_DEMO_BENCHMARK`.

## What differs from `examples/conf/lv_conf.h`

Three lines, and nothing else — keep it that way so the two files stay easy to
diff:

```
LV_BUILD_DEMOS        0 -> 1
LV_USE_DEMO_WIDGETS   0 -> 1   (the benchmark's last scene runs the widgets demo)
LV_USE_DEMO_BENCHMARK 0 -> 1
```

`LV_USE_SYSMON` and `LV_USE_PERF_MONITOR` are already `1` in the default
configuration. They are not optional here: the demo takes its samples from the
sysmon performance subject, and `oxivgl::demo::benchmark` refuses to start
without them rather than reporting a run of zeroes.

## No translation-unit shims needed

`oxivgl-sys` compiles LVGL's `demos/` tree whenever this configuration sets
`LV_BUILD_DEMOS 1` (`lv_conf_builds_demos` in `oxivgl-sys/build.rs`). An
application enables the benchmark by editing its `lv_conf.h` and nothing else —
it does not have to add `#include` shims of its own.

Compiling the whole tree also builds demos this configuration leaves off, and
their image assets, which carry no `LV_USE_DEMO_*` guard. That costs build time
but not image size: each asset is its own archive member, so the linker never
pulls one nothing references. Verified — a linked `demo_benchmark` binary
contains no `lv_demo_music` symbol.

## Cost

Roughly 900 KB of flash, nearly all of it the image assets. That is the reason
this is a separate configuration directory rather than a flag on the default
one — **never ship it in a production image.**
