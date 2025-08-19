//! Physics System
//! 
//! This crate provides physics simulation functionality for the playground system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhysicsError {
    #[error("Physics initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Invalid physics body: {0}")]
    InvalidBody(String),
    #[error("Simulation error: {0}")]
    SimulationError(String),
}

pub type PhysicsResult<T> = Result<T, PhysicsError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        } else {
            Self::zero()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsBody {
    pub id: String,
    pub position: Vector2,
    pub velocity: Vector2,
    pub acceleration: Vector2,
    pub mass: f32,
    pub is_static: bool,
}

impl PhysicsBody {
    pub fn new(id: String, position: Vector2) -> Self {
        Self {
            id,
            position,
            velocity: Vector2::zero(),
            acceleration: Vector2::zero(),
            mass: 1.0,
            is_static: false,
        }
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn as_static(mut self) -> Self {
        self.is_static = true;
        self
    }
}

/// Main physics system struct
pub struct PhysicsSystem {
    bodies: HashMap<String, PhysicsBody>,
    gravity: Vector2,
    time_step: f32,
    initialized: bool,
}

impl PhysicsSystem {
    /// Create a new physics system
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            gravity: Vector2::new(0.0, -9.81), // Default gravity
            time_step: 1.0 / 60.0, // 60 FPS
            initialized: false,
        }
    }

    /// Initialize the physics system
    pub fn initialize(&mut self) -> PhysicsResult<()> {
        if self.initialized {
            return Err(PhysicsError::InitializationFailed("Already initialized".to_string()));
        }
        
        self.initialized = true;
        Ok(())
    }

    /// Add a physics body to the simulation
    pub fn add_body(&mut self, body: PhysicsBody) -> PhysicsResult<()> {
        if self.bodies.contains_key(&body.id) {
            return Err(PhysicsError::InvalidBody(
                format!("Body with id '{}' already exists", body.id)
            ));
        }
        
        self.bodies.insert(body.id.clone(), body);
        Ok(())
    }

    /// Remove a physics body from the simulation
    pub fn remove_body(&mut self, body_id: &str) -> PhysicsResult<()> {
        if self.bodies.remove(body_id).is_none() {
            return Err(PhysicsError::InvalidBody(
                format!("Body with id '{}' not found", body_id)
            ));
        }
        Ok(())
    }

    /// Get a physics body by ID
    pub fn get_body(&self, body_id: &str) -> Option<&PhysicsBody> {
        self.bodies.get(body_id)
    }

    /// Get a mutable reference to a physics body by ID
    pub fn get_body_mut(&mut self, body_id: &str) -> Option<&mut PhysicsBody> {
        self.bodies.get_mut(body_id)
    }

    /// Set the gravity for the simulation
    pub fn set_gravity(&mut self, gravity: Vector2) {
        self.gravity = gravity;
    }

    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) -> PhysicsResult<()> {
        if !self.initialized {
            return Err(PhysicsError::InitializationFailed("Physics system not initialized".to_string()));
        }

        let dt = if delta_time > 0.0 { delta_time } else { self.time_step };

        for body in self.bodies.values_mut() {
            if body.is_static {
                continue;
            }

            // Apply gravity
            body.acceleration.x += self.gravity.x;
            body.acceleration.y += self.gravity.y;

            // Update velocity
            body.velocity.x += body.acceleration.x * dt;
            body.velocity.y += body.acceleration.y * dt;

            // Update position
            body.position.x += body.velocity.x * dt;
            body.position.y += body.velocity.y * dt;

            // Reset acceleration
            body.acceleration = Vector2::zero();
        }

        Ok(())
    }

    /// Apply a force to a physics body
    pub fn apply_force(&mut self, body_id: &str, force: Vector2) -> PhysicsResult<()> {
        if let Some(body) = self.bodies.get_mut(body_id) {
            if !body.is_static && body.mass > 0.0 {
                body.acceleration.x += force.x / body.mass;
                body.acceleration.y += force.y / body.mass;
            }
        } else {
            return Err(PhysicsError::InvalidBody(
                format!("Body with id '{}' not found", body_id)
            ));
        }
        Ok(())
    }

    /// Get all physics bodies
    pub fn get_all_bodies(&self) -> Vec<&PhysicsBody> {
        self.bodies.values().collect()
    }
}

impl Default for PhysicsSystem {
    fn default() -> Self {
        Self::new()
    }
}