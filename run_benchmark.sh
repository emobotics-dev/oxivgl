#!/usr/bin/env bash
# Run LVGL's benchmark demo (oxivgl::demo::benchmark) via the demo_benchmark
# example.
#
# Usage:
#   ./run_benchmark.sh                 Host, interactive SDL window
#   ./run_benchmark.sh host            Same
#   ./run_benchmark.sh fire27          Flash + monitor on M5Stack Fire27 (ESP32)
#   ./run_benchmark.sh cores3          Flash + monitor on M5Stack CoreS3 (ESP32-S3)
#
# DEP_LV_CONFIG_PATH must be ABSOLUTE: oxivgl-sys resolves a relative value
# against its own package directory, not the workspace root.
# A separate CARGO_TARGET_DIR keeps this config's LVGL build beside the default.
set -e

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export DEP_LV_CONFIG_PATH="$ROOT/examples/conf-benchmark"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}/benchmark"

board="${1:-host}"
shift 2>/dev/null || true
[[ "${1:-}" == "--" ]] && shift

case "$board" in
  host)
    cargo run --features demo-benchmark --example demo_benchmark \
        --target x86_64-unknown-linux-gnu "$@"
    ;;
  fire27)
    cargo +esp -Zbuild-std=alloc,core run --release \
        --target xtensa-esp32-none-elf \
        --features fire27,demo-benchmark --example demo_benchmark "$@"
    ;;
  cores3)
    cargo +esp -Zbuild-std=alloc,core run --release \
        --target xtensa-esp32s3-none-elf \
        --features cores3,demo-benchmark --example demo_benchmark "$@"
    ;;
  *)
    echo "Usage: $0 [host|fire27|cores3] [-- extra cargo args]" >&2
    exit 1
    ;;
esac
