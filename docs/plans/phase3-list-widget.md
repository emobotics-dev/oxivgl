# Phase 3: List widget + list1, list2, scroll3

## Steps
1. Enable LV_USE_LIST in lv_conf.h
2. Create src/symbols.rs — safe wrappers for LV_SYMBOL_* UTF-8 constants
3. Create src/widgets/list.rs — List wrapper with add_text, add_button, get_button_text
4. Add Obj methods: move_to_index, get_index, move_background, get_style_pad_right
5. Examples: widget_list1, widget_list2, scroll3
6. Integration tests for List and new Obj methods
7. Leak test for List
8. Update run_host.sh, README.md, prelude.rs
