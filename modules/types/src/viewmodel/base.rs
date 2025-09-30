//! ViewModel base class for implementations (System modules only)
//! Using concrete base class instead of trait (NO dyn allowed!)

/// ViewModel base class for View API implementations
///
/// System modules implement ViewModels that:
/// - Implement View API contracts
/// - Contain business logic
/// - Handle I/O operations
/// - NO data storage (use Model from Core)
///
/// This is a concrete base class, not a trait!
pub struct ViewModel {
    /// Which Core module View this implements
    pub implements_view: &'static str,

    /// ViewModel name
    pub viewmodel_name: &'static str,

    /// Which features this ViewModel provides
    pub provides_features: Vec<&'static str>,
}