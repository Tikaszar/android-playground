//! View - API contracts EXPORTS ONLY
//!
//! This module defines the View layer of the MVVM architecture.
//! These are the API contracts that systems/ecs will implement.

pub mod entity;
pub mod component;
pub mod event;
pub mod query;
pub mod storage;
pub mod system;
pub mod world;