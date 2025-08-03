use std::path::PathBuf;
use anyhow::Result;

// This demonstrates how to integrate the enhanced shader validation into your UI
// You can copy this pattern into your game_properties.rs or wherever you handle shaders

pub struct ShaderValidationDemo {
    shader_manager: crate::utils::graphics::ShaderManager,
}

impl ShaderValidationDemo {
    pub fn new() -> Self {
        let shader_path = PathBuf::from("shaders"); // Your shader directory
        Self {
            shader_manager: crate::utils::graphics::ShaderManager::new(shader_path),
        }
    }

    /// Example: Validate a shader file and show results in UI
    pub fn validate_shader_file(&self, shader_path: &str) -> (bool, Vec<String>) {
        if shader_path.is_empty() {
            return (true, vec![]);
        }

        // Try to read the shader file
        match std::fs::read_to_string(shader_path) {
            Ok(shader_content) => {
                // Determine shader type from file extension
                let shader_type = if shader_path.ends_with(".vert") {
                    "vertex"
                } else {
                    "fragment"
                };

                // Validate the shader using the enhanced validator
                match self.shader_manager.validate_glsl_syntax(&shader_content, shader_type) {
                    Ok(()) => (true, vec![]),
                    Err(e) => (false, vec![e.to_string()])
                }
            }
            Err(_) => (false, vec!["File not found or cannot be read".to_string()])
        }
    }

    /// Example: Show validation status in egui UI
    pub fn show_validation_status(&self, ui: &mut egui::Ui, shader_path: &str, slot_name: &str) {
        if shader_path.is_empty() {
            return;
        }

        let (is_valid, errors) = self.validate_shader_file(shader_path);

        if is_valid {
            ui.colored_label(egui::Color32::GREEN, format!("✓ {}: Valid", slot_name));
        } else {
            ui.colored_label(egui::Color32::RED, format!("✗ {}: Invalid", slot_name));
            for error in errors {
                ui.colored_label(egui::Color32::RED, format!("  {}", error));
            }
        }
    }

    /// Example: Real-time validation during shader editing
    pub fn validate_shader_code(&self, shader_code: &str, shader_type: &str) -> Result<()> {
        self.shader_manager.validate_glsl_syntax(shader_code, shader_type)
    }
}

/// Example usage in your UI:
/// 
/// ```rust
/// // In your game_properties.rs or wherever you handle shaders:
/// 
/// // 1. Add shader manager to your dialog struct
/// pub struct GamePropertiesDialog {
///     // ... existing fields ...
///     shader_validator: ShaderValidationDemo,
/// }
/// 
/// // 2. Initialize it in your constructor
/// impl GamePropertiesDialog {
///     pub fn new_with_config(game: Option<&Game>, config: &AppConfig) -> Self {
///         // ... existing code ...
///         Self {
///             // ... existing fields ...
///             shader_validator: ShaderValidationDemo::new(),
///         }
///     }
/// }
/// 
/// // 3. Use it in your shader input fields
/// fn show_shader_input(&self, ui: &mut egui::Ui, shader_path: &str, slot_name: &str) {
///     ui.horizontal(|ui| {
///         ui.label(format!("{}", slot_name));
///         
///         let mut path = shader_path.to_string();
///         if ui.text_edit_singleline(&mut path).changed() {
///             // Update your shader path here
///         }
///         
///         // Show validation status
///         self.shader_validator.show_validation_status(ui, &path, slot_name);
///     });
/// }
/// ```
