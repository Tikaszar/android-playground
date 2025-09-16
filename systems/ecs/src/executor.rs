//! Command executor for ECS operations

use bytes::Bytes;
use serde::{Serialize, Deserialize};
use playground_core_ecs::{
    VTableCommand, VTableResponse,
    EntityId, Component, ComponentId,
    Query
};
use playground_core_types::{Handle, CoreResult};

use crate::implementation::EcsImplementation;

/// Operations that can be performed on the ECS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcsOperation {
    SpawnEntity,
    DespawnEntity { entity: EntityId },
    AddComponent { entity: EntityId, component: Component },
    RemoveComponent { entity: EntityId, component_id: ComponentId },
    GetComponent { entity: EntityId, component_id: ComponentId },
    Query { query: Query },
    Update { delta_time: f32 },
    GetStats,
}

/// Response from ECS operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcsOperationResponse {
    EntitySpawned(EntityId),
    EntityDespawned,
    ComponentAdded,
    ComponentRemoved,
    Component(Component),
    QueryResult(Vec<EntityId>),
    Updated,
    Stats { entities: u32, components: u32 },
}

/// Executes ECS commands
pub struct EcsExecutor {
    implementation: Handle<EcsImplementation>,
}

impl EcsExecutor {
    pub fn new(implementation: Handle<EcsImplementation>) -> Self {
        Self { implementation }
    }
    
    pub async fn handle_command(&mut self, cmd: VTableCommand) -> CoreResult<()> {
        // Deserialize the operation
        let operation: EcsOperation = match bincode::deserialize(&cmd.payload) {
            Ok(op) => op,
            Err(e) => {
                let response = VTableResponse {
                    success: false,
                    payload: None,
                    error: Some(format!("Failed to deserialize operation: {}", e)),
                };
                let _ = cmd.response.send(response).await;
                return Ok(());
            }
        };
        
        // Execute the operation
        let result = self.execute_operation(operation).await;
        
        // Send response
        let response = match result {
            Ok(op_response) => {
                let payload = bincode::serialize(&op_response)
                    .ok()
                    .map(|data| Bytes::from(data));
                VTableResponse {
                    success: true,
                    payload,
                    error: None,
                }
            }
            Err(e) => VTableResponse {
                success: false,
                payload: None,
                error: Some(e.to_string()),
            }
        };
        
        let _ = cmd.response.send(response).await;
        Ok(())
    }
    
    async fn execute_operation(&self, op: EcsOperation) -> CoreResult<EcsOperationResponse> {
        match op {
            EcsOperation::SpawnEntity => {
                let entity = self.implementation.spawn_entity().await?;
                Ok(EcsOperationResponse::EntitySpawned(entity))
            }
            EcsOperation::DespawnEntity { entity } => {
                self.implementation.despawn_entity(entity).await?;
                Ok(EcsOperationResponse::EntityDespawned)
            }
            EcsOperation::AddComponent { entity, component } => {
                self.implementation.add_component(entity, component).await?;
                Ok(EcsOperationResponse::ComponentAdded)
            }
            EcsOperation::RemoveComponent { entity, component_id } => {
                self.implementation.remove_component(entity, component_id).await?;
                Ok(EcsOperationResponse::ComponentRemoved)
            }
            EcsOperation::GetComponent { entity, component_id } => {
                let component = self.implementation.get_component(entity, component_id).await?;
                Ok(EcsOperationResponse::Component(component))
            }
            EcsOperation::Query { query } => {
                let result = self.implementation.query(query).await?;
                Ok(EcsOperationResponse::QueryResult(result.entities().to_vec()))
            }
            EcsOperation::Update { delta_time } => {
                self.implementation.update(delta_time).await?;
                Ok(EcsOperationResponse::Updated)
            }
            EcsOperation::GetStats => {
                let (entities, components) = self.implementation.stats();
                Ok(EcsOperationResponse::Stats { entities, components })
            }
        }
    }
}