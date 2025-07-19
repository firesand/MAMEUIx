use std::path::PathBuf;
use std::collections::HashMap;
mod graphics;
use graphics::{GraphicsConfig, BGFXBackend, BGFXOptions, ShaderManager, BGFXManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing BGFX and GLSL Integration");
    println!("=====================================");

    // Test 1: Graphics Configuration
    println!("\n1. Testing Graphics Configuration...");
    let mut graphics_config = GraphicsConfig::default();
    
    // Test preset selection
    graphics_config.global_preset = "CRT Classic".to_string();
    if let Some(preset) = graphics_config.current_preset() {
        println!("✅ Current preset: {}", preset.name);
        println!("   Description: {}", preset.description);
        println!("   Backend: {:?}", preset.bgfx_backend);
        println!("   Shader chain: {:?}", preset.shader_chain);
    } else {
        println!("❌ Failed to get current preset");
    }

    // Test BGFX argument generation
    println!("\n2. Testing BGFX Argument Generation...");
    let bgfx_args = graphics_config.generate_bgfx_args();
    println!("Generated BGFX arguments:");
    for arg in &bgfx_args {
        println!("   {}", arg);
    }

    // Test 3: Shader Manager
    println!("\n3. Testing Shader Manager...");
    let temp_dir = std::env::temp_dir().join("mameuix_test_shaders");
    let mut shader_manager = ShaderManager::new(temp_dir.clone());
    
    // Create test shader
    match shader_manager.create_shader_template("test-crt", "crt-geom") {
        Ok(_) => println!("✅ Created CRT shader template"),
        Err(e) => println!("❌ Failed to create shader template: {}", e),
    }

    // List available shaders
    let shaders = shader_manager.list_shaders();
    println!("Available shaders: {:?}", shaders);

    // Test 4: BGFX Manager
    println!("\n4. Testing BGFX Manager...");
    let bgfx_path = PathBuf::from("/usr/local/bin"); // Example path
    let bgfx_manager = BGFXManager::new(bgfx_path);
    
    // Test argument generation
    let mut options = HashMap::new();
    options.insert("debug".to_string(), "1".to_string());
    options.insert("gamma".to_string(), "1.2".to_string());
    
    let bgfx_args = bgfx_manager.generate_args("vulkan", &options);
    println!("BGFX manager arguments:");
    for arg in &bgfx_args {
        println!("   {}", arg);
    }

    // Test 5: Shader Parameter Generation
    println!("\n5. Testing Shader Parameter Generation...");
    graphics_config.current_shader = Some("crt-geom".to_string());
    let shader_params = graphics_config.generate_shader_params();
    println!("Shader parameters: {:?}", shader_params);

    // Test 6: Complete MAME Command Generation
    println!("\n6. Testing Complete MAME Command Generation...");
    let mut all_args = Vec::new();
    all_args.extend(bgfx_args);
    all_args.push("game_name".to_string());
    
    println!("Complete MAME command:");
    println!("mame {}", all_args.join(" "));

    // Test 7: Validation Tests
    println!("\n7. Testing Validation...");
    
    // Test GLSL syntax validation
    let valid_glsl = r#"
#version 330 core
in vec2 vTexCoord;
out vec4 fragColor;
uniform sampler2D uTexture;
void main() {
    fragColor = texture(uTexture, vTexCoord);
}
"#;
    
    match shader_manager.validate_glsl_syntax(valid_glsl, "test") {
        Ok(_) => println!("✅ Valid GLSL syntax"),
        Err(e) => println!("❌ GLSL validation failed: {}", e),
    }

    // Test invalid GLSL
    let invalid_glsl = r#"
#version 330 core
void main() {
    // Missing closing brace
"#;
    
    match shader_manager.validate_glsl_syntax(invalid_glsl, "test") {
        Ok(_) => println!("❌ Should have failed validation"),
        Err(e) => println!("✅ Correctly caught invalid GLSL: {}", e),
    }

    println!("\n🎉 BGFX and GLSL Integration Test Complete!");
    println!("=============================================");
    
    // Cleanup
    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)?;
    }

    Ok(())
} 