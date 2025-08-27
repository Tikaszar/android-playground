use crate::system::UiSystem;
use crate::error::{UiError, UiResult};
use crate::components::*;
use playground_core_ecs::{Component, EntityId};
use playground_core_rendering::Viewport;

impl UiSystem {
    pub async fn initialize(&mut self) -> UiResult<()> {
        self.log("Info", format!("[UiSystem] initialize() called, initialized={}", self.initialized)).await;
        
        if self.initialized {
            return Err(UiError::AlreadyInitialized);
        }
        
        self.log("Info", "[UiSystem] Registering components...".to_string()).await;
        self.register_components().await?;
        self.log("Info", "[UiSystem] Components registered".to_string()).await;
        
        self.log("Info", "[UiSystem] Loading default themes...".to_string()).await;
        let mut theme_mgr = self.theme_manager.write().await;
        theme_mgr.load_default_themes()?;
        drop(theme_mgr);
        self.log("Info", "[UiSystem] Themes loaded".to_string()).await;
        
        self.log("Info", "[UiSystem] Creating root element...".to_string()).await;
        let root_entity = self.create_root().await?;
        self.log("Info", format!("[UiSystem] Root element created: {:?}", root_entity)).await;
        self.root_entity = Some(root_entity);
        self.log("Info", format!("[UiSystem] root_entity set to: {:?}", self.root_entity)).await;
        
        self.log("Info", "[UiSystem] Initializing mobile features...".to_string()).await;
        let mut mobile = self.mobile_features.write().await;
        mobile.initialize().await?;
        drop(mobile);
        self.log("Info", "[UiSystem] Mobile features initialized".to_string()).await;
        
        self.initialized = true;
        self.log("Info", format!("[UiSystem] Initialization complete, initialized={}, root_entity={:?}", 
                     self.initialized, self.root_entity)).await;
        Ok(())
    }
    
    pub async fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
        self.screen_size = [viewport.width as f32, viewport.height as f32];
        
        if let Some(root) = self.root_entity {
            self.mark_subtree_dirty(root).await.ok();
        }
    }
    
    pub(super) async fn register_components(&self) -> UiResult<()> {
        self.registry.register::<UiElementComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiLayoutComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiStyleComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiInputComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.registry.register::<UiTextComponent>().await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiStyleComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiInputComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Dense
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.world.register_component_storage(
            <UiTextComponent as playground_core_ecs::ComponentData>::component_id(),
            playground_core_ecs::StorageType::Sparse
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        Ok(())
    }
    
    pub(super) async fn create_root(&self) -> UiResult<EntityId> {
        let entities = self.world.spawn_batch(vec![vec![]]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let entity = entities.into_iter().next()
            .ok_or_else(|| UiError::CreationFailed("Failed to create root entity".into()))?;
        
        let mut root_element = UiElementComponent::new("root");
        root_element.visible = true;
        let component = Component::new(root_element).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity, 
            Box::new(component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let mut root_layout = UiLayoutComponent::default();
        root_layout.bounds = ElementBounds {
            x: 0.0,
            y: 0.0,
            width: self.screen_size[0],
            height: self.screen_size[1],
        };
        root_layout.layout_type = LayoutType::Absolute;
        let component = Component::new(root_layout).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(component),
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let mut root_style = UiStyleComponent::default();
        root_style.visible = true;
        let style_component = Component::new(root_style).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(style_component),
            <UiStyleComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let input_component = Component::new(UiInputComponent::default()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(input_component),
            <UiInputComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        Ok(entity)
    }
}