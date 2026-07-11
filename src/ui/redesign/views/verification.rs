use super::super::fonts;
use super::super::tokens::RedesignTokens;
use super::super::widgets::{accent_button, card_frame, secondary_button};
use crate::app::MameApp;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, app: &mut MameApp) {
    egui::Frame::new()
        .inner_margin(egui::Margin::same(RedesignTokens::PAGE_PADDING as i8))
        .show(ui, |ui| {
            let compact = ui.available_width() < 680.0;
            let page_needs_scroll = compact || ui.available_height() < 520.0;
            if page_needs_scroll {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| show_content(ui, app, compact));
            } else {
                show_content(ui, app, false);
            }
        });
}

fn show_content(ui: &mut egui::Ui, app: &mut MameApp, compact: bool) {
    let version = app
        .config
        .mame_executables
        .get(app.config.selected_mame_index)
        .map(|m| m.version.as_str())
        .unwrap_or("—")
        .to_owned();

    if compact {
        show_heading(ui, &version);
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| show_actions(ui, app));
    } else {
        ui.horizontal(|ui| {
            show_heading(ui, &version);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                show_actions(ui, app);
            });
        });
    }

    ui.add_space(16.0);
    card_frame().show(ui, |ui| {
        app.dialog_manager
            .rom_verify_dialog()
            .show_redesign_panel(ui);
    });

    ui.add_space(16.0);
    card_frame().show(ui, |ui| {
        app.dialog_manager
            .rom_verify_dialog()
            .show_redesign_results(ui);
    });
}

fn show_heading(ui: &mut egui::Ui, version: &str) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new("ROM Verification")
                .font(fonts::bold(18.0))
                .color(RedesignTokens::TEXT_BRIGHT),
        );
        ui.label(
            egui::RichText::new(format!(
                "CLRMamePro Lite · CRC check against MAME {version} datfile"
            ))
            .font(fonts::regular(12.0))
            .color(RedesignTokens::TEXT_MUTED),
        );
    });
}

fn show_actions(ui: &mut egui::Ui, app: &mut MameApp) {
    let dialog = app.dialog_manager.rom_verify_dialog();
    let running = dialog.is_verifying();
    if accent_button(ui, if running { "⟲ Restart" } else { "▶ Start" }).clicked() {
        if running {
            dialog.stop_verification();
        }
        dialog.start_verification_all(&app.config, &app.games);
    }
    if running {
        if secondary_button(
            ui,
            if dialog.is_paused() {
                "Resume"
            } else {
                "Pause"
            },
        )
        .clicked()
        {
            dialog.toggle_pause();
        }
        if secondary_button(ui, "Stop").clicked() {
            dialog.stop_verification();
        }
    }
}
