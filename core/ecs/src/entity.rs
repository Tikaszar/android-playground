//! Entity contracts for the ECS

use serde::{Serialize, Deserialize};

/// Generation counter for entity IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generation(u32);

impl Generation {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn increment(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Entity identifier with generation for safe references
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    index: u32,
    generation: Generation,
}

impl EntityId {
    pub fn new(index: u32, generation: Generation) -> Self {
        Self { index, generation }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn generation(&self) -> Generation {
        self.generation
    }

    pub fn null() -> Self {
        Self {
            index: u32::MAX,
            generation: Generation(0),
        }
    }

    pub fn is_null(&self) -> bool {
        self.index == u32::MAX
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({}/{})", self.index, self.generation.0)
    }
}