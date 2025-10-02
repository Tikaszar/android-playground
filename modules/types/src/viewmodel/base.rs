//! ViewModel trait for implementations (System modules only)
//! ViewModels implement View API contracts

use crate::view::base::{ViewId, FragmentId};

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

/// ViewModel Fragment trait for pieces of a ViewModel
///
/// ViewModels are composed of fragments matching View fragments.
/// Each fragment implements the logic for its corresponding View fragment.
#[async_trait::async_trait]
pub trait ViewModelFragmentTrait: Send + Sync {
    /// Get the ViewId this fragment implements
    fn view_id(&self) -> ViewId;

    /// Get the FragmentId this fragment implements
    fn fragment_id(&self) -> FragmentId;
}
