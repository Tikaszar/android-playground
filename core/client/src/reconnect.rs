use gloo_timers::future::TimeoutFuture;
use web_sys::console;

#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    pub initial_delay_ms: u32,
    pub max_delay_ms: u32,
    pub multiplier: f32,
    pub max_attempts: Option<u32>,
    pub jitter: bool,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
            multiplier: 1.5,
            max_attempts: None,
            jitter: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReconnectState {
    Connected,
    Disconnected,
    Reconnecting { attempt: u32, next_delay_ms: u32 },
    Failed { reason: String },
}

pub struct ReconnectManager {
    config: ReconnectConfig,
    state: ReconnectState,
    current_attempt: u32,
    current_delay_ms: u32,
}

impl ReconnectManager {
    pub fn new(config: ReconnectConfig) -> Self {
        Self {
            current_delay_ms: config.initial_delay_ms,
            config,
            state: ReconnectState::Disconnected,
            current_attempt: 0,
        }
    }
    
    pub fn with_default() -> Self {
        Self::new(ReconnectConfig::default())
    }
    
    pub fn state(&self) -> &ReconnectState {
        &self.state
    }
    
    pub fn reset(&mut self) {
        self.current_attempt = 0;
        self.current_delay_ms = self.config.initial_delay_ms;
        self.state = ReconnectState::Connected;
    }
    
    pub fn on_disconnected(&mut self) {
        console::log_1(&"Connection lost, initiating reconnection".into());
        self.state = ReconnectState::Disconnected;
        self.current_attempt = 0;
        self.current_delay_ms = self.config.initial_delay_ms;
    }
    
    pub fn should_reconnect(&self) -> bool {
        match &self.state {
            ReconnectState::Disconnected | ReconnectState::Reconnecting { .. } => {
                if let Some(max) = self.config.max_attempts {
                    self.current_attempt < max
                } else {
                    true
                }
            }
            _ => false,
        }
    }
    
    pub async fn wait_before_reconnect(&mut self) -> Result<(), String> {
        if !self.should_reconnect() {
            return Err("Max reconnection attempts reached".to_string());
        }
        
        self.current_attempt += 1;
        
        let delay = if self.config.jitter {
            let jitter = (js_sys::Math::random() * 0.3 - 0.15) as f32;
            let jittered = self.current_delay_ms as f32 * (1.0 + jitter);
            jittered as u32
        } else {
            self.current_delay_ms
        };
        
        self.state = ReconnectState::Reconnecting {
            attempt: self.current_attempt,
            next_delay_ms: delay,
        };
        
        console::log_1(&format!(
            "Reconnection attempt {} in {}ms",
            self.current_attempt, delay
        ).into());
        
        TimeoutFuture::new(delay).await;
        
        self.current_delay_ms = (self.current_delay_ms as f32 * self.config.multiplier) as u32;
        if self.current_delay_ms > self.config.max_delay_ms {
            self.current_delay_ms = self.config.max_delay_ms;
        }
        
        Ok(())
    }
    
    pub fn on_reconnect_failed(&mut self, reason: String) {
        console::error_1(&format!("Reconnection attempt {} failed: {}", 
            self.current_attempt, reason).into());
        
        if let Some(max) = self.config.max_attempts {
            if self.current_attempt >= max {
                self.state = ReconnectState::Failed {
                    reason: format!("Max attempts ({}) reached. Last error: {}", max, reason),
                };
                console::error_1(&"Reconnection failed permanently".into());
            }
        }
    }
    
    pub fn on_connected(&mut self) {
        console::log_1(&format!(
            "Successfully reconnected after {} attempts",
            self.current_attempt
        ).into());
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exponential_backoff() {
        let config = ReconnectConfig {
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            multiplier: 2.0,
            max_attempts: None,
            jitter: false,
        };
        
        let mut manager = ReconnectManager::new(config);
        
        assert_eq!(manager.current_delay_ms, 1000);
        manager.current_delay_ms = (manager.current_delay_ms as f32 * manager.config.multiplier) as u32;
        assert_eq!(manager.current_delay_ms, 2000);
        manager.current_delay_ms = (manager.current_delay_ms as f32 * manager.config.multiplier) as u32;
        assert_eq!(manager.current_delay_ms, 4000);
        manager.current_delay_ms = (manager.current_delay_ms as f32 * manager.config.multiplier) as u32;
        assert_eq!(manager.current_delay_ms, 8000);
        manager.current_delay_ms = (manager.current_delay_ms as f32 * manager.config.multiplier) as u32;
        if manager.current_delay_ms > manager.config.max_delay_ms {
            manager.current_delay_ms = manager.config.max_delay_ms;
        }
        assert_eq!(manager.current_delay_ms, 10000);
    }
}