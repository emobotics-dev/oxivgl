// SPDX-License-Identifier: MIT OR Apache-2.0
//! Safe wrapper around LVGL events (`lv_event_t`).

use lvgl_rust_sys::*;

use super::{
    obj::{AsLvHandle, Obj},
    EventCode,
};

/// Safe wrapper around an LVGL event (`lv_event_t`).
///
/// Passed to [`View::on_event`](crate::view::View::on_event) and
/// [`Obj::on`] callbacks — valid only for the duration of the callback.
pub struct Event {
    raw: *mut lv_event_t,
}

impl Event {
    pub(crate) fn from_raw(raw: *mut lv_event_t) -> Self {
        Self { raw }
    }

    /// Event code (e.g. `EventCode::CLICKED`).
    pub fn code(&self) -> EventCode {
        // SAFETY: raw pointer valid for callback duration.
        EventCode(unsafe { lv_event_get_code(self.raw) })
    }

    /// Raw handle of the widget that originally received the event.
    pub fn target_handle(&self) -> *mut lv_obj_t {
        // SAFETY: raw pointer valid for callback duration.
        unsafe { lv_event_get_target_obj(self.raw) }
    }

    /// Non-owning reference to the event target widget.
    /// The returned `Obj` does NOT own the LVGL object — do not store it.
    pub fn target(&self) -> super::Child<Obj<'_>> {
        super::Child::new(Obj::from_raw(self.target_handle()))
    }

    /// Raw handle of the widget whose event handler is currently running
    /// (differs from target when events bubble).
    pub fn current_target_handle(&self) -> *mut lv_obj_t {
        // SAFETY: raw pointer valid for callback duration.
        unsafe { lv_event_get_current_target_obj(self.raw) }
    }

    /// Check if this event matches a specific widget and event code.
    ///
    /// ```ignore
    /// fn on_event(&mut self, event: &Event) {
    ///     if event.matches(&self.btn, EventCode::CLICKED) {
    ///         // handle click
    ///     }
    /// }
    /// ```
    pub fn matches(&self, widget: &impl AsLvHandle, code: EventCode) -> bool {
        self.code() == code && self.target_handle() == widget.lv_handle()
    }

    /// Set a style property on the event target. Convenience for event handlers
    /// that need to modify the originating widget (e.g. event bubbling).
    pub fn target_style_bg_color(&self, color: lv_color_t, selector: u32) {
        // SAFETY: target_handle() returns a valid LVGL object for callback duration.
        unsafe { lv_obj_set_style_bg_color(self.target_handle(), color, selector) };
    }
}
