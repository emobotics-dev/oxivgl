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
    # Capture rather than pipe. Two traps this avoids:
    #  - Broken intra-doc links are `error:`, not `warning:`, and a failed doc
    #    build emits no HTML at all — so grepping only for "warning:" reports a
    #    clean audit while the module you just added is missing from the docs.
    #  - Piping discards cargo's exit status, and the old `|| true; exit 0` made
    #    this check incapable of ever failing.
    # `set -e` would abort at the capture on a failed doc build, printing
    # nothing at all — so suspend it and handle the status explicitly.
    set +e
    # CARGO_TERM_COLOR=always (CI) prefixes every cargo line with ANSI, so
    # grepping `^(error|warning)` matches nothing and the real rustdoc
    # errors vanish — CI then only prints "FAIL: doc build failed".
    out=$(CARGO_TERM_COLOR=never RUSTDOCFLAGS="-W missing-docs" \
        cargo +nightly doc \
            --target "$TARGET" \
            --no-deps \
            --color never \
            -j1 \
            2>&1)
    status=$?
    set -e

    plain=$(printf '%s\n' "$out" | sed $'s/\033\\[[0-9;]*[mK]//g')

    if [ "$status" -ne 0 ]; then
        printf '%s\n' "$plain"
        echo "FAIL: doc build failed — see errors above" >&2
        exit 1
    fi
    # Cargo also prints `warning: patch … was not used` for firmware-only
    # [patch] entries; those are not rustdoc missing-docs.
    if printf '%s\n' "$plain" | grep -E 'warning: missing documentation' >/dev/null; then
        printf '%s\n' "$plain" | grep -E '^(error|warning)' | grep -v 'warning: patch ' || true
        echo "FAIL: undocumented public items" >&2
        exit 1
    fi
    echo "Doc check passed: build succeeded, no undocumented public items."
    exit 0
fi

OPEN_FLAG=""
[ "$OPEN" -eq 1 ] && OPEN_FLAG="--open"

cargo +nightly doc \
    --target "$TARGET" \
    --no-deps \
    -j1 \
    $OPEN_FLAG
