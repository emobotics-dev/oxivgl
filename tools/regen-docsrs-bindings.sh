#!/usr/bin/env bash
# Regenerate oxivgl-sys/bindings_docsrs.rs.
#
# docs.rs has no network access, so `oxivgl-sys` cannot download LVGL or run
# bindgen there. Under DOCS_RS it uses this committed snapshot instead and skips
# the download + cc compilation entirely (build.rs, the DOCS_RS branch).
#
# The snapshot therefore has to be refreshed whenever `default-conf/lv_conf.h`
# changes anything bindgen surfaces — fonts, widget enables, and in particular
# `LV_USE_STDLIB_MALLOC`, which gates whether `oxivgl::mem` is compiled at all.
# A stale snapshot does not fail the build: it silently publishes docs that are
# missing modules. That is exactly what happened when the allocator moved to
# LV_STDLIB_BUILTIN and the snapshot still reported CLIB.
#
# Generated from `default-conf`, not `examples/conf`: the snapshot should
# reflect what a consumer gets with the shipped default, not this repo's
# example/test configuration.
#
# Usage: ./tools/regen-docsrs-bindings.sh
set -euo pipefail

cd "$(dirname "$0")/.."

TARGET="x86_64-unknown-linux-gnu"
CONF="$PWD/oxivgl-sys/default-conf"
OUT="$PWD/oxivgl-sys/bindings_docsrs.rs"

echo "Regenerating $OUT from $CONF"

# build.rs does not declare rerun-if-env-changed for DEP_LV_CONFIG_PATH, so a
# changed config alone will not re-run it. Force it.
touch oxivgl-sys/build.rs

DEP_LV_CONFIG_PATH="$CONF" cargo build -p oxivgl-sys --target "$TARGET"

generated=$(find "target/$TARGET" -path '*/oxivgl-sys-*/out/bindings.rs' \
    -newer oxivgl-sys/build.rs -print0 2>/dev/null \
    | xargs -0 ls -t 2>/dev/null | head -1)

if [ -z "$generated" ]; then
    echo "error: no freshly generated bindings.rs found" >&2
    exit 1
fi

cp "$generated" "$OUT"
echo "Copied $generated"

# Sanity-check the one setting whose drift silently removes a module from the
# published docs.
backend=$(grep -oE 'LV_USE_STDLIB_MALLOC *: u32 = [0-9]+' "$OUT" | grep -oE '[0-9]+$')
builtin=$(grep -oE 'LV_STDLIB_BUILTIN *: u32 = [0-9]+' "$OUT" | grep -oE '[0-9]+$')
if [ "$backend" = "$builtin" ]; then
    echo "OK: allocator backend is LV_STDLIB_BUILTIN — oxivgl::mem will be documented"
else
    echo "NOTE: allocator backend is not BUILTIN — oxivgl::mem will NOT appear on docs.rs"
fi

# Leave the tree building against the workspace default again.
touch oxivgl-sys/build.rs
echo "Done. Rebuild normally before running tests."
