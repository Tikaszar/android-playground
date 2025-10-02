//! View trait for API contracts (Core modules only)
//! Views define API contracts with no implementation

/// 64-bit unique identifier for a View
pub type ViewId = u64;

/// 64-bit unique identifier for a View Fragment
pub type FragmentId = u64;

/// View trait for API contracts with no implementation
///
/// Core modules define Views that specify:
/// - API surface through trait methods
/// - Parameter types
/// - Return types
/// - NO implementation logic
///
/// Views are implemented by ViewModels in System modules
#[async_trait::async_trait]
pub trait ViewTrait: Send + Sync {
    /// Get the unique ID of this View
    fn view_id(&self) -> ViewId;
}

/// View Fragment trait for pieces of a View
///
/// Views are composed of fragments, each handling a logical domain.
/// Fragments share the View's ViewId but have their own FragmentId.
#[async_trait::async_trait]
pub trait ViewFragmentTrait: Send + Sync {
    /// Get the ViewId this fragment belongs to
    fn view_id(&self) -> ViewId;

    /// Get the unique FragmentId for this specific fragment
    fn fragment_id(&self) -> FragmentId;
}
