//! Component metadata

use crate::model::component::ComponentId;

/// Component metadata - describes a component type
///
/// The actual component data is stored in ComponentPool<T> in Systems.
/// This struct just provides metadata about the component type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Component {
    /// The component type ID
    pub component_id: ComponentId,

    /// The component type name (for debugging)
    pub component_name: String,

    /// Size hint for allocation
    pub size_hint: usize,
}

impl Component {
    /// Create new component metadata
    pub fn new(component_id: ComponentId, component_name: String, size_hint: usize) -> Self {
        Self {
            component_id,
            component_name,
            size_hint,
        }
    }

    /// Create component metadata from a type
    pub fn from_type<T: 'static>() -> Self {
        Self {
            component_id: ComponentId::from_type_name::<T>(),
            component_name: std::any::type_name::<T>().to_string(),
            size_hint: std::mem::size_of::<T>(),
        }
    }

    /// Check if this component is of a specific type
    pub fn is_type<T: 'static>(&self) -> bool {
        self.component_id == ComponentId::from_type_name::<T>()
    }

    /// Get the component type name
    pub fn type_name(&self) -> &str {
        &self.component_name
    }

    /// Get the size hint
    pub fn size_hint(&self) -> usize {
        self.size_hint
    }
}