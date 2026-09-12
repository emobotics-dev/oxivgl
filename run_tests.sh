#!/usr/bin/env bash
# Run oxivgl tests on the host.
# Usage: ./run_tests.sh [unit|int|all] [-- extra cargo args]
set -e

# Host bindgen needs the system libclang and no ESP --sysroot: a devcontainer
# pre-exports both for cross builds, and either makes bindgen read host headers
# as 32-bit and abort on pointer width. llvm-config, as /usr/lib64 is Fedora-only.
if command -v llvm-config >/dev/null 2>&1; then
    export LIBCLANG_PATH="$(llvm-config --libdir)"
fi
unset BINDGEN_EXTRA_CLANG_ARGS

TARGET="x86_64-unknown-linux-gnu"

mode="${1:-all}"
shift 2>/dev/null || true  # consume mode arg
# consume optional '--' separator
[[ "${1:-}" == "--" ]] && shift

case "$mode" in
  unit)
    echo "=== Unit tests ==="
    cargo test --lib --target "$TARGET" "$@"
    echo "=== Doc tests ==="
    cargo test --doc --target "$TARGET" "$@"
    ;;
  int|integration)
    echo "=== Integration tests ==="
    SDL_VIDEODRIVER=dummy cargo test --test integration --target "$TARGET" -- --test-threads=1 "$@"
    ;;
  pool)
    echo "=== Memory pool tests ==="
    SDL_VIDEODRIVER=dummy cargo test --test mem_pool --target "$TARGET" -- --test-threads=1 "$@"
    ;;
  leak)
    echo "=== Leak check tests ==="
    SDL_VIDEODRIVER=dummy cargo test --test leak_check --target "$TARGET" -- --test-threads=1 "$@"
    ;;
  bench)
    echo "=== Benchmark-demo unit tests (examples/conf-benchmark) ==="
    # Absolute path: oxivgl-sys resolves a relative one against its own
    # package dir. Separate target dir to keep the default build's cache.
    DEP_LV_CONFIG_PATH="$PWD/examples/conf-benchmark" \
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}/benchmark" \
      cargo test --lib --target "$TARGET" demo:: "$@"
    ;;
  all)
    echo "=== Unit tests ==="
    cargo test --lib --target "$TARGET" "$@"
    echo ""
    echo "=== Doc tests ==="
    cargo test --doc --target "$TARGET" "$@"
    echo ""
    echo "=== Integration tests ==="
    SDL_VIDEODRIVER=dummy cargo test --test integration --target "$TARGET" -- --test-threads=1 "$@"
    echo ""
    echo "=== Memory pool tests ==="
    SDL_VIDEODRIVER=dummy cargo test --test mem_pool --target "$TARGET" -- --test-threads=1 "$@"
    echo ""
    echo "=== Leak check tests ==="
    SDL_VIDEODRIVER=dummy cargo test --test leak_check --target "$TARGET" -- --test-threads=1 "$@"
    echo ""
    echo "=== Benchmark-demo unit tests (examples/conf-benchmark) ==="
    DEP_LV_CONFIG_PATH="$PWD/examples/conf-benchmark" \
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$PWD/target}/benchmark" \
      cargo test --lib --target "$TARGET" demo:: "$@"
    ;;
  *)
    echo "Usage: $0 [unit|int|pool|leak|bench|all] [-- extra cargo args]"
    echo "  unit  — unit tests + doctests"
    echo "  int   — integration tests (headless LVGL)"
    echo "  pool  — LVGL runtime memory pool registration"
    echo "  leak  — memory leak detection tests"
    echo "  bench — oxivgl::demo tests (needs examples/conf-benchmark)"
    echo "  all   — all of the above (default)"
    exit 1
    ;;
esac
