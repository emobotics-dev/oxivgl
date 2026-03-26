#!/usr/bin/env bash
# Run LVGL examples on the host via SDL2.
#
# Usage:
#   ./run_host.sh <example>           Interactive SDL window
#   ./run_host.sh -s <example>        Screenshot only (no window)
#   ./run_host.sh -s                  Screenshot all examples
set -e

# Use system 64-bit libclang for host builds.
# The ESP toolchain may set LIBCLANG_PATH to a 32-bit clang which
# breaks x86_64 bindgen — override it when detected.
if [[ "${LIBCLANG_PATH:-}" == *"xtensa"* || "${LIBCLANG_PATH:-}" == *"esp"* ]]; then
    export LIBCLANG_PATH=/usr/lib64
elif [[ -z "${LIBCLANG_PATH:-}" ]]; then
    export LIBCLANG_PATH=/usr/lib64
fi
TARGET="x86_64-unknown-linux-gnu"

SCREENSHOT=0
if [[ "${1:-}" == "-s" ]]; then
    SCREENSHOT=1
    shift
fi

ALL_EXAMPLES=(
    getting_started{1,2,3,4,5,6,7,8}
    style{1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,20,21}
    anim{1,2,3,4} anim_timeline1
    event_{click,button,bubble,trickle,streak,draw}
    flex{1,2,3,4,5,6}
    grid{1,2,3,4,5,6}
    scroll{1,2,3,4,6,7,8,9}
    widget_obj{1,2,3}
    widget_arc{1,2,3}
    widget_arclabel1
    calendar_1
    calendar_2
    image1
    widget_bar{1,2,3,4,5,6,7}
    widget_button{1,2,3}
    widget_checkbox{1,2}
    widget_dropdown{1,2,3}
    widget_image{2,3,4,5}
    widget_label{1,2,5}
    widget_label{3,6,7}
    widget_led1
    widget_list{1,2}
    widget_line1
    widget_menu{1,2,3,4,5}
    widget_msgbox{1,2,3}
    widget_roller{1,2}
    widget_scale{1,2,3,4,5,6,7,8,9,10,11,12}
    widget_slider{1,2,3,4}
    widget_switch{1,2}
    table_{1,2}
    tabview_{1,2}
    widget_textarea{1,2,3,4}
    canvas_{1,2,3,4,5,6,7,9,10,11,12}
    spinner_1
    spinbox_1
    span_1
    tileview_1
    imagebutton_1
    win_1
    animimg_1
    widget_label4
    widget_roller3
    widget_scale12
    widget_chart{1,2,3}
    widget_buttonmatrix1
    widget_keyboard{1,2}
    widget_chart{6,7,8}
    widget_buttonmatrix3
    widget_buttonmatrix2
    widget_chart4
    widget_keyboard3
    observer1
    observer2
    scroll5
)

run_example() {
    local ex="$1"
    if [[ "$SCREENSHOT" == 1 ]]; then
        echo "=== $ex ==="
        SCREENSHOT_ONLY=1 SDL_VIDEODRIVER=dummy \
            cargo +nightly run --example "$ex" --target "$TARGET"
    else
        echo "Running $ex (SDL window)… Close the window or press Ctrl-C to exit."
        cargo +nightly run --example "$ex" --target "$TARGET"
    fi
}

if [[ $# -eq 0 && "$SCREENSHOT" == 1 ]]; then
    # Build all examples first
    cargo +nightly build --examples --target "$TARGET"
    # Run built binaries directly in parallel (avoids cargo lock contention)
    BIN_DIR="target/$TARGET/debug/examples"
    N=${SCREENSHOT_JOBS:-$(nproc 2>/dev/null || echo 4)}
    export SCREENSHOT_ONLY=1 SDL_VIDEODRIVER=dummy
    run_one_screenshot() {
        echo "=== $1 ==="
        "$BIN_DIR/$1"
    }
    export -f run_one_screenshot
    export BIN_DIR
    printf '%s\n' "${ALL_EXAMPLES[@]}" | xargs -P"$N" -I{} bash -c 'run_one_screenshot {}'
elif [[ $# -ge 1 ]]; then
    run_example "$1"
else
    echo "Usage: $0 [-s] [<example>]"
    echo "  $0 <example>       Interactive SDL window"
    echo "  $0 -s <example>    Screenshot only (no window)"
    echo "  $0 -s              Screenshot all examples"
    exit 1
fi
