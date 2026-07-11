//! Main entry point for the experimental redesign shell.
//! Completely isolated from legacy UI — toggle via `Preferences → ui_shell`.

use super::state::{RedesignScreen, RedesignState};
use super::tokens::RedesignTokens;
use super::topbar;
use super::views;
use crate::app::MameApp;
use crate::models::UiShellMode;
use eframe::egui;

#[derive(Default)]
pub struct RedesignShell {
    pub state: RedesignState,
}

impl RedesignShell {
    pub fn show(&mut self, ctx: &egui::Context, app: &mut MameApp) {
        if !self.state.style_applied {
            self.state.previous_style = Some((*ctx.style()).clone());
            RedesignTokens::apply(ctx);
            self.state.style_applied = true;
        }

        self.state.invalidate_on_games_loaded(app.games.len());

        let mame_version = app
            .config
            .mame_executables
            .get(app.config.selected_mame_index)
            .map(|m| m.version.as_str())
            .unwrap_or("—");
        let total = app.games.len();
        let available = if self.state.sidebar_stats.games_len == total {
            self.state.sidebar_stats.available
        } else {
            app.games
                .iter()
                .filter(|g| matches!(g.status, crate::models::RomStatus::Available))
                .count()
        };

        egui::TopBottomPanel::top("redesign_topbar")
            .exact_height(RedesignTokens::TOP_BAR_HEIGHT)
            .frame(
                egui::Frame::new()
                    .fill(RedesignTokens::BG_PANEL)
                    .stroke(egui::Stroke::new(1.0, RedesignTokens::BORDER))
                    .inner_margin(egui::Margin::symmetric(16, 0)),
            )
            .show(ctx, |ui| {
                topbar::show_top_bar(ui, &mut self.state, mame_version, total, available);
            });

        match self.state.screen {
            RedesignScreen::Library => {
                let action = views::library::show(ctx, app, &mut self.state);
                if let Some(idx) = action.open_detail {
                    app.selected_game = Some(idx);
                    self.state.open_detail(idx);
                }
                if let Some(name) = action.toggle_favorite {
                    app.toggle_favorite(&name);
                    self.state.mark_sidebar_stats_dirty();
                }
            }
            RedesignScreen::Detail => {
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(RedesignTokens::BG_ROOT)
                            .inner_margin(0.0),
                    )
                    .show(ctx, |ui| {
                        let action = views::detail::show(ui, app, &mut self.state);
                        if action.back {
                            self.state.back_to_library();
                        }
                        if action.play
                            && let Some(idx) = self.state.detail_game_index
                        {
                            app.launch_game_at_index(idx);
                        }
                        if action.toggle_favorite
                            && let Some(idx) = self.state.detail_game_index
                            && let Some(name) = app.games.get(idx).map(|g| g.name.clone())
                        {
                            app.toggle_favorite(&name);
                        }
                        if action.verify {
                            app.dialog_manager
                                .rom_verify_dialog()
                                .start_verification_all(&app.config, &app.games);
                            self.state
                                .navigate_to(super::state::RedesignNavTab::Verification);
                        }
                    });
            }
            RedesignScreen::Verification => {
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(RedesignTokens::BG_ROOT)
                            .inner_margin(0.0),
                    )
                    .show(ctx, |ui| {
                        views::verification::show(ui, app);
                    });
            }
            RedesignScreen::Settings => {
                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(RedesignTokens::BG_ROOT)
                            .inner_margin(0.0),
                    )
                    .show(ctx, |ui| {
                        let action =
                            views::settings::show(ui, app, &mut self.state.settings_section);
                        if action.style_changed
                            && app.config.preferences.ui_shell == UiShellMode::RedesignPreview
                        {
                            // Theme cards configure the legacy shell theme. Keep the
                            // redesign token palette active until the user leaves it.
                            RedesignTokens::apply(ctx);
                            app.theme_applied = true;
                        }
                        if action.save_config {
                            self.state.artwork_loader.clear_cache();
                        }
                        if action.rescan_needed {
                            app.on_directories_changed();
                            self.state.mark_sidebar_stats_dirty();
                            self.state.mark_table_dirty();
                        } else if action.save_config {
                            app.save_config();
                        }
                        if app.config.preferences.ui_shell != UiShellMode::RedesignPreview {
                            if let Some(previous_style) = self.state.previous_style.take() {
                                ctx.set_style(previous_style);
                            }
                            app.config.theme.apply(ctx);
                            self.state.style_applied = false;
                        }
                    });
            }
        }

        self.handle_shortcuts(ctx, app);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context, app: &mut MameApp) {
        ctx.input(|i| {
            if i.modifiers.command_only() && i.key_pressed(egui::Key::F) {
                self.state.request_search_focus();
                self.state.screen = RedesignScreen::Library;
                ctx.request_repaint();
            }
            if i.key_pressed(egui::Key::Escape) && !self.state.search_text_buf.is_empty() {
                self.state.search_text_buf.clear();
                app.config.filter_settings.search_text.clear();
                app.game_index_manager.mark_cache_dirty();
                self.state.search_debounce_deadline = None;
                self.state.mark_table_dirty();
            }
        });
    }
}
