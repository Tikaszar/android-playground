//! Visibility component - controls whether an entity is rendered

use serde::{Serialize, Deserialize};
use playground_core_ecs::impl_component_data;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Visibility {
    pub visible: bool,
    pub visible_in_hierarchy: bool,
}

impl Default for Visibility {
    fn default() -> Self {
        Self {
            visible: true,
            visible_in_hierarchy: true,
        }
    }
}

impl_component_data!(Visibility);