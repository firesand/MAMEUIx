# BGFX dan GLSL Integration dalam MAMEUIx

## 📚 Overview

MAMEUIx telah diintegrasikan dengan BGFX (Bare Graphics Framework) dan GLSL (OpenGL Shading Language) untuk memberikan pengalaman visual yang lebih baik dan fleksibilitas dalam rendering game arcade.

## 🎯 Fitur Utama

### BGFX Integration
- **Multi-backend Support**: OpenGL, DirectX 11/12, Vulkan, Metal, Gnm (PS4), Nvn (Switch)
- **Cross-platform Rendering**: Konsisten di berbagai platform
- **Performance Optimization**: Rendering pipeline yang dioptimalkan
- **Post-processing Effects**: Filter dan efek visual yang dapat dikustomisasi

### GLSL Shader System
- **Custom Shaders**: Vertex dan Fragment shader yang dapat dikustomisasi
- **Shader Presets**: Template shader untuk efek umum
- **Parameter Control**: Kontrol real-time terhadap parameter shader
- **Validation**: Validasi syntax GLSL otomatis

## 🏗️ Arsitektur

### Graphics Module Structure
```
src/graphics/
├── mod.rs              # Main graphics configuration
├── shader_manager.rs   # GLSL shader management
└── shader_templates/   # Pre-built shader templates
    ├── crt-geom.vert   # CRT geometry vertex shader
    ├── crt-geom.frag   # CRT geometry fragment shader
    ├── lcd.vert        # LCD vertex shader
    ├── lcd.frag        # LCD fragment shader
    └── scanlines.frag  # Scanlines fragment shader
```

### BGFX Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BGFXBackend {
    Auto,           // Auto-detect best backend
    OpenGL,         // OpenGL rendering
    DirectX11,      // DirectX 11
    DirectX12,      // DirectX 12
    Vulkan,         // Vulkan
    Metal,          // Metal (macOS)
    Gnm,            // PlayStation 4
    Nvn,            // Nintendo Switch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BGFXOptions {
    pub debug: bool,              // Debug mode
    pub profile: bool,            // Profile mode
    pub vsync: bool,              // V-Sync
    pub max_frame_latency: u32,   // Frame latency
    pub gamma: f32,               // Gamma correction
    pub brightness: f32,          // Brightness adjustment
    pub contrast: f32,            // Contrast adjustment
    pub saturation: f32,          // Saturation adjustment
}
```

## 🎨 Shader Presets

### 1. CRT Classic
**Deskripsi**: Simulasi monitor arcade dengan scanlines dan curvature
**Parameter**:
- `curvature`: 4.0 - Intensitas curvature CRT
- `scanlines`: 0.5 - Intensitas scanlines
- `phosphor`: 0.3 - Efek phosphor persistence

**GLSL Fragment Shader**:
```glsl
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
```

### 2. LCD Sharp
**Deskripsi**: Simulasi LCD dengan pixel grid yang tajam
**Parameter**:
- `pixel_size`: 1.0 - Ukuran pixel grid
- `sharpness`: 0.8 - Tingkat ketajaman

### 3. Retro Scanlines
**Deskripsi**: Efek scanlines sederhana
**Parameter**:
- `intensity`: 0.7 - Intensitas scanlines
- `frequency`: 2.0 - Frekuensi scanlines

## 🔧 Konfigurasi

### Video Settings Dialog
Dialog video settings yang diperluas mencakup:

1. **Basic Settings**
   - Video backend selection
   - Window mode options
   - V-Sync settings
   - Filter options
   - Prescale settings

2. **BGFX Configuration**
   - Backend selection (Auto, OpenGL, DirectX, Vulkan, etc.)
   - Debug dan profile mode
   - Frame latency control
   - Color adjustments (gamma, brightness, contrast, saturation)

3. **Graphics Presets**
   - Preset selection
   - Preset description
   - Shader chain configuration

4. **Shader Configuration**
   - Shader selection
   - Parameter adjustment
   - Real-time preview

5. **Paths Configuration**
   - BGFX path setup
   - Shader path setup

### Command Line Arguments
MAMEUIx secara otomatis menghasilkan argument command line untuk MAME:

```bash
# Contoh argument yang dihasilkan
mame game_name \
  -bgfx_backend vulkan \
  -bgfx_screen_chains crt-geom \
  -bgfx_gamma 1.2 \
  -bgfx_brightness 1.1 \
  -bgfx_contrast 1.1 \
  -bgfx_saturation 1.0 \
  -filter 1 \
  -prescale 1
```

## 🚀 Penggunaan

### 1. Setup BGFX
```rust
// Inisialisasi BGFX manager
let bgfx_manager = BGFXManager::new(PathBuf::from("/path/to/bgfx"));

// Validasi instalasi
bgfx_manager.validate_installation()?;

// Generate arguments
let options = HashMap::new();
let args = bgfx_manager.generate_args("vulkan", &options);
```

### 2. Setup Shader Manager
```rust
// Inisialisasi shader manager
let shader_manager = ShaderManager::new(PathBuf::from("/path/to/shaders"));

// Load shader
let shader_info = shader_manager.load_shader("crt-geom")?;

// Generate BGFX chain
let chain_config = shader_manager.generate_bgfx_chain("crt-geom")?;
```

### 3. Graphics Configuration
```rust
// Load graphics configuration
let mut graphics_config = GraphicsConfig::default();

// Set preset
graphics_config.global_preset = "CRT Classic".to_string();

// Generate MAME arguments
let bgfx_args = graphics_config.generate_bgfx_args();
```

## 📁 File Structure

### Shader Directory Structure
```
shaders/
├── crt-geom/
│   ├── vertex.glsl      # Vertex shader
│   ├── fragment.glsl    # Fragment shader
│   └── parameters.json  # Shader parameters
├── lcd-sharp/
│   ├── vertex.glsl
│   ├── fragment.glsl
│   └── parameters.json
└── scanlines/
    ├── fragment.glsl
    └── parameters.json
```

### Parameters JSON Format
```json
{
  "curvature": 4.0,
  "scanlines": 0.5,
  "phosphor": 0.3
}
```

## 🎮 Preset Examples

### Original Preset
```rust
GraphicsPreset {
    name: "Original".to_string(),
    description: "Raw pixels without filtering".to_string(),
    shader_chain: None,
    filter: false,
    prescale: 1,
    bgfx_backend: BGFXBackend::Auto,
    bgfx_options: BGFXOptions::default(),
}
```

### CRT Classic Preset
```rust
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
}
```

## 🔍 Troubleshooting

### Common Issues

1. **BGFX Backend Not Found**
   - Pastikan BGFX terinstall dengan benar
   - Periksa path BGFX di settings
   - Validasi instalasi dengan `BGFXManager::validate_installation()`

2. **Shader Compilation Error**
   - Periksa syntax GLSL
   - Pastikan semua uniform variables didefinisikan
   - Validasi shader dengan `ShaderManager::validate_shader()`

3. **Performance Issues**
   - Gunakan backend yang sesuai dengan hardware
   - Kurangi prescale jika diperlukan
   - Nonaktifkan debug mode untuk production

### Debug Mode
```rust
// Enable debug mode
preset.bgfx_options.debug = true;

// Check generated arguments
let args = graphics_config.generate_bgfx_args();
println!("Generated args: {:?}", args);
```

## 📈 Performance Optimization

### Backend Selection
- **Vulkan**: Best performance pada GPU modern
- **DirectX 12**: Optimal untuk Windows dengan GPU modern
- **OpenGL**: Compatible dengan semua platform
- **Metal**: Optimal untuk macOS

### Shader Optimization
- Gunakan texture sampling yang efisien
- Minimalkan conditional statements
- Optimalkan loop dalam shader
- Gunakan appropriate precision qualifiers

## 🔮 Future Enhancements

1. **Real-time Shader Editor**
   - Live shader editing
   - Hot reloading
   - Visual shader graph

2. **Advanced Effects**
   - Bloom effects
   - Motion blur
   - Depth of field
   - Anti-aliasing

3. **Shader Marketplace**
   - Community shader sharing
   - Rating system
   - Automatic updates

4. **Performance Profiling**
   - Frame time analysis
   - GPU utilization monitoring
   - Bottleneck detection

## 📚 References

- [MAME BGFX Documentation](https://docs.mamedev.org/advanced/bgfx.html)
- [MAME GLSL Documentation](https://docs.mamedev.org/advanced/glsl.html)
- [BGFX GitHub Repository](https://github.com/bkaradzic/bgfx)
- [OpenGL Shading Language](https://www.khronos.org/opengl/wiki/OpenGL_Shading_Language) 