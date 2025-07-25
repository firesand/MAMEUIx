use eframe::egui;
use crate::models::game_properties::*;
use crate::models::Game;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ShaderStatus {
    Available,
    NotFound,
    Unknown,
}

pub struct GamePropertiesDialog {
    properties: GameProperties,
    selected_tab: PropertiesTab,
    is_default_game: bool,
    original_properties: GameProperties,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PropertiesTab {
    Display,
    Advanced,
    Screen,
    Sound,
    Miscellaneous,
    SDLDrivers,  // New tab for SDL driver options
    OSDOptions,  // New tab for OSD input/output options
}

impl GamePropertiesDialog {
    pub fn new_with_config(game: Option<&Game>, config: &crate::models::AppConfig) -> Self {
        let properties = if let Some(g) = game {
            // Load saved properties for this specific game
            config.game_properties.get(&g.name)
                .cloned()
                .unwrap_or_else(|| GameProperties {
                    game_name: g.name.clone(),
                    ..config.default_game_properties.clone()
                })
        } else {
            // Use default properties
            config.default_game_properties.clone()
        };
        

        
        Self {
            original_properties: properties.clone(),
            properties,
            selected_tab: PropertiesTab::Display,
            is_default_game: game.is_none(),
        }
    }
    
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        config: &mut crate::models::AppConfig,
    ) -> bool {
        let mut apply_clicked = false;
        let mut ok_clicked = false;
        let mut should_close = false;
        
        let title = if self.is_default_game {
            "Properties for Default Game"
        } else {
            &format!("Properties for {}", self.properties.game_name)
        };
        
        egui::Window::new(title)
            .open(open)
            .default_size([600.0, 550.0])
            .min_size([500.0, 400.0])
            .max_size([1200.0, 900.0])
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                // Main tabs
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_tab, PropertiesTab::Display, "Display");
                    ui.selectable_value(&mut self.selected_tab, PropertiesTab::Advanced, "Advanced");
                    ui.selectable_value(&mut self.selected_tab, PropertiesTab::Screen, "Screen");
                    ui.selectable_value(&mut self.selected_tab, PropertiesTab::Sound, "Sound");
                    ui.selectable_value(&mut self.selected_tab, PropertiesTab::Miscellaneous, "Miscellaneous");
                    ui.selectable_value(&mut self.selected_tab, PropertiesTab::SDLDrivers, "SDL Drivers");
                    ui.selectable_value(&mut self.selected_tab, PropertiesTab::OSDOptions, "OSD/Input");
                });
                
                ui.separator();
                
                // Global game options info
                ui.horizontal(|ui| {
                    // Placeholder for icon
                    ui.label("🎮");
                    ui.vertical(|ui| {
                        ui.label("Global game options");
                        ui.label("Default options used by all games");
                    });
                });
                
                ui.separator();
                
                // Tab content
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(ui.available_height() - 100.0) // Dynamic height based on available space
                    .show(ui, |ui| {
                        match self.selected_tab {
                            PropertiesTab::Display => self.show_display_tab(ui),
                            PropertiesTab::Advanced => self.show_advanced_tab(ui),
                            PropertiesTab::Screen => self.show_screen_tab(ui),
                            PropertiesTab::Sound => self.show_sound_tab(ui),
                            PropertiesTab::Miscellaneous => self.show_miscellaneous_tab(ui),
                            PropertiesTab::SDLDrivers => self.show_sdl_drivers_tab(ui),
                            PropertiesTab::OSDOptions => self.show_osd_options_tab(ui),
                        }
                    });
                
                ui.separator();
                
                // Bottom buttons
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        ok_clicked = true;
                        apply_clicked = true;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        should_close = true;
                    }
                    
                    if ui.button("Apply").clicked() {
                        apply_clicked = true;
                    }
                });
            });
        
        if ok_clicked || should_close {
            *open = false;
        }
        
        if apply_clicked {
            self.apply_changes(config);
        }
        
        // Note: Window size and position are automatically saved by egui
        // when the window is resized or moved
        
        apply_clicked
    }
    
    fn show_display_tab(&mut self, ui: &mut egui::Ui) {
        let display = &mut self.properties.display;
        
        ui.checkbox(&mut display.run_in_window, "Run in a window");
        ui.checkbox(&mut display.start_out_maximized, "Start out maximized");
        ui.checkbox(&mut display.enforce_aspect_ratio, "Enforce aspect ratio");
        ui.checkbox(&mut display.throttle, "Throttle");
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Bitmap prescaling:");
            ui.add(egui::DragValue::new(&mut display.bitmap_prescaling)
                .speed(1.0)
                .range(1..=8));
        });
        
        ui.horizontal(|ui| {
            ui.label("Gamma Correction:");
            ui.add(egui::Slider::new(&mut display.gamma_correction, 0.1..=3.0)
                .text("1.00"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Brightness Correction:");
            ui.add(egui::Slider::new(&mut display.brightness_correction, 0.1..=2.0)
                .text("1.00"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Contrast Correction:");
            ui.add(egui::Slider::new(&mut display.contrast_correction, 0.1..=2.0)
                .text("1.00"));
        });
        
        ui.horizontal(|ui| {
            ui.label("Pause Brightness:");
            ui.add(egui::Slider::new(&mut display.pause_brightness, 0.0..=1.0)
                .text("0.65"));
        });
        
        ui.separator();
        
        ui.checkbox(&mut display.use_bilinear_filtering, "Use Bilinear filtering");
        ui.checkbox(&mut display.update_main_window_during_pause, 
                   "Update Main window during paused emulation");
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Video Mode:");
            let old_video_mode = display.video_mode.clone();
            egui::ComboBox::from_id_salt("video_mode_dropdown")
                .selected_text(display.video_mode.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut display.video_mode, VideoMode::Auto, "Auto");
                    ui.selectable_value(&mut display.video_mode, VideoMode::OpenGL, "OpenGL");
                    
                    // Only show Direct3D on Windows
                    #[cfg(target_os = "windows")]
                    {
                        ui.selectable_value(&mut display.video_mode, VideoMode::Direct3D, "Direct3D");
                    }
                    
                    ui.selectable_value(&mut display.video_mode, VideoMode::BGFX, "BGFX");
                    ui.selectable_value(&mut display.video_mode, VideoMode::Software, "Software");
                });
            if old_video_mode != display.video_mode {
                // Video mode changed
            }
        });
        
        // Platform-specific recommendations and warnings
        #[cfg(target_os = "linux")]
        {
            ui.colored_label(
                egui::Color32::from_rgb(100, 150, 100),
                "💡 Linux recommendations: OpenGL (best compatibility), BGFX+Vulkan (newer GPUs), Auto (safe choice)"
            );
            
            // Warn if Direct3D is somehow selected (shouldn't happen with our UI, but just in case)
            if matches!(display.video_mode, VideoMode::Direct3D) {
                ui.colored_label(
                    egui::Color32::RED,
                    "⚠ Direct3D is not available on Linux. Please select OpenGL, BGFX, or Auto."
                );
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            ui.colored_label(
                egui::Color32::from_rgb(100, 150, 100),
                "💡 Windows recommendations: Direct3D (best performance), OpenGL (compatibility), BGFX (advanced features)"
            );
        }
        
        ui.horizontal(|ui| {
            ui.label("Rotation:");
            let old_rotation = display.rotation.clone();
            egui::ComboBox::from_id_salt("rotation_dropdown")
                .selected_text(display.rotation.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut display.rotation, RotationMode::Default, "Default");
                    ui.selectable_value(&mut display.rotation, RotationMode::Rotate0, "0");
                    ui.selectable_value(&mut display.rotation, RotationMode::Rotate90, "90");
                    ui.selectable_value(&mut display.rotation, RotationMode::Rotate180, "180");
                    ui.selectable_value(&mut display.rotation, RotationMode::Rotate270, "270");
                });
            if old_rotation != display.rotation {
                // Rotation changed
            }
        });
        
        let old_flip_upside = display.flip_screen_upside_down;
        let old_flip_left_right = display.flip_screen_left_right;
        ui.checkbox(&mut display.flip_screen_upside_down, "Flip screen upside-down");
        ui.checkbox(&mut display.flip_screen_left_right, "Flip screen left-right");
        if old_flip_upside != display.flip_screen_upside_down {
            // Flip upside-down changed
        }
        if old_flip_left_right != display.flip_screen_left_right {
            // Flip left-right changed
        }
        
        // Integer Scaling group with better organization
        ui.group(|ui| {
            ui.label("Integer Scaling Options");
            ui.add_space(5.0);
            
            // Main non-integer scaling toggle
            let was_checked = display.use_non_integer_scaling;
            ui.checkbox(&mut display.use_non_integer_scaling, 
                       "Allow non-integer scaling ratios")
                .on_hover_text("Allows fractional scaling (e.g., 2.5x) for better screen fit. May cause slight blur.");
            
            // Only show axis options if non-integer scaling is enabled
            if display.use_non_integer_scaling {
                ui.indent("stretch_options", |ui| {
                    ui.checkbox(&mut display.stretch_only_x_axis, 
                               "Allow non-integer stretch on X axis only")
                        .on_hover_text("Only stretch horizontally with fractional scaling, keep vertical scaling integer");
                    ui.checkbox(&mut display.stretch_only_y_axis, 
                               "Allow non-integer stretch on Y axis only")
                        .on_hover_text("Only stretch vertically with fractional scaling, keep horizontal scaling integer");
                    
                    // Show a note if both are checked
                    if display.stretch_only_x_axis && display.stretch_only_y_axis {
                        ui.colored_label(
                            egui::Color32::from_rgb(150, 150, 150),
                            "ℹ Both axes selected - will use general uneven stretch"
                        );
                    }
                });
            }
            
            ui.add_space(5.0);
            
            // Auto stretch option
            ui.checkbox(&mut display.auto_select_stretch_axis, 
                       "Auto-select stretch axis based on game orientation")
                .on_hover_text("Automatically choose which axis to stretch based on whether game is vertical or horizontal");
            
            if display.auto_select_stretch_axis && display.use_non_integer_scaling {
                ui.colored_label(
                    egui::Color32::from_rgb(150, 150, 150),
                    "ℹ Auto mode will override manual axis selection"
                );
            }
            
            ui.add_space(5.0);
            
            // Overscan option
            ui.checkbox(&mut display.overscan_on_targets, 
                       "Allow overscan on integer scaled targets")
                .on_hover_text("Allow the image to extend slightly beyond screen edges when using integer scaling");
            
            ui.add_space(10.0);
            ui.separator();
            
            // Integer scale factors
            ui.label("Manual Integer Scale Factors:");
            ui.label("Set to 0 for automatic scaling, or specify exact multiplier");
            
            ui.horizontal(|ui| {
                ui.label("Horizontal scale factor:");
                ui.add(egui::DragValue::new(&mut display.horizontal_scale_factor)
                    .speed(1)
                    .range(0..=10)
                    .suffix("x"));
                
                if display.horizontal_scale_factor > 0 {
                    ui.label(format!("({}x native width)", display.horizontal_scale_factor));
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Vertical scale factor:");
                ui.add(egui::DragValue::new(&mut display.vertical_scale_factor)
                    .speed(1)
                    .range(0..=10)
                    .suffix("x"));
                
                if display.vertical_scale_factor > 0 {
                    ui.label(format!("({}x native height)", display.vertical_scale_factor));
                }
            });
            
            // Preset buttons for common integer scales
            ui.horizontal(|ui| {
                ui.label("Common presets:");
                if ui.button("2x").clicked() {
                    display.horizontal_scale_factor = 2;
                    display.vertical_scale_factor = 2;
                }
                if ui.button("3x").clicked() {
                    display.horizontal_scale_factor = 3;
                    display.vertical_scale_factor = 3;
                }
                if ui.button("4x").clicked() {
                    display.horizontal_scale_factor = 4;
                    display.vertical_scale_factor = 4;
                }
                if ui.button("Auto").clicked() {
                    display.horizontal_scale_factor = 0;
                    display.vertical_scale_factor = 0;
                }
            });
        });
    }
    
    fn show_advanced_tab(&mut self, ui: &mut egui::Ui) {
        let advanced = &mut self.properties.advanced;
        
        ui.group(|ui| {
            ui.label("OpenGL settings");
            ui.add_space(5.0);
            
            // Basic OpenGL options
            ui.checkbox(&mut advanced.enable_glsl, "Enable GLSL shaders");
            ui.checkbox(&mut advanced.force_power_of_two_textures, "Force power-of-two textures");
            ui.checkbox(&mut advanced.dont_use_gl_arb_texture_rectangle, 
                       "Don't use GL_ARB_texture_rectangle");
            ui.checkbox(&mut advanced.enable_vbo, "Enable VBO (Vertex Buffer Objects)");
            ui.checkbox(&mut advanced.enable_pbo, "Enable PBO (Pixel Buffer Objects)");
            
            ui.add_space(10.0);
            
            // GLSL Filter (only show if GLSL is enabled)
            if advanced.enable_glsl {
                ui.horizontal(|ui| {
                    ui.label("GLSL Filter:");
                    egui::ComboBox::from_id_salt("glsl_filter")
                        .selected_text(match advanced.glsl_filter {
                            GLSLFilter::Plain => "Plain",
                            GLSLFilter::Bilinear => "Bilinear",
                            GLSLFilter::Bicubic => "Bicubic",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut advanced.glsl_filter, 
                                              GLSLFilter::Plain, "Plain");
                            ui.selectable_value(&mut advanced.glsl_filter, 
                                              GLSLFilter::Bilinear, "Bilinear (default)");
                            ui.selectable_value(&mut advanced.glsl_filter, 
                                              GLSLFilter::Bicubic, "Bicubic");
                        });
                });
                
                ui.add_space(10.0);
                
                // GLSL Shaders section
                ui.collapsing("GLSL Shaders", |ui| {
                    ui.label("Configure custom GLSL shaders for different render passes:");
                    ui.add_space(5.0);
                    
                    // Common shader presets
                    ui.horizontal(|ui| {
                        ui.label("Quick presets:");
                        if ui.button("CRT").clicked() {
                            Self::apply_shader_preset_static(advanced, "crt");
                        }
                        if ui.button("Scanlines").clicked() {
                            Self::apply_shader_preset_static(advanced, "scanlines");
                        }
                        if ui.button("LCD").clicked() {
                            Self::apply_shader_preset_static(advanced, "lcd");
                        }
                        if ui.button("Clear All").clicked() {
                            advanced.glsl_shader_mame = vec![String::new(); 10];
                            advanced.glsl_shader_screen = vec![String::new(); 10];
                        }
                    });
                    
                    ui.separator();
                    
                    // MAME bitmap shaders
                    ui.label("MAME Bitmap Shaders:");
                    egui::ScrollArea::vertical()
                        .id_salt("mame_shaders_scroll")
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for i in 0..10 {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Slot {}:", i));
                                    
                                    let mut shader_text = advanced.glsl_shader_mame[i].clone();
                                    
                                    if ui.text_edit_singleline(&mut shader_text)
                                        .on_hover_text("Path to GLSL shader file")
                                        .changed() 
                                    {
                                        advanced.glsl_shader_mame[i] = shader_text;
                                    }
                                    
                                    if ui.button(format!("Browse##mame_browse_{}", i)).clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("GLSL Shader", &["glsl", "frag", "vert"])
                                            .add_filter("All files", &["*"])
                                            .pick_file()
                                        {
                                            advanced.glsl_shader_mame[i] = 
                                                path.to_string_lossy().to_string();
                                        }
                                    }
                                    
                                    if !advanced.glsl_shader_mame[i].is_empty() {
                                        if ui.button(format!("Clear##mame_clear_{}", i)).clicked() {
                                            advanced.glsl_shader_mame[i] = String::new();
                                        }
                                    }
                                });
                            }
                        });
                    
                    ui.add_space(10.0);
                    
                    // Screen bitmap shaders
                    ui.label("Screen Bitmap Shaders:");
                    egui::ScrollArea::vertical()
                        .id_salt("screen_shaders_scroll")
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for i in 0..10 {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Slot {}:", i));
                                    
                                    let mut shader_text = advanced.glsl_shader_screen[i].clone();
                                    
                                    if ui.text_edit_singleline(&mut shader_text)
                                        .on_hover_text("Path to GLSL shader file")
                                        .changed() 
                                    {
                                        advanced.glsl_shader_screen[i] = shader_text;
                                    }
                                    
                                    if ui.button(format!("Browse##screen_browse_{}", i)).clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("GLSL Shader", &["glsl", "frag", "vert"])
                                            .add_filter("All files", &["*"])
                                            .pick_file()
                                        {
                                            advanced.glsl_shader_screen[i] = 
                                                path.to_string_lossy().to_string();
                                        }
                                    }
                                    
                                    if !advanced.glsl_shader_screen[i].is_empty() {
                                        if ui.button(format!("Clear##screen_clear_{}", i)).clicked() {
                                            advanced.glsl_shader_screen[i] = String::new();
                                        }
                                    }
                                });
                            }
                        });
                });
                

            }
        });
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("BGFX settings");
            ui.add_space(5.0);
            
            // Backend selection with platform detection
            ui.horizontal(|ui| {
                ui.label("Backend:");
                egui::ComboBox::from_id_salt("bgfx_backend_dropdown")
                    .selected_text(advanced.bgfx_settings.backend.to_string())
                    .show_ui(ui, |ui| {
                        // Only show available backends for current platform
                        for backend in BGFXBackend::available_backends() {
                            let label = match backend {
                                BGFXBackend::Auto => "Auto-detect",
                                BGFXBackend::D3D9 => "Direct3D 9",
                                BGFXBackend::D3D11 => "Direct3D 11", 
                                BGFXBackend::D3D12 => "Direct3D 12",
                                BGFXBackend::OpenGL => "OpenGL",
                                BGFXBackend::Metal => "Metal",
                                BGFXBackend::Vulkan => "Vulkan",
                            };
                            ui.selectable_value(
                                &mut advanced.bgfx_settings.backend,
                                backend,
                                label
                            );
                        }
                    });
            });
            
            // Show info about selected backend
            if !advanced.bgfx_settings.backend.is_available() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("⚠ {} is not available on this platform", 
                            advanced.bgfx_settings.backend)
                );
            }
            
            ui.add_space(5.0);
            
            // Screen chains
            ui.horizontal(|ui| {
                ui.label("Screen Chains:");
                ui.add_space(10.0);
                ui.text_edit_singleline(&mut advanced.bgfx_settings.screen_chains);
            });
            
            // Preset buttons for common shader chains
            ui.horizontal(|ui| {
                if ui.button("CRT-geom").clicked() {
                    advanced.bgfx_settings.screen_chains = "crt-geom".to_string();
                }
                if ui.button("LCD").clicked() {
                    advanced.bgfx_settings.screen_chains = "lcd".to_string();
                }
                if ui.button("HLSL").clicked() {
                    advanced.bgfx_settings.screen_chains = "hlsl".to_string();
                }
                if ui.button("Default").clicked() {
                    advanced.bgfx_settings.screen_chains = "default".to_string();
                }
            });
            
            // Platform-specific note about HLSL
            #[cfg(target_os = "linux")]
            {
                ui.colored_label(
                    egui::Color32::from_rgb(150, 150, 150),
                    "ℹ HLSL shaders work with BGFX on Linux, but GLSL shaders are more common"
                );
            }
            
            // Shader validation status
            let shader_name = &advanced.bgfx_settings.screen_chains;
            if !shader_name.is_empty() && shader_name != "default" {
                let shader_status = Self::validate_shader_availability_static(shader_name);
                match shader_status {
                    ShaderStatus::Available => {
                        ui.colored_label(egui::Color32::GREEN, "✅ Shader available");
                    }
                    ShaderStatus::NotFound => {
                        ui.colored_label(egui::Color32::RED, "❌ Shader not found");
                        ui.label("Common BGFX shader locations:");
                        ui.label("• ~/.mame/bgfx/chains/");
                        ui.label("• /usr/share/mame/bgfx/chains/");
                        ui.label("• ./bgfx/chains/");
                    }
                    ShaderStatus::Unknown => {
                        ui.colored_label(egui::Color32::YELLOW, "⚠ Shader status unknown");
                    }
                }
            }
            
            ui.add_space(5.0);
            
            // Debug option
            ui.checkbox(&mut advanced.bgfx_settings.enable_debug, "Enable BGFX debug statistics");
            
            ui.add_space(5.0);
            
            // Shadow mask
            ui.horizontal(|ui| {
                ui.label("Shadow Mask:");
                let mut shadow_mask = advanced.bgfx_settings.shadow_mask.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut shadow_mask).changed() {
                    advanced.bgfx_settings.shadow_mask = if shadow_mask.is_empty() {
                        None
                    } else {
                        Some(shadow_mask)
                    };
                }
            });
            
            // LUT texture
            ui.horizontal(|ui| {
                ui.label("LUT Texture:");
                let mut lut = advanced.bgfx_settings.lut_texture.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut lut).changed() {
                    advanced.bgfx_settings.lut_texture = if lut.is_empty() {
                        None
                    } else {
                        Some(lut)
                    };
                }
            });
        });
        

    }
    
    fn show_screen_tab(&mut self, ui: &mut egui::Ui) {
        let screen = &mut self.properties.screen;
        
        ui.checkbox(&mut screen.triple_buffering, "Triple buffering");
        ui.checkbox(&mut screen.sync_to_monitor_refresh, "Sync to monitor refresh");
        ui.checkbox(&mut screen.wait_for_vertical_sync, "Wait for vertical sync");
        ui.checkbox(&mut screen.refresh_speed, "Refresh speed");
        ui.checkbox(&mut screen.low_latency, "Low latency");
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("Frame skipping");
            ui.radio_value(&mut screen.frame_skipping.automatic, true, "Automatic");
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut screen.frame_skipping.automatic, false, "Draw every frame");
                ui.add(egui::DragValue::new(&mut screen.frame_skipping.draw_every_frame)
                    .speed(1)
                    .range(1..=10));
            });
        });
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("Core Performance Options");
            
            ui.checkbox(&mut screen.auto_frameskip, "Auto-frameskip (maintain emulation speed)");
            
            ui.horizontal(|ui| {
                ui.label("Frameskip value:");
                ui.add(egui::DragValue::new(&mut screen.frameskip_value)
                    .speed(1)
                    .range(0..=10));
                ui.label("(0-10, used with auto-frameskip as upper limit)");
            });
            
            ui.checkbox(&mut screen.sleep_when_idle, "Sleep when idle (give time to other apps)");
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Emulation speed:");
            let speed = screen.emulation_speed;
            ui.add(egui::Slider::new(&mut screen.emulation_speed, 0.1..=2.0)
                .text(format!("{:.1}", speed)));
        });
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("Effect");
            
            ui.horizontal(|ui| {
                let effect_text = screen.effect.get_or_insert_with(|| "none".to_string());
                ui.text_edit_singleline(effect_text);
            });
            
            ui.horizontal(|ui| {
                if ui.button("Select Effect").clicked() {
                    // TODO: Open effect selection dialog
                }
                
                if ui.button("Reset Effect").clicked() {
                    screen.effect = None;
                }
            });
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Full screen gamma:");
            let gamma = screen.full_screen_gamma;
            ui.add(egui::Slider::new(&mut screen.full_screen_gamma, 0.1..=3.0)
                .text(format!("{:.2}", gamma)));
        });
        
        ui.horizontal(|ui| {
            ui.label("Full screen brightness:");
            let brightness = screen.full_screen_brightness;
            ui.add(egui::Slider::new(&mut screen.full_screen_brightness, 0.1..=2.0)
                .text(format!("{:.2}", brightness)));
        });
        
        ui.horizontal(|ui| {
            ui.label("Full screen contrast:");
            let contrast = screen.full_screen_contrast;
            ui.add(egui::Slider::new(&mut screen.full_screen_contrast, 0.1..=2.0)
                .text(format!("{:.2}", contrast)));
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Seconds to run:");
            ui.add(egui::DragValue::new(&mut screen.seconds_to_run)
                .speed(1)
                .range(0..=3600));
        });
    }
    
    fn show_sound_tab(&mut self, ui: &mut egui::Ui) {
        let sound = &mut self.properties.sound;
        
        ui.horizontal(|ui| {
            ui.label("Sound Mode:");
            egui::ComboBox::from_id_salt("sound_mode_dropdown")
                .selected_text(sound.sound_mode.to_string())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut sound.sound_mode, SoundMode::Auto, "Auto");
                    ui.selectable_value(&mut sound.sound_mode, SoundMode::SDL, "SDL");
                    ui.selectable_value(&mut sound.sound_mode, SoundMode::PortAudio, "PortAudio");
                    ui.selectable_value(&mut sound.sound_mode, SoundMode::PulseAudio, "PulseAudio");
                    ui.selectable_value(&mut sound.sound_mode, SoundMode::None, "None");
                });
        });
        
        ui.separator();
        
        ui.checkbox(&mut sound.use_samples, "Use samples");
        
        ui.horizontal(|ui| {
            ui.label("Sample rate:");
            egui::ComboBox::from_id_salt("sample_rate_dropdown")
                .selected_text(format!("{}", sound.sample_rate))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut sound.sample_rate, 11025, "11025");
                    ui.selectable_value(&mut sound.sample_rate, 22050, "22050");
                    ui.selectable_value(&mut sound.sample_rate, 44100, "44100");
                    ui.selectable_value(&mut sound.sample_rate, 48000, "48000");
                });
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Volume attenuation:");
            let volume = sound.volume_attenuation;
            ui.add(egui::Slider::new(&mut sound.volume_attenuation, -32..=0)
                .text(format!("{}dB", volume)));
        });
        
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Audio latency:");
            let latency = sound.audio_latency;
            ui.add(egui::Slider::new(&mut sound.audio_latency, 0.0..=5.0)
                .text(format!("{:.1}", latency)));
        });
    }
    
    fn show_miscellaneous_tab(&mut self, ui: &mut egui::Ui) {
        let misc = &mut self.properties.miscellaneous;
        
        ui.horizontal(|ui| {
            ui.label("Number of screens:");
            ui.add(egui::DragValue::new(&mut misc.num_screens)
                .speed(1)
                .range(1..=4));
        });
        
        ui.horizontal(|ui| {
            ui.label("Screen Number:");
            egui::ComboBox::from_id_salt("screen_number_dropdown")
                .selected_text(match &misc.screen_number {
                    ScreenSelection::Default => "Default".to_string(),
                    ScreenSelection::Screen(n) => format!("Screen {}", n),
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut misc.screen_number, 
                                      ScreenSelection::Default, "Default");
                    for i in 1..=4 {
                        ui.selectable_value(&mut misc.screen_number, 
                                          ScreenSelection::Screen(i), 
                                          format!("Screen {}", i));
                    }
                });
        });
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("Performance");
            
            ui.horizontal(|ui| {
                ui.label("Number of processors:");
                
                let mut use_custom = misc.num_processors.is_some();
                if ui.checkbox(&mut use_custom, "Override system default").changed() {
                    if use_custom {
                        // Get system CPU count as default
                        let cpu_count = std::thread::available_parallelism()
                            .map(|n| n.get() as u32)
                            .unwrap_or(4);
                        misc.num_processors = Some(cpu_count);
                    } else {
                        misc.num_processors = None;
                    }
                }
                
                if let Some(num_procs) = &mut misc.num_processors {
                    ui.add(egui::DragValue::new(num_procs)
                        .speed(1)
                        .range(1..=64)
                        .suffix(" cores"));
                }
            });
            
            if misc.num_processors.is_some() {
                ui.colored_label(
                    egui::Color32::from_rgb(150, 150, 150),
                    "ℹ Overriding system processor count may affect performance"
                );
            }
        });
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("Screen");
            
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                egui::ComboBox::from_id_salt("resolution_dropdown")
                    .selected_text(match &misc.resolution {
                        Resolution::Auto => "Auto".to_string(),
                        Resolution::Custom(w, h) => format!("{}x{}", w, h),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut misc.resolution, Resolution::Auto, "Auto");
                        ui.selectable_value(&mut misc.resolution, Resolution::Custom(640, 480), "640x480");
                        ui.selectable_value(&mut misc.resolution, Resolution::Custom(800, 600), "800x600");
                        ui.selectable_value(&mut misc.resolution, Resolution::Custom(1024, 768), "1024x768");
                        ui.selectable_value(&mut misc.resolution, Resolution::Custom(1280, 720), "1280x720");
                        ui.selectable_value(&mut misc.resolution, Resolution::Custom(1920, 1080), "1920x1080");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Size:");
                egui::ComboBox::from_id_salt("size_dropdown")
                    .selected_text("Auto")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut 0, 0, "Auto");
                    });
            });
            
            ui.horizontal(|ui| {
                ui.label("Refresh:");
                egui::ComboBox::from_id_salt("refresh_dropdown")
                    .selected_text("Auto")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut 0, 0, "Auto");
                    });
            });
            
            ui.checkbox(&mut misc.switch_resolutions_to_fit, "Switch resolutions to fit");
        });
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("View");
            
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt("view_dropdown")
                    .selected_text(match &misc.view {
                        ViewSelection::Auto => "Auto".to_string(),
                        ViewSelection::Custom(v) => v.clone(),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut misc.view, ViewSelection::Auto, "Auto");
                    });
            });
            
            ui.checkbox(&mut misc.autoselect_aspect, "Autoselect aspect");
            
            ui.horizontal(|ui| {
                ui.label("Aspect ratio:");
                ui.add(egui::DragValue::new(&mut misc.aspect_ratio.0)
                    .speed(1)
                    .range(1..=50));
                ui.label(":");
                ui.add(egui::DragValue::new(&mut misc.aspect_ratio.1)
                    .speed(1)
                    .range(1..=50));
            });
        });
        
        ui.separator();
        
        ui.group(|ui| {
            ui.label("Custom Arguments");
            ui.label("Additional MAME command line arguments:");
            
            ui.add(egui::TextEdit::multiline(&mut misc.custom_args)
                .desired_rows(3)
                .hint_text("e.g., -bench 60 -monitorprovider sdl"));
            
            ui.colored_label(
                egui::Color32::from_rgb(150, 150, 150),
                "ℹ Separate multiple arguments with spaces"
            );
        });
    }
    
    fn show_sdl_drivers_tab(&mut self, ui: &mut egui::Ui) {
        let sdl_options = &mut self.properties.sdl_options;
        
        ui.heading("SDL-Specific Options (Linux/Unix)");
        ui.separator();
        
        // Performance Options
        ui.group(|ui| {
            ui.label("Performance Monitoring");
            ui.checkbox(&mut sdl_options.show_video_fps, "Show SDL video FPS");
        });
        
        ui.add_space(10.0);
        
        // Video Options
        ui.group(|ui| {
            ui.label("Video Options");
            ui.checkbox(&mut sdl_options.center_horizontal, "Center horizontally");
            ui.checkbox(&mut sdl_options.center_vertical, "Center vertically");
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Scale mode (software video only):");
                egui::ComboBox::from_id_salt("sdl_scale_mode")
                    .selected_text(sdl_options.scale_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut sdl_options.scale_mode, crate::models::game_properties::SDLScaleMode::None, "None");
                        ui.selectable_value(&mut sdl_options.scale_mode, crate::models::game_properties::SDLScaleMode::HWBlit, "Hardware Blit");
                        ui.selectable_value(&mut sdl_options.scale_mode, crate::models::game_properties::SDLScaleMode::HWBest, "Hardware Best");
                        ui.selectable_value(&mut sdl_options.scale_mode, crate::models::game_properties::SDLScaleMode::YV12, "YV12");
                        ui.selectable_value(&mut sdl_options.scale_mode, crate::models::game_properties::SDLScaleMode::YUY2, "YUY2");
                        ui.selectable_value(&mut sdl_options.scale_mode, crate::models::game_properties::SDLScaleMode::YV12x2, "YV12 x2");
                        ui.selectable_value(&mut sdl_options.scale_mode, crate::models::game_properties::SDLScaleMode::YUY2x2, "YUY2 x2");
                    });
            });
        });
        
        ui.add_space(10.0);
        
        // Full Screen Options
        ui.group(|ui| {
            ui.label("Full Screen Options");
            ui.checkbox(&mut sdl_options.use_all_heads, "Use all monitors (split image across screens)");
            
            ui.horizontal(|ui| {
                ui.label("Attach to window:");
                let mut attach_str = sdl_options.attach_window.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut attach_str).changed() {
                    sdl_options.attach_window = if attach_str.is_empty() {
                        None
                    } else {
                        Some(attach_str)
                    };
                }
                ui.label("(Window ID or handle)");
            });
        });
        
        ui.add_space(10.0);
        
        // Input Options
        ui.group(|ui| {
            ui.label("Input Options");
            ui.checkbox(&mut sdl_options.enable_touch, "Enable touch input");
            ui.checkbox(&mut sdl_options.sixaxis_support, "PS3 Sixaxis controller support");
            ui.checkbox(&mut sdl_options.dual_lightgun, "Dual lightgun support");
        });
        
        ui.add_space(10.0);
        
        // Keyboard Mapping
        ui.group(|ui| {
            ui.label("Keyboard Mapping");
            ui.checkbox(&mut sdl_options.enable_keymap, "Enable custom keymap");
            
            if sdl_options.enable_keymap {
                ui.horizontal(|ui| {
                    ui.label("Keymap file:");
                    let mut keymap_str = sdl_options.keymap_file.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut keymap_str).changed() {
                        sdl_options.keymap_file = if keymap_str.is_empty() {
                            None
                        } else {
                            Some(keymap_str)
                        };
                    }
                    if ui.button("Browse...").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Keymap files", &["map", "txt"])
                            .add_filter("All files", &["*"])
                            .pick_file()
                        {
                            sdl_options.keymap_file = Some(path.display().to_string());
                        }
                    }
                });
            }
        });
        
        ui.add_space(10.0);
        
        // Lightgun Mapping (collapsible)
        ui.collapsing("Lightgun Mappings", |ui| {
            for i in 0..8 {
                ui.horizontal(|ui| {
                    ui.label(format!("Lightgun #{}:", i + 1));
                    let mut lightgun_str = sdl_options.lightgun_mappings[i].clone();
                    if ui.text_edit_singleline(&mut lightgun_str).changed() {
                        sdl_options.lightgun_mappings[i] = lightgun_str;
                    }
                });
            }
        });
        
        ui.add_space(10.0);
        
        // Low-Level Driver Options
        ui.group(|ui| {
            ui.label("Low-Level Driver Options");
            
            ui.label("Video Driver:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut sdl_options.video_driver, None, "Auto (SDL Default)");
                ui.radio_value(&mut sdl_options.video_driver, Some("x11".to_string()), "X11");
                ui.radio_value(&mut sdl_options.video_driver, Some("wayland".to_string()), "Wayland");
                ui.radio_value(&mut sdl_options.video_driver, Some("directfb".to_string()), "DirectFB");
            });
            
            ui.add_space(5.0);
            
            ui.label("Render Driver:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut sdl_options.render_driver, None, "Auto (SDL Default)");
                ui.radio_value(&mut sdl_options.render_driver, Some("software".to_string()), "Software");
                ui.radio_value(&mut sdl_options.render_driver, Some("opengl".to_string()), "OpenGL");
                ui.radio_value(&mut sdl_options.render_driver, Some("opengles2".to_string()), "OpenGL ES 2");
            });
            
            ui.add_space(5.0);
            
            ui.label("Audio Driver:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut sdl_options.audio_driver, None, "Auto (SDL Default)");
                ui.radio_value(&mut sdl_options.audio_driver, Some("alsa".to_string()), "ALSA");
                ui.radio_value(&mut sdl_options.audio_driver, Some("pulse".to_string()), "PulseAudio");
                ui.radio_value(&mut sdl_options.audio_driver, Some("jack".to_string()), "JACK");
                ui.radio_value(&mut sdl_options.audio_driver, Some("pipewire".to_string()), "PipeWire");
            });
            
            ui.add_space(5.0);
            
            ui.label("Alternative GL Library:");
            ui.horizontal(|ui| {
                let mut gl_lib_str = sdl_options.gl_lib.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut gl_lib_str).changed() {
                    sdl_options.gl_lib = if gl_lib_str.is_empty() {
                        None
                    } else {
                        Some(gl_lib_str)
                    };
                }
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Shared Libraries", &["so", "so.*"])
                        .add_filter("All files", &["*"])
                        .pick_file()
                    {
                        sdl_options.gl_lib = Some(path.display().to_string());
                    }
                }
                if ui.button("Auto").clicked() {
                    sdl_options.gl_lib = None;
                }
            });
            ui.label("Leave empty for system default libGL.so");
        });
        
        ui.add_space(10.0);
        
        ui.separator();
        ui.label("ℹ️ These options are SDL-specific and primarily for Linux/Unix systems.");
        ui.label("Most users should leave these on 'Auto' unless experiencing issues.");
    }
    
    fn show_osd_options_tab(&mut self, ui: &mut egui::Ui) {
        let osd = &mut self.properties.osd_options;
        
        ui.heading("OSD Input/Output Options");
        ui.separator();
        
        // Input Mapping
        ui.group(|ui| {
            ui.label("Input Mapping");
            
            ui.horizontal(|ui| {
                ui.label("UI Mode Key:");
                let mut key_str = osd.ui_mode_key.clone().unwrap_or_else(|| "SCRLOCK".to_string());
                if ui.text_edit_singleline(&mut key_str).changed() {
                    osd.ui_mode_key = if key_str.is_empty() || key_str == "SCRLOCK" {
                        None
                    } else {
                        Some(key_str)
                    };
                }
                ui.label("(Default: ScrollLock)");
            });
            
            ui.checkbox(&mut osd.background_input, "Keep input when window loses focus");
            
            ui.horizontal(|ui| {
                ui.label("Controller mapping file:");
                let mut map_str = osd.controller_map_file.clone().unwrap_or_default();
                if ui.text_edit_singleline(&mut map_str).changed() {
                    osd.controller_map_file = if map_str.is_empty() {
                        None
                    } else {
                        Some(map_str)
                    };
                }
                if ui.button("Browse...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Controller maps", &["cfg", "txt"])
                        .add_filter("All files", &["*"])
                        .pick_file()
                    {
                        osd.controller_map_file = Some(path.display().to_string());
                    }
                }
            });
        });
        
        ui.add_space(10.0);
        
        // Input Providers
        ui.group(|ui| {
            ui.label("Input Providers (Linux/SDL)");
            ui.add_space(5.0);
            
            // Keyboard provider
            ui.horizontal(|ui| {
                ui.label("Keyboard:");
                ui.radio_value(&mut osd.keyboard_provider, crate::models::game_properties::OSDProvider::Auto, "Auto");
                ui.radio_value(&mut osd.keyboard_provider, crate::models::game_properties::OSDProvider::SDL, "SDL");
                ui.radio_value(&mut osd.keyboard_provider, crate::models::game_properties::OSDProvider::None, "None");
            });
            
            // Mouse provider
            ui.horizontal(|ui| {
                ui.label("Mouse:");
                ui.radio_value(&mut osd.mouse_provider, crate::models::game_properties::OSDProvider::Auto, "Auto");
                ui.radio_value(&mut osd.mouse_provider, crate::models::game_properties::OSDProvider::SDL, "SDL");
                ui.radio_value(&mut osd.mouse_provider, crate::models::game_properties::OSDProvider::None, "None");
            });
            
            // Lightgun provider
            ui.horizontal(|ui| {
                ui.label("Lightgun:");
                ui.radio_value(&mut osd.lightgun_provider, crate::models::game_properties::LightgunProvider::Auto, "Auto");
                ui.radio_value(&mut osd.lightgun_provider, crate::models::game_properties::LightgunProvider::SDL, "SDL");
                ui.radio_value(&mut osd.lightgun_provider, crate::models::game_properties::LightgunProvider::X11, "X11");
                ui.radio_value(&mut osd.lightgun_provider, crate::models::game_properties::LightgunProvider::None, "None");
            });
            
            // Joystick provider
            ui.horizontal(|ui| {
                ui.label("Joystick:");
                ui.radio_value(&mut osd.joystick_provider, crate::models::game_properties::JoystickProvider::Auto, "Auto");
                ui.radio_value(&mut osd.joystick_provider, crate::models::game_properties::JoystickProvider::SDLGame, "SDL Game");
                ui.radio_value(&mut osd.joystick_provider, crate::models::game_properties::JoystickProvider::SDLJoy, "SDL Joy");
                ui.radio_value(&mut osd.joystick_provider, crate::models::game_properties::JoystickProvider::None, "None");
            });
        });
        
        ui.add_space(10.0);
        
        // Output Options
        ui.group(|ui| {
            ui.label("Output Options");
            
            ui.horizontal(|ui| {
                ui.label("UI Font Provider:");
                ui.radio_value(&mut osd.ui_font_provider, crate::models::game_properties::OSDProvider::Auto, "Auto");
                ui.radio_value(&mut osd.ui_font_provider, crate::models::game_properties::OSDProvider::SDL, "SDL");
                ui.radio_value(&mut osd.ui_font_provider, crate::models::game_properties::OSDProvider::None, "None");
            });
            
            ui.horizontal(|ui| {
                ui.label("Output notifications:");
                ui.radio_value(&mut osd.output_provider, crate::models::game_properties::OutputProvider::None, "None");
                ui.radio_value(&mut osd.output_provider, crate::models::game_properties::OutputProvider::Console, "Console");
                ui.radio_value(&mut osd.output_provider, crate::models::game_properties::OutputProvider::Network, "Network");
            });
        });
        
        ui.separator();
        ui.label("ℹ️ Provider Settings:");
        ui.label("• SDL Game: Modern game controller API (recommended)");
        ui.label("• SDL Joy: Legacy joystick API");
        ui.label("• X11: Better for some lightgun setups");
        ui.label("• None: Disable specific input type");
    }
    
    fn apply_changes(&mut self, config: &mut crate::models::AppConfig) {
        
        if self.is_default_game {
            // Save as default properties
            config.default_game_properties = self.properties.clone();
        } else {
            // Save as per-game properties
            config.game_properties.insert(self.properties.game_name.clone(), self.properties.clone());
        }
        
        // Mark properties as saved
        self.original_properties = self.properties.clone();
    }
    
    pub fn generate_mame_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        let props = &self.properties;
        

        
        // Window mode
        if props.display.run_in_window {
            args.push("-window".to_string());
        }
        
        // Aspect ratio (this should come before integer scaling options)
        if props.display.enforce_aspect_ratio {
            args.push("-keepaspect".to_string());
        }
        
        // Integer Scaling options
        if props.display.use_non_integer_scaling {
            // Check which type of non-integer scaling to use
            if props.display.stretch_only_x_axis && props.display.stretch_only_y_axis {
                // Both axes - use general unevenstretch
                args.push("-unevenstretch".to_string());
            } else if props.display.stretch_only_x_axis {
                // Only X axis
                args.push("-unevenstretchx".to_string());
            } else if props.display.stretch_only_y_axis {
                // Only Y axis
                args.push("-unevenstretchy".to_string());
            } else {
                // General uneven stretch if checkbox is checked but no specific axis
                args.push("-unevenstretch".to_string());
            }
        }
        
        // Auto stretch based on orientation
        if props.display.auto_select_stretch_axis {
            args.push("-autostretchxy".to_string());
        }
        
        // Overscan for integer scaled targets
        if props.display.overscan_on_targets {
            args.push("-intoverscan".to_string());
        }
        
        // Integer scale factors (only apply if > 0)
        if props.display.horizontal_scale_factor > 0 {
            args.push("-intscalex".to_string());
            args.push(props.display.horizontal_scale_factor.to_string());
        }
        
        if props.display.vertical_scale_factor > 0 {
            args.push("-intscaley".to_string());
            args.push(props.display.vertical_scale_factor.to_string());
        }
        
        // Throttle
        if !props.display.throttle {
            args.push("-nothrottle".to_string());
        }
        
        // Bitmap prescaling
        if props.display.bitmap_prescaling > 1 {
            args.push("-prescale".to_string());
            args.push(props.display.bitmap_prescaling.to_string());
        }
        
        // Video mode
        match props.display.video_mode {
            VideoMode::OpenGL => {
                args.push("-video".to_string());
                args.push("opengl".to_string());
            }
            VideoMode::Direct3D => {
                args.push("-video".to_string());
                args.push("d3d".to_string());
            }
            VideoMode::BGFX => {
                args.push("-video".to_string());
                args.push("bgfx".to_string());
            }
            VideoMode::Software => {
                args.push("-video".to_string());
                args.push("soft".to_string());
            }
            VideoMode::Auto => {} // Let MAME decide
        }
        
        // CORRECTED: Rotation handling
        match props.display.rotation {
            RotationMode::Default => {
                // Don't add any rotation arguments - let MAME handle it automatically
            }
            RotationMode::Rotate0 => {
                // No rotation needed - this is the normal orientation
                // Don't add any arguments
            }
            RotationMode::Rotate90 => {
                args.push("-ror".to_string()); // Rotate right (clockwise) 90 degrees
            }
            RotationMode::Rotate180 => {
                // For 180 degrees, we can either use -ror -ror or use flip X and Y
                // Using double rotation is more reliable
                args.push("-ror".to_string());
                args.push("-ror".to_string());
            }
            RotationMode::Rotate270 => {
                args.push("-rol".to_string()); // Rotate left (counter-clockwise) 90 degrees
            }
        }
        
        // CORRECTED: Flip options (these work independently of rotation)
        if props.display.flip_screen_upside_down {
            args.push("-flipy".to_string()); // Flip vertically
        }
        
        if props.display.flip_screen_left_right {
            args.push("-flipx".to_string()); // Flip horizontally
        }
        
        // OpenGL-specific options (when using OpenGL video mode)
        if matches!(props.display.video_mode, VideoMode::OpenGL) {
            let advanced = &props.advanced;
            
            // Force power-of-two textures
            if advanced.force_power_of_two_textures {
                args.push("-gl_forcepow2texture".to_string());
            }
            
            // GL_ARB_texture_rectangle
            if advanced.dont_use_gl_arb_texture_rectangle {
                args.push("-gl_notexturerect".to_string());
            }
            
            // VBO (Vertex Buffer Objects)
            if !advanced.enable_vbo {
                args.push("-nogl_vbo".to_string());
            }
            
            // PBO (Pixel Buffer Objects)
            if !advanced.enable_pbo {
                args.push("-nogl_pbo".to_string());
            }
            
            // GLSL
            if advanced.enable_glsl {
                args.push("-gl_glsl".to_string());
                
                // GLSL filter type
                match advanced.glsl_filter {
                    GLSLFilter::Plain => {
                        args.push("-gl_glsl_filter".to_string());
                        args.push("0".to_string());
                    }
                    GLSLFilter::Bilinear => {
                        // This is default, so we could skip it
                        args.push("-gl_glsl_filter".to_string());
                        args.push("1".to_string());
                    }
                    GLSLFilter::Bicubic => {
                        args.push("-gl_glsl_filter".to_string());
                        args.push("2".to_string());
                    }
                }
                
                // GLSL shader paths - MAME bitmaps
                for (i, shader_path) in advanced.glsl_shader_mame.iter().enumerate() {
                    if !shader_path.is_empty() {
                        args.push(format!("-glsl_shader_mame{}", i));
                        args.push(shader_path.clone());
                    }
                }
                
                // GLSL shader paths - Screen bitmaps
                for (i, shader_path) in advanced.glsl_shader_screen.iter().enumerate() {
                    if !shader_path.is_empty() {
                        args.push(format!("-glsl_shader_screen{}", i));
                        args.push(shader_path.clone());
                    }
                }
            }
        }
        
        // BGFX-specific options (when using BGFX video mode)
        if matches!(props.display.video_mode, VideoMode::BGFX) {
            let bgfx = &props.advanced.bgfx_settings;
            
            // BGFX backend
            match bgfx.backend {
                BGFXBackend::Auto => {
                    // Don't specify backend, let MAME auto-detect
                }
                BGFXBackend::D3D9 => {
                    args.push("-bgfx_backend".to_string());
                    args.push("d3d9".to_string());
                }
                BGFXBackend::D3D11 => {
                    args.push("-bgfx_backend".to_string());
                    args.push("d3d11".to_string());
                }
                BGFXBackend::D3D12 => {
                    args.push("-bgfx_backend".to_string());
                    args.push("d3d12".to_string());
                }
                BGFXBackend::OpenGL => {
                    args.push("-bgfx_backend".to_string());
                    args.push("opengl".to_string());
                }
                BGFXBackend::Metal => {
                    args.push("-bgfx_backend".to_string());
                    args.push("metal".to_string());
                }
                BGFXBackend::Vulkan => {
                    args.push("-bgfx_backend".to_string());
                    args.push("vulkan".to_string());
                }
            }
            
            // BGFX screen chains (shader chains)
            if !bgfx.screen_chains.is_empty() && bgfx.screen_chains != "default" {
                args.push("-bgfx_screen_chains".to_string());
                args.push(bgfx.screen_chains.clone());
            }
            
            // BGFX debug
            if bgfx.enable_debug {
                args.push("-bgfx_debug".to_string());
            }
            
            // Shadow mask
            if let Some(shadow_mask) = &bgfx.shadow_mask {
                if !shadow_mask.is_empty() {
                    args.push("-bgfx_shadow_mask".to_string());
                    args.push(shadow_mask.clone());
                }
            }
            
            // LUT (Look-Up Table)
            if let Some(lut) = &bgfx.lut_texture {
                if !lut.is_empty() {
                    args.push("-bgfx_lut".to_string());
                    args.push(lut.clone());
                }
            }
        }
        
        // Wait vsync
        if props.screen.wait_for_vertical_sync {
            args.push("-waitvsync".to_string());
        }
        
        // Sound
        match props.sound.sound_mode {
            SoundMode::None => {
                args.push("-sound".to_string());
                args.push("none".to_string());
            }
            SoundMode::SDL => {
                args.push("-sound".to_string());
                args.push("sdl".to_string());
            }
            _ => {}
        }
        
        // Samples
        if props.sound.use_samples {
            args.push("-samples".to_string());
        }
        
        // Sample rate
        if props.sound.sample_rate != 48000 {
            args.push("-samplerate".to_string());
            args.push(props.sound.sample_rate.to_string());
        }
        
        // Volume
        if props.sound.volume_attenuation != 0 {
            args.push("-volume".to_string());
            args.push(props.sound.volume_attenuation.to_string());
        }
        
        args
    }
    

    
    fn validate_shader_availability_static(shader_name: &str) -> ShaderStatus {
        // Common BGFX shader paths
        let shader_paths = vec![
            std::env::var("HOME").map(|home| format!("{}/.mame/bgfx/chains/{}.json", home, shader_name)).ok(),
            Some("/usr/share/mame/bgfx/chains/".to_string() + shader_name + ".json"),
            Some("./bgfx/chains/".to_string() + shader_name + ".json"),
            Some(format!("./bgfx/chains/{}.json", shader_name)),
        ];
        
        for path in shader_paths {
            if let Some(shader_path) = path {
                if Path::new(&shader_path).exists() {
                    return ShaderStatus::Available;
                }
            }
        }
        
        ShaderStatus::NotFound
    }
    
    fn apply_shader_preset(&mut self, preset: &str) {
        let advanced = &mut self.properties.advanced;
        
        // Clear existing shaders
        advanced.glsl_shader_mame = vec![String::new(); 10];
        advanced.glsl_shader_screen = vec![String::new(); 10];
        
        // Apply preset
        match preset {
            "crt" => {
                // Example CRT shader setup
                advanced.glsl_shader_screen[0] = "shaders/crt-geom.glsl".to_string();
            }
            "scanlines" => {
                advanced.glsl_shader_screen[0] = "shaders/scanlines.glsl".to_string();
            }
            "lcd" => {
                advanced.glsl_shader_screen[0] = "shaders/lcd-grid.glsl".to_string();
            }
            _ => {}
        }
    }
    
    fn apply_shader_preset_static(advanced: &mut AdvancedProperties, preset: &str) {
        // Clear existing shaders
        advanced.glsl_shader_mame = vec![String::new(); 10];
        advanced.glsl_shader_screen = vec![String::new(); 10];
        
        // Apply preset
        match preset {
            "crt" => {
                // Example CRT shader setup
                advanced.glsl_shader_screen[0] = "shaders/crt-geom.glsl".to_string();
            }
            "scanlines" => {
                advanced.glsl_shader_screen[0] = "shaders/scanlines.glsl".to_string();
            }
            "lcd" => {
                advanced.glsl_shader_screen[0] = "shaders/lcd-grid.glsl".to_string();
            }
            _ => {}
        }
    }
}