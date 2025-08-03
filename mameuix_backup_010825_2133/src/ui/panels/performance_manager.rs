// src/ui/performance_manager.rs
// Performance monitoring and optimization module

use eframe::egui;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct PerformanceManager {
    // Core performance monitoring
    frame_times: VecDeque<Duration>,
    last_frame: Instant,
    slow_frame_threshold: Duration,
    pub frame_count: u64,
    total_time: Duration,
    
    // FPS calculation optimization
    last_fps_calculation: Instant,
    cached_fps: f32,
    
    // Performance settings
    enable_fps_limit: bool,
    target_fps: u32,
    enable_low_quality_mode: bool,
    
    // Debug information
    debug_info_visible: bool,
    performance_history: VecDeque<f32>, // Store FPS history for graphs
    max_history_size: usize,
}

impl PerformanceManager {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120), // Track 2 seconds at 60fps
            last_frame: Instant::now(),
            slow_frame_threshold: Duration::from_millis(33), // Target 30fps minimum
            frame_count: 0,
            total_time: Duration::ZERO,
            last_fps_calculation: Instant::now(),
            cached_fps: 60.0,
            enable_fps_limit: false,
            target_fps: 60,
            enable_low_quality_mode: false,
            debug_info_visible: false,
            performance_history: VecDeque::with_capacity(300), // 5 seconds at 60fps
            max_history_size: 300,
        }
    }

    /// Initialize with performance settings
    pub fn with_settings(mut self, enable_fps_limit: bool, target_fps: u32, enable_low_quality_mode: bool) -> Self {
        self.enable_fps_limit = enable_fps_limit;
        self.target_fps = target_fps;
        self.enable_low_quality_mode = enable_low_quality_mode;
        self
    }

    /// Call at frame start - tracks frame timing
    pub fn frame_start(&mut self) {
        let now = Instant::now();
        let frame_time = now - self.last_frame;
        self.last_frame = now;

        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }

        self.frame_count += 1;
        self.total_time += frame_time;
    }

    /// Get average FPS - cached for performance
    pub fn get_average_fps(&mut self) -> f32 {
        // Only recalculate FPS every second
        if self.last_fps_calculation.elapsed() >= Duration::from_secs(1) {
            if !self.frame_times.is_empty() {
                let avg_frame_time = self.frame_times.iter()
                    .sum::<Duration>() / self.frame_times.len() as u32;
                
                if avg_frame_time.as_secs_f32() > 0.0 {
                    self.cached_fps = 1.0 / avg_frame_time.as_secs_f32();
                } else {
                    self.cached_fps = 60.0; // Default to 60 FPS if calculation fails
                }
                
                // Store in history for graphs
                self.performance_history.push_back(self.cached_fps);
                if self.performance_history.len() > self.max_history_size {
                    self.performance_history.pop_front();
                }
                
                self.last_fps_calculation = Instant::now();
            }
        }
        
        self.cached_fps
    }

    /// Check if experiencing lag
    pub fn is_lagging(&self) -> bool {
        if let Some(last_frame) = self.frame_times.back() {
            *last_frame > self.slow_frame_threshold
        } else {
            false
        }
    }

    /// Get lag spike count
    pub fn get_lag_spike_count(&self) -> usize {
        self.frame_times.iter()
        .filter(|&&time| time > self.slow_frame_threshold)
        .count()
    }

    /// Get current frame time
    pub fn get_current_frame_time(&self) -> Duration {
        self.frame_times.back().copied().unwrap_or(Duration::ZERO)
    }

    /// Get average frame time
    pub fn get_average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }
        self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32
    }

    /// Check if FPS limiting should be applied
    pub fn should_limit_fps(&mut self) -> bool {
        self.enable_fps_limit && self.get_average_fps() > self.target_fps as f32
    }

    /// Get target frame time for FPS limiting
    pub fn get_target_frame_time(&self) -> Duration {
        if self.enable_fps_limit {
            Duration::from_secs_f32(1.0 / self.target_fps as f32)
        } else {
            Duration::ZERO
        }
    }

    /// Check if low quality mode should be enabled
    pub fn should_use_low_quality_mode(&mut self) -> bool {
        self.enable_low_quality_mode || self.get_average_fps() < 20.0
    }

    /// Get performance statistics
    pub fn get_stats(&mut self) -> PerformanceStats {
        let fps = self.get_average_fps();
        PerformanceStats {
            fps,
            frame_time: self.get_current_frame_time(),
            average_frame_time: self.get_average_frame_time(),
            lag_spikes: self.get_lag_spike_count(),
            is_lagging: self.is_lagging(),
            frame_count: self.frame_count,
            total_time: self.total_time,
        }
    }

    /// Toggle debug info visibility
    pub fn toggle_debug_info(&mut self) {
        self.debug_info_visible = !self.debug_info_visible;
    }

    /// Show debug information in UI
    pub fn show_debug_info(&mut self, ui: &mut egui::Ui) {
        if !self.debug_info_visible {
            return;
        }

        let fps = self.get_average_fps();
        let color = if fps < 20.0 {
            egui::Color32::RED
        } else if fps < 30.0 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::GREEN
        };

        ui.colored_label(color, format!("FPS: {:.1}", fps));

        if self.is_lagging() {
            ui.colored_label(egui::Color32::RED, "⚠ Lag detected");
        }

        let lag_spikes = self.get_lag_spike_count();
        if lag_spikes > 0 {
            ui.label(format!("Lag spikes: {}", lag_spikes));
        }

        // Show frame time
        let frame_time = self.get_current_frame_time();
        ui.label(format!("Frame time: {:.1}ms", frame_time.as_secs_f32() * 1000.0));

        // Show performance mode
        if self.should_use_low_quality_mode() {
            ui.colored_label(egui::Color32::YELLOW, "Low quality mode");
        }
    }

    /// Show performance graph
    pub fn show_performance_graph(&mut self, ui: &mut egui::Ui) {
        if !self.debug_info_visible || self.performance_history.is_empty() {
            return;
        }

        ui.label("Performance History:");
        
        // Show recent FPS values as text instead of plot
        let recent_fps: Vec<String> = self.performance_history
            .iter()
            .rev()
            .take(10)
            .map(|fps| format!("{:.1}", fps))
            .collect();
        
        ui.label(format!("Recent FPS: {}", recent_fps.join(", ")));
    }

    /// Reset monitor
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.frame_count = 0;
        self.total_time = Duration::ZERO;
        self.last_frame = Instant::now();
        self.performance_history.clear();
        self.cached_fps = 60.0;
    }

    /// Update performance settings
    pub fn update_settings(&mut self, enable_fps_limit: bool, target_fps: u32, enable_low_quality_mode: bool) {
        self.enable_fps_limit = enable_fps_limit;
        self.target_fps = target_fps;
        self.enable_low_quality_mode = enable_low_quality_mode;
    }

    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            frame_times_capacity: self.frame_times.capacity(),
            frame_times_len: self.frame_times.len(),
            history_capacity: self.performance_history.capacity(),
            history_len: self.performance_history.len(),
        }
    }
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub fps: f32,
    pub frame_time: Duration,
    pub average_frame_time: Duration,
    pub lag_spikes: usize,
    pub is_lagging: bool,
    pub frame_count: u64,
    pub total_time: Duration,
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub frame_times_capacity: usize,
    pub frame_times_len: usize,
    pub history_capacity: usize,
    pub history_len: usize,
} 