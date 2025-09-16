//! VTable registration for console system

use tokio::sync::mpsc;
use playground_core_ecs::{VTableCommand, VTableResponse};
use playground_core_types::{CoreResult};
use crate::vtable_handlers;

/// Register the console system with the World's VTable
pub async fn register() -> CoreResult<()> {
    let world = playground_core_ecs::get_world().await?;
    
    // Create channels for console capabilities
    let capabilities = vec![
        ("console.output", 100),
        ("console.logging", 100),
        ("console.progress", 100),
        ("console.input", 100),
        ("console.registry", 10),
    ];
    
    for (capability, buffer_size) in capabilities {
        let (tx, mut rx) = mpsc::channel::<VTableCommand>(buffer_size);
        
        // Register with VTable
        world.vtable.register(capability.to_string(), tx).await?;
        
        // Spawn handler task for this capability
        let capability_name = capability.to_string();
        tokio::spawn(async move {
            while let Some(cmd) = rx.recv().await {
                let response = match capability_name.as_str() {
                    "console.output" => vtable_handlers::handle_output_command(cmd.operation, cmd.payload).await,
                    "console.logging" => vtable_handlers::handle_logging_command(cmd.operation, cmd.payload).await,
                    "console.progress" => vtable_handlers::handle_progress_command(cmd.operation, cmd.payload).await,
                    "console.input" => vtable_handlers::handle_input_command(cmd.operation, cmd.payload).await,
                    "console.registry" => vtable_handlers::handle_registry_command(cmd.operation, cmd.payload).await,
                    _ => VTableResponse {
                        success: false,
                        payload: None,
                        error: Some(format!("Unknown capability: {}", capability_name)),
                    },
                };
                
                let _ = cmd.response.send(response).await;
            }
        });
    }
    
    // Initialize the console implementation
    vtable_handlers::initialize().await?;
    
    Ok(())
}