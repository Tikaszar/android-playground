//! Component management ViewModel functions

mod add_component;
mod add_components;
mod remove_component;
mod remove_components;
mod get_component;
mod get_components;
mod has_component;
mod has_components;
mod get_all_components;
mod clear_components;
mod count_components;
mod replace_component;
mod get_entities_with_component;
mod get_entities_with_components;

pub use add_component::add_component;
pub use add_components::add_components;
pub use remove_component::remove_component;
pub use remove_components::remove_components;
pub use get_component::get_component;
pub use get_components::get_components;
pub use has_component::has_component;
pub use has_components::has_components;
pub use get_all_components::get_all_components;
pub use clear_components::clear_components;
pub use count_components::count_components;
pub use replace_component::replace_component;
pub use get_entities_with_component::get_entities_with_component;
pub use get_entities_with_components::get_entities_with_components;