use crate::error::LogicResult;
use crate::system::{Stage, SystemExecutor};
use crate::world::World;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// System scheduler with parallel execution support
pub struct Scheduler {
    executor: Arc<SystemExecutor>,
    metrics: Arc<RwLock<SchedulerMetrics>>,
    paused: Arc<RwLock<bool>>,
    time_budget: Duration,
}

#[derive(Default)]
pub struct SchedulerMetrics {
    pub frame_count: u64,
    pub total_time: Duration,
    pub stage_times: fnv::FnvHashMap<Stage, Duration>,
    pub system_times: fnv::FnvHashMap<String, Duration>,
    pub slowest_frame: Duration,
    pub fastest_frame: Duration,
}


impl Scheduler {
    pub fn new() -> Self {
        Self {
            executor: Arc::new(SystemExecutor::new()),
            metrics: Arc::new(RwLock::new(SchedulerMetrics::default())),
            paused: Arc::new(RwLock::new(false)),
            time_budget: Duration::from_millis(16), // 60 FPS target
        }
    }
    
    pub fn with_time_budget(mut self, budget: Duration) -> Self {
        self.time_budget = budget;
        self
    }
    
    pub fn executor(&self) -> &SystemExecutor {
        &self.executor
    }
    
    pub fn pause(&self) {
        *self.paused.write() = true;
    }
    
    pub fn resume(&self) {
        *self.paused.write() = false;
    }
    
    pub fn is_paused(&self) -> bool {
        *self.paused.read()
    }
    
    /// Run one frame of systems
    pub async fn run_frame(&self, world: &World, delta_time: f32) -> LogicResult<()> {
        if self.is_paused() {
            return Ok(());
        }
        
        let frame_start = Instant::now();
        let mut metrics = self.metrics.write();
        
        // Run stages in order
        let stages = [
            Stage::PreUpdate,
            Stage::Update,
            Stage::PostUpdate,
            Stage::PreRender,
            Stage::Render,
            Stage::PostRender,
        ];
        
        for stage in stages {
            let stage_start = Instant::now();
            
            // Check time budget
            if frame_start.elapsed() > self.time_budget {
                // Skip rendering stages if over budget
                if stage == Stage::PreRender || stage == Stage::Render || stage == Stage::PostRender {
                    continue;
                }
            }
            
            self.executor.run_stage(stage, world, delta_time).await?;
            
            let stage_time = stage_start.elapsed();
            *metrics.stage_times.entry(stage).or_insert(Duration::ZERO) += stage_time;
        }
        
        // Update metrics
        let frame_time = frame_start.elapsed();
        metrics.frame_count += 1;
        metrics.total_time += frame_time;
        
        if metrics.frame_count == 1 {
            metrics.slowest_frame = frame_time;
            metrics.fastest_frame = frame_time;
        } else {
            metrics.slowest_frame = metrics.slowest_frame.max(frame_time);
            metrics.fastest_frame = metrics.fastest_frame.min(frame_time);
        }
        
        Ok(())
    }
    
    /// Get current metrics
    pub fn metrics(&self) -> SchedulerMetrics {
        self.metrics.read().clone()
    }
    
    /// Reset metrics
    pub fn reset_metrics(&self) {
        *self.metrics.write() = SchedulerMetrics::default();
    }
}

impl Clone for SchedulerMetrics {
    fn clone(&self) -> Self {
        Self {
            frame_count: self.frame_count,
            total_time: self.total_time,
            stage_times: self.stage_times.clone(),
            system_times: self.system_times.clone(),
            slowest_frame: self.slowest_frame,
            fastest_frame: self.fastest_frame,
        }
    }
}

/// Fixed timestep scheduler for deterministic simulation
pub struct FixedScheduler {
    base: Scheduler,
    fixed_timestep: Duration,
    accumulator: Arc<RwLock<Duration>>,
    max_steps_per_frame: usize,
}

impl FixedScheduler {
    pub fn new(timestep: Duration) -> Self {
        Self {
            base: Scheduler::new(),
            fixed_timestep: timestep,
            accumulator: Arc::new(RwLock::new(Duration::ZERO)),
            max_steps_per_frame: 5,
        }
    }
    
    pub async fn run_frame(&self, world: &World, frame_time: Duration) -> LogicResult<()> {
        let mut accumulator = self.accumulator.write();
        *accumulator += frame_time;
        
        let mut steps = 0;
        while *accumulator >= self.fixed_timestep && steps < self.max_steps_per_frame {
            let delta = self.fixed_timestep.as_secs_f32();
            self.base.run_frame(world, delta).await?;
            
            *accumulator -= self.fixed_timestep;
            steps += 1;
        }
        
        Ok(())
    }
}

/// Adaptive scheduler that adjusts to maintain target framerate
pub struct AdaptiveScheduler {
    base: Scheduler,
    target_fps: f32,
    time_scale: Arc<RwLock<f32>>,
    auto_adjust: bool,
}

impl AdaptiveScheduler {
    pub fn new(target_fps: f32) -> Self {
        Self {
            base: Scheduler::new(),
            target_fps,
            time_scale: Arc::new(RwLock::new(1.0)),
            auto_adjust: true,
        }
    }
    
    pub async fn run_frame(&self, world: &World, delta_time: f32) -> LogicResult<()> {
        let scaled_delta = delta_time * *self.time_scale.read();
        
        let frame_start = Instant::now();
        self.base.run_frame(world, scaled_delta).await?;
        let frame_time = frame_start.elapsed();
        
        // Auto-adjust time scale to maintain target FPS
        if self.auto_adjust {
            let target_frame_time = Duration::from_secs_f32(1.0 / self.target_fps);
            if frame_time > target_frame_time * 2 {
                // Running too slow, reduce time scale
                let mut scale = self.time_scale.write();
                *scale = (*scale * 0.95).max(0.1);
            } else if frame_time < target_frame_time / 2 {
                // Running too fast, increase time scale
                let mut scale = self.time_scale.write();
                *scale = (*scale * 1.05).min(2.0);
            }
        }
        
        Ok(())
    }
    
    pub fn set_time_scale(&self, scale: f32) {
        *self.time_scale.write() = scale.clamp(0.1, 10.0);
    }
    
    pub fn time_scale(&self) -> f32 {
        *self.time_scale.read()
    }
}