use serde::{Deserialize, Serialize};

pub trait Stateful {
    type State: Serialize + for<'de> Deserialize<'de>;
    
    fn save_state(&self) -> Self::State;
    
    fn restore_state(&mut self, state: Self::State);
}