#!/usr/bin/env bash
# Capture screenshots for all examples.
set -e
for ex in getting_started{1,2,3,4} style{1,2,3,4,5,6,7,8,9,10,11,12,13,15,16,17,18} anim{1,2} anim_timeline1 event_{click,button,bubble,trickle} flex{1,2,3,4,5,6} grid{1,2,3,4,5,6} scroll{1,2,4} widget_obj1 widget_arc{1,2} image1 widget_bar{1,2,3,5} widget_button{1,2} widget_checkbox1 widget_dropdown{1,2} widget_label{1,2} widget_led1 widget_roller1 widget_slider2; do
  echo "=== $ex ==="
  SCREENSHOT_ONLY=1 LIBCLANG_PATH=/usr/lib64 cargo +nightly run --example "$ex" --target x86_64-unknown-linux-gnu
done
