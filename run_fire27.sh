#!/usr/bin/env bash
# Flash and monitor an LVGL example on fire27 (ESP32) via espflash.
# Usage: ./run_fire27.sh <example_name>
set -e
cargo +esp -Zbuild-std=alloc,core run --example "${1:?usage: $0 <example>}"
