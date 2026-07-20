#!/usr/bin/env bash
# Flash and monitor an LVGL example on the M5Stack CoreS3 (ESP32-S3) via espflash.
# Usage: ./run_cores3.sh <example_name>
#
# The `cores3` feature selects the ESP32-S3 board across the esp-hal family, the
# m5stack-core BSP (touch input + AXP2101/AW9523B display power/reset), and
# oxivgl's chip feature (see examples/common). The esp-hal family resolves to
# the emobotics fork via the workspace [patch].
set -e
cargo +esp -Zbuild-std=alloc,core run \
    --target xtensa-esp32s3-none-elf --release \
    --features cores3 \
    --example "${1:?usage: $0 <example>}"
