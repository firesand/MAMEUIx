//! Advanced MAME Settings Viewport Demo
//! 
//! This example demonstrates the Advanced MAME Settings dialog
//! using the viewport API to create a separate native window.

use eframe::egui;
use mameui::ui::components::AdvancedMameSettingsViewport;

struct DemoApp {
    settings_dialog: AdvancedMameSettingsViewport,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            settings_dialog: AdvancedMameSettingsViewport::new(),
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply dark theme
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = egui::Color32::from_rgb(22, 22, 22);
        style.visuals.panel_fill = egui::Color32::from_rgb(22, 22, 22);
        style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 51, 51));
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(42, 42, 42);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(51, 51, 51);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 60, 60);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(74, 158, 255);
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 224, 224));
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 224, 224));
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
        ctx.set_style(style);
        
        // Show the settings dialog as a separate window
        self.settings_dialog.show(ctx);
        
        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                
                ui.heading("Advanced MAME Settings Viewport Demo");
                
                ui.add_space(20.0);
                
                ui.label("This demo shows the Advanced MAME Settings dialog as a separate window.");
                ui.label("The dialog uses egui's viewport API to create a native window that can be moved independently.");
                
                ui.add_space(40.0);
                
                // Button to open the settings dialog
                let button_text = if self.settings_dialog.is_open() {
                    "Settings Window is Open"
                } else {
                    "Open Advanced MAME Settings"
                };
                
                let button = ui.add_sized(
                    egui::Vec2::new(250.0, 40.0),
                    egui::Button::new(button_text)
                        .fill(if self.settings_dialog.is_open() {
                            egui::Color32::from_rgb(60, 120, 60)
                        } else {
                            egui::Color32::from_rgb(74, 158, 255)
                        })
                );
                
                if button.clicked() && !self.settings_dialog.is_open() {
                    self.settings_dialog.open();
                }
                
                ui.add_space(20.0);
                
                // Status
                if self.settings_dialog.is_open() {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 200, 100),
                        "✓ Settings window is open. You can drag it around!"
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(136, 136, 136),
                        "Settings window is closed."
                    );
                }
                
                ui.add_space(40.0);
                
                // Instructions
                ui.group(|ui| {
                    ui.label("Instructions:");
                    ui.label("• Click the button to open the Advanced MAME Settings window");
                    ui.label("• The settings window can be moved independently from this main window");
                    ui.label("• Close the settings window using the X button or the Cancel/Apply buttons");
                    ui.label("• The window remembers its position while the app is running");
                });
                
                ui.add_space(40.0);
                
                // Show current settings (if any were applied)
                if let Some(settings) = self.get_applied_settings() {
                    ui.separator();
                    ui.add_space(20.0);
                    ui.heading("Applied Settings:");
                    ui.label(format!("ROM Path: {}", settings.paths.rom_path));
                    ui.label(format!("Sample Path: {}", settings.paths.sample_path));
                    // Add more settings display as needed
                }
            });
        });
    }
}

impl DemoApp {
    fn get_applied_settings(&self) -> Option<&mameui::ui::components::advanced_mame_settings::MameSettings> {
        // In a real implementation, this would return settings if they were applied
        None
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Advanced MAME Settings Viewport Demo"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Advanced MAME Settings Viewport Demo",
        options,
        Box::new(|_cc| Ok(Box::new(DemoApp::default()))),
    )
}