// src/ui/icon_manager.rs
// Icon management module - handles loading, caching, and displaying game icons
// THREAD POOL VERSION: Uses rayon thread pool for parallel icon loading

use eframe::egui;
use crate::models::{AppConfig, IconInfo};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, mpsc};
use rayon::prelude::*;
use std::path::PathBuf;
use super::icon_performance_monitor::IconPerformanceMonitor;

// Icon loading result from background thread
#[derive(Debug)]
pub struct IconLoadResult {
    rom_name: String,
    icon_data: Option<Vec<u8>>,
    width: u32,
    height: u32,
    load_time: Option<Duration>,
    success: bool,
}

pub struct IconManager {
    // Icon storage
    pub rom_icons: HashMap<String, egui::TextureHandle>,
    pub default_icon_texture: Option<egui::TextureHandle>,
    
    // Thread pool based loading
    pub icon_load_queue: Arc<Mutex<VecDeque<String>>>,
    pub icon_info: HashMap<String, IconInfo>,
    pub last_icon_cleanup: Instant,
    
    // Thread pool results
    pub icon_results_rx: Option<mpsc::Receiver<IconLoadResult>>,
    pub icon_results_tx: Option<mpsc::Sender<IconLoadResult>>,
    
    // Performance settings
    pub max_cached_icons: usize,
    pub icon_lifetime: u64, // seconds
    pub max_concurrent_loads: usize,
    
    // Thread pool state
    pub thread_pool_initialized: bool,
    
    // Performance monitoring
    pub performance_monitor: IconPerformanceMonitor,
}

impl IconManager {
    pub fn new(config: &AppConfig) -> Self {
        let (tx, rx) = mpsc::channel();
        
        Self {
            rom_icons: HashMap::new(),
            default_icon_texture: None,
            icon_load_queue: Arc::new(Mutex::new(VecDeque::new())),
            icon_info: HashMap::new(),
            last_icon_cleanup: Instant::now(),
            icon_results_rx: Some(rx),
            icon_results_tx: Some(tx),
            max_cached_icons: config.max_cached_icons,
            icon_lifetime: 300, // 5 minutes default
            max_concurrent_loads: num_cpus::get().min(8), // Use CPU cores, max 8
            thread_pool_initialized: false,
            performance_monitor: IconPerformanceMonitor::new(),
        }
    }

    /// Initialize the default icon texture
    pub fn init_default_icon(&mut self, ctx: &egui::Context, icon_size: u32) {
        let size = icon_size as usize;
        let pixels = vec![80u8; size * size * 4];

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [size, size],
            &pixels,
        );

        self.default_icon_texture = Some(ctx.load_texture(
            "default_icon",
            color_image,
            egui::TextureOptions::default(),
        ));
    }

    /// Queue an icon for loading (lazy loading)
    pub fn queue_icon_load(&mut self, rom_name: String, enable_lazy_icons: bool) {
        if enable_lazy_icons {
            if !self.rom_icons.contains_key(&rom_name)
                && !self.icon_info.contains_key(&rom_name) {
                    // Check if already in queue
                    if let Ok(mut queue) = self.icon_load_queue.lock() {
                        if !queue.contains(&rom_name) {
                            queue.push_back(rom_name);
                        }
                    }
                }
        }
    }

    /// Load icon from file system (thread-safe version)
    fn load_icon_from_file_threaded(rom_name: String, icons_path: PathBuf, icon_size: u32) -> IconLoadResult {
        let start_time = Instant::now();
        // Try to load .ico file
        let ico_path = icons_path.join(format!("{}.ico", rom_name));
        
        if ico_path.exists() {
            // Read the ico file
            if let Ok(ico_data) = std::fs::read(&ico_path) {
                // Try to load as ICO format
                if let Ok(image) = image::load_from_memory_with_format(&ico_data, image::ImageFormat::Ico) {
                    // Convert to RGBA8
                    let rgba_image = image.to_rgba8();
                    
                    // Resize to configured icon size if needed
                    let resized = if rgba_image.width() != icon_size || rgba_image.height() != icon_size {
                        image::imageops::resize(
                            &rgba_image,
                            icon_size,
                            icon_size,
                            image::imageops::FilterType::Lanczos3
                        )
                    } else {
                        rgba_image
                    };
                    
                    let width = resized.width();
                    let height = resized.height();
                    let pixels = resized.into_raw();
                    
                    let load_time = start_time.elapsed();
                    return IconLoadResult {
                        rom_name,
                        icon_data: Some(pixels),
                        width,
                        height,
                        load_time: Some(load_time),
                        success: true,
                    };
                }
            }
        }
        
        // Return empty result if loading failed
        let load_time = start_time.elapsed();
        IconLoadResult {
            rom_name,
            icon_data: None,
            width: 0,
            height: 0,
            load_time: Some(load_time),
            success: false,
        }
    }

    /// Process the icon loading queue with thread pool
    pub fn process_icon_queue(&mut self, ctx: &egui::Context, config: &AppConfig, current_fps: f32) {
        if !config.show_rom_icons {
            return;
        }

        // Initialize thread pool if needed
        if !self.thread_pool_initialized {
            self.initialize_thread_pool(config);
        }

        // Adaptive loading based on current FPS
        let max_per_frame = if current_fps < 25.0 {
            1
        } else if current_fps < 40.0 {
            3
        } else if current_fps < 50.0 {
            5
        } else {
            8
        };

        // Collect icons to load from queue
        let mut icons_to_load = Vec::new();
        if let Ok(mut queue) = self.icon_load_queue.lock() {
            for _ in 0..max_per_frame {
                if let Some(rom_name) = queue.pop_front() {
                    // Skip if already loaded
                    if !self.rom_icons.contains_key(&rom_name) {
                        icons_to_load.push(rom_name);
                    }
                } else {
                    break;
                }
            }
        }

        // Process icons in parallel using rayon thread pool
        if !icons_to_load.is_empty() {
            self.process_icons_parallel(icons_to_load, config);
        }

        // Process results from background threads
        self.process_icon_results(ctx);
    }

    /// Initialize the thread pool for icon loading
    fn initialize_thread_pool(&mut self, config: &AppConfig) {
        // Create a custom thread pool for icon loading
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.max_concurrent_loads)
            .thread_name(|i| format!("icon-loader-{}", i))
            .build_global()
            .expect("Failed to initialize icon loading thread pool");
        
        self.thread_pool_initialized = true;
        println!("Icon loading thread pool initialized with {} threads", self.max_concurrent_loads);
    }

    /// Process icons in parallel using rayon
    fn process_icons_parallel(&mut self, icons_to_load: Vec<String>, config: &AppConfig) {
        // Get icons path
        let icons_path = if let Some(path) = &config.icons_path {
            path.clone()
        } else {
            return; // No icons path configured
        };

        let icon_size = config.icon_size;
        let tx = if let Some(tx) = &self.icon_results_tx {
            tx.clone()
        } else {
            return;
        };

        // Process icons in parallel
        icons_to_load.into_par_iter().for_each(|rom_name| {
            let result = Self::load_icon_from_file_threaded(rom_name, icons_path.clone(), icon_size);
            let _ = tx.send(result); // Ignore send errors
        });
    }

    /// Process results from background icon loading threads
    fn process_icon_results(&mut self, ctx: &egui::Context) {
        if let Some(ref rx) = self.icon_results_rx {
            // Process all available results
            while let Ok(result) = rx.try_recv() {
                // Record performance metrics
                if let Some(load_time) = result.load_time {
                    self.performance_monitor.record_load(
                        load_time,
                        result.width,
                        result.success
                    );
                }
                
                if let Some(icon_data) = result.icon_data {
                    // Create egui ColorImage from loaded data
                    let size = [result.width as usize, result.height as usize];
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &icon_data);
                    
                    // Create texture
                    let texture = ctx.load_texture(
                        format!("icon_{}", result.rom_name),
                        color_image,
                        egui::TextureOptions::default(),
                    );
                    
                    // Store in cache
                    self.rom_icons.insert(result.rom_name.clone(), texture);
                    self.icon_info.insert(result.rom_name.clone(), IconInfo {
                        path: result.rom_name,
                        loaded: true,
                        last_accessed: Instant::now(),
                    });
                } else {
                    // Mark as failed to avoid retrying
                    self.icon_info.insert(result.rom_name.clone(), IconInfo {
                        path: result.rom_name,
                        loaded: false,
                        last_accessed: Instant::now(),
                    });
                }
            }
        }
    }

    /// Get an icon for a ROM, with fallback to default
    pub fn get_rom_icon(&mut self, rom_name: &str) -> Option<egui::TextureHandle> {
        // Update last accessed time
        if let Some(info) = self.icon_info.get_mut(rom_name) {
            info.last_accessed = Instant::now();
        }

        // Return cached icon or default
        self.rom_icons.get(rom_name).cloned()
            .or_else(|| self.default_icon_texture.clone())
    }

    /// Clean up old icons to free memory
    pub fn cleanup_old_icons(&mut self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        // Find icons that are too old
        for (rom_name, info) in &self.icon_info {
            if now.duration_since(info.last_accessed).as_secs() > self.icon_lifetime {
                to_remove.push(rom_name.clone());
            }
        }

        // Remove old icons
        for rom_name in to_remove {
            self.rom_icons.remove(&rom_name);
            self.icon_info.remove(&rom_name);
        }

        // If we still have too many icons, remove the least recently used
        if self.rom_icons.len() > self.max_cached_icons {
            let mut icon_ages: Vec<_> = self.icon_info.iter()
                .map(|(name, info)| (name.clone(), info.last_accessed))
                .collect();
            
            icon_ages.sort_by(|a, b| a.1.cmp(&b.1));
            
            let to_remove_count = self.rom_icons.len() - self.max_cached_icons;
            for (rom_name, _) in icon_ages.iter().take(to_remove_count) {
                self.rom_icons.remove(rom_name);
                self.icon_info.remove(rom_name);
            }
        }

        self.last_icon_cleanup = now;
    }

    /// Clear all cached icons
    pub fn clear_cache(&mut self) {
        self.rom_icons.clear();
        self.icon_info.clear();
        if let Ok(mut queue) = self.icon_load_queue.lock() {
            queue.clear();
        }
    }

    /// Remove a specific icon from cache
    pub fn remove_from_cache(&mut self, rom_name: &str) {
        self.rom_icons.remove(rom_name);
        self.icon_info.remove(rom_name);
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize, usize) {
        let queue_size = if let Ok(queue) = self.icon_load_queue.lock() {
            queue.len()
        } else {
            0
        };
        
        (
            self.rom_icons.len(),
            queue_size,
            self.icon_info.len()
        )
    }

    /// Check if cleanup is needed
    pub fn needs_cleanup(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_icon_cleanup).as_secs() > 60 // Clean up every minute
    }

    /// Get thread pool statistics
    pub fn get_thread_pool_stats(&self) -> (usize, bool) {
        (self.max_concurrent_loads, self.thread_pool_initialized)
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> String {
        self.performance_monitor.get_performance_summary()
    }
    
    /// Get performance monitor reference
    pub fn get_performance_monitor(&self) -> &IconPerformanceMonitor {
        &self.performance_monitor
    }
} 