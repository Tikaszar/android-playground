//! View base class for API contracts (Core modules only)
//! Using concrete base class instead of trait (NO dyn allowed!)

/// View base class for API contracts with no implementation
///
/// Core modules define Views that specify:
/// - Function signatures
/// - Parameter types
/// - Return types
/// - NO implementation logic
///
/// This is a concrete base class, not a trait!
pub struct View {
    /// Unique identifier for this view
    pub view_id: &'static str,

    /// View name
    pub view_name: &'static str,

    /// List of API methods this view exposes
    pub methods: Vec<&'static str>,
}