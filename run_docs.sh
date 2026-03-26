#!/usr/bin/env bash
# Build and open API documentation for the host target.
# Usage: ./run_docs.sh [--no-open] [--check]
#
#   --no-open   build without opening the browser
#   --check     warn-on-missing-docs only, no output (for CI)
set -e

export LIBCLANG_PATH="${LIBCLANG_PATH:-/usr/lib64}"
TARGET="x86_64-unknown-linux-gnu"

OPEN=1
CHECK=0

for arg in "$@"; do
    case "$arg" in
        --no-open) OPEN=0 ;;
        --check)   CHECK=1; OPEN=0 ;;
    esac
done

if [ "$CHECK" -eq 1 ]; then
    RUSTDOCFLAGS="-W missing-docs" \
        cargo +nightly doc \
            --target "$TARGET" \
            --no-deps \
            -j1 \
            2>&1 | grep "warning:" || true
    exit 0
fi

OPEN_FLAG=""
[ "$OPEN" -eq 1 ] && OPEN_FLAG="--open"

cargo +nightly doc \
    --target "$TARGET" \
    --no-deps \
    -j1 \
    $OPEN_FLAG
