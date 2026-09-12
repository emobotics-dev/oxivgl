# oxivgl v0.9.2

A consumer that lends LVGL a pool could not reliably get it back. Fixing that
moved event-list memory off LVGL's heap, which changes what the two heaps
report. Nothing to change in your code.

## A borrowed pool stayed borrowed

Lending LVGL memory for the duration of an operation — `lv_mem_add_pool`
before, `lv_mem_remove_pool` after — is how a board hands the renderer a window
it needs back afterwards, typically DRAM the radio wants returned. It did not
come back.

Any label marked for a text refresh registers a callback on the **display** and
removes it on the next layout pass, so a display's event list churns
continuously while anything at all is drawing. Each churn reallocates, and the
allocator is free to place the new buffer in the lent pool — where it stays,
because removing a pool requires it empty and a block that small cannot be
trimmed away. A few bytes hold a 32 KiB window hostage.

Event-list memory — the pointer array and the per-callback descriptors both —
now comes from an allocator that is not LVGL's, so it cannot enter a lent pool
at all.

## What you have to re-check

Event-list memory counts against the Rust global allocator instead of LVGL's
heap, so `lv_mem_monitor` totals fall by roughly 1.4 KB and the same memory
appears on the Rust side. **An application that budgets the two separately must
re-check both.**

The Rust-side high-water may rise by much less than the LVGL-side fall, because
the event-list peak need not coincide with the draw-buffer peak that dominates
that allocator. The memory moved regardless — read the LVGL-side drop, not the
Rust-side rise.

Measured over three benchmark runs per boot on both boards, lending the window
and taking it back each run:

| | ESP32 (Fire) | ESP32-S3 (CoreS3) |
|---|---|---|
| lent window returned | 3/3 | 3/3 |
| LVGL heap peak | 44,776 → 43,232 B (−1,544) | 48,816 → 47,440 B (−1,376) |
| Rust heap high-water | +84 B | +1,152 B |
| benchmark fps | unchanged | unchanged |

The ESP32 shows the *larger* LVGL-side drop and the *smaller* Rust-side rise,
which is the asymmetry above in one line. Render throughput is unaffected — the
benchmark sits at 21–22 fps on both boards before and after, quantised against
the refresh period.

Corroborated by a full 23-test hardware suite on both boards, three iterations
each, each from a fresh boot: 82 passes and no target failures, including the
page-cycling leak test that exercises the callback churn this release fixes.

## Changes to LVGL are patches now

`oxivgl-sys` modifies LVGL before compiling it, and those modifications used to
be string replacements inside `build.rs`. They are now a quilt-style numbered
series in `oxivgl-sys/patches/`, applied in filename order — one patch per
change, each carrying the reasoning behind it. A hunk that stops matching is a
build error naming the patch and the file, where the old helpers returned
quietly.

This matters to you only if you carry local LVGL changes: they belong in that
series, and the extracted tree is regenerated whenever the series changes, so
editing it directly achieves nothing.

No behaviour change — the tree the new mechanism produces is byte-identical to
the one the old code produced.

## The host suite could not run in an ESP container

If your devcontainer is set up to cross-compile for ESP, `./run_tests.sh`,
`./run_host.sh` and `./run_docs.sh` could not run at all: they inherited
`LIBCLANG_PATH` and `BINDGEN_EXTRA_CLANG_ARGS` from the ESP toolchain, so
bindgen read the host headers through a 32-bit target and failed. Two
independent faults, both fixed — the scripts now pin the host libclang and clear
the cross args, and `oxivgl-sys` names the target on host builds instead of only
when cross-compiling.

## Upgrading

Nothing to change. No public API moved. `oxivgl-sys` moves to 0.2.6 and
`oxivgl`'s requirement with it, because the patch series lives there.
