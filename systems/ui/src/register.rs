/// Register UI system with core/ecs
/// This is called at startup to register the system type
pub async fn register() -> Result<(), playground_core_ecs::EcsError> {
    playground_core_ecs::register_ui_system("ui".to_string()).await
}