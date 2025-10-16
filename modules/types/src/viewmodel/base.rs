//! ViewModel trait for implementations (System modules only)
//! ViewModels implement View API contracts

use crate::view::base::ViewId;
use crate::error::ModuleError;

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

    fn api_version(&self) -> u32;

    async fn save_state(&self) -> Option<Result<Vec<u8>, ModuleError>> {
        None
    }

    async fn restore_state(&self, _state: Vec<u8>) -> Option<Result<(), ModuleError>> {
        None
    }
}
