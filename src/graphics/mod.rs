use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsPreset {
    pub name: String,
    pub description: String,
    pub shader_chain: Option<String>,
    pub filter: bool,
    pub prescale: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub presets: Vec<GraphicsPreset>,
    pub global_preset: String,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            presets: vec![
                GraphicsPreset {
                    name: "Original".to_string(),
                    description: "Raw pixels".to_string(),
                    shader_chain: None,
                    filter: false,
                    prescale: 1,
                },
                GraphicsPreset {
                    name: "CRT Classic".to_string(),
                    description: "Arcade monitor with scanlines".to_string(),
                    shader_chain: Some("crt-geom".to_string()),
                    filter: true,
                    prescale: 1,
                },
            ],
            global_preset: "Original".to_string(),
        }
    }
}
