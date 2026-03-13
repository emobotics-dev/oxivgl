#!/usr/bin/env bash
# Run oxivgl tests on the host.
# Usage: ./run_tests.sh [unit|int|all] [-- extra cargo args]
set -e

export LIBCLANG_PATH=/usr/lib64
TARGET="x86_64-unknown-linux-gnu"

mode="${1:-all}"
shift 2>/dev/null || true  # consume mode arg
# consume optional '--' separator
[[ "${1:-}" == "--" ]] && shift

case "$mode" in
  unit)
    echo "=== Unit tests ==="
    cargo +nightly test --lib --target "$TARGET" "$@"
    echo "=== Doc tests ==="
    cargo +nightly test --doc --target "$TARGET" "$@"
    ;;
  int|integration)
    echo "=== Integration tests ==="
    SDL_VIDEODRIVER=dummy cargo +nightly test --test integration --target "$TARGET" -- --test-threads=1 "$@"
    ;;
  all)
    echo "=== Unit tests ==="
    cargo +nightly test --lib --target "$TARGET" "$@"
    echo ""
    echo "=== Doc tests ==="
    cargo +nightly test --doc --target "$TARGET" "$@"
    echo ""
    echo "=== Integration tests ==="
    SDL_VIDEODRIVER=dummy cargo +nightly test --test integration --target "$TARGET" -- --test-threads=1 "$@"
    ;;
  *)
    echo "Usage: $0 [unit|int|all] [-- extra cargo args]"
    echo "  unit  — unit tests + doctests"
    echo "  int   — integration tests (headless LVGL)"
    echo "  all   — both (default)"
    exit 1
    ;;
esac
