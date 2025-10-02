//! View trait for API contracts (Core modules only)
//! Views define API contracts with no implementation

/// 64-bit unique identifier for a View
pub type ViewId = u64;

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
