pub mod theme;
pub mod manager;

pub use theme::{Theme, ThemeColors, ThemeTypography};
pub use manager::ThemeManager;

/// Theme identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeId(pub u32);