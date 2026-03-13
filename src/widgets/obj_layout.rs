// SPDX-License-Identifier: MIT OR Apache-2.0
//! Layout methods (flex, grid) for [`Obj`]. These are `impl` blocks on the
//! same type defined in `obj.rs` — no new types introduced.

use core::ptr::null_mut;

use lvgl_rust_sys::*;

use super::obj::{FlexAlign, FlexFlow, GridAlign, Obj};

impl<'p> Obj<'p> {
    pub fn set_flex_flow(&self, flow: FlexFlow) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_flex_flow(self.handle(), flow as lv_flex_flow_t) };
        self
    }

    pub fn set_flex_align(&self, main: FlexAlign, cross: FlexAlign, track: FlexAlign) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe {
            lv_obj_set_flex_align(
                self.handle(),
                main as lv_flex_align_t,
                cross as lv_flex_align_t,
                track as lv_flex_align_t,
            )
        };
        self
    }

    pub fn set_flex_grow(&self, grow: u8) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_flex_grow(self.handle(), grow) };
        self
    }

    pub fn set_layout(&self, layout: super::Layout) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe { lv_obj_set_layout(self.handle(), layout as u32) };
        self
    }

    /// Set grid column/row descriptors and enable grid layout.
    /// The slices must be `'static` — LVGL stores the pointers internally.
    pub fn set_grid_dsc_array(&self, col_dsc: &'static [i32], row_dsc: &'static [i32]) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above); slices are 'static.
        unsafe { lv_obj_set_grid_dsc_array(self.handle(), col_dsc.as_ptr(), row_dsc.as_ptr()) };
        self
    }

    pub fn set_grid_cell(&self, col: super::grid::GridCell, row: super::grid::GridCell) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe {
            lv_obj_set_grid_cell(
                self.handle(),
                col.align as lv_grid_align_t,
                col.pos,
                col.span,
                row.align as lv_grid_align_t,
                row.pos,
                row.span,
            )
        };
        self
    }

    pub fn set_grid_align(&self, col_align: GridAlign, row_align: GridAlign) -> &Self {
        assert_ne!(self.handle(), null_mut());
        // SAFETY: handle non-null (asserted above).
        unsafe {
            lv_obj_set_grid_align(
                self.handle(),
                col_align as lv_grid_align_t,
                row_align as lv_grid_align_t,
            )
        };
        self
    }
}
