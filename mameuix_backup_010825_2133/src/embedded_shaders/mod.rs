// Embedded shaders module - automatically includes shaders in the binary
// Users get these shaders without needing to download anything

use std::collections::HashMap;
use std::path::Path;

/// Embedded shaders that are included in the binary
pub struct EmbeddedShaders {
    shaders: HashMap<String, String>,
}

impl EmbeddedShaders {
    pub fn new() -> Self {
        let mut shaders = HashMap::new();
        
        // CRT Shaders
        shaders.insert("crt-geom.frag".to_string(), include_str!("crt-geom.frag").to_string());
        shaders.insert("crt-geom.vert".to_string(), include_str!("crt-geom.vert").to_string());
        shaders.insert("crt-royale.frag".to_string(), include_str!("crt-royale.frag").to_string());
        shaders.insert("crt-royale.vert".to_string(), include_str!("crt-royale.vert").to_string());
        
        // LCD Shaders
        shaders.insert("lcd.frag".to_string(), include_str!("lcd.frag").to_string());
        shaders.insert("lcd.vert".to_string(), include_str!("lcd.vert").to_string());
        
        // NTSC Shaders
        shaders.insert("ntsc.frag".to_string(), include_str!("ntsc.frag").to_string());
        shaders.insert("ntsc.vert".to_string(), include_str!("ntsc.vert").to_string());
        
        // Pixel Perfect Shaders
        shaders.insert("pixel-perfect.frag".to_string(), include_str!("pixel-perfect.frag").to_string());
        shaders.insert("pixel-perfect.vert".to_string(), include_str!("pixel-perfect.vert").to_string());
        
        // Scanline Shaders
        shaders.insert("scanlines.frag".to_string(), include_str!("scanlines.frag").to_string());
        
        Self { shaders }
    }
    
    /// Get a shader by name
    pub fn get_shader(&self, name: &str) -> Option<&String> {
        self.shaders.get(name)
    }
    
    /// List all available embedded shaders
    pub fn list_shaders(&self) -> Vec<String> {
        self.shaders.keys().cloned().collect()
    }
    
    /// Get shader categories for UI organization
    pub fn get_shader_categories(&self) -> HashMap<String, Vec<String>> {
        let mut categories = HashMap::new();
        
        // CRT Shaders
        categories.insert("CRT Effects".to_string(), vec![
            "crt-geom.frag".to_string(),
            "crt-geom.vert".to_string(),
            "crt-royale.frag".to_string(),
            "crt-royale.vert".to_string(),
        ]);
        
        // LCD Shaders
        categories.insert("LCD Effects".to_string(), vec![
            "lcd.frag".to_string(),
            "lcd.vert".to_string(),
        ]);
        
        // Retro Effects
        categories.insert("Retro Effects".to_string(), vec![
            "ntsc.frag".to_string(),
            "ntsc.vert".to_string(),
            "scanlines.frag".to_string(),
        ]);
        
        // Scaling
        categories.insert("Scaling".to_string(), vec![
            "pixel-perfect.frag".to_string(),
            "pixel-perfect.vert".to_string(),
        ]);
        
        categories
    }
    
    /// Extract all embedded shaders to a directory
    pub fn extract_to_directory(&self, target_dir: &Path) -> std::io::Result<()> {
        use std::fs;
        
        // Create target directory if it doesn't exist
        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }
        
        // Write each shader to a file
        for (name, content) in &self.shaders {
            let file_path = target_dir.join(name);
            fs::write(file_path, content)?;
        }
        
        Ok(())
    }
    
    /// Get shader description for UI tooltips
    pub fn get_shader_description(&self, name: &str) -> Option<&'static str> {
        match name {
            "crt-geom.frag" => Some("CRT geometry simulation with scanlines and curvature"),
            "crt-geom.vert" => Some("Vertex shader for CRT geometry effects"),
            "crt-royale.frag" => Some("Advanced CRT simulation with bloom and phosphor effects"),
            "crt-royale.vert" => Some("Vertex shader for CRT-Royale effects"),
            "lcd.frag" => Some("LCD grid effect for handheld games"),
            "lcd.vert" => Some("Vertex shader for LCD effects"),
            "ntsc.frag" => Some("NTSC color space simulation for authentic retro look"),
            "ntsc.vert" => Some("Vertex shader for NTSC effects"),
            "pixel-perfect.frag" => Some("Integer scaling for crisp, pixel-perfect rendering"),
            "pixel-perfect.vert" => Some("Vertex shader for pixel-perfect scaling"),
            "scanlines.frag" => Some("Simple scanline effect for retro CRT look"),
            _ => None,
        }
    }
}

impl Default for EmbeddedShaders {
    fn default() -> Self {
        Self::new()
    }
}
