//! Absolute positioning layout

use crate::layout::{LayoutConstraints, LayoutResult, LayoutEngine, LayoutChild};
use nalgebra::Vector2;

/// Absolute positioning layout
#[derive(Debug, Clone, Default)]
pub struct AbsoluteLayout;

impl LayoutEngine for AbsoluteLayout {
    fn calculate(
        &self,
        constraints: &LayoutConstraints,
        children: &[LayoutChild],
    ) -> Vec<LayoutResult> {
        children.iter().map(|child| {
            // For absolute layout, use preferred size or available size
            let size = child.preferred_size.unwrap_or(constraints.available_size);
            let position = Vector2::zeros(); // Position will be set by element itself
            LayoutResult::new(size, position)
        }).collect()
    }
}