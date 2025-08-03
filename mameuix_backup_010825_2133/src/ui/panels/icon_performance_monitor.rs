// src/ui/panels/icon_performance_monitor.rs
// Performance monitoring for icon loading system

use std::time::{Instant, Duration};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct IconLoadMetrics {
    pub load_time: Duration,
    pub icon_size: u32,
    pub success: bool,
    pub timestamp: Instant,
}

pub struct IconPerformanceMonitor {
    // Performance tracking
    pub load_times: VecDeque<IconLoadMetrics>,
    pub total_icons_loaded: usize,
    pub total_load_time: Duration,
    pub failed_loads: usize,
    
    // Thread pool metrics
    pub concurrent_loads: usize,
    pub thread_pool_utilization: f32,
    
    // Performance windows
    pub window_size: usize,
    pub last_reset: Instant,
    
    // Performance thresholds
    pub slow_load_threshold: Duration,
    pub target_load_time: Duration,
}

impl IconPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            load_times: VecDeque::new(),
            total_icons_loaded: 0,
            total_load_time: Duration::ZERO,
            failed_loads: 0,
            concurrent_loads: 0,
            thread_pool_utilization: 0.0,
            window_size: 100, // Track last 100 loads
            last_reset: Instant::now(),
            slow_load_threshold: Duration::from_millis(100),
            target_load_time: Duration::from_millis(50),
        }
    }
    
    /// Record an icon load completion
    pub fn record_load(&mut self, load_time: Duration, icon_size: u32, success: bool) {
        let metrics = IconLoadMetrics {
            load_time,
            icon_size,
            success,
            timestamp: Instant::now(),
        };
        
        self.load_times.push_back(metrics);
        self.total_icons_loaded += 1;
        self.total_load_time += load_time;
        
        if !success {
            self.failed_loads += 1;
        }
        
        // Maintain window size
        while self.load_times.len() > self.window_size {
            if let Some(old_metrics) = self.load_times.pop_front() {
                self.total_load_time -= old_metrics.load_time;
            }
        }
    }
    
    /// Update thread pool metrics
    pub fn update_thread_pool_metrics(&mut self, concurrent_loads: usize, utilization: f32) {
        self.concurrent_loads = concurrent_loads;
        self.thread_pool_utilization = utilization;
    }
    
    /// Get average load time for the current window
    pub fn get_average_load_time(&self) -> Duration {
        if self.load_times.is_empty() {
            return Duration::ZERO;
        }
        
        self.total_load_time / self.load_times.len() as u32
    }
    
    /// Get load time percentiles
    pub fn get_load_time_percentiles(&self) -> (Duration, Duration, Duration) {
        if self.load_times.is_empty() {
            return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
        }
        
        let mut times: Vec<Duration> = self.load_times
            .iter()
            .map(|m| m.load_time)
            .collect();
        times.sort();
        
        let len = times.len();
        let p50 = times[len / 2];
        let p90 = times[(len * 9) / 10];
        let p99 = times[(len * 99) / 100];
        
        (p50, p90, p99)
    }
    
    /// Get success rate
    pub fn get_success_rate(&self) -> f32 {
        if self.total_icons_loaded == 0 {
            return 0.0;
        }
        
        let successful = self.total_icons_loaded - self.failed_loads;
        (successful as f32) / (self.total_icons_loaded as f32) * 100.0
    }
    
    /// Get icons per second
    pub fn get_icons_per_second(&self) -> f32 {
        if self.load_times.is_empty() {
            return 0.0;
        }
        
        let window_duration = self.load_times.back().unwrap().timestamp
            .duration_since(self.load_times.front().unwrap().timestamp);
        
        if window_duration.is_zero() {
            return 0.0;
        }
        
        (self.load_times.len() as f32) / window_duration.as_secs_f32()
    }
    
    /// Check if performance is meeting targets
    pub fn is_performance_acceptable(&self) -> bool {
        let avg_time = self.get_average_load_time();
        avg_time <= self.target_load_time
    }
    
    /// Get performance summary
    pub fn get_performance_summary(&self) -> String {
        let avg_time = self.get_average_load_time();
        let (p50, p90, p99) = self.get_load_time_percentiles();
        let success_rate = self.get_success_rate();
        let icons_per_sec = self.get_icons_per_second();
        
        format!(
            "Avg: {:.1}ms, P50: {:.1}ms, P90: {:.1}ms, P99: {:.1}ms\n\
             Success: {:.1}%, Rate: {:.1} icons/sec\n\
             Threads: {}, Utilization: {:.1}%",
            avg_time.as_micros() as f32 / 1000.0,
            p50.as_micros() as f32 / 1000.0,
            p90.as_micros() as f32 / 1000.0,
            p99.as_micros() as f32 / 1000.0,
            success_rate,
            icons_per_sec,
            self.concurrent_loads,
            self.thread_pool_utilization * 100.0
        )
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        self.load_times.clear();
        self.total_icons_loaded = 0;
        self.total_load_time = Duration::ZERO;
        self.failed_loads = 0;
        self.last_reset = Instant::now();
    }
    
    /// Get metrics for the last N loads
    pub fn get_recent_metrics(&self, count: usize) -> Vec<&IconLoadMetrics> {
        self.load_times
            .iter()
            .rev()
            .take(count)
            .collect()
    }
} 