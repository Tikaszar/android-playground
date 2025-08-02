//! Logic System
//! 
//! This crate provides game logic and business rules functionality for the playground system.

use playground_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LogicError {
    #[error("Logic initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Invalid game state: {0}")]
    InvalidGameState(String),
    #[error("Rule execution failed: {0}")]
    RuleExecutionFailed(String),
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
}

pub type LogicResult<T> = Result<T, LogicError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntity {
    pub id: String,
    pub entity_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub active: bool,
}

impl GameEntity {
    pub fn new(id: String, entity_type: String) -> Self {
        Self {
            id,
            entity_type,
            properties: HashMap::new(),
            active: true,
        }
    }

    pub fn with_property(mut self, key: String, value: serde_json::Value) -> Self {
        self.properties.insert(key, value);
        self
    }

    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }
}

#[derive(Debug, Clone)]
pub struct GameRule {
    pub id: String,
    pub name: String,
    pub condition: String, // TODO: Replace with actual condition type
    pub action: String,    // TODO: Replace with actual action type
    pub enabled: bool,
}

impl GameRule {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            condition: String::new(),
            action: String::new(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub current_level: u32,
    pub score: u64,
    pub player_data: HashMap<String, serde_json::Value>,
    pub game_time: f64,
    pub paused: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            current_level: 1,
            score: 0,
            player_data: HashMap::new(),
            game_time: 0.0,
            paused: false,
        }
    }
}

/// Main logic system struct
pub struct LogicSystem {
    entities: HashMap<String, GameEntity>,
    rules: HashMap<String, GameRule>,
    game_state: GameState,
    initialized: bool,
}

impl LogicSystem {
    /// Create a new logic system
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            rules: HashMap::new(),
            game_state: GameState::new(),
            initialized: false,
        }
    }

    /// Initialize the logic system
    pub fn initialize(&mut self) -> LogicResult<()> {
        if self.initialized {
            return Err(LogicError::InitializationFailed("Already initialized".to_string()));
        }
        
        self.initialized = true;
        Ok(())
    }

    /// Add a game entity
    pub fn add_entity(&mut self, entity: GameEntity) -> LogicResult<()> {
        if self.entities.contains_key(&entity.id) {
            return Err(LogicError::InvalidGameState(
                format!("Entity with id '{}' already exists", entity.id)
            ));
        }
        
        self.entities.insert(entity.id.clone(), entity);
        Ok(())
    }

    /// Remove a game entity
    pub fn remove_entity(&mut self, entity_id: &str) -> LogicResult<()> {
        if self.entities.remove(entity_id).is_none() {
            return Err(LogicError::EntityNotFound(entity_id.to_string()));
        }
        Ok(())
    }

    /// Get a game entity
    pub fn get_entity(&self, entity_id: &str) -> Option<&GameEntity> {
        self.entities.get(entity_id)
    }

    /// Get a mutable reference to a game entity
    pub fn get_entity_mut(&mut self, entity_id: &str) -> Option<&mut GameEntity> {
        self.entities.get_mut(entity_id)
    }

    /// Add a game rule
    pub fn add_rule(&mut self, rule: GameRule) -> LogicResult<()> {
        self.rules.insert(rule.id.clone(), rule);
        Ok(())
    }

    /// Remove a game rule
    pub fn remove_rule(&mut self, rule_id: &str) -> LogicResult<()> {
        self.rules.remove(rule_id);
        Ok(())
    }

    /// Enable or disable a rule
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) -> LogicResult<()> {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.enabled = enabled;
            Ok(())
        } else {
            Err(LogicError::RuleExecutionFailed(
                format!("Rule '{}' not found", rule_id)
            ))
        }
    }

    /// Update the logic system
    pub fn update(&mut self, delta_time: f32) -> LogicResult<()> {
        if !self.initialized {
            return Err(LogicError::InitializationFailed("Logic system not initialized".to_string()));
        }

        // Update game time
        if !self.game_state.paused {
            self.game_state.game_time += delta_time as f64;
        }

        // Execute game rules
        self.execute_rules()?;

        // Update entities
        self.update_entities(delta_time)?;

        Ok(())
    }

    /// Execute all enabled game rules
    fn execute_rules(&mut self) -> LogicResult<()> {
        // TODO: Implement actual rule execution logic
        for (_rule_id, rule) in &self.rules {
            if rule.enabled {
                // Rule execution would go here
            }
        }
        Ok(())
    }

    /// Update all active entities
    fn update_entities(&mut self, _delta_time: f32) -> LogicResult<()> {
        // TODO: Implement entity update logic
        for (_entity_id, entity) in &mut self.entities {
            if entity.active {
                // Entity update logic would go here
            }
        }
        Ok(())
    }

    /// Get the current game state
    pub fn get_game_state(&self) -> &GameState {
        &self.game_state
    }

    /// Get a mutable reference to the game state
    pub fn get_game_state_mut(&mut self) -> &mut GameState {
        &mut self.game_state
    }

    /// Pause or unpause the game
    pub fn set_paused(&mut self, paused: bool) {
        self.game_state.paused = paused;
    }

    /// Reset the game state
    pub fn reset_game(&mut self) -> LogicResult<()> {
        self.game_state = GameState::new();
        self.entities.clear();
        Ok(())
    }

    /// Get all entities
    pub fn get_all_entities(&self) -> Vec<&GameEntity> {
        self.entities.values().collect()
    }

    /// Get all rules
    pub fn get_all_rules(&self) -> Vec<&GameRule> {
        self.rules.values().collect()
    }
}

impl Default for LogicSystem {
    fn default() -> Self {
        Self::new()
    }
}