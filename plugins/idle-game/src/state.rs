use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct IdleGameState {
    pub currency: u64,
    pub generators: Vec<Generator>,
    pub multiplier: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Generator {
    pub name: String,
    pub count: u32,
    pub base_cost: u64,
    pub production: f64,
}