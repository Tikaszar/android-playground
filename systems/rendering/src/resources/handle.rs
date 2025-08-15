use std::marker::PhantomData;
use std::fmt::{Debug, Display};
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Handle<T> {
    id: u64,
    generation: u32,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(id: u64, generation: u32) -> Self {
        Self {
            id,
            generation,
            _phantom: PhantomData,
        }
    }
    
    pub fn invalid() -> Self {
        Self {
            id: 0,
            generation: 0,
            _phantom: PhantomData,
        }
    }
    
    pub fn id(&self) -> u64 {
        self.id
    }
    
    pub fn generation(&self) -> u32 {
        self.generation
    }
    
    pub fn is_valid(&self) -> bool {
        self.id != 0
    }
}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({}.{})", self.id, self.generation)
    }
}

impl<T> Display for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.id, self.generation)
    }
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::invalid()
    }
}