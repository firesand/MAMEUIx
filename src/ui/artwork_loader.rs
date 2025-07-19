// src/ui/artwork_loader.rs
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use eframe::egui;

pub struct ArtworkLoader {
    // Cache loaded textures to avoid reloading
    texture_cache: HashMap<String, egui::TextureHandle>,
}

impl ArtworkLoader {
    pub fn new() -> Self {
        Self {
            texture_cache: HashMap::new(),
        }
    }

    /// Load artwork for a game from the specified directory
    pub fn load_artwork(
        &mut self,
        ctx: &egui::Context,
        game_name: &str,
        artwork_type: ArtworkType,
        config: &crate::models::config::AppConfig,
    ) -> Option<egui::TextureHandle> {
        // Generate cache key
        let cache_key = format!("{}-{:?}", game_name, artwork_type);
        
        // Check cache first
        if let Some(texture) = self.texture_cache.get(&cache_key) {
            return Some(texture.clone());
        }

        // Determine which directory to search based on artwork type
        let search_dir = match artwork_type {
            ArtworkType::Screenshot => {
                if let Some(path) = config.snap_path.as_ref() {
                    path
                } else {
                    return None;
                }
            },
            ArtworkType::Cabinet => config.cabinet_path.as_ref()?,
            ArtworkType::Marquee => config.marquee_path.as_ref()?,
            ArtworkType::Title => config.title_path.as_ref()?,
            ArtworkType::Flyer => config.flyer_path.as_ref()?,
        };
        
        // Check if the directory exists
        if !search_dir.exists() {
            return None;
        }

        // Try to load the artwork
        if let Some(texture) = self.load_from_directory(ctx, game_name, search_dir, artwork_type) {
            // Cache the texture
            self.texture_cache.insert(cache_key, texture.clone());
            return Some(texture);
        }

        None
    }

    /// Load artwork from a specific directory
    fn load_from_directory(
        &self,
        ctx: &egui::Context,
        game_name: &str,
        dir: &Path,
        artwork_type: ArtworkType,
    ) -> Option<egui::TextureHandle> {
        // The configured path should already point to the correct directory
        // Don't add subdirectories - the user has already configured the full path
        let search_path = dir.to_path_buf();

        // Try to find and load the artwork file
        // First try ZIP file
        let zip_path = search_path.join(format!("{}.zip", game_name));
        if zip_path.exists() {
            if let Some(image_data) = self.extract_from_zip(&zip_path, game_name) {
                return self.create_texture(ctx, &image_data, game_name);
            }
        }

        // Try common image formats
        for ext in &["png", "jpg", "jpeg", "bmp", "gif"] {
            let image_path = search_path.join(format!("{}.{}", game_name, ext));
            if image_path.exists() {
                if let Ok(image_data) = std::fs::read(&image_path) {
                    return self.create_texture(ctx, &image_data, game_name);
                }
            }
        }

        // For screenshots, also check for subdirectory with game name
        if artwork_type == ArtworkType::Screenshot {
            let game_subdir = search_path.join(game_name);
            if game_subdir.exists() && game_subdir.is_dir() {
                // First try direct game name files in the subdirectory
                for ext in &["png", "jpg", "jpeg", "bmp", "gif"] {
                    let image_path = game_subdir.join(format!("{}.{}", game_name, ext));
                    if image_path.exists() {
                        if let Ok(image_data) = std::fs::read(&image_path) {
                            return self.create_texture(ctx, &image_data, game_name);
                        }
                    }
                }
                
                // Look for numbered PNG files (0000.png, 0001.png, etc.)
                if let Ok(entries) = std::fs::read_dir(&game_subdir) {
                    let mut numbered_files: Vec<(u32, std::path::PathBuf)> = Vec::new();
                    
                    for entry in entries.flatten() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            // Check if it matches the pattern 000*.png
                            if file_name.ends_with(".png") {
                                if let Some(num_part) = file_name.strip_suffix(".png") {
                                    // Try to parse as number (handles 0000, 0001, etc.)
                                    if let Ok(num) = num_part.parse::<u32>() {
                                        numbered_files.push((num, entry.path()));
                                    }
                                }
                            }
                        }
                    }
                    
                    // Sort by number (highest first) and take the first one
                    numbered_files.sort_by(|a, b| b.0.cmp(&a.0));
                    
                    if let Some((num, path)) = numbered_files.first() {
                        if let Ok(image_data) = std::fs::read(path) {
                            return self.create_texture(ctx, &image_data, game_name);
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract image from ZIP file
    fn extract_from_zip(&self, zip_path: &Path, game_name: &str) -> Option<Vec<u8>> {
        let file = File::open(zip_path).ok()?;
        let mut archive = zip::ZipArchive::new(file).ok()?;
        
        let archive_len = archive.len();

        // Look for image files in the ZIP
        for i in 0..archive_len {
            if let Ok(mut file) = archive.by_index(i) {
                let file_name = file.name().to_lowercase();
                
                // Check if this is an image file
                if file_name.ends_with(".png") ||
                   file_name.ends_with(".jpg") ||
                   file_name.ends_with(".jpeg") ||
                   file_name.ends_with(".bmp") ||
                   file_name.ends_with(".gif") {
                    
                    // Some ZIPs might have the game name in the filename
                    if file_name.contains(game_name) ||
                       file_name == format!("{}.png", game_name) ||
                       file_name == format!("{}.jpg", game_name) ||
                       file_name == "0000.png" || // Some ZIPs use 0000.png as the main image
                       archive_len == 1 { // If only one file, use it
                        
                        let mut buffer = Vec::new();
                        if file.read_to_end(&mut buffer).is_ok() {
                            return Some(buffer);
                        }
                    }
                }
            }
        }

        None
    }

    /// Create egui texture from image data
    fn create_texture(
        &self,
        ctx: &egui::Context,
        image_data: &[u8],
        name: &str,
    ) -> Option<egui::TextureHandle> {
        // Try to load the image
        let image = image::load_from_memory(image_data).ok()?;
        
        // Convert to RGBA8
        let rgba_image = image.to_rgba8();
        let size = [rgba_image.width() as usize, rgba_image.height() as usize];
        let pixels = rgba_image.into_raw();

        // Create egui ColorImage
        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);

        // Create texture
        Some(ctx.load_texture(
            name,
            color_image,
            egui::TextureOptions::default(),
        ))
    }

    /// Clear cache to free memory
    pub fn clear_cache(&mut self) {
        self.texture_cache.clear();
    }

    /// Remove specific item from cache
    pub fn remove_from_cache(&mut self, game_name: &str, artwork_type: ArtworkType) {
        let cache_key = format!("{}-{:?}", game_name, artwork_type);
        self.texture_cache.remove(&cache_key);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArtworkType {
    Screenshot,
    Cabinet,
    Marquee,
    Title,
    Flyer,
}