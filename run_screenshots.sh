#!/usr/bin/env bash
# Capture screenshots for all examples.
set -e
for ex in getting_started{1,2,3,4} style{1,2,3,4,5,7,8,9,10,11,12,13,15,16,17,18}; do
  echo "=== $ex ==="
  SCREENSHOT_ONLY=1 LIBCLANG_PATH=/usr/lib64 cargo +nightly run --example "$ex" --target x86_64-unknown-linux-gnu
done
