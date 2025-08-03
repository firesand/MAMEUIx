use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsPreset {
    pub name: String,
    pub description: String,
    pub shader_chain: Option<String>,
    pub filter: bool,
    pub prescale: u8,
    pub bgfx_backend: BGFXBackend,
    pub bgfx_options: BGFXOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BGFXBackend {
    Auto,
    OpenGL,
    DirectX11,
    DirectX12,
    Vulkan,
    Metal,
    Gnm, // PlayStation 4
    Nvn, // Nintendo Switch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BGFXOptions {
    pub debug: bool,
    pub profile: bool,
    pub vsync: bool,
    pub max_frame_latency: u32,
    pub gamma: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPreset {
    pub name: String,
    pub description: String,
    pub vertex_shader: Option<String>,
    pub fragment_shader: Option<String>,
    pub parameters: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub presets: Vec<GraphicsPreset>,
    pub shader_presets: Vec<ShaderPreset>,
    pub global_preset: String,
    pub current_shader: Option<String>,
    pub bgfx_path: Option<String>,
    pub shader_path: Option<String>,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            presets: vec![
                GraphicsPreset {
                    name: "Original".to_string(),
                    description: "Raw pixels without filtering".to_string(),
                    shader_chain: None,
                    filter: false,
                    prescale: 1,
                    bgfx_backend: BGFXBackend::Auto,
                    bgfx_options: BGFXOptions::default(),
                },
                GraphicsPreset {
                    name: "CRT Classic".to_string(),
                    description: "Arcade monitor with scanlines".to_string(),
                    shader_chain: Some("crt-geom".to_string()),
                    filter: true,
                    prescale: 1,
                    bgfx_backend: BGFXBackend::OpenGL,
                    bgfx_options: BGFXOptions {
                        gamma: 1.2,
                        brightness: 1.0,
                        contrast: 1.1,
                        saturation: 1.0,
                        ..Default::default()
                    },
                },
                GraphicsPreset {
                    name: "LCD Sharp".to_string(),
                    description: "Sharp LCD display simulation".to_string(),
                    shader_chain: Some("lcd-sharp".to_string()),
                    filter: true,
                    prescale: 2,
                    bgfx_backend: BGFXBackend::Vulkan,
                    bgfx_options: BGFXOptions {
                        gamma: 1.0,
                        brightness: 1.1,
                        contrast: 1.2,
                        saturation: 0.9,
                        ..Default::default()
                    },
                },
                GraphicsPreset {
                    name: "Retro Scanlines".to_string(),
                    description: "Classic scanline effect".to_string(),
                    shader_chain: Some("scanlines".to_string()),
                    filter: true,
                    prescale: 1,
                    bgfx_backend: BGFXBackend::Auto,
                    bgfx_options: BGFXOptions::default(),
                },
            ],
            shader_presets: vec![
                ShaderPreset {
                    name: "crt-geom".to_string(),
                    description: "CRT geometry with curvature".to_string(),
                    vertex_shader: Some("crt-geom.vert".to_string()),
                    fragment_shader: Some("crt-geom.frag".to_string()),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("curvature".to_string(), 4.0);
                        params.insert("scanlines".to_string(), 0.5);
                        params.insert("phosphor".to_string(), 0.3);
                        params
                    },
                },
                ShaderPreset {
                    name: "lcd-sharp".to_string(),
                    description: "Sharp LCD pixel grid".to_string(),
                    vertex_shader: Some("lcd.vert".to_string()),
                    fragment_shader: Some("lcd.frag".to_string()),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("pixel_size".to_string(), 1.0);
                        params.insert("sharpness".to_string(), 0.8);
                        params
                    },
                },
                ShaderPreset {
                    name: "scanlines".to_string(),
                    description: "Simple scanline effect".to_string(),
                    vertex_shader: None,
                    fragment_shader: Some("scanlines.frag".to_string()),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("intensity".to_string(), 0.7);
                        params.insert("frequency".to_string(), 2.0);
                        params
                    },
                },
            ],
            global_preset: "Original".to_string(),
            current_shader: None,
            bgfx_path: None,
            shader_path: None,
        }
    }
}

impl Default for BGFXOptions {
    fn default() -> Self {
        Self {
            debug: false,
            profile: false,
            vsync: true,
            max_frame_latency: 1,
            gamma: 1.0,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

impl GraphicsConfig {
    /// Get current preset
    pub fn current_preset(&self) -> Option<&GraphicsPreset> {
        self.presets.iter().find(|p| p.name == self.global_preset)
    }

    /// Get shader preset by name
    pub fn get_shader_preset(&self, name: &str) -> Option<&ShaderPreset> {
        self.shader_presets.iter().find(|s| s.name == name)
    }

    /// Generate MAME BGFX command line arguments
    pub fn generate_bgfx_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        if let Some(preset) = self.current_preset() {
            // Backend selection
            match preset.bgfx_backend {
                BGFXBackend::Auto => args.push("-bgfx_backend".to_string()),
                BGFXBackend::OpenGL => {
                    args.push("-bgfx_backend".to_string());
                    args.push("opengl".to_string());
                },
                BGFXBackend::DirectX11 => {
                    args.push("-bgfx_backend".to_string());
                    args.push("d3d11".to_string());
                },
                BGFXBackend::DirectX12 => {
                    args.push("-bgfx_backend".to_string());
                    args.push("d3d12".to_string());
                },
                BGFXBackend::Vulkan => {
                    args.push("-bgfx_backend".to_string());
                    args.push("vulkan".to_string());
                },
                BGFXBackend::Metal => {
                    args.push("-bgfx_backend".to_string());
                    args.push("metal".to_string());
                },
                BGFXBackend::Gnm => {
                    args.push("-bgfx_backend".to_string());
                    args.push("gnm".to_string());
                },
                BGFXBackend::Nvn => {
                    args.push("-bgfx_backend".to_string());
                    args.push("nvn".to_string());
                },
            }

            // Shader chain
            if let Some(ref shader_chain) = preset.shader_chain {
                args.push("-bgfx_screen_chains".to_string());
                args.push(shader_chain.clone());
            }

            // Filter options
            if preset.filter {
                args.push("-filter".to_string());
                args.push("1".to_string());
            }

            // Prescale
            if preset.prescale > 1 {
                args.push("-prescale".to_string());
                args.push(preset.prescale.to_string());
            }

            // BGFX options
            let options = &preset.bgfx_options;
            if options.debug {
                args.push("-bgfx_debug".to_string());
                args.push("1".to_string());
            }
            if options.profile {
                args.push("-bgfx_profile".to_string());
                args.push("1".to_string());
            }
            if !options.vsync {
                args.push("-vsync".to_string());
                args.push("0".to_string());
            }
            if options.max_frame_latency != 1 {
                args.push("-bgfx_max_frame_latency".to_string());
                args.push(options.max_frame_latency.to_string());
            }

            // Color adjustments
            if options.gamma != 1.0 {
                args.push("-bgfx_gamma".to_string());
                args.push(options.gamma.to_string());
            }
            if options.brightness != 1.0 {
                args.push("-bgfx_brightness".to_string());
                args.push(options.brightness.to_string());
            }
            if options.contrast != 1.0 {
                args.push("-bgfx_contrast".to_string());
                args.push(options.contrast.to_string());
            }
            if options.saturation != 1.0 {
                args.push("-bgfx_saturation".to_string());
                args.push(options.saturation.to_string());
            }
        }

        args
    }

    /// Generate GLSL shader parameters
    pub fn generate_shader_params(&self) -> HashMap<String, f32> {
        let mut params = HashMap::new();
        
        if let Some(shader_name) = &self.current_shader {
            if let Some(shader_preset) = self.get_shader_preset(shader_name) {
                params.extend(shader_preset.parameters.clone());
            }
        }

        params
    }
}

// Example GLSL shader templates
pub mod shader_templates {
    pub const CRT_GEOM_VERT: &str = r#"
#version 330 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texCoord;

out vec2 vTexCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vTexCoord = texCoord;
}
"#;

    pub const CRT_GEOM_FRAG: &str = r#"
#version 330 core

in vec2 vTexCoord;
out vec4 fragColor;

uniform sampler2D uTexture;
uniform float uCurvature;
uniform float uScanlines;
uniform float uPhosphor;

void main() {
    // Apply CRT curvature
    vec2 uv = vTexCoord - 0.5;
    float dist = length(uv);
    uv = uv * (1.0 + dist * dist * uCurvature);
    uv = uv + 0.5;
    
    // Sample texture
    vec4 color = texture(uTexture, uv);
    
    // Apply scanlines
    float scanline = sin(uv.y * 1000.0) * 0.5 + 0.5;
    color.rgb *= 1.0 - uScanlines * (1.0 - scanline);
    
    // Phosphor effect
    color.rgb = mix(color.rgb, color.rgb * 0.8, uPhosphor);
    
    fragColor = color;
}
"#;

    pub const LCD_FRAG: &str = r#"
#version 330 core

in vec2 vTexCoord;
out vec4 fragColor;

uniform sampler2D uTexture;
uniform float uPixelSize;
uniform float uSharpness;

void main() {
    vec2 uv = vTexCoord;
    
    // Create pixel grid effect
    vec2 pixel = floor(uv * uPixelSize) / uPixelSize;
    vec4 color = texture(uTexture, pixel);
    
    // Apply sharpening
    vec4 neighbor1 = texture(uTexture, pixel + vec2(1.0/uPixelSize, 0.0));
    vec4 neighbor2 = texture(uTexture, pixel + vec2(-1.0/uPixelSize, 0.0));
    vec4 neighbor3 = texture(uTexture, pixel + vec2(0.0, 1.0/uPixelSize));
    vec4 neighbor4 = texture(uTexture, pixel + vec2(0.0, -1.0/uPixelSize));
    
    color.rgb = color.rgb + uSharpness * (4.0 * color.rgb - neighbor1.rgb - neighbor2.rgb - neighbor3.rgb - neighbor4.rgb);
    
    fragColor = color;
}
"#;

    pub const SCANLINES_FRAG: &str = r#"
#version 330 core

in vec2 vTexCoord;
out vec4 fragColor;

uniform sampler2D uTexture;
uniform float uIntensity;
uniform float uFrequency;

void main() {
    vec4 color = texture(uTexture, vTexCoord);
    
    // Create scanline effect
    float scanline = sin(vTexCoord.y * uFrequency * 100.0) * 0.5 + 0.5;
    color.rgb *= 1.0 - uIntensity * (1.0 - scanline);
    
    fragColor = color;
}
"#;
}

pub mod enhanced_validator;
pub mod shader_manager;
pub use shader_manager::{ShaderManager, BGFXManager};
