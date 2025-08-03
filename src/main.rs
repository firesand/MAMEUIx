mod models;
mod ui;
mod mame;
mod config;
mod app;
mod utils;
mod embedded_shaders;

use eframe::egui;
use anyhow::Result;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_decorations(true) // Keep decorations for window movement/resize
            .with_resizable(true)
            .with_transparent(false), // Disable transparency
        ..Default::default()
    };

    eframe::run_native(
        "MAMEUIx",
        options,
        Box::new(|cc| Ok(Box::new(app::MameApp::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}
