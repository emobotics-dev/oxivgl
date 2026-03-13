// SPDX-License-Identifier: MIT OR Apache-2.0
//! Grid cell placement helper.

/// Grid cell placement (alignment + position + span).
/// Used with [`Obj::set_grid_cell`](super::Obj::set_grid_cell) to avoid
/// positional argument confusion.
///
/// ```ignore
/// use oxivgl::widgets::{GridAlign, GridCell};
/// obj.set_grid_cell(
///     GridCell::new(GridAlign::Stretch, 0, 1),
///     GridCell::new(GridAlign::Center, 0, 1),
/// );
/// ```
#[derive(Clone, Copy, Debug)]
pub struct GridCell {
    pub align: super::obj::GridAlign,
    pub pos: i32,
    pub span: i32,
}

impl GridCell {
    /// Create a grid cell placement.
    pub fn new(align: super::obj::GridAlign, pos: i32, span: i32) -> Self {
        Self { align, pos, span }
    }

    /// Single-cell at given position with Start alignment and span 1.
    pub fn at(pos: i32) -> Self {
        Self {
            align: super::obj::GridAlign::Start,
            pos,
            span: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::obj::GridAlign;

    #[test]
    fn at_defaults_to_start_span1() {
        let c = GridCell::at(2);
        assert_eq!(c.pos, 2);
        assert_eq!(c.span, 1);
        // GridAlign doesn't impl PartialEq, so check via debug
        assert!(format!("{:?}", c.align).contains("Start"));
    }

    #[test]
    fn new_preserves_fields() {
        let c = GridCell::new(GridAlign::Center, 3, 2);
        assert_eq!(c.pos, 3);
        assert_eq!(c.span, 2);
    }
}
