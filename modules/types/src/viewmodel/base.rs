//! ViewModel trait for implementations (System modules only)
//! ViewModels implement View API contracts

use super::super::view::base::ViewId;

/// ViewModel trait for View API implementations
///
/// System modules implement ViewModels that:
/// - Implement View API contracts
/// - Contain business logic
/// - Handle I/O operations
/// - NO data storage (use Model from Core)
///
/// ViewModels are bound to Views via matching ViewId
#[async_trait::async_trait]
pub trait ViewModelTrait: Send + Sync {
    /// Get the ViewId this ViewModel implements
    fn view_id(&self) -> ViewId;
}
