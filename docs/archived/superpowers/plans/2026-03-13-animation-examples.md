# LVGL Animation Examples Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `Anim`, `AnimTimeline`, `Switch` wrappers + `Obj`/`Slider` extensions, then implement LVGL animation chapter examples (anim1–3, anim_timeline1).

**Architecture:** New `src/widgets/anim.rs` module wraps `lv_anim_t` as a stack-local builder (LVGL copies it on `start()`). `src/widgets/anim_timeline.rs` owns `lv_anim_timeline_t*` with Drop. `src/widgets/switch.rs` follows existing widget pattern (Deref to Obj). Existing `Obj` gains `get_x`, `get_width`, `add_state`, `has_state`, `add_flag`, `set_scrollbar_mode`. `lv_conf.h` enables `LV_USE_SWITCH`, `LV_USE_FLEX`.

**Tech Stack:** Rust no_std, lvgl_rust_sys FFI bindings, LVGL 9.x C lib

**Skip rationale for anim3:** Requires `LV_USE_CHART` + `LV_USE_GRID` + scatter chart API + global mutable state. Significant scope for one example — documented but skipped.

---

## Chunk 1: Config + Core Lib Additions

### Task 1: Enable LV_USE_SWITCH and LV_USE_FLEX in lv_conf.h

**Files:**
- Modify: `conf/lv_conf.h:269` (SWITCH 0→1)
- Modify: `conf/lv_conf.h:294` (FLEX 0→1)

- [ ] **Step 1: Edit lv_conf.h**

```c
#define LV_USE_SWITCH     1   // was 0
#define LV_USE_FLEX 1         // was 0
```

- [ ] **Step 2: Verify build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`
Expected: compiles (new symbols now available in bindings)

- [ ] **Step 3: Commit**

```bash
git add conf/lv_conf.h
git commit -m "feat: enable LV_USE_SWITCH and LV_USE_FLEX in lv_conf.h"
```

---

### Task 2: Add Obj methods — get_x, get_width, add_state, has_state, add_flag, set_scrollbar_mode

**Files:**
- Modify: `src/widgets/obj.rs` (add methods to `impl Obj`)
- Modify: `src/widgets/mod.rs` (re-export new constants)

- [ ] **Step 1: Add methods to Obj**

In `src/widgets/obj.rs`, add to `impl<'p> Obj<'p>`:

```rust
pub fn get_x(&self) -> i32 {
    assert_ne!(self.handle, null_mut());
    unsafe { lv_obj_get_x(self.handle) }
}

pub fn get_width(&self) -> i32 {
    assert_ne!(self.handle, null_mut());
    unsafe { lv_obj_get_width(self.handle) }
}

pub fn add_state(&self, state: lv_state_t) -> &Self {
    assert_ne!(self.handle, null_mut());
    unsafe { lv_obj_add_state(self.handle, state) };
    self
}

pub fn has_state(&self, state: lv_state_t) -> bool {
    assert_ne!(self.handle, null_mut());
    unsafe { lv_obj_has_state(self.handle, state) }
}

pub fn add_flag(&self, flag: lv_obj_flag_t) -> &Self {
    assert_ne!(self.handle, null_mut());
    unsafe { lv_obj_add_flag(self.handle, flag) };
    self
}

pub fn set_scrollbar_mode(&self, mode: lv_scrollbar_mode_t) -> &Self {
    assert_ne!(self.handle, null_mut());
    unsafe { lv_obj_set_scrollbar_mode(self.handle, mode) };
    self
}
```

- [ ] **Step 2: Add constants to mod.rs**

In `src/widgets/mod.rs`, add re-exports:

```rust
pub use lvgl_rust_sys::{
    lv_state_t_LV_STATE_CHECKED as LV_STATE_CHECKED_RAW,
    lv_obj_flag_t_LV_OBJ_FLAG_CHECKABLE as LV_OBJ_FLAG_CHECKABLE,
    lv_obj_flag_t_LV_OBJ_FLAG_IGNORE_LAYOUT as LV_OBJ_FLAG_IGNORE_LAYOUT,
    lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF as LV_SCROLLBAR_MODE_OFF,
};
```

Also add `LV_EVENT_CLICKED`:
```rust
pub const LV_EVENT_CLICKED: lv_event_code_t =
    lvgl_rust_sys::lv_event_code_t_LV_EVENT_CLICKED;
```

- [ ] **Step 3: Verify build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`

- [ ] **Step 4: Commit**

```bash
git add src/widgets/obj.rs src/widgets/mod.rs
git commit -m "feat: add Obj methods — get_x, get_width, add_state, has_state, add_flag, set_scrollbar_mode"
```

---

### Task 3: Add Slider::set_range

**Files:**
- Modify: `src/widgets/slider.rs`

- [ ] **Step 1: Add set_range method**

```rust
pub fn set_range(&self, min: i32, max: i32) -> &Self {
    assert_ne!(self.obj.handle(), null_mut());
    unsafe { lv_slider_set_range(self.obj.handle(), min, max) };
    self
}
```

- [ ] **Step 2: Verify build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`

- [ ] **Step 3: Commit**

```bash
git add src/widgets/slider.rs
git commit -m "feat: add Slider::set_range"
```

---

### Task 4: Create Anim wrapper

**Files:**
- Create: `src/widgets/anim.rs`
- Modify: `src/widgets/mod.rs` (add `mod anim; pub use anim::*;`)

- [ ] **Step 1: Create src/widgets/anim.rs**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
use core::ffi::c_void;
use lvgl_rust_sys::*;

use super::obj::AsLvHandle;

/// Stack-local animation builder. LVGL copies the descriptor on `start()`,
/// so this can be dropped after starting.
pub struct Anim {
    pub(crate) inner: lv_anim_t,
}

impl Anim {
    pub fn new() -> Self {
        let mut inner = unsafe { core::mem::zeroed::<lv_anim_t>() };
        unsafe { lv_anim_init(&mut inner) };
        Self { inner }
    }

    /// Set the animated variable (the raw `lv_obj_t*` pointer).
    pub fn set_var(&mut self, obj: &impl AsLvHandle) -> &mut Self {
        unsafe { lv_anim_set_var(&mut self.inner, obj.lv_handle() as *mut c_void) };
        self
    }

    pub fn set_values(&mut self, start: i32, end: i32) -> &mut Self {
        unsafe { lv_anim_set_values(&mut self.inner, start, end) };
        self
    }

    pub fn set_duration(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_duration(&mut self.inner, ms) };
        self
    }

    pub fn set_exec_cb(&mut self, cb: lv_anim_exec_xcb_t) -> &mut Self {
        unsafe { lv_anim_set_exec_cb(&mut self.inner, cb) };
        self
    }

    pub fn set_custom_exec_cb(&mut self, cb: lv_anim_custom_exec_cb_t) -> &mut Self {
        unsafe { lv_anim_set_custom_exec_cb(&mut self.inner, cb) };
        self
    }

    pub fn set_path_cb(&mut self, cb: lv_anim_path_cb_t) -> &mut Self {
        unsafe { lv_anim_set_path_cb(&mut self.inner, cb) };
        self
    }

    pub fn set_reverse_duration(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_reverse_duration(&mut self.inner, ms) };
        self
    }

    pub fn set_reverse_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_reverse_delay(&mut self.inner, ms) };
        self
    }

    pub fn set_repeat_count(&mut self, cnt: u32) -> &mut Self {
        unsafe { lv_anim_set_repeat_count(&mut self.inner, cnt) };
        self
    }

    pub fn set_repeat_delay(&mut self, ms: u32) -> &mut Self {
        unsafe { lv_anim_set_repeat_delay(&mut self.inner, ms) };
        self
    }

    /// Start the animation. LVGL copies the descriptor internally.
    pub fn start(&self) {
        unsafe { lv_anim_start(&self.inner) };
    }
}

// Animation path functions
pub unsafe extern "C" fn anim_path_overshoot(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_overshoot(a) }
}

pub unsafe extern "C" fn anim_path_ease_in(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_in(a) }
}

pub unsafe extern "C" fn anim_path_ease_out(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_out(a) }
}

pub unsafe extern "C" fn anim_path_ease_in_out(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_ease_in_out(a) }
}

pub unsafe extern "C" fn anim_path_bounce(a: *const lv_anim_t) -> i32 {
    unsafe { lv_anim_path_bounce(a) }
}

/// `LV_ANIM_REPEAT_INFINITE`
pub const ANIM_REPEAT_INFINITE: u32 = LV_ANIM_REPEAT_INFINITE;
```

- [ ] **Step 2: Register module in mod.rs**

Add `mod anim;` and:
```rust
pub use anim::{
    Anim, ANIM_REPEAT_INFINITE,
    anim_path_overshoot, anim_path_ease_in, anim_path_ease_out,
    anim_path_ease_in_out, anim_path_bounce,
};
```

- [ ] **Step 3: Verify build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`

- [ ] **Step 4: Commit**

```bash
git add src/widgets/anim.rs src/widgets/mod.rs
git commit -m "feat: add Anim wrapper with path functions"
```

---

### Task 5: Create AnimTimeline wrapper

**Files:**
- Create: `src/widgets/anim_timeline.rs`
- Modify: `src/widgets/mod.rs`

- [ ] **Step 1: Create src/widgets/anim_timeline.rs**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
use lvgl_rust_sys::*;

use super::anim::Anim;

/// Owning wrapper around `lv_anim_timeline_t*`. Calls `lv_anim_timeline_delete` on drop.
pub struct AnimTimeline {
    handle: *mut lv_anim_timeline_t,
}

impl AnimTimeline {
    pub fn new() -> Self {
        let handle = unsafe { lv_anim_timeline_create() };
        assert!(!handle.is_null(), "lv_anim_timeline_create returned NULL");
        Self { handle }
    }

    pub fn add(&mut self, start_time: u32, anim: &Anim) -> &mut Self {
        unsafe { lv_anim_timeline_add(self.handle, start_time, &anim.inner) };
        self
    }

    pub fn start(&self) -> u32 {
        unsafe { lv_anim_timeline_start(self.handle) }
    }

    pub fn pause(&self) {
        unsafe { lv_anim_timeline_pause(self.handle) }
    }

    pub fn set_reverse(&self, reverse: bool) {
        unsafe { lv_anim_timeline_set_reverse(self.handle, reverse) }
    }

    pub fn set_progress(&self, progress: u16) {
        unsafe { lv_anim_timeline_set_progress(self.handle, progress) }
    }

    pub fn handle(&self) -> *mut lv_anim_timeline_t {
        self.handle
    }
}

impl Drop for AnimTimeline {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { lv_anim_timeline_delete(self.handle) };
        }
    }
}

/// `LV_ANIM_TIMELINE_PROGRESS_MAX`
pub const ANIM_TIMELINE_PROGRESS_MAX: u16 = LV_ANIM_TIMELINE_PROGRESS_MAX as u16;
```

- [ ] **Step 2: Register in mod.rs**

```rust
mod anim_timeline;
pub use anim_timeline::{AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX};
```

- [ ] **Step 3: Verify build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`

- [ ] **Step 4: Commit**

```bash
git add src/widgets/anim_timeline.rs src/widgets/mod.rs
git commit -m "feat: add AnimTimeline wrapper"
```

---

### Task 6: Create Switch widget

**Files:**
- Create: `src/widgets/switch.rs`
- Modify: `src/widgets/mod.rs`

- [ ] **Step 1: Create src/widgets/switch.rs**

```rust
// SPDX-License-Identifier: MIT OR Apache-2.0
use core::{ops::Deref, ptr::null_mut};

use lvgl_rust_sys::*;

use super::{
    WidgetError,
    obj::{AsLvHandle, Obj},
};

/// LVGL switch (toggle) widget.
#[derive(Debug)]
pub struct Switch<'p> {
    obj: Obj<'p>,
}

impl<'p> AsLvHandle for Switch<'p> {
    fn lv_handle(&self) -> *mut lv_obj_t {
        self.obj.lv_handle()
    }
}

impl<'p> Deref for Switch<'p> {
    type Target = Obj<'p>;
    fn deref(&self) -> &Obj<'p> {
        &self.obj
    }
}

impl<'p> Switch<'p> {
    pub fn new(parent: &impl AsLvHandle) -> Result<Self, WidgetError> {
        let parent_ptr = parent.lv_handle();
        assert_ne!(parent_ptr, null_mut(), "Parent widget cannot be null");
        let handle = unsafe { lv_switch_create(parent_ptr) };
        if handle.is_null() {
            Err(WidgetError::LvglNullPointer)
        } else {
            Ok(Switch { obj: Obj::from_raw(handle) })
        }
    }
}
```

- [ ] **Step 2: Register in mod.rs**

```rust
mod switch;
pub use switch::Switch;
```

- [ ] **Step 3: Verify build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`

- [ ] **Step 4: Commit**

```bash
git add src/widgets/switch.rs src/widgets/mod.rs
git commit -m "feat: add Switch widget wrapper"
```

---

### Task 7: Add Flex layout helpers to Screen/Obj

**Files:**
- Modify: `src/widgets/obj.rs`
- Modify: `src/widgets/mod.rs`

- [ ] **Step 1: Add flex methods to Obj**

```rust
pub fn set_flex_flow(&self, flow: FlexFlow) -> &Self {
    assert_ne!(self.handle, null_mut());
    unsafe { lv_obj_set_flex_flow(self.handle, flow as lv_flex_flow_t) };
    self
}

pub fn set_flex_align(&self, main: FlexAlign, cross: FlexAlign, track: FlexAlign) -> &Self {
    assert_ne!(self.handle, null_mut());
    unsafe {
        lv_obj_set_flex_align(
            self.handle,
            main as lv_flex_align_t,
            cross as lv_flex_align_t,
            track as lv_flex_align_t,
        )
    };
    self
}
```

- [ ] **Step 2: Add FlexFlow and FlexAlign enums to obj.rs**

```rust
/// Type-safe wrapper for `lv_flex_flow_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum FlexFlow {
    Row = 0x00,
    Column = 0x01,
    RowWrap = 0x02 | 0x01 << 2,  // LV_FLEX_FLOW_ROW_WRAP
    // add more as needed
}

/// Type-safe wrapper for `lv_flex_align_t`.
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum FlexAlign {
    Start = 0,
    End = 1,
    Center = 2,
    SpaceEvenly = 3,
    SpaceAround = 4,
    SpaceBetween = 5,
}
```

Note: exact enum values must match the bindings. Check `lv_flex_flow_t_*` and `lv_flex_align_t_*` constants in generated bindings after enabling LV_USE_FLEX.

- [ ] **Step 3: Re-export in mod.rs**

```rust
pub use obj::{Align, AsLvHandle, FlexAlign, FlexFlow, Obj, Part, Screen, TextAlign};
```

- [ ] **Step 4: Verify build**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly check --target x86_64-unknown-linux-gnu`

- [ ] **Step 5: Commit**

```bash
git add src/widgets/obj.rs src/widgets/mod.rs
git commit -m "feat: add Flex layout helpers to Obj"
```

---

### Task 8: Run all existing tests to verify no regressions

- [ ] **Step 1: Run tests**

Run: `LIBCLANG_PATH=/usr/lib64 cargo +nightly test --target x86_64-unknown-linux-gnu`
Expected: all pass

- [ ] **Step 2: Run existing examples**

Run: `./run_host.sh getting_started1` and `./run_host.sh style10`
Expected: both succeed (screenshot captured)

---

## Chunk 2: Animation Examples

### Task 9: Implement anim1 — Start animation on event

**Files:**
- Create: `examples/anim1.rs`

C original: Switch toggles label X-position animation with overshoot/ease_in paths.

- [ ] **Step 1: Create examples/anim1.rs**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 1 — Start animation on an event

use core::ffi::c_void;
use oxivgl::{
    view::View,
    widgets::{
        Align, Anim, Label, LV_EVENT_VALUE_CHANGED, Screen, Switch, WidgetError,
        anim_path_ease_in, anim_path_overshoot,
    },
};
use lvgl_rust_sys::*;

struct Anim1 {
    _label: Label<'static>,
    _sw: Switch<'static>,
}

unsafe extern "C" fn anim_x_cb(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_x(var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn sw_event_cb(e: *mut lv_event_t) {
    unsafe {
        let sw = lv_event_get_target_obj(e);
        let label = lv_event_get_user_data(e) as *mut lv_obj_t;
        let checked = lv_obj_has_state(sw, lv_state_t_LV_STATE_CHECKED);

        let mut a = core::mem::zeroed::<lv_anim_t>();
        lv_anim_init(&mut a);
        lv_anim_set_var(&mut a, label as *mut c_void);
        lv_anim_set_duration(&mut a, 500);
        lv_anim_set_exec_cb(&mut a, Some(anim_x_cb));

        if checked {
            lv_anim_set_values(&mut a, lv_obj_get_x(label), 100);
            lv_anim_set_path_cb(&mut a, Some(anim_path_overshoot));
        } else {
            lv_anim_set_values(&mut a, lv_obj_get_x(label), -lv_obj_get_width(label));
            lv_anim_set_path_cb(&mut a, Some(anim_path_ease_in));
        }
        lv_anim_start(&a);
    }
}

impl View for Anim1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let label = Label::new(&screen)?;
        label.text("Hello animations!\0")?.pos(100, 10);

        let sw = Switch::new(&screen)?;
        sw.center();
        sw.add_state(lv_state_t_LV_STATE_CHECKED);
        sw.on_event(sw_event_cb, LV_EVENT_VALUE_CHANGED, label.handle() as *mut c_void);

        Ok(Self { _label: label, _sw: sw })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Anim1);
```

- [ ] **Step 2: Test**

Run: `./run_host.sh anim1`
Expected: window shows label + switch, screenshot captured

- [ ] **Step 3: Commit**

```bash
git add examples/anim1.rs
git commit -m "feat: anim1 example — start animation on event"
```

---

### Task 10: Implement anim2 — Playback animation

**Files:**
- Create: `examples/anim2.rs`

C original: Red circle animates size + X position with repeat/reverse/playback.

- [ ] **Step 1: Create examples/anim2.rs**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim 2 — Playback animation

use core::ffi::c_void;
use oxivgl::{
    view::View,
    widgets::{
        Align, Anim, ANIM_REPEAT_INFINITE, Obj, Palette, Screen, WidgetError,
        anim_path_ease_in_out, palette_main,
    },
};
use lvgl_rust_sys::*;

struct Anim2 {
    _obj: Obj<'static>,
}

unsafe extern "C" fn anim_x_cb(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_x(var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn anim_size_cb(var: *mut c_void, v: i32) {
    unsafe { lv_obj_set_size(var as *mut lv_obj_t, v, v) };
}

impl View for Anim2 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let obj = Obj::new(&screen)?;
        obj.remove_scrollable();
        obj.style_bg_color(palette_main(Palette::Red), 0);
        obj.radius(0x7fff, 0); // LV_RADIUS_CIRCLE
        obj.align(Align::LeftMid, 10, 0);

        let mut a = Anim::new();
        a.set_var(&obj)
            .set_values(10, 50)
            .set_duration(1000)
            .set_reverse_delay(100)
            .set_reverse_duration(300)
            .set_repeat_delay(500)
            .set_repeat_count(ANIM_REPEAT_INFINITE)
            .set_path_cb(Some(anim_path_ease_in_out));

        a.set_exec_cb(Some(anim_size_cb));
        a.start();

        a.set_exec_cb(Some(anim_x_cb));
        a.set_values(10, 240);
        a.start();

        Ok(Self { _obj: obj })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(Anim2);
```

- [ ] **Step 2: Test**

Run: `./run_host.sh anim2`
Expected: red circle animates

- [ ] **Step 3: Commit**

```bash
git add examples/anim2.rs
git commit -m "feat: anim2 example — playback animation"
```

---

### Task 11: Implement anim3 or document skip

**Files:**
- Create: `examples/anim3.rs` (if feasible)

C original: Cubic bezier with Chart widget, Grid layout, scatter plot, sliders.

**Assessment:** Requires `LV_USE_CHART=1`, `LV_USE_GRID=1`, Chart Rust wrapper, scatter chart API, grid cell API — all currently missing. This is a major widget addition tangent.

- [ ] **Step 1: Decide — attempt or skip**

If skipping: document in commit message why. Proceed to Task 12.

If attempting: enable `LV_USE_CHART=1` and `LV_USE_GRID=1` in `lv_conf.h`, verify bindings generate, then implement minimal Chart wrapper + grid helpers. This is estimated at 3-4 additional tasks.

---

### Task 12: Implement anim_timeline1 — Animation timeline

**Files:**
- Create: `examples/anim_timeline1.rs`

C original: 3 objects with coordinated width/height animations via timeline, start/pause buttons, progress slider. Uses flex layout.

- [ ] **Step 1: Create examples/anim_timeline1.rs**

```rust
#![cfg_attr(target_arch = "xtensa", no_std, no_main)]
#![cfg_attr(target_arch = "xtensa", feature(impl_trait_in_assoc_type, type_alias_impl_trait))]
// SPDX-License-Identifier: MIT OR Apache-2.0
//! Anim Timeline 1 — Animation timeline

extern crate alloc;

use alloc::boxed::Box;
use core::ffi::c_void;
use oxivgl::{
    view::View,
    widgets::{
        Align, Anim, AnimTimeline, ANIM_TIMELINE_PROGRESS_MAX, Button, FlexAlign, FlexFlow,
        Label, LV_EVENT_CLICKED, LV_EVENT_VALUE_CHANGED, LV_OBJ_FLAG_CHECKABLE,
        LV_OBJ_FLAG_IGNORE_LAYOUT, Obj, Screen, Slider, WidgetError,
        anim_path_ease_out, anim_path_overshoot, anim_path_linear,
    },
};
use lvgl_rust_sys::*;

const OBJ_WIDTH: i32 = 90;
const OBJ_HEIGHT: i32 = 70;

struct AnimTimeline1 {
    _timeline: Box<AnimTimeline>,
    _btn_start: Button<'static>,
    _btn_pause: Button<'static>,
    _slider: Slider<'static>,
    _obj1: Obj<'static>,
    _obj2: Obj<'static>,
    _obj3: Obj<'static>,
    _label_start: Label<'static>,
    _label_pause: Label<'static>,
}

unsafe extern "C" fn set_width(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_obj_set_width((*a).var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn set_height(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_obj_set_height((*a).var as *mut lv_obj_t, v) };
}

unsafe extern "C" fn set_slider_value(a: *mut lv_anim_t, v: i32) {
    unsafe { lv_slider_set_value((*a).var as *mut lv_obj_t, v, false) };
}

unsafe extern "C" fn btn_start_event_handler(e: *mut lv_event_t) {
    unsafe {
        let btn = lv_event_get_current_target_obj(e);
        let timeline = lv_event_get_user_data(e) as *mut lv_anim_timeline_t;
        let reverse = lv_obj_has_state(btn, lv_state_t_LV_STATE_CHECKED);
        lv_anim_timeline_set_reverse(timeline, reverse);
        lv_anim_timeline_start(timeline);
    }
}

unsafe extern "C" fn btn_pause_event_handler(e: *mut lv_event_t) {
    unsafe {
        let timeline = lv_event_get_user_data(e) as *mut lv_anim_timeline_t;
        lv_anim_timeline_pause(timeline);
    }
}

unsafe extern "C" fn slider_prg_event_handler(e: *mut lv_event_t) {
    unsafe {
        let slider = lv_event_get_current_target_obj(e);
        let timeline = lv_event_get_user_data(e) as *mut lv_anim_timeline_t;
        let progress = lv_slider_get_value(slider);
        lv_anim_timeline_set_progress(timeline, progress as u16);
    }
}

impl View for AnimTimeline1 {
    fn create() -> Result<Self, WidgetError> {
        let screen = Screen::active().ok_or(WidgetError::LvglNullPointer)?;

        let mut timeline = Box::new(AnimTimeline::new());
        let tl_ptr = timeline.handle() as *mut c_void;

        // Setup flex on screen
        // Screen doesn't have set_flex_flow — use raw calls
        unsafe {
            lv_obj_set_flex_flow(screen.handle(), 0x00); // LV_FLEX_FLOW_ROW
            lv_obj_set_flex_align(screen.handle(), 4, 2, 2); // SPACE_AROUND, CENTER, CENTER
        }

        // Start button
        let btn_start = Button::new(&screen)?;
        btn_start.add_flag(LV_OBJ_FLAG_IGNORE_LAYOUT);
        btn_start.add_flag(LV_OBJ_FLAG_CHECKABLE);
        btn_start.align(Align::TopMid, -100, 20);
        btn_start.on_event(btn_start_event_handler, LV_EVENT_VALUE_CHANGED, tl_ptr);
        let label_start = Label::new(&btn_start)?;
        label_start.text("Start\0")?.center();

        // Pause button
        let btn_pause = Button::new(&screen)?;
        btn_pause.add_flag(LV_OBJ_FLAG_IGNORE_LAYOUT);
        btn_pause.align(Align::TopMid, 100, 20);
        btn_pause.on_event(btn_pause_event_handler, LV_EVENT_CLICKED, tl_ptr);
        let label_pause = Label::new(&btn_pause)?;
        label_pause.text("Pause\0")?.center();

        // Progress slider
        let slider = Slider::new(&screen)?;
        slider.add_flag(LV_OBJ_FLAG_IGNORE_LAYOUT);
        slider.align(Align::BottomMid, 0, -20);
        slider.set_range(0, ANIM_TIMELINE_PROGRESS_MAX as i32);
        slider.on_event(slider_prg_event_handler, LV_EVENT_VALUE_CHANGED, tl_ptr);

        // 3 objects
        let obj1 = Obj::new(&screen)?;
        obj1.size(OBJ_WIDTH, OBJ_HEIGHT);
        obj1.set_scrollbar_mode(lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF);

        let obj2 = Obj::new(&screen)?;
        obj2.size(OBJ_WIDTH, OBJ_HEIGHT);
        obj2.set_scrollbar_mode(lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF);

        let obj3 = Obj::new(&screen)?;
        obj3.size(OBJ_WIDTH, OBJ_HEIGHT);
        obj3.set_scrollbar_mode(lv_scrollbar_mode_t_LV_SCROLLBAR_MODE_OFF);

        // Animations — slider progress
        let mut a_slider = Anim::new();
        a_slider.set_var(&slider)
            .set_values(0, ANIM_TIMELINE_PROGRESS_MAX as i32)
            .set_custom_exec_cb(Some(set_slider_value))
            .set_path_cb(Some(anim_path_linear))
            .set_duration(700);

        // obj1 width + height
        let mut a1 = Anim::new();
        a1.set_var(&obj1).set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a2 = Anim::new();
        a2.set_var(&obj1).set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj2
        let mut a3 = Anim::new();
        a3.set_var(&obj2).set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a4 = Anim::new();
        a4.set_var(&obj2).set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // obj3
        let mut a5 = Anim::new();
        a5.set_var(&obj3).set_values(0, OBJ_WIDTH)
            .set_custom_exec_cb(Some(set_width))
            .set_path_cb(Some(anim_path_overshoot))
            .set_duration(300);

        let mut a6 = Anim::new();
        a6.set_var(&obj3).set_values(0, OBJ_HEIGHT)
            .set_custom_exec_cb(Some(set_height))
            .set_path_cb(Some(anim_path_ease_out))
            .set_duration(300);

        // Add to timeline
        timeline.add(0, &a_slider);
        timeline.add(0, &a1);
        timeline.add(0, &a2);
        timeline.add(200, &a3);
        timeline.add(200, &a4);
        timeline.add(400, &a5);
        timeline.add(400, &a6);

        timeline.set_progress(ANIM_TIMELINE_PROGRESS_MAX);

        Ok(Self {
            _timeline: timeline,
            _btn_start: btn_start,
            _btn_pause: btn_pause,
            _slider: slider,
            _obj1: obj1,
            _obj2: obj2,
            _obj3: obj3,
            _label_start: label_start,
            _label_pause: label_pause,
        })
    }

    fn update(&mut self) -> Result<(), WidgetError> {
        Ok(())
    }
}

oxivgl_examples_common::example_main!(AnimTimeline1);
```

- [ ] **Step 2: Test**

Run: `./run_host.sh anim_timeline1`
Expected: 3 objects + buttons + slider visible

- [ ] **Step 3: Commit**

```bash
git add examples/anim_timeline1.rs
git commit -m "feat: anim_timeline1 example — animation timeline"
```

---

### Task 13: Capture screenshots + final verification

- [ ] **Step 1: Run all new examples**

```bash
./run_host.sh anim1
./run_host.sh anim2
./run_host.sh anim_timeline1
```

- [ ] **Step 2: Verify no regressions**

Run: `./run_screenshots.sh`
Expected: all existing + new screenshots captured

- [ ] **Step 3: Final commit with screenshots**

```bash
git add examples/doc/screenshots/
git commit -m "feat: animation example screenshots"
```
