//! Layout system types

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};

/// Layout constraints passed to elements
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LayoutConstraints {
    /// Minimum size
    pub min_size: Vector2<f32>,
    /// Maximum size
    pub max_size: Vector2<f32>,
    /// Available space
    pub available_size: Vector2<f32>,
}

impl LayoutConstraints {
    pub fn new(available_size: Vector2<f32>) -> Self {
        Self {
            min_size: Vector2::zeros(),
            max_size: available_size,
            available_size,
        }
    }
    
    pub fn with_min(mut self, min_size: Vector2<f32>) -> Self {
        self.min_size = min_size;
        self
    }
    
    pub fn with_max(mut self, max_size: Vector2<f32>) -> Self {
        self.max_size = max_size;
        self
    }
}

/// Result of layout calculation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LayoutResult {
    /// Calculated size
    pub size: Vector2<f32>,
    /// Position relative to parent
    pub position: Vector2<f32>,
}

impl LayoutResult {
    pub fn new(size: Vector2<f32>, position: Vector2<f32>) -> Self {
        Self { size, position }
    }
}

/// Layout engine trait
pub trait LayoutEngine: Send + Sync {
    /// Calculate layout for children
    fn calculate(
        &self,
        constraints: &LayoutConstraints,
        children: &[LayoutChild],
    ) -> Vec<LayoutResult>;
}

/// Information about a child for layout
#[derive(Debug, Clone)]
pub struct LayoutChild {
    /// Preferred size
    pub preferred_size: Option<Vector2<f32>>,
    /// Flex grow factor
    pub flex_grow: f32,
    /// Flex shrink factor
    pub flex_shrink: f32,
    /// Flex basis
    pub flex_basis: Option<f32>,
    /// Margin
    pub margin: Margin,
    /// Padding
    pub padding: Padding,
}

impl Default for LayoutChild {
    fn default() -> Self {
        Self {
            preferred_size: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
            margin: Margin::default(),
            padding: Padding::default(),
        }
    }
}

/// Margin values
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
    
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

impl Default for Margin {
    fn default() -> Self {
        Self::all(0.0)
    }
}

/// Padding values
pub type Padding = Margin;

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// Justify content alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Align items alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

/// Align content alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    Stretch,
}

/// Position type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    Relative,
    Absolute,
    Fixed,
}