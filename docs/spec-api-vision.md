# API Vision

Design principles for oxivgl's public API.

---

## 1. Core Promise

oxivgl provides **safe, `no_std` Rust bindings for LVGL** that feel
native to Rust while staying close to LVGL's mental model. Consumer code
and examples never touch `unsafe` or raw LVGL bindings — the library
absorbs all unsafety internally and leverages the Rust compiler to
enforce soundness at the type level.

---

## 2. Safety Over Ergonomics

When safety or soundness and convenience conflict, safety wins. Specifically:

- APIs that store pointers in LVGL require `'static` at the type level
  — no runtime checks, no leaking allocations.
- Widget lifetimes are expressed through Rust's ownership and borrow
  system, not through reference counting or GC.
- Style ownership uses a two-phase model (mutable builder → frozen
  shared handle) to prevent mutation after LVGL has taken a pointer.

See `docs/spec-memory-lifetime.md` for the full invariant catalog.

---

## 3. Thin Wrappers, Not Abstractions

Each widget wrapper maps directly to one LVGL widget type. Methods
correspond to LVGL API calls — same names where possible, same argument
semantics, same defaults. The goal is that someone reading the LVGL docs
can predict what the Rust API looks like.

Do not invent abstractions that hide LVGL's structure. Helpers and
builders are welcome when they reduce error-prone boilerplate (e.g.
`ScaleBuilder` for 13-argument constructors), but they must not obscure
which LVGL calls happen underneath.

---

## 4. No `std` Required

The library must compile on `no_std` + `alloc` targets. This means:

- No `std::` types in public APIs.
- Use `alloc::vec::Vec`, `alloc::boxed::Box`, `alloc::rc::Rc`.
- All examples must compile for both host (`x86_64`, `std`) and
  embedded (`xtensa`, `no_std`).

---

## 5. One Canonical Path

Every public type lives in exactly one module and is imported from there.
No cross-module re-exports that create multiple paths to the same type.
`prelude` is the only exception — it re-exports everything for
convenience globs.

---

## 6. Extend, Don't Work Around

When an example or consumer needs LVGL functionality not yet wrapped:

1. Add the wrapper to the core library with proper safety invariants.
2. Then use it from the example.

Never add `unsafe` to user code as a shortcut. Never call `lvgl_rust_sys`
from outside the library. If the `View` trait lacks a needed hook,
extend the trait.

---

## 7. Minimal Public Surface

Only expose what is used. Do not speculatively wrap LVGL APIs "just in
case" — wrap them when an example or consumer needs them. This keeps the
API surface small, reviewed, and tested.

---

## 8. Consistency

- Widget methods return `&Self` for chaining.
- `new()` returns `Result<Self, WidgetError>`.
- Setter methods take the same argument types as the underlying LVGL
  function, translated to Rust idioms (enums for constants, `bool` for
  enable/disable, `i32` for coordinates).
- Style properties are set through `StyleBuilder` (build phase) or
  `Obj::style_*` / `Obj::add_style` (runtime).
- Names follow Rust conventions (`set_value`, not `setValue` or
  `lv_slider_set_value`), but stay recognizable from the LVGL docs.

---

## 9. Example-Driven Development

The API grows by porting LVGL examples — see
`docs/spec-example-porting.md` for the full process and rationale.
