//! Audio state component

use crate::types::*;
use playground_core_ecs::impl_component_data;
use serde::{Deserialize, Serialize};

/// Audio state as an ECS component
#[cfg(feature = "audio")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStateComponent {
    pub master_volume: Float,
    pub music_volume: Float,
    pub sfx_volume: Float,
    pub voice_volume: Float,
    pub muted: bool,
}

#[cfg(feature = "audio")]
impl_component_data!(AudioStateComponent);

#[cfg(feature = "audio")]
impl AudioStateComponent {
    pub fn new() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,
            voice_volume: 1.0,
            muted: false,
        }
    }

    pub fn set_master_volume(&mut self, volume: Float) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_music_volume(&mut self, volume: Float) {
        self.music_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_sfx_volume(&mut self, volume: Float) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
    }

    pub fn get_effective_music_volume(&self) -> Float {
        if self.muted {
            0.0
        } else {
            self.master_volume * self.music_volume
        }
    }

    pub fn get_effective_sfx_volume(&self) -> Float {
        if self.muted {
            0.0
        } else {
            self.master_volume * self.sfx_volume
        }
    }
}

#[cfg(feature = "audio")]
impl Default for AudioStateComponent {
    fn default() -> Self {
        Self::new()
    }
}