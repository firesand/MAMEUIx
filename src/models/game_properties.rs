use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProperties {
    pub game_name: String,
    pub display: DisplayProperties,
    pub advanced: AdvancedProperties,
    pub screen: ScreenProperties,
    pub sound: SoundProperties,
    pub miscellaneous: MiscProperties,
    pub sdl_options: SDLDriverOptions,  // New field for SDL options
    pub osd_options: OSDOptions,        // New field for OSD options
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayProperties {
    pub run_in_window: bool,
    pub start_out_maximized: bool,
    pub enforce_aspect_ratio: bool,
    pub throttle: bool,
    pub bitmap_prescaling: u8,
    pub gamma_correction: f32,
    pub brightness_correction: f32,
    pub contrast_correction: f32,
    pub pause_brightness: f32,
    pub use_bilinear_filtering: bool,
    pub update_main_window_during_pause: bool,
    pub video_mode: VideoMode,
    pub rotation: RotationMode,
    pub flip_screen_upside_down: bool,
    pub flip_screen_left_right: bool,
    pub use_non_integer_scaling: bool,
    pub stretch_only_x_axis: bool,
    pub stretch_only_y_axis: bool,
    pub auto_select_stretch_axis: bool,
    pub overscan_on_targets: bool,
    pub horizontal_scale_factor: u8,
    pub vertical_scale_factor: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VideoMode {
    Auto,
    OpenGL,
    Direct3D,
    Software,
    BGFX,
}

impl std::fmt::Display for VideoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VideoMode::Auto => write!(f, "Auto"),
            VideoMode::OpenGL => write!(f, "OpenGL"),
            VideoMode::Direct3D => write!(f, "Direct3D"),
            VideoMode::Software => write!(f, "Software"),
            VideoMode::BGFX => write!(f, "BGFX"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RotationMode {
    Default,
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl std::fmt::Display for RotationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RotationMode::Default => write!(f, "Default"),
            RotationMode::Rotate0 => write!(f, "0"),
            RotationMode::Rotate90 => write!(f, "90"),
            RotationMode::Rotate180 => write!(f, "180"),
            RotationMode::Rotate270 => write!(f, "270"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedProperties {
    // OpenGL settings
    pub enable_glsl: bool,
    pub glsl_filter: GLSLFilter,
    pub force_power_of_two_textures: bool,
    pub dont_use_gl_arb_texture_rectangle: bool,
    pub enable_vbo: bool,
    pub enable_pbo: bool,
    
    // GLSL Shader paths (10 slots each for mame and screen)
    // Use Vec<String> instead of Vec<Option<String>> to avoid TOML serialization issues
    // Empty strings represent None values
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub glsl_shader_mame: Vec<String>,  // 0-9, empty string = None
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub glsl_shader_screen: Vec<String>, // 0-9, empty string = None
    
    // BGFX settings
    pub bgfx_settings: BGFXSettings,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GLSLFilter {
    Plain,      // 0
    Bilinear,   // 1 (default)
    Bicubic,    // 2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BGFXSettings {
    pub screen_chains: String,
    pub backend: BGFXBackend,
    pub enable_debug: bool,
    pub shadow_mask: Option<String>,
    pub lut_texture: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum BGFXBackend {
    Auto,
    D3D9,
    D3D11,
    D3D12,
    OpenGL,
    Metal,
    Vulkan,
}

impl std::fmt::Display for BGFXBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BGFXBackend::Auto => write!(f, "Auto"),
            BGFXBackend::D3D9 => write!(f, "D3D9"),
            BGFXBackend::D3D11 => write!(f, "D3D11"),
            BGFXBackend::D3D12 => write!(f, "D3D12"),
            BGFXBackend::OpenGL => write!(f, "OpenGL"),
            BGFXBackend::Metal => write!(f, "Metal"),
            BGFXBackend::Vulkan => write!(f, "Vulkan"),
        }
    }
}

impl BGFXBackend {
    /// Get available backends for current platform
    pub fn available_backends() -> Vec<BGFXBackend> {
        #[cfg(target_os = "linux")]
        {
            vec![
                BGFXBackend::Auto,
                BGFXBackend::OpenGL,
                BGFXBackend::Vulkan,
            ]
        }
        
        #[cfg(target_os = "windows")]
        {
            vec![
                BGFXBackend::Auto,
                BGFXBackend::D3D9,
                BGFXBackend::D3D11,
                BGFXBackend::D3D12,
                BGFXBackend::OpenGL,
                BGFXBackend::Vulkan,
            ]
        }
        
        #[cfg(target_os = "macos")]
        {
            vec![
                BGFXBackend::Auto,
                BGFXBackend::Metal,
                BGFXBackend::OpenGL,
            ]
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            vec![
                BGFXBackend::Auto,
                BGFXBackend::OpenGL,
            ]
        }
    }
    
    /// Check if this backend is available on current platform
    pub fn is_available(&self) -> bool {
        Self::available_backends().contains(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenProperties {
    pub triple_buffering: bool,
    pub sync_to_monitor_refresh: bool,
    pub wait_for_vertical_sync: bool,
    pub refresh_speed: bool,
    pub low_latency: bool,
    pub frame_skipping: FrameSkipping,
    pub emulation_speed: f32,
    pub effect: Option<String>,
    pub full_screen_gamma: f32,
    pub full_screen_brightness: f32,
    pub full_screen_contrast: f32,
    pub seconds_to_run: u32,
    // Core Performance Options
    pub auto_frameskip: bool,      // -autoframeskip
    pub frameskip_value: u8,       // -frameskip (0-10)
    pub sleep_when_idle: bool,     // -sleep
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSkipping {
    pub automatic: bool,
    pub draw_every_frame: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundProperties {
    pub sound_mode: SoundMode,
    pub use_samples: bool,
    pub sample_rate: u32,
    pub volume_attenuation: i8,
    pub audio_latency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SoundMode {
    Auto,
    SDL,
    PortAudio,
    PulseAudio,
    None,
}

impl std::fmt::Display for SoundMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoundMode::Auto => write!(f, "Auto"),
            SoundMode::SDL => write!(f, "SDL"),
            SoundMode::PortAudio => write!(f, "PortAudio"),
            SoundMode::PulseAudio => write!(f, "PulseAudio"),
            SoundMode::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscProperties {
    pub num_screens: u8,
    pub screen_number: ScreenSelection,
    pub resolution: Resolution,
    pub view: ViewSelection,
    pub autoselect_aspect: bool,
    pub aspect_ratio: (u8, u8),
    pub switch_resolutions_to_fit: bool,
    pub custom_args: String,
    // Performance options
    pub num_processors: Option<u32>,  // None means use system default, Some(n) means override
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScreenSelection {
    Default,
    Screen(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Resolution {
    Auto,
    Custom(u32, u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewSelection {
    Auto,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDLDriverOptions {
    pub video_driver: Option<String>,      // x11, wayland, directfb, auto
    pub render_driver: Option<String>,     // software, opengl, opengles2, auto
    pub audio_driver: Option<String>,      // alsa, pulse, jack, auto
    pub gl_lib: Option<String>,           // path to alternative libGL.so
    
    // SDL Performance Options
    pub show_video_fps: bool,
    
    // SDL Video Options
    pub center_horizontal: bool,
    pub center_vertical: bool,
    pub scale_mode: SDLScaleMode,
    
    // SDL Full Screen Options
    pub use_all_heads: bool,
    pub attach_window: Option<String>,
    
    // SDL Keyboard Mapping
    pub enable_keymap: bool,
    pub keymap_file: Option<String>,
    
    // SDL Input Options
    pub enable_touch: bool,
    pub sixaxis_support: bool,
    pub dual_lightgun: bool,
    
    // SDL Lightgun Mapping (8 slots)
    // Use Vec<String> instead of Vec<Option<String>> to avoid TOML serialization issues
    // Empty strings represent None values
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lightgun_mappings: Vec<String>, // Empty string = None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SDLScaleMode {
    None,
    HWBlit,
    HWBest,
    YV12,
    YUY2,
    YV12x2,
    YUY2x2,
}

impl std::fmt::Display for SDLScaleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SDLScaleMode::None => write!(f, "None"),
            SDLScaleMode::HWBlit => write!(f, "Hardware Blit"),
            SDLScaleMode::HWBest => write!(f, "Hardware Best"),
            SDLScaleMode::YV12 => write!(f, "YV12"),
            SDLScaleMode::YUY2 => write!(f, "YUY2"),
            SDLScaleMode::YV12x2 => write!(f, "YV12 x2"),
            SDLScaleMode::YUY2x2 => write!(f, "YUY2 x2"),
        }
    }
}

impl Default for SDLDriverOptions {
    fn default() -> Self {
        Self {
            video_driver: None,    // None means don't pass the option (use SDL default)
            render_driver: None,
            audio_driver: None,
            gl_lib: None,
            
            // SDL Performance Options
            show_video_fps: false,
            
            // SDL Video Options
            center_horizontal: false,
            center_vertical: false,
            scale_mode: SDLScaleMode::None,
            
            // SDL Full Screen Options
            use_all_heads: false,
            attach_window: None,
            
            // SDL Keyboard Mapping
            enable_keymap: false,
            keymap_file: None,
            
            // SDL Input Options
            enable_touch: false,
            sixaxis_support: false,
            dual_lightgun: false,
            
            // SDL Lightgun Mapping
            lightgun_mappings: vec![String::new(); 8], // Empty strings instead of None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSDOptions {
    // Input Mapping
    pub ui_mode_key: Option<String>,
    pub controller_map_file: Option<String>,
    pub background_input: bool,
    
    // Providers
    pub ui_font_provider: OSDProvider,
    pub output_provider: OutputProvider,
    pub keyboard_provider: OSDProvider,
    pub mouse_provider: OSDProvider,
    pub lightgun_provider: LightgunProvider,
    pub joystick_provider: JoystickProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OSDProvider {
    Auto,
    SDL,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputProvider {
    None,
    Console,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LightgunProvider {
    Auto,
    SDL,
    X11,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JoystickProvider {
    Auto,
    SDLGame,
    SDLJoy,
    None,
}

impl Default for OSDOptions {
    fn default() -> Self {
        Self {
            ui_mode_key: None,  // None means use MAME default (ScrollLock)
            controller_map_file: None,
            background_input: false,
            ui_font_provider: OSDProvider::Auto,
            output_provider: OutputProvider::None,
            keyboard_provider: OSDProvider::Auto,
            mouse_provider: OSDProvider::Auto,
            lightgun_provider: LightgunProvider::Auto,
            joystick_provider: JoystickProvider::Auto,
        }
    }
}

impl Default for GameProperties {
    fn default() -> Self {
        Self {
            game_name: "Default Game".to_string(),
            display: DisplayProperties::default(),
            advanced: AdvancedProperties::default(),
            screen: ScreenProperties::default(),
            sound: SoundProperties::default(),
            miscellaneous: MiscProperties::default(),
            sdl_options: SDLDriverOptions::default(),
            osd_options: OSDOptions::default(),
        }
    }
}

impl Default for DisplayProperties {
    fn default() -> Self {
        Self {
            run_in_window: true,
            start_out_maximized: true,
            enforce_aspect_ratio: true,
            throttle: true,
            bitmap_prescaling: 1,
            gamma_correction: 1.0,
            brightness_correction: 1.0,
            contrast_correction: 1.0,
            pause_brightness: 0.65,
            use_bilinear_filtering: false,
            update_main_window_during_pause: false,
            video_mode: VideoMode::Auto,
            rotation: RotationMode::Default,
            flip_screen_upside_down: false,
            flip_screen_left_right: false,
            use_non_integer_scaling: true,
            stretch_only_x_axis: false,
            stretch_only_y_axis: false,
            auto_select_stretch_axis: false,
            overscan_on_targets: false,
            horizontal_scale_factor: 0,
            vertical_scale_factor: 0,
        }
    }
}

impl Default for AdvancedProperties {
    fn default() -> Self {
        Self {
            enable_glsl: false,
            glsl_filter: GLSLFilter::Bilinear,
            force_power_of_two_textures: false,
            dont_use_gl_arb_texture_rectangle: true, // default on per MAME
            enable_vbo: true,  // default on per MAME
            enable_pbo: true,  // default on per MAME
            glsl_shader_mame: vec![String::new(); 10], // Empty strings instead of None
            glsl_shader_screen: vec![String::new(); 10], // Empty strings instead of None
            bgfx_settings: BGFXSettings::default(),
        }
    }
}

impl Default for BGFXSettings {
    fn default() -> Self {
        Self {
            screen_chains: "default".to_string(),
            backend: BGFXBackend::Auto,
            enable_debug: false,
            shadow_mask: None,
            lut_texture: None,
        }
    }
}

impl Default for ScreenProperties {
    fn default() -> Self {
        Self {
            triple_buffering: false,
            sync_to_monitor_refresh: false,
            wait_for_vertical_sync: false,
            refresh_speed: false,
            low_latency: false,
            frame_skipping: FrameSkipping::default(),
            emulation_speed: 1.0,
            effect: None,
            full_screen_gamma: 1.10,
            full_screen_brightness: 1.00,
            full_screen_contrast: 1.00,
            seconds_to_run: 0,
            // Core Performance Options defaults
            auto_frameskip: false,
            frameskip_value: 0,
            sleep_when_idle: true,
        }
    }
}

impl Default for FrameSkipping {
    fn default() -> Self {
        Self {
            automatic: false,
            draw_every_frame: 1,
        }
    }
}

impl Default for SoundProperties {
    fn default() -> Self {
        Self {
            sound_mode: SoundMode::Auto,
            use_samples: true,
            sample_rate: 48000,
            volume_attenuation: 0,
            audio_latency: 0.0,
        }
    }
}

impl Default for MiscProperties {
    fn default() -> Self {
        Self {
            num_screens: 1,
            screen_number: ScreenSelection::Default,
            resolution: Resolution::Auto,
            view: ViewSelection::Auto,
            autoselect_aspect: true,
            aspect_ratio: (4, 3),
            switch_resolutions_to_fit: false,
            custom_args: String::new(),
            num_processors: None,  // Use system default
        }
    }
} 