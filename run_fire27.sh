#!/usr/bin/env bash
# Flash and monitor an LVGL example on the M5Stack Fire27 (ESP32) via espflash.
# Usage: ./run_fire27.sh <example_name>
#
# The `fire27` feature selects the ESP32 board across the esp-hal family, the
# m5stack-core BSP, and oxivgl's chip feature (see examples/common). The
# esp-hal family resolves to the emobotics fork via the workspace [patch].
set -e
cargo +esp -Zbuild-std=alloc,core run \
    --target xtensa-esp32-none-elf --release \
    --features fire27 \
    --example "${1:?usage: $0 <example>}"
