use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use anyhow::{Result, Context};

/// Manages GLSL shaders and BGFX integration
pub struct ShaderManager {
    shader_path: PathBuf,
    compiled_shaders: HashMap<String, ShaderInfo>,
}

#[derive(Debug, Clone)]
pub struct ShaderInfo {
    pub name: String,
    pub vertex_shader: Option<String>,
    pub fragment_shader: Option<String>,
    pub parameters: HashMap<String, f32>,
    pub compiled: bool,
}

impl ShaderManager {
    pub fn new(shader_path: PathBuf) -> Self {
        Self {
            shader_path,
            compiled_shaders: HashMap::new(),
        }
    }

    /// Load shader from file system
    pub fn load_shader(&mut self, shader_name: &str) -> Result<ShaderInfo> {
        let shader_dir = self.shader_path.join(shader_name);
        
        if !shader_dir.exists() {
            return Err(anyhow::anyhow!("Shader directory not found: {:?}", shader_dir));
        }

        let mut shader_info = ShaderInfo {
            name: shader_name.to_string(),
            vertex_shader: None,
            fragment_shader: None,
            parameters: HashMap::new(),
            compiled: false,
        };

        // Load vertex shader
        let vert_path = shader_dir.join("vertex.glsl");
        if vert_path.exists() {
            shader_info.vertex_shader = Some(fs::read_to_string(&vert_path)
                .context("Failed to read vertex shader")?);
        }

        // Load fragment shader
        let frag_path = shader_dir.join("fragment.glsl");
        if frag_path.exists() {
            shader_info.fragment_shader = Some(fs::read_to_string(&frag_path)
                .context("Failed to read fragment shader")?);
        }

        // Load parameters
        let params_path = shader_dir.join("parameters.json");
        if params_path.exists() {
            let params_data = fs::read_to_string(&params_path)
                .context("Failed to read shader parameters")?;
            shader_info.parameters = serde_json::from_str(&params_data)
                .context("Failed to parse shader parameters")?;
        }

        // Validate shader
        self.validate_shader(&shader_info)?;
        
        shader_info.compiled = true;
        self.compiled_shaders.insert(shader_name.to_string(), shader_info.clone());
        
        Ok(shader_info)
    }

    /// Validate GLSL shader syntax
    fn validate_shader(&self, shader_info: &ShaderInfo) -> Result<()> {
        // Basic GLSL validation
        if let Some(ref vert_shader) = shader_info.vertex_shader {
            self.validate_glsl_syntax(vert_shader, "vertex")?;
        }
        
        if let Some(ref frag_shader) = shader_info.fragment_shader {
            self.validate_glsl_syntax(frag_shader, "fragment")?;
        }
        
        Ok(())
    }

    /// Basic GLSL syntax validation
    pub(crate) fn validate_glsl_syntax(&self, shader_code: &str, shader_type: &str) -> Result<()> {
        // Check for required GLSL version
        if !shader_code.contains("#version") {
            return Err(anyhow::anyhow!("{} shader missing #version directive", shader_type));
        }

        // Check for main function
        if !shader_code.contains("main()") {
            return Err(anyhow::anyhow!("{} shader missing main() function", shader_type));
        }

        // Basic syntax checks
        let open_braces = shader_code.matches('{').count();
        let close_braces = shader_code.matches('}').count();
        
        if open_braces != close_braces {
            return Err(anyhow::anyhow!("{} shader has mismatched braces", shader_type));
        }

        Ok(())
    }

    /// Generate BGFX shader chain configuration
    pub fn generate_bgfx_chain(&self, shader_name: &str) -> Result<String> {
        if let Some(shader_info) = self.compiled_shaders.get(shader_name) {
            let mut chain_config = format!("chain={}", shader_name);
            
            // Add parameters
            for (param_name, param_value) in &shader_info.parameters {
                chain_config.push_str(&format!(",{}={}", param_name, param_value));
            }
            
            Ok(chain_config)
        } else {
            Err(anyhow::anyhow!("Shader not found: {}", shader_name))
        }
    }

    /// Create shader template
    pub fn create_shader_template(&self, shader_name: &str, template_type: &str) -> Result<()> {
        let shader_dir = self.shader_path.join(shader_name);
        fs::create_dir_all(&shader_dir)?;

        match template_type {
            "crt-geom" => {
                // Create CRT geometry shader
                let vert_shader = include_str!("shader_templates/crt-geom.vert");
                let frag_shader = include_str!("shader_templates/crt-geom.frag");
                
                fs::write(shader_dir.join("vertex.glsl"), vert_shader)?;
                fs::write(shader_dir.join("fragment.glsl"), frag_shader)?;
                
                // Create parameters file
                let params = serde_json::json!({
                    "curvature": 4.0,
                    "scanlines": 0.5,
                    "phosphor": 0.3
                });
                fs::write(shader_dir.join("parameters.json"), serde_json::to_string_pretty(&params)?)?;
            },
            "lcd-sharp" => {
                // Create LCD sharp shader
                let vert_shader = include_str!("shader_templates/lcd.vert");
                let frag_shader = include_str!("shader_templates/lcd.frag");
                
                fs::write(shader_dir.join("vertex.glsl"), vert_shader)?;
                fs::write(shader_dir.join("fragment.glsl"), frag_shader)?;
                
                // Create parameters file
                let params = serde_json::json!({
                    "pixel_size": 1.0,
                    "sharpness": 0.8
                });
                fs::write(shader_dir.join("parameters.json"), serde_json::to_string_pretty(&params)?)?;
            },
            "scanlines" => {
                // Create scanlines shader
                let frag_shader = include_str!("shader_templates/scanlines.frag");
                
                fs::write(shader_dir.join("fragment.glsl"), frag_shader)?;
                
                // Create parameters file
                let params = serde_json::json!({
                    "intensity": 0.7,
                    "frequency": 2.0
                });
                fs::write(shader_dir.join("parameters.json"), serde_json::to_string_pretty(&params)?)?;
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown shader template: {}", template_type));
            }
        }

        Ok(())
    }

    /// List available shaders
    pub fn list_shaders(&self) -> Vec<String> {
        if !self.shader_path.exists() {
            return Vec::new();
        }

        let mut shaders = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.shader_path) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        shaders.push(name.to_string());
                    }
                }
            }
        }
        
        shaders
    }

    /// Get shader info
    pub fn get_shader_info(&self, shader_name: &str) -> Option<&ShaderInfo> {
        self.compiled_shaders.get(shader_name)
    }

    /// Clear shader cache
    pub fn clear_cache(&mut self) {
        self.compiled_shaders.clear();
    }
}

/// BGFX integration utilities
pub struct BGFXManager {
    bgfx_path: PathBuf,
}

impl BGFXManager {
    pub fn new(bgfx_path: PathBuf) -> Self {
        Self { bgfx_path }
    }

    /// Generate BGFX command line arguments
    pub fn generate_args(&self, backend: &str, options: &HashMap<String, String>) -> Vec<String> {
        let mut args = Vec::new();
        
        // Backend
        if backend != "auto" {
            args.push("-bgfx_backend".to_string());
            args.push(backend.to_string());
        }
        
        // Options
        for (key, value) in options {
            args.push(format!("-bgfx_{}", key));
            args.push(value.clone());
        }
        
        args
    }

    /// Validate BGFX installation
    pub fn validate_installation(&self) -> Result<()> {
        if !self.bgfx_path.exists() {
            return Err(anyhow::anyhow!("BGFX path not found: {:?}", self.bgfx_path));
        }
        
        // Check for required BGFX files
        let required_files = ["bgfx", "bgfx_shaderc"];
        for file in &required_files {
            let file_path = self.bgfx_path.join(file);
            if !file_path.exists() {
                return Err(anyhow::anyhow!("BGFX file not found: {:?}", file_path));
            }
        }
        
        Ok(())
    }
} 