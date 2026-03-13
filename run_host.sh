#!/usr/bin/env bash
# Run an LVGL example on the host via SDL2.
# Usage: ./run_host.sh <example_name>
set -e
LIBCLANG_PATH=/usr/lib64 cargo +nightly run --example "${1:?usage: $0 <example>}" --target x86_64-unknown-linux-gnu
