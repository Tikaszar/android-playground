//! Registration with core/ecs VTable
//! 
//! This module handles registering all ECS operation handlers with the World's VTable.

use playground_core_ecs::CoreResult;

/// Register the ECS system with the VTable
/// 
/// This registers all ECS operation handlers so that core/ecs can
/// delegate operations to systems/ecs through the VTable.
pub async fn register() -> CoreResult<()> {
    // Register all VTable handlers
    crate::vtable_handlers::register_handlers().await?;
    
    Ok(())
}