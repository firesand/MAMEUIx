use eframe::egui;
use crate::models::VideoSettings;

pub struct VideoSettingsDialog;

impl VideoSettingsDialog {
    pub fn show(ctx: &egui::Context, settings: &mut VideoSettings, open: &mut bool) {
        let mut close = false;

        egui::Window::new("Video Settings")
        .default_size([400.0, 500.0])
        .open(open)
        .show(ctx, |ui| {
            ui.heading("Video Options");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Video Backend:");
                egui::ComboBox::from_label("")
                .selected_text(&settings.video_backend)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings.video_backend, "auto".to_string(), "Auto");
                    ui.selectable_value(&mut settings.video_backend, "opengl".to_string(), "OpenGL");
                    ui.selectable_value(&mut settings.video_backend, "bgfx".to_string(), "BGFX");
                });
            });

            ui.checkbox(&mut settings.window_mode, "Run in window");
            ui.checkbox(&mut settings.maximize, "Start maximized");
            ui.checkbox(&mut settings.wait_vsync, "Wait for V-Sync");
            ui.checkbox(&mut settings.keep_aspect, "Keep aspect ratio");
            ui.checkbox(&mut settings.filter, "Bilinear filtering");

            ui.horizontal(|ui| {
                ui.label("Prescale:");
                ui.add(egui::Slider::new(&mut settings.prescale, 0..=3));
            });

            ui.separator();

            ui.label("Custom arguments:");
            ui.text_edit_singleline(&mut settings.custom_args);

            ui.separator();

            if ui.button("Close").clicked() {
                close = true;
            }
        });

        if close {
            *open = false;
        }
    }
}
