use crate::system::UiSystem;
use crate::error::{UiError, UiResult};
use crate::element::ElementId;
use crate::components::*;
use crate::input::InputEvent;
use crate::theme::ThemeId;
use playground_core_ecs::{Component, EntityId};
use uuid::Uuid;

impl UiSystem {
    pub async fn handle_input(&mut self, event: InputEvent) -> UiResult<bool> {
        let mut input_mgr = self.input_manager.write().await;
        let handled = input_mgr.process_event(event, &self.element_graph, &self.world).await?;
        
        if handled {
            if let Some(focused) = input_mgr.get_focused_element() {
                self.dirty_elements.write().await.push(focused);
            }
        }
        
        Ok(handled)
    }
    
    pub async fn create_element(
        &mut self,
        element_type: &str,
        parent: Option<EntityId>,
    ) -> UiResult<EntityId> {
        self.log("Info", format!("[UiSystem] create_element called: type={}, parent={:?}", element_type, parent)).await;
        
        let entities = self.world.spawn_batch(vec![vec![]]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let entity = entities.into_iter().next()
            .ok_or_else(|| UiError::CreationFailed("Failed to create element".into()))?;
        
        let element_component = Component::new(UiElementComponent::new(element_type)).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(element_component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let layout_component = Component::new(UiLayoutComponent::default()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(layout_component),
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let style_component = Component::new(UiStyleComponent::default()).await
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
        
        if element_type == "text" || element_type == "input" {
            let text_component = Component::new(UiTextComponent::new(String::new())).await
                .map_err(|e| UiError::EcsError(e.to_string()))?;
            self.world.add_component_raw(
                entity,
                Box::new(text_component),
                <UiTextComponent as playground_core_ecs::ComponentData>::component_id()
            ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        }
        
        self.log("Info", format!("[UiSystem] Entity created: {:?}", entity)).await;
        
        if let Some(parent) = parent {
            self.log("Info", format!("[UiSystem] Adding entity {:?} as child of {:?}", entity, parent)).await;
            let mut graph = self.element_graph.write().await;
            graph.add_child(parent, entity)?;
            drop(graph);
            self.log("Info", "[UiSystem] Added to element graph".to_string()).await;
        }
        
        self.log("Info", format!("[UiSystem] Marking entity {:?} as dirty", entity)).await;
        self.dirty_elements.write().await.push(entity);
        
        self.log("Info", format!("[UiSystem] create_element complete, returning entity: {:?}", entity)).await;
        Ok(entity)
    }
    
    pub async fn remove_element(&mut self, element: EntityId) -> UiResult<()> {
        let mut graph = self.element_graph.write().await;
        graph.remove_element(element);
        drop(graph);
        
        self.world.despawn_batch(vec![element]).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        Ok(())
    }
    
    pub async fn set_element_text(&mut self, element: EntityId, text: String) -> UiResult<()> {
        let current_component = self.world.get_component::<UiElementComponent>(element).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let mut updated_component = current_component.clone();
        updated_component.text_content = Some(text);
        
        self.world.remove_component_raw(element, <UiElementComponent as playground_core_ecs::ComponentData>::component_id()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let component = Component::new(updated_component).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            element,
            Box::new(component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.dirty_elements.write().await.push(element);
        
        Ok(())
    }
    
    pub async fn create_terminal(&mut self, parent: EntityId) -> UiResult<Uuid> {
        let terminal_id = Uuid::new_v4();
        let entity = self.create_element("terminal", Some(parent)).await?;
        
        self.terminal_connections.write().await.insert(terminal_id, entity);
        
        let mut term_mgr = self.terminal_manager.write().await;
        term_mgr.create_terminal(terminal_id).await?;
        
        Ok(terminal_id)
    }
    
    pub async fn set_theme(&mut self, theme_id: ThemeId) -> UiResult<()> {
        let theme_mgr = self.theme_manager.read().await;
        if !theme_mgr.has_theme(theme_id) {
            return Err(UiError::ThemeNotFound(theme_id.to_string()));
        }
        drop(theme_mgr);
        
        self.current_theme = theme_id;
        
        if let Some(root) = self.root_entity {
            self.mark_subtree_dirty(root).await?;
        }
        
        Ok(())
    }
    
    pub async fn set_element_style(&mut self, element: ElementId, style: crate::types::ElementStyle) -> UiResult<()> {
        self.log("Info", format!("[UiSystem] set_element_style called for element: {:?}", element)).await;
        
        let ui_style = UiStyleComponent {
            background_color: Some(nalgebra::Vector4::new(
                style.background_color[0],
                style.background_color[1],
                style.background_color[2],
                style.background_color[3],
            )),
            text_color: Some(nalgebra::Vector4::new(
                style.text_color[0],
                style.text_color[1],
                style.text_color[2],
                style.text_color[3],
            )),
            border_color: Some(nalgebra::Vector4::new(
                style.border_color[0],
                style.border_color[1],
                style.border_color[2],
                style.border_color[3],
            )),
            border_width: style.border_width,
            border_radius: style.border_radius,
            opacity: style.opacity,
            font_size: style.font_size,
            font_family: Some(style.font_family),
            font_weight: match style.font_weight {
                crate::types::FontWeight::Light => crate::components::FontWeight::Light,
                crate::types::FontWeight::Normal => crate::components::FontWeight::Normal,
                crate::types::FontWeight::Bold => crate::components::FontWeight::Bold,
                crate::types::FontWeight::ExtraBold => crate::components::FontWeight::ExtraBold,
            },
            text_align: match style.text_align {
                crate::types::TextAlign::Left => crate::components::TextAlign::Left,
                crate::types::TextAlign::Center => crate::components::TextAlign::Center,
                crate::types::TextAlign::Right => crate::components::TextAlign::Right,
                crate::types::TextAlign::Justify => crate::components::TextAlign::Justify,
            },
            visible: style.visible,
            z_index: style.z_index,
            cursor: None,
            custom_styles: std::collections::HashMap::new(),
        };
        
        self.log("Info", "[UiSystem] Updating style component...".to_string()).await;
        
        let component_id = <UiStyleComponent as playground_core_ecs::ComponentData>::component_id();
        
        let has_component = self.world.has_component(element, component_id.clone()).await;
        self.log("Info", format!("[UiSystem] Entity has style component: {}", has_component)).await;
        
        if has_component {
            self.log("Info", format!("[UiSystem] Removing existing component for entity {:?}", element)).await;
            let _ = self.world.remove_component_raw(element, component_id.clone()).await;
            self.log("Info", "[UiSystem] Existing component removed".to_string()).await;
        }
        
        self.log("Info", format!("[UiSystem] Adding new component for entity {:?}", element)).await;
        let style_component = Component::new(ui_style).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            element,
            Box::new(style_component),
            component_id
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.log("Info", "[UiSystem] Style component updated".to_string()).await;
        
        self.log("Info", format!("[UiSystem] Style component updated for element: {:?}", element)).await;
        
        self.dirty_elements.write().await.push(element);
        
        self.log("Info", format!("[UiSystem] set_element_style complete for element: {:?}", element)).await;
        Ok(())
    }
    
    pub async fn set_element_bounds(&mut self, element: ElementId, bounds: crate::types::ElementBounds) -> UiResult<()> {
        let layout = UiLayoutComponent {
            bounds: crate::components::ElementBounds {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
            },
            padding: [0.0; 4],
            margin: [0.0; 4],
            layout_type: crate::components::LayoutType::Absolute,
            flex_direction: crate::components::FlexDirection::Row,
            justify_content: crate::components::JustifyContent::FlexStart,
            align_items: crate::components::AlignItems::FlexStart,
            position_type: crate::components::PositionType::Absolute,
            size: crate::components::Size {
                width: Some(bounds.width),
                height: Some(bounds.height),
            },
            min_size: crate::components::Size { width: None, height: None },
            max_size: crate::components::Size { width: None, height: None },
        };
        
        let _ = self.world.remove_component_raw(element, <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()).await;
        
        let component = Component::new(layout).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            element,
            Box::new(component),
            <UiLayoutComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.dirty_elements.write().await.push(element);
        
        Ok(())
    }
    
    pub async fn create_element_with_id(
        &mut self,
        id: String,
        element_type: String,
        parent: Option<ElementId>,
    ) -> UiResult<ElementId> {
        self.log("Info", format!("[UiSystem] create_element_with_id called: id={}, type={}, parent={:?}", 
                     id, element_type, parent)).await;
        
        let entity = self.create_element(&element_type, parent).await?;
        self.log("Info", format!("[UiSystem] Element created with entity: {:?}", entity)).await;
        
        self.log("Info", "[UiSystem] Updating component with id...".to_string()).await;
        
        let elem_box = self.world.get_component_raw(entity, <UiElementComponent as playground_core_ecs::ComponentData>::component_id()).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        
        let bytes = elem_box.serialize();
        let mut elem = <UiElementComponent as playground_core_ecs::ComponentData>::deserialize(&bytes).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        elem.id = id.clone();
        
        let _ = self.world.remove_component_raw(entity, <UiElementComponent as playground_core_ecs::ComponentData>::component_id()).await;
        let component = Component::new(elem).await
            .map_err(|e| UiError::EcsError(e.to_string()))?;
        self.world.add_component_raw(
            entity,
            Box::new(component),
            <UiElementComponent as playground_core_ecs::ComponentData>::component_id()
        ).await.map_err(|e| UiError::EcsError(e.to_string()))?;
        
        self.log("Info", format!("[UiSystem] Component updated with id={}", id)).await;
        Ok(entity)
    }
}