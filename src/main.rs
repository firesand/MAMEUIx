mod models;
mod ui;
mod mame;
mod config;
mod rom_utils;
mod graphics;

use eframe::egui;
use anyhow::Result;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MAME Frontend",
        options,
        Box::new(|cc| Ok(Box::new(ui::MameApp::new(cc)))),
    ).map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}
