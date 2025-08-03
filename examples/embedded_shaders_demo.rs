use crate::embedded_shaders::EmbeddedShaders;

fn main() {
    println!("🎮 MAMEUI Embedded Shaders Demo");
    println!("=================================");

    let embedded_shaders = EmbeddedShaders::new();

    println!("📁 Available Embedded Shaders:");
    let shaders = embedded_shaders.list_shaders();
    for shader in &shaders {
        if let Some(desc) = embedded_shaders.get_shader_description(shader) {
            println!("   • {} - {}", shader, desc);
        } else {
            println!("   • {}", shader);
        }
    }
}
