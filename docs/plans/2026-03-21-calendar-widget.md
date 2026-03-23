# Calendar Widget

## Goal
Wrap LVGL's `lv_calendar` widget.

## Steps
1. Enable `LV_USE_CALENDAR 1` in `conf/lv_conf.h` (+ HEADER_ARROW, HEADER_DROPDOWN)
2. `src/widgets/calendar.rs` — `CalendarDate` + `Calendar<'p>`
3. Register in `mod.rs`, `prelude.rs`
4. `examples/calendar_1.rs` — port C example 1
5. Integration tests (4) + leak test (1)
6. Screenshot + README update

## API surface
- `CalendarDate { year: u32, month: u8, day: u8 }`
- `Calendar::new` → `Result<Self, WidgetError>`
- `set_today_date(year, month, day)` / `set_month_shown(year, month)`
- `set_highlighted_dates(&[CalendarDate])` — copies into internal `RefCell<Vec>`, passes ptr to LVGL
- `get_pressed_date()` → `Option<CalendarDate>`
- `get_today_date()` / `get_showed_date()` → `CalendarDate`
- `add_header_arrow()` / `add_header_dropdown()` → `Child<Obj>`
- `get_btnmatrix()` → `Child<Obj>`

## Skipped
- `set_day_names` (pointer-to-pointer, not in examples)
- `calendar_2` (requires LV_USE_CALENDAR_CHINESE + CJK font)
