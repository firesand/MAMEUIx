use crate::models::AppConfig;
use std::collections::HashMap;
use std::ffi::OsString;
use std::process::{Child, Command};

pub fn launch_game(
    rom_name: &str,
    config: &AppConfig,
) -> Result<Child, Box<dyn std::error::Error>> {
    if let Some(mame) = config.mame_executables.get(config.selected_mame_index) {
        let mut cmd = Command::new(&mame.path);

        cmd.args(rom_search_args(config));

        // Add MAME internal directories
        if let Some(cfg_path) = &config.cfg_path {
            cmd.arg("-cfg_directory")
                .arg(cfg_path.to_string_lossy().to_string());
        }

        if let Some(nvram_path) = &config.nvram_path {
            cmd.arg("-nvram_directory")
                .arg(nvram_path.to_string_lossy().to_string());
        }

        if let Some(input_path) = &config.input_path {
            cmd.arg("-input_directory")
                .arg(input_path.to_string_lossy().to_string());
        }

        if let Some(state_path) = &config.state_path {
            cmd.arg("-state_directory")
                .arg(state_path.to_string_lossy().to_string());
        }

        if let Some(diff_path) = &config.diff_path {
            cmd.arg("-diff_directory")
                .arg(diff_path.to_string_lossy().to_string());
        }

        if let Some(comment_path) = &config.comment_path {
            cmd.arg("-comment_directory")
                .arg(comment_path.to_string_lossy().to_string());
        }

        // Get game-specific properties or use defaults
        let game_properties = config
            .game_properties
            .get(rom_name)
            .cloned()
            .unwrap_or_else(|| config.default_game_properties.clone());

        // Apply game properties directly
        apply_game_properties(&mut cmd, &game_properties);

        // IMPORTANT: High score support
        // Enable the hiscore plugin if hiscore.dat is configured
        // Note: Temporarily disabled due to plugin loading issues
        // if let Some(hiscore_path) = &config.hiscore_dat_path {
        //     if hiscore_path.exists() {
        //         // Enable the hiscore plugin
        //         cmd.arg("-plugin").arg("hiscore");
        //
        //         // Note: The hiscore plugin automatically uses the plugin data folder
        //         (~/.mame/hi/) for storing high scores, no additional arguments needed
        //     }
        // }

        // Cheats support
        if let Some(cheats_path) = &config.cheats_path
            && cheats_path.exists()
        {
            cmd.arg("-cheat");
            cmd.arg("-cheatpath")
                .arg(cheats_path.to_string_lossy().to_string());
        }

        // Artwork paths
        if let Some(artwork_path) = &config.artwork_path {
            if artwork_path.exists() {
                cmd.arg("-artpath")
                    .arg(artwork_path.to_string_lossy().to_string());
            }
        } else {
            // Fallback to MAME installation directory if no custom path is configured
            if let Some(mame_dir) = std::path::Path::new(&mame.path).parent() {
                let artwork_path = mame_dir.join("artwork");
                if artwork_path.exists() {
                    cmd.arg("-artpath")
                        .arg(artwork_path.to_string_lossy().to_string());
                }
            }
        }

        // Sample paths
        if !config.sample_paths.is_empty() {
            let sample_paths = config
                .sample_paths
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join(";");
            cmd.arg("-samplepath").arg(&sample_paths);
        }

        // Additional MAME Search Paths
        if let Some(ctrlr_path) = &config.ctrlr_path
            && ctrlr_path.exists()
        {
            cmd.arg("-ctrlrpath")
                .arg(ctrlr_path.to_string_lossy().to_string());
        }

        if let Some(crosshair_path) = &config.crosshair_path
            && crosshair_path.exists()
        {
            cmd.arg("-crosshairpath")
                .arg(crosshair_path.to_string_lossy().to_string());
        }

        if let Some(font_path) = &config.font_path
            && font_path.exists()
        {
            cmd.arg("-fontpath")
                .arg(font_path.to_string_lossy().to_string());
        }

        if let Some(plugins_path) = &config.plugins_path
            && plugins_path.exists()
        {
            cmd.arg("-pluginspath")
                .arg(plugins_path.to_string_lossy().to_string());
        }

        if let Some(language_path) = &config.language_path
            && language_path.exists()
        {
            cmd.arg("-languagepath")
                .arg(language_path.to_string_lossy().to_string());
        }

        if let Some(sw_path) = &config.sw_path
            && sw_path.exists()
        {
            cmd.arg("-swpath")
                .arg(sw_path.to_string_lossy().to_string());
        }

        if let Some(hash_path) = &config.hash_path
            && hash_path.exists()
        {
            cmd.arg("-hashpath")
                .arg(hash_path.to_string_lossy().to_string());
        }

        // Use the same effective properties selected for the rest of this game.
        cmd.args(game_properties.miscellaneous.custom_args.split_whitespace());

        // Finally, add the ROM name
        cmd.arg(rom_name);

        // Debug: Print the full command
        #[cfg(debug_assertions)]
        {
            println!("Launching MAME with command:");
            println!("{:?}", cmd);
        }

        Ok(cmd.spawn()?)
    } else {
        Err("No MAME executable configured".into())
    }
}

/// Shared structured ROM/configuration overrides for launch and verification.
/// Keep their order and existence rules identical so both commands select the
/// same collection; unrelated custom launch arguments are deliberately separate.
pub(crate) fn rom_search_args(config: &AppConfig) -> Vec<OsString> {
    let mut args = Vec::new();
    let rom_paths = config
        .rom_paths
        .iter()
        .chain(config.extra_rom_dirs.iter())
        .chain(config.software_rom_paths.iter())
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(";");
    if !rom_paths.is_empty() {
        args.extend([OsString::from("-rompath"), OsString::from(rom_paths)]);
    }
    for (option, path) in [
        ("-inipath", &config.ini_path),
        ("-homepath", &config.home_path),
    ] {
        if let Some(path) = path
            && path.exists()
        {
            args.extend([OsString::from(option), path.as_os_str().to_owned()]);
        }
    }
    args
}

/// Immutable worker snapshot of the structured paths and the effective custom
/// path/configuration overrides. Per-game properties replace defaults, including
/// when their custom argument string is empty, just as they do in `launch_game`.
pub(crate) struct VerificationSearchPaths {
    configured: Vec<OsString>,
    defaults: Vec<OsString>,
    per_game: HashMap<String, Vec<OsString>>,
}

impl VerificationSearchPaths {
    pub(crate) fn new(config: &AppConfig) -> Self {
        Self {
            configured: rom_search_args(config),
            defaults: verification_custom_args(
                &config.default_game_properties.miscellaneous.custom_args,
            ),
            per_game: config
                .game_properties
                .iter()
                .map(|(name, properties)| {
                    (
                        name.clone(),
                        verification_custom_args(&properties.miscellaneous.custom_args),
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn for_game(&self, game_name: &str) -> Vec<OsString> {
        let mut args = self.configured.clone();
        args.extend(
            self.per_game
                .get(game_name)
                .unwrap_or(&self.defaults)
                .iter()
                .cloned(),
        );
        args
    }
}

fn verification_custom_args(custom_args: &str) -> Vec<OsString> {
    // Match launch_game's existing whitespace tokenization. Keep every relevant
    // occurrence in original order: MAME resolves aliases and the last option
    // wins. Custom paths, unlike structured INI/home paths, are not gated on
    // existence because the launcher forwards them unconditionally too.
    let mut args = Vec::new();
    let mut tokens = custom_args.split_whitespace();
    while let Some(option) = tokens.next() {
        match option {
            "-rompath" | "-rp" | "-biospath" | "-bp" | "-inipath" | "-homepath" => {
                args.push(OsString::from(option));
                if let Some(value) = tokens.next() {
                    args.push(OsString::from(value));
                }
            }
            "-readconfig" | "-rc" | "-noreadconfig" | "-norc" => {
                args.push(OsString::from(option));
            }
            // Verification must never inherit actions such as -bench,
            // -autoboot_script, -plugin, -http, or arbitrary launch options.
            _ => {}
        }
    }
    args
}

// Helper function to check if hiscore plugin is available
pub fn check_hiscore_plugin(mame_path: &str) -> bool {
    if let Ok(output) = Command::new(mame_path).arg("-showplugins").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("hiscore")
    } else {
        false
    }
}

// Helper function to verify MAME installation supports plugins
pub fn verify_plugin_support(mame_path: &str) -> Result<PluginSupport, String> {
    // First check if MAME supports plugins by looking at config
    match Command::new(mame_path).arg("-showconfig").output() {
        Ok(output) => {
            let config_output = String::from_utf8_lossy(&output.stdout);
            let mut support = PluginSupport::default();

            // Check if plugin system is enabled
            if config_output
                .lines()
                .any(|line| line.trim().starts_with("plugins") && line.trim().ends_with("1"))
            {
                support.has_plugin_support = true;

                // Check for specific plugins by reading plugin.ini
                let home_dir = std::env::var("HOME").unwrap_or_default();
                let plugin_ini_path = format!("{}/.mame/plugin.ini", home_dir);

                if let Ok(plugin_ini) = std::fs::read_to_string(&plugin_ini_path) {
                    if plugin_ini.contains("hiscore") {
                        support.hiscore_available = true;
                        support.available_plugins.push("hiscore".to_string());
                    }

                    if plugin_ini.contains("cheat") {
                        support.cheat_available = true;
                        support.available_plugins.push("cheat".to_string());
                    }

                    if plugin_ini.contains("autofire") {
                        support.autofire_available = true;
                        support.available_plugins.push("autofire".to_string());
                    }
                }
            }

            Ok(support)
        }
        Err(e) => Err(format!("Failed to check plugin support: {}", e)),
    }
}

#[derive(Default, Debug)]
pub struct PluginSupport {
    pub has_plugin_support: bool,
    pub hiscore_available: bool,
    pub cheat_available: bool,
    pub autofire_available: bool,
    pub available_plugins: Vec<String>,
}

// Helper function to apply game properties to MAME command
fn apply_game_properties(cmd: &mut Command, props: &crate::models::GameProperties) {
    // Window mode
    if props.display.run_in_window {
        cmd.arg("-window");
    }

    // Aspect ratio
    if props.display.enforce_aspect_ratio {
        cmd.arg("-keepaspect");
    }

    // Throttle
    if !props.display.throttle {
        cmd.arg("-nothrottle");
    }

    // Bitmap prescaling
    if props.display.bitmap_prescaling > 1 {
        cmd.arg("-prescale")
            .arg(props.display.bitmap_prescaling.to_string());
    }

    // Video mode
    match props.display.video_mode {
        crate::models::game_properties::VideoMode::OpenGL => {
            cmd.arg("-video").arg("opengl");
        }
        crate::models::game_properties::VideoMode::Direct3D => {
            cmd.arg("-video").arg("d3d");
        }
        crate::models::game_properties::VideoMode::BGFX => {
            cmd.arg("-video").arg("bgfx");
        }
        crate::models::game_properties::VideoMode::Software => {
            cmd.arg("-video").arg("soft");
        }
        crate::models::game_properties::VideoMode::Auto => {} // Let MAME decide
    }

    // CORRECTED: Rotation handling
    match props.display.rotation {
        crate::models::game_properties::RotationMode::Default => {
            // Don't add any rotation arguments - let MAME handle it automatically
        }
        crate::models::game_properties::RotationMode::Rotate0 => {
            // No rotation needed - this is the normal orientation
            // Don't add any arguments
        }
        crate::models::game_properties::RotationMode::Rotate90 => {
            cmd.arg("-ror"); // Rotate right (clockwise) 90 degrees
        }
        crate::models::game_properties::RotationMode::Rotate180 => {
            // For 180 degrees, we can either use -ror -ror or use flip X and Y
            // Using double rotation is more reliable
            cmd.arg("-ror");
            cmd.arg("-ror");
        }
        crate::models::game_properties::RotationMode::Rotate270 => {
            cmd.arg("-rol"); // Rotate left (counter-clockwise) 90 degrees
        }
    }

    // CORRECTED: Flip options (these work independently of rotation)
    if props.display.flip_screen_upside_down {
        cmd.arg("-flipy"); // Flip vertically
    }

    if props.display.flip_screen_left_right {
        cmd.arg("-flipx"); // Flip horizontally
    }

    // Auto rotation options
    if props.display.auto_rotate_right {
        cmd.arg("-autoror");
    }

    if props.display.auto_rotate_left {
        cmd.arg("-autorol");
    }

    // Core Configuration options
    if props.miscellaneous.write_config {
        cmd.arg("-writeconfig");
    }

    if props.miscellaneous.auto_save {
        cmd.arg("-autosave");
    }

    if props.miscellaneous.rewind {
        cmd.arg("-rewind");
    }

    if props.miscellaneous.exit_after {
        cmd.arg("-exit_after");
    }

    if props.miscellaneous.bilinear {
        cmd.arg("-bilinear");
    }

    if props.miscellaneous.burnin {
        cmd.arg("-burnin");
    }

    if props.miscellaneous.crop {
        cmd.arg("-crop");
    }

    // Core Input options
    if props.miscellaneous.multi_keyboard {
        cmd.arg("-multikeyboard");
    }

    if props.miscellaneous.multi_mouse {
        cmd.arg("-multimouse");
    }

    if props.miscellaneous.steady_key {
        cmd.arg("-steadykey");
    }

    if props.miscellaneous.ui_active {
        cmd.arg("-ui_active");
    }

    if props.miscellaneous.offscreen_reload {
        cmd.arg("-offscreen_reload");
    }

    if props.miscellaneous.contradictory {
        cmd.arg("-joystick_contradictory");
    }

    if props.miscellaneous.natural {
        cmd.arg("-natural");
    }

    // Core Debug options
    if props.miscellaneous.verbose {
        cmd.arg("-verbose");
    }

    if props.miscellaneous.log {
        cmd.arg("-log");
    }

    if props.miscellaneous.oslog {
        cmd.arg("-oslog");
    }

    if props.miscellaneous.debug {
        cmd.arg("-debug");
    }

    if props.miscellaneous.update_pause {
        cmd.arg("-update_in_pause");
    }

    if props.miscellaneous.debuglog {
        cmd.arg("-debuglog");
    }

    if props.miscellaneous.drc_c {
        cmd.arg("-drc_use_c");
    }

    if props.miscellaneous.log_uml {
        cmd.arg("-drc_log_uml");
    }

    if props.miscellaneous.log_native {
        cmd.arg("-drc_log_native");
    }

    if props.miscellaneous.cheat {
        cmd.arg("-cheat");
    }

    if props.miscellaneous.skip {
        cmd.arg("-skip_gameinfo");
    }

    if props.miscellaneous.confirm {
        cmd.arg("-confirm_quit");
    }

    if props.miscellaneous.console {
        cmd.arg("-console");
    }

    if props.miscellaneous.switchres {
        cmd.arg("-switchres");
    }

    // Number of processors
    if let Some(num_procs) = props.miscellaneous.num_processors {
        cmd.arg("-numprocessors").arg(num_procs.to_string());
    }

    // Number of screens
    if props.miscellaneous.num_screens > 1 {
        cmd.arg("-numscreens")
            .arg(props.miscellaneous.num_screens.to_string());
    }

    // Screen selection
    match props.miscellaneous.screen_number {
        crate::models::game_properties::ScreenSelection::Screen(n) => {
            cmd.arg("-screen").arg(n.to_string());
        }
        crate::models::game_properties::ScreenSelection::Default => {
            // Use default screen
        }
    }

    // Resolution
    match &props.miscellaneous.resolution {
        crate::models::game_properties::Resolution::Custom(width, height) => {
            cmd.arg("-resolution").arg(format!("{}x{}", width, height));
        }
        crate::models::game_properties::Resolution::Auto => {
            // Use auto resolution
        }
    }

    // Switch resolutions to fit
    if props.miscellaneous.switch_resolutions_to_fit {
        cmd.arg("-switchres");
    }

    // Aspect ratio (if not auto-selecting)
    if !props.miscellaneous.autoselect_aspect {
        let (x, y) = props.miscellaneous.aspect_ratio;
        cmd.arg("-aspect").arg(format!("{}:{}", x, y));
    }

    // BGFX-specific options (when using BGFX video mode)
    if matches!(
        props.display.video_mode,
        crate::models::game_properties::VideoMode::BGFX
    ) {
        let bgfx = &props.advanced.bgfx_settings;

        // BGFX backend
        match bgfx.backend {
            crate::models::game_properties::BGFXBackend::Auto => {
                // Don't specify backend, let MAME auto-detect
            }
            crate::models::game_properties::BGFXBackend::D3D9 => {
                cmd.arg("-bgfx_backend").arg("d3d9");
            }
            crate::models::game_properties::BGFXBackend::D3D11 => {
                cmd.arg("-bgfx_backend").arg("d3d11");
            }
            crate::models::game_properties::BGFXBackend::D3D12 => {
                cmd.arg("-bgfx_backend").arg("d3d12");
            }
            crate::models::game_properties::BGFXBackend::OpenGL => {
                cmd.arg("-bgfx_backend").arg("opengl");
            }
            crate::models::game_properties::BGFXBackend::Metal => {
                cmd.arg("-bgfx_backend").arg("metal");
            }
            crate::models::game_properties::BGFXBackend::Vulkan => {
                cmd.arg("-bgfx_backend").arg("vulkan");
            }
        }

        // BGFX screen chains (shader chains)
        if !bgfx.screen_chains.is_empty() && bgfx.screen_chains != "default" {
            cmd.arg("-bgfx_screen_chains").arg(&bgfx.screen_chains);
        }

        // BGFX debug
        if bgfx.enable_debug {
            cmd.arg("-bgfx_debug");
        }

        // Shadow mask
        if let Some(shadow_mask) = &bgfx.shadow_mask
            && !shadow_mask.is_empty()
        {
            cmd.arg("-bgfx_shadow_mask").arg(shadow_mask);
        }

        // LUT (Look-Up Table)
        if let Some(lut) = &bgfx.lut_texture
            && !lut.is_empty()
        {
            cmd.arg("-bgfx_lut").arg(lut);
        }
    }

    // Wait vsync
    if props.screen.wait_for_vertical_sync {
        cmd.arg("-waitvsync");
    }

    // Sound
    match props.sound.sound_mode {
        crate::models::game_properties::SoundMode::None => {
            cmd.arg("-sound").arg("none");
        }
        crate::models::game_properties::SoundMode::SDL => {
            cmd.arg("-sound").arg("sdl");
        }
        _ => {} // Auto or others
    }

    // Samples
    if props.sound.use_samples {
        cmd.arg("-samples");
    }

    // Sample rate
    if props.sound.sample_rate != 48000 {
        cmd.arg("-samplerate")
            .arg(props.sound.sample_rate.to_string());
    }

    // Volume
    if props.sound.volume_attenuation != 0 {
        cmd.arg("-volume")
            .arg(props.sound.volume_attenuation.to_string());
    }

    // Core Performance Options
    if props.screen.auto_frameskip {
        cmd.arg("-autoframeskip");
    }

    if props.screen.frameskip_value > 0 {
        cmd.arg("-frameskip")
            .arg(props.screen.frameskip_value.to_string());
    }

    if props.screen.sleep_when_idle {
        cmd.arg("-sleep");
    }

    // Emulation speed (already handled via emulation_speed field)
    if (props.screen.emulation_speed - 1.0).abs() > 0.01 {
        cmd.arg("-speed")
            .arg(format!("{:.2}", props.screen.emulation_speed));
    }

    // Seconds to run
    if props.screen.seconds_to_run > 0 {
        cmd.arg("-seconds_to_run")
            .arg(props.screen.seconds_to_run.to_string());
    }

    // Refresh speed
    if props.screen.refresh_speed {
        cmd.arg("-refreshspeed");
    }

    // Low latency
    if props.screen.low_latency {
        cmd.arg("-lowlatency");
    }

    // Apply SDL driver options (Linux/Unix specific)
    if let Some(video_driver) = &props.sdl_options.video_driver
        && !video_driver.is_empty()
        && video_driver != "auto"
    {
        cmd.arg("-videodriver").arg(video_driver);
    }

    if let Some(render_driver) = &props.sdl_options.render_driver
        && !render_driver.is_empty()
        && render_driver != "auto"
    {
        cmd.arg("-renderdriver").arg(render_driver);
    }

    if let Some(audio_driver) = &props.sdl_options.audio_driver
        && !audio_driver.is_empty()
        && audio_driver != "auto"
    {
        cmd.arg("-audiodriver").arg(audio_driver);
    }

    if let Some(gl_lib) = &props.sdl_options.gl_lib
        && !gl_lib.is_empty()
        && gl_lib != "auto"
    {
        cmd.arg("-gl_lib").arg(gl_lib);
    }

    // SDL Performance Options
    if props.sdl_options.show_video_fps {
        cmd.arg("-sdlvideofps");
    }

    // SDL Video Options
    if props.sdl_options.center_horizontal {
        cmd.arg("-centerh");
    }

    if props.sdl_options.center_vertical {
        cmd.arg("-centerv");
    }

    if props.sdl_options.scale_mode != crate::models::game_properties::SDLScaleMode::None {
        cmd.arg("-scalemode");
        let scale_mode_arg = match props.sdl_options.scale_mode {
            crate::models::game_properties::SDLScaleMode::HWBlit => "hwblit",
            crate::models::game_properties::SDLScaleMode::HWBest => "hwbest",
            crate::models::game_properties::SDLScaleMode::YV12 => "yv12",
            crate::models::game_properties::SDLScaleMode::YUY2 => "yuy2",
            crate::models::game_properties::SDLScaleMode::YV12x2 => "yv12x2",
            crate::models::game_properties::SDLScaleMode::YUY2x2 => "yuy2x2",
            crate::models::game_properties::SDLScaleMode::None => "none",
        };
        cmd.arg(scale_mode_arg);
    }

    // SDL Full Screen Options
    if props.sdl_options.use_all_heads {
        cmd.arg("-useallheads");
    }

    if let Some(window_id) = &props.sdl_options.attach_window {
        cmd.arg("-attach_window").arg(window_id);
    }

    // SDL Keyboard Mapping
    if props.sdl_options.enable_keymap {
        cmd.arg("-keymap");
        if let Some(keymap_file) = &props.sdl_options.keymap_file {
            cmd.arg("-keymap_file").arg(keymap_file);
        }
    }

    // SDL Input Options
    if props.sdl_options.enable_touch {
        cmd.arg("-enable_touch");
    }

    if props.sdl_options.sixaxis_support {
        cmd.arg("-sixaxis");
    }

    if props.sdl_options.dual_lightgun {
        cmd.arg("-dual_lightgun");
    }

    // SDL Lightgun Mappings
    for (i, mapping) in props.sdl_options.lightgun_mappings.iter().enumerate() {
        if !mapping.is_empty() {
            cmd.arg(format!("-lightgun_index{}", i + 1)).arg(mapping);
        }
    }

    // OSD Options
    if let Some(ui_key) = &props.osd_options.ui_mode_key {
        cmd.arg("-uimodekey").arg(ui_key);
    }

    if let Some(controller_map) = &props.osd_options.controller_map_file {
        cmd.arg("-controller_map").arg(controller_map);
    }

    if props.osd_options.background_input {
        cmd.arg("-background_input");
    }

    // Providers
    match props.osd_options.keyboard_provider {
        crate::models::game_properties::OSDProvider::SDL => {
            cmd.arg("-keyboardprovider").arg("sdl");
        }
        crate::models::game_properties::OSDProvider::None => {
            cmd.arg("-keyboardprovider").arg("none");
        }
        crate::models::game_properties::OSDProvider::Auto => {} // Don't specify, use MAME default
    };

    match props.osd_options.mouse_provider {
        crate::models::game_properties::OSDProvider::SDL => {
            cmd.arg("-mouseprovider").arg("sdl");
        }
        crate::models::game_properties::OSDProvider::None => {
            cmd.arg("-mouseprovider").arg("none");
        }
        crate::models::game_properties::OSDProvider::Auto => {}
    };

    match props.osd_options.lightgun_provider {
        crate::models::game_properties::LightgunProvider::SDL => {
            cmd.arg("-lightgunprovider").arg("sdl");
        }
        crate::models::game_properties::LightgunProvider::X11 => {
            cmd.arg("-lightgunprovider").arg("x11");
        }
        crate::models::game_properties::LightgunProvider::None => {
            cmd.arg("-lightgunprovider").arg("none");
        }
        crate::models::game_properties::LightgunProvider::Auto => {}
    };

    match props.osd_options.joystick_provider {
        crate::models::game_properties::JoystickProvider::SDLGame => {
            cmd.arg("-joystickprovider").arg("sdlgame");
        }
        crate::models::game_properties::JoystickProvider::SDLJoy => {
            cmd.arg("-joystickprovider").arg("sdljoy");
        }
        crate::models::game_properties::JoystickProvider::None => {
            cmd.arg("-joystickprovider").arg("none");
        }
        crate::models::game_properties::JoystickProvider::Auto => {}
    };

    match props.osd_options.ui_font_provider {
        crate::models::game_properties::OSDProvider::SDL => {
            cmd.arg("-uifontprovider").arg("sdl");
        }
        crate::models::game_properties::OSDProvider::None => {
            cmd.arg("-uifontprovider").arg("none");
        }
        crate::models::game_properties::OSDProvider::Auto => {}
    };

    match props.osd_options.output_provider {
        crate::models::game_properties::OutputProvider::Console => {
            cmd.arg("-output").arg("console");
        }
        crate::models::game_properties::OutputProvider::Network => {
            cmd.arg("-output").arg("network");
        }
        crate::models::game_properties::OutputProvider::None => {
            cmd.arg("-output").arg("none");
        }
    };

    #[cfg(debug_assertions)]
    println!("Applied game properties for {}", props.game_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_args_preserve_root_order_and_configuration_overrides() {
        let temp = tempfile::tempdir().unwrap();
        let ini = temp.path().join("ini with spaces");
        let home = temp.path().join("home");
        std::fs::create_dir(&ini).unwrap();
        std::fs::create_dir(&home).unwrap();
        let config = AppConfig {
            rom_paths: vec![
                "/collection/arcade one".into(),
                "/collection/arcade two".into(),
            ],
            extra_rom_dirs: vec!["/collection/extra".into()],
            software_rom_paths: vec!["/collection/software".into()],
            ini_path: Some(ini.clone()),
            home_path: Some(home.clone()),
            ..Default::default()
        };
        assert_eq!(
            rom_search_args(&config),
            vec![
                OsString::from("-rompath"),
                OsString::from(
                    "/collection/arcade one;/collection/arcade two;/collection/extra;/collection/software"
                ),
                OsString::from("-inipath"),
                ini.into_os_string(),
                OsString::from("-homepath"),
                home.into_os_string(),
            ]
        );
    }

    #[test]
    fn absent_search_overrides_preserve_mame_defaults() {
        let temp = tempfile::tempdir().unwrap();
        let config = AppConfig {
            ini_path: Some(temp.path().join("missing")),
            home_path: Some(temp.path().join("also-missing")),
            ..Default::default()
        };
        assert!(rom_search_args(&config).is_empty());
    }

    #[test]
    fn custom_search_overrides_preserve_aliases_order_and_exclude_launch_actions() {
        let args = verification_custom_args(
            "-bench 60 -rompath first -plugin hiscore -rp second -biospath third -bp final \
             -inipath custom-ini -homepath custom-home -readconfig -norc -rc -noreadconfig \
             -autoboot_script boot.lua -http",
        );
        let expected: Vec<OsString> = [
            "-rompath",
            "first",
            "-rp",
            "second",
            "-biospath",
            "third",
            "-bp",
            "final",
            "-inipath",
            "custom-ini",
            "-homepath",
            "custom-home",
            "-readconfig",
            "-norc",
            "-rc",
            "-noreadconfig",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        assert_eq!(args, expected);
    }

    #[test]
    fn search_snapshot_uses_per_game_custom_overrides_without_default_leakage() {
        let mut config = AppConfig {
            rom_paths: vec!["structured".into()],
            ..Default::default()
        };
        config.default_game_properties.miscellaneous.custom_args = "-rp default -norc".into();
        let mut per_game = config.default_game_properties.clone();
        per_game.miscellaneous.custom_args = "-bp game-specific -readconfig -bench 60".into();
        config.game_properties.insert("pacman".into(), per_game);
        config
            .game_properties
            .insert("empty".into(), Default::default());
        let snapshot = VerificationSearchPaths::new(&config);
        // A worker must retain the settings from when it started.
        config.default_game_properties.miscellaneous.custom_args = "-rp changed".into();
        for (name, suffix) in [
            ("pacman", vec!["-bp", "game-specific", "-readconfig"]),
            ("puckman", vec!["-rp", "default", "-norc"]),
            ("empty", vec![]),
        ] {
            let expected: Vec<OsString> = ["-rompath", "structured"]
                .into_iter()
                .chain(suffix)
                .map(OsString::from)
                .collect();
            assert_eq!(snapshot.for_game(name), expected, "{name}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn launch_uses_per_game_arguments_and_falls_back_to_defaults() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let executable = temp.path().join("mame-stub");
        std::fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$(dirname \"$0\")/args\"\n",
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let mut config = AppConfig {
            mame_executables: vec![crate::models::MameExecutable {
                name: "Test MAME".into(),
                path: executable.to_string_lossy().into_owned(),
                version: "test".into(),
                total_games: 0,
                working_games: 0,
            }],
            rom_paths: vec![temp.path().join("roms with spaces")],
            ..Default::default()
        };
        config.default_game_properties.miscellaneous.custom_args = "-bench 10".into();
        let mut per_game = config.default_game_properties.clone();
        per_game.miscellaneous.custom_args = "-bench 60".into();
        config.game_properties.insert("pacman".into(), per_game);

        for (game, duration) in [("pacman", "60"), ("puckman", "10")] {
            assert!(
                launch_game(game, &config)
                    .unwrap()
                    .wait()
                    .unwrap()
                    .success()
            );
            let captured = std::fs::read_to_string(temp.path().join("args")).unwrap();
            let args: Vec<_> = captured.lines().collect();
            let bench = args.iter().position(|arg| *arg == "-bench").unwrap();
            assert_eq!(args[bench + 1], duration);
            assert_eq!(args.iter().filter(|arg| **arg == "-bench").count(), 1);
            assert_eq!(args.last(), Some(&game));
            let expected_paths: Vec<_> = rom_search_args(&config)
                .into_iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            assert_eq!(&args[..expected_paths.len()], expected_paths.as_slice());
        }
    }

    #[test]
    #[ignore = "set MAMEUIX_TEST_MAME to an installed MAME executable"]
    fn native_mame_launch_applies_per_game_benchmark_and_exits() {
        let executable = std::env::var("MAMEUIX_TEST_MAME").expect("set MAMEUIX_TEST_MAME");
        let temp = tempfile::tempdir().unwrap();
        let mut config = AppConfig {
            mame_executables: vec![crate::models::MameExecutable {
                name: "Native test MAME".into(),
                path: executable,
                version: "native".into(),
                total_games: 0,
                working_games: 0,
            }],
            rom_paths: vec![temp.path().join("roms")],
            ini_path: Some(temp.path().join("ini")),
            cfg_path: Some(temp.path().join("cfg")),
            nvram_path: Some(temp.path().join("nvram")),
            input_path: Some(temp.path().join("input")),
            state_path: Some(temp.path().join("state")),
            diff_path: Some(temp.path().join("diff")),
            comment_path: Some(temp.path().join("comment")),
            home_path: Some(temp.path().join("home")),
            ..Default::default()
        };
        for directory in [
            "roms", "ini", "cfg", "nvram", "input", "state", "diff", "comment", "home",
        ] {
            std::fs::create_dir(temp.path().join(directory)).unwrap();
        }
        // Defaults deliberately lack -bench: applying the per-game setting is
        // what makes this real invocation terminate after one emulated second.
        let mut properties = config.default_game_properties.clone();
        properties.miscellaneous.custom_args = "-noreadconfig -bench 1".into();
        config.game_properties.insert("pong".into(), properties);
        let mut child = launch_game("pong", &config).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    assert!(
                        status.success(),
                        "Native MAME benchmark exited with {status}"
                    );
                    break;
                }
                Ok(None) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
                result => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("Native MAME benchmark failed to finish within 20s: {result:?}");
                }
            }
        }
    }
}
