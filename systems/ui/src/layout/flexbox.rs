//! Flexbox layout implementation

use crate::layout::{
    Direction, JustifyContent, AlignItems, LayoutConstraints, 
    LayoutResult, LayoutEngine, LayoutChild
};
use nalgebra::Vector2;

/// Flexbox layout engine
#[derive(Debug, Clone)]
pub struct FlexLayout {
    pub direction: Direction,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub gap: f32,
    pub wrap: bool,
}

impl Default for FlexLayout {
    fn default() -> Self {
        Self {
            direction: Direction::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            gap: 0.0,
            wrap: false,
        }
    }
}

impl FlexLayout {
    pub fn column() -> Self {
        Self {
            direction: Direction::Column,
            ..Default::default()
        }
    }
    
    pub fn row() -> Self {
        Self::default()
    }
    
    pub fn with_justify(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }
    
    pub fn with_align(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }
    
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

impl LayoutEngine for FlexLayout {
    fn calculate(
        &self,
        constraints: &LayoutConstraints,
        children: &[LayoutChild],
    ) -> Vec<LayoutResult> {
        if children.is_empty() {
            return Vec::new();
        }
        
        let is_row = matches!(self.direction, Direction::Row | Direction::RowReverse);
        let is_reverse = matches!(self.direction, Direction::RowReverse | Direction::ColumnReverse);
        
        let main_size = if is_row { constraints.available_size.x } else { constraints.available_size.y };
        let cross_size = if is_row { constraints.available_size.y } else { constraints.available_size.x };
        
        // Calculate flex basis and growth
        let mut total_flex_grow = 0.0;
        let mut total_fixed_size = 0.0;
        
        for child in children {
            if child.flex_grow > 0.0 {
                total_flex_grow += child.flex_grow;
            } else if let Some(pref_size) = child.preferred_size {
                total_fixed_size += if is_row { pref_size.x } else { pref_size.y };
            }
        }
        
        // Add gaps
        total_fixed_size += self.gap * (children.len() - 1) as f32;
        
        // Calculate available space for flex items
        let flex_space = (main_size - total_fixed_size).max(0.0);
        let flex_unit = if total_flex_grow > 0.0 {
            flex_space / total_flex_grow
        } else {
            0.0
        };
        
        // Calculate positions and sizes
        let mut results = Vec::with_capacity(children.len());
        let mut main_pos = 0.0;
        
        // Handle justify-content spacing
        let (start_offset, between_spacing) = match self.justify_content {
            JustifyContent::Start => (0.0, 0.0),
            JustifyContent::End => (main_size - total_fixed_size, 0.0),
            JustifyContent::Center => ((main_size - total_fixed_size) / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                if children.len() > 1 {
                    (0.0, flex_space / (children.len() - 1) as f32)
                } else {
                    (0.0, 0.0)
                }
            }
            JustifyContent::SpaceAround => {
                let spacing = flex_space / children.len() as f32;
                (spacing / 2.0, spacing)
            }
            JustifyContent::SpaceEvenly => {
                let spacing = flex_space / (children.len() + 1) as f32;
                (spacing, spacing)
            }
        };
        
        main_pos += start_offset;
        
        for (i, child) in children.iter().enumerate() {
            // Calculate size
            let child_main_size = if child.flex_grow > 0.0 {
                flex_unit * child.flex_grow
            } else if let Some(pref_size) = child.preferred_size {
                if is_row { pref_size.x } else { pref_size.y }
            } else {
                0.0
            };
            
            let child_cross_size = match self.align_items {
                AlignItems::Stretch => cross_size,
                _ => {
                    if let Some(pref_size) = child.preferred_size {
                        if is_row { pref_size.y } else { pref_size.x }
                    } else {
                        cross_size
                    }
                }
            };
            
            // Calculate cross position based on alignment
            let cross_pos = match self.align_items {
                AlignItems::Start => 0.0,
                AlignItems::End => cross_size - child_cross_size,
                AlignItems::Center => (cross_size - child_cross_size) / 2.0,
                AlignItems::Stretch | AlignItems::Baseline => 0.0,
            };
            
            // Create result
            let position = if is_row {
                Vector2::new(main_pos, cross_pos)
            } else {
                Vector2::new(cross_pos, main_pos)
            };
            
            let size = if is_row {
                Vector2::new(child_main_size, child_cross_size)
            } else {
                Vector2::new(child_cross_size, child_main_size)
            };
            
            results.push(LayoutResult::new(size, position));
            
            // Advance position
            main_pos += child_main_size + self.gap + between_spacing;
        }
        
        // Reverse if needed
        if is_reverse {
            results.reverse();
        }
        
        results
    }
}