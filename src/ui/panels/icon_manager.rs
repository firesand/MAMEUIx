// src/ui/icon_manager.rs
// Icon management module - handles loading, caching, and displaying game icons

use eframe::egui;
use crate::models::{AppConfig, IconInfo};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

pub struct IconManager {
    // Icon storage
    pub rom_icons: HashMap<String, egui::TextureHandle>,
    pub default_icon_texture: Option<egui::TextureHandle>,
    
    // Loading queue and state
    pub icon_load_queue: VecDeque<String>,
    pub icon_info: HashMap<String, IconInfo>,
    pub last_icon_cleanup: Instant,
    
    // Performance settings
    pub max_cached_icons: usize,
    pub icon_lifetime: u64, // seconds
}

impl IconManager {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            rom_icons: HashMap::new(),
            default_icon_texture: None,
            icon_load_queue: VecDeque::new(),
            icon_info: HashMap::new(),
            last_icon_cleanup: Instant::now(),
            max_cached_icons: config.max_cached_icons,
            icon_lifetime: 300, // 5 minutes default
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
                && !self.icon_load_queue.contains(&rom_name)
                && !self.icon_info.contains_key(&rom_name) {
                    self.icon_load_queue.push_back(rom_name);
                }
        }
    }

    /// Load icon from file system
    pub fn load_icon_from_file(&self, ctx: &egui::Context, rom_name: &str, config: &AppConfig) -> Option<egui::TextureHandle> {
        // Check if icons path is configured
        let icons_path = config.icons_path.as_ref()?;
        
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
                    let icon_size = config.icon_size;
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
                    
                    let size = [resized.width() as usize, resized.height() as usize];
                    let pixels = resized.into_raw();
                    
                    // Create egui ColorImage
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    
                    // Create texture
                    return Some(ctx.load_texture(
                        format!("icon_{}", rom_name),
                        color_image,
                        egui::TextureOptions::default(),
                    ));
                }
            }
        }
        
        None
    }

    /// Process the icon loading queue with adaptive performance
    pub fn process_icon_queue(&mut self, ctx: &egui::Context, config: &AppConfig, current_fps: f32) {
        if !config.show_rom_icons || self.icon_load_queue.is_empty() {
            return;
        }

        // Adaptive loading based on current FPS
        let max_per_frame = if current_fps < 25.0 {
            2
        } else if current_fps < 40.0 {
            5
        } else if current_fps < 50.0 {
            8
        } else {
            12
        };

        // Pre-allocate vector for batch loading
        let mut icons_to_load = Vec::with_capacity(max_per_frame);
        
        // Collect icons to load
        for _ in 0..max_per_frame {
            if let Some(rom_name) = self.icon_load_queue.pop_front() {
                // Skip if already loaded
                if !self.rom_icons.contains_key(&rom_name) {
                    icons_to_load.push(rom_name);
                }
            } else {
                break;
            }
        }

        // Load icons in batch
        for rom_name in icons_to_load {
            // Try to load icon from file
            let icon_texture = self.load_icon_from_file(ctx, &rom_name, config)
                .or_else(|| self.default_icon_texture.clone());

            if let Some(texture) = icon_texture {
                self.rom_icons.insert(rom_name.clone(), texture);
                self.icon_info.insert(rom_name.clone(), IconInfo {
                    path: rom_name,
                    loaded: true,
                    last_accessed: Instant::now(),
                });
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
        self.icon_load_queue.clear();
    }

    /// Remove a specific icon from cache
    pub fn remove_from_cache(&mut self, rom_name: &str) {
        self.rom_icons.remove(rom_name);
        self.icon_info.remove(rom_name);
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize, usize) {
        (
            self.rom_icons.len(),
            self.icon_load_queue.len(),
            self.icon_info.len()
        )
    }

    /// Check if cleanup is needed
    pub fn needs_cleanup(&self) -> bool {
        let now = Instant::now();
        now.duration_since(self.last_icon_cleanup).as_secs() > 60 // Clean up every minute
    }
} 