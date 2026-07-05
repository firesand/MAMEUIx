#![allow(dead_code)] // Roadmap modules (hardware_filter, embedded_shaders, ini_utils, etc.)
#![allow(clippy::too_many_arguments)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::upper_case_acronyms)]

mod app;
mod config;
mod embedded_shaders;
mod mame;
mod models;
mod ui;
mod utils;

use anyhow::Result;
use eframe::egui;

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
    )
    .map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}
