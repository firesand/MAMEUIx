use super::super::fonts;
use super::super::state::RedesignState;
use super::super::tokens::RedesignTokens;
use super::super::widgets::{accent_button, card_frame, secondary_button, status_dot, text_link};
use crate::app::MameApp;
use crate::models::{AppConfig, Game, RomStatus};
use crate::ui::panels::artwork_loader::{ArtworkLoader, ArtworkType};
use eframe::egui;

const NARROW_BREAKPOINT: f32 = 720.0;
const MEDIA_RAIL_WIDTH: f32 = 340.0;
const CONTENT_GAP: f32 = 20.0;
const THUMBNAIL_GAP: f32 = 10.0;

pub struct DetailAction {
    pub back: bool,
    pub play: bool,
    pub toggle_favorite: bool,
    pub verify: bool,
}

pub fn show(ui: &mut egui::Ui, app: &mut MameApp, state: &mut RedesignState) -> DetailAction {
    let mut action = DetailAction {
        back: false,
        play: false,
        toggle_favorite: false,
        verify: false,
    };

    let Some(idx) = state.detail_game_index else {
        state.back_to_library();
        return action;
    };
    let Some(game) = app.games.get(idx).cloned() else {
        state.back_to_library();
        return action;
    };
    let narrow = ui.available_width() < NARROW_BREAKPOINT;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(RedesignTokens::PAGE_PADDING);
                if text_link(
                    ui,
                    "← LIBRARY",
                    fonts::semibold(12.0),
                    RedesignTokens::TEXT_SECONDARY,
                )
                .clicked()
                {
                    action.back = true;
                }
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(RedesignTokens::PAGE_PADDING);
                let available_width = ui.available_width().max(0.0);
                let hero_width = (available_width - RedesignTokens::PAGE_PADDING).max(0.0);
                show_hero_artwork(ui, app, state, &game, hero_width);
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(RedesignTokens::PAGE_PADDING);
                if narrow {
                    let content_width =
                        (ui.available_width() - RedesignTokens::PAGE_PADDING).max(0.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(content_width, 0.0),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            show_title(ui, &game);
                            ui.add_space(10.0);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center)
                                    .with_main_wrap(true),
                                |ui| {
                                    show_actions(
                                        ui,
                                        &mut action,
                                        app.config.favorite_games.contains(&game.name),
                                    );
                                },
                            );
                        },
                    );
                } else {
                    show_title(ui, &game);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(RedesignTokens::PAGE_PADDING);
                        show_actions(
                            ui,
                            &mut action,
                            app.config.favorite_games.contains(&game.name),
                        );
                    });
                }
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(RedesignTokens::PAGE_PADDING);
                let total_w = ui.available_width() - RedesignTokens::PAGE_PADDING;
                ui.allocate_ui_with_layout(
                    egui::vec2(total_w.max(0.0), 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        if narrow {
                            show_info_card(ui, app, &game, true);
                            ui.add_space(CONTENT_GAP);

                            let rail_width = ui.available_width().clamp(0.0, MEDIA_RAIL_WIDTH);
                            ui.allocate_ui_with_layout(
                                egui::vec2(rail_width, 0.0),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| show_media_rail(ui, app, state, &game, rail_width),
                            );
                            ui.add_space(CONTENT_GAP);
                            show_history_card(ui, app, &game, true);
                        } else {
                            ui.horizontal_top(|ui| {
                                let left_w =
                                    (ui.available_width() - MEDIA_RAIL_WIDTH - CONTENT_GAP)
                                        .max(280.0);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(left_w, 0.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| show_info_card(ui, app, &game, false),
                                );

                                ui.add_space(CONTENT_GAP);
                                ui.allocate_ui_with_layout(
                                    egui::vec2(MEDIA_RAIL_WIDTH, 0.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| show_media_rail(ui, app, state, &game, MEDIA_RAIL_WIDTH),
                                );
                            });
                            ui.add_space(16.0);
                            show_history_card(ui, app, &game, false);
                        }
                    },
                );
            });
            ui.add_space(32.0);
        });

    action
}

fn show_title(ui: &mut egui::Ui, game: &Game) {
    ui.vertical(|ui| {
        ui.label(
            egui::RichText::new(&game.description)
                .font(fonts::bold(26.0))
                .color(RedesignTokens::TEXT_BRIGHT),
        );
        ui.label(
            egui::RichText::new(format!(
                "{} · {} · {}",
                game.manufacturer, game.year, game.category
            ))
            .font(fonts::regular(13.0))
            .color(RedesignTokens::TEXT_SECONDARY),
        );
    });
}

fn show_actions(ui: &mut egui::Ui, action: &mut DetailAction, is_favorite: bool) {
    if accent_button(ui, "▶ PLAY").clicked() {
        action.play = true;
    }
    ui.add_space(10.0);
    let star = if is_favorite { "★" } else { "☆" };
    let star_color = if is_favorite {
        RedesignTokens::STATUS_WARN
    } else {
        RedesignTokens::STAR_INACTIVE
    };
    if ui
        .add_sized(
            [40.0, 40.0],
            egui::Button::new(
                egui::RichText::new(star)
                    .font(fonts::regular(16.0))
                    .color(star_color),
            ),
        )
        .clicked()
    {
        action.toggle_favorite = true;
    }
    ui.add_space(10.0);
    if secondary_button(ui, "Verify ROM").clicked() {
        action.verify = true;
    }
}

fn show_info_card(ui: &mut egui::Ui, app: &mut MameApp, game: &Game, narrow: bool) {
    card_frame().show(ui, |ui| {
        let label_width = if narrow {
            adaptive_info_label_width(ui.available_width())
        } else {
            140.0
        };

        info_row(ui, "ROM set", &format!("{}.zip", game.name), label_width);
        info_row_status(ui, "Status", game.status, label_width);
        if game.is_clone {
            if let Some(parent) = &game.parent {
                let parent_title = app
                    .games
                    .iter()
                    .find(|g| &g.name == parent)
                    .map(|g| g.description.as_str())
                    .unwrap_or(parent);
                info_row(
                    ui,
                    "Clone of",
                    &format!("{parent} — {parent_title}"),
                    label_width,
                );
            }
        } else {
            let clones = app
                .games
                .iter()
                .filter(|g| g.parent.as_deref() == Some(game.name.as_str()))
                .count();
            if clones > 0 {
                info_row(
                    ui,
                    "Clones",
                    &format!("{clones} sets in collection"),
                    label_width,
                );
            }
        }
        info_row(ui, "Driver", &game.driver, label_width);
        info_row(ui, "Driver status", &game.driver_status, label_width);
        info_row(ui, "Category", &game.category, label_width);
        info_row(ui, "Play count", &game.play_count.to_string(), label_width);
    });
}

fn show_history_card(ui: &mut egui::Ui, app: &mut MameApp, game: &Game, narrow: bool) {
    card_frame().show(ui, |ui| {
        ui.label(
            egui::RichText::new("HISTORY")
                .font(fonts::bold(11.0))
                .color(RedesignTokens::TEXT_FAINT)
                .extra_letter_spacing(0.8),
        );
        ui.add_space(8.0);
        app.history_panel.set_selected_game(
            Some(game.name.clone()),
            Some(game.name.clone()),
            &app.config,
        );
        let min_body_height = if narrow { 240.0 } else { 360.0 };
        app.history_panel.show_redesign_reader(ui, min_body_height);
    });
}

fn adaptive_info_label_width(available_width: f32) -> f32 {
    let available_width = available_width.max(0.0);
    (available_width * 0.36)
        .clamp(72.0, 140.0)
        .min(available_width * 0.5)
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str, label_width: f32) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(label_width, 24.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    egui::RichText::new(label)
                        .font(fonts::regular(13.0))
                        .color(RedesignTokens::TEXT_MUTED),
                );
            },
        );
        ui.add(
            egui::Label::new(
                egui::RichText::new(value)
                    .font(fonts::medium(13.0))
                    .color(RedesignTokens::TEXT_PRIMARY),
            )
            .wrap(),
        );
    });
    ui.separator();
}

fn info_row_status(ui: &mut egui::Ui, label: &str, status: RomStatus, label_width: f32) {
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(label_width, 24.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(13.0)
                        .color(RedesignTokens::TEXT_MUTED),
                );
            },
        );
        ui.horizontal(|ui| {
            status_dot(ui, RedesignTokens::status_color(status));
            ui.label(
                egui::RichText::new(RedesignTokens::status_label(status))
                    .font(fonts::medium(13.0))
                    .color(RedesignTokens::status_color(status)),
            );
        });
    });
    ui.separator();
}

fn show_hero_artwork(
    ui: &mut egui::Ui,
    app: &MameApp,
    state: &mut RedesignState,
    game: &Game,
    width: f32,
) {
    let texture = load_game_artwork(
        ui.ctx(),
        &mut state.artwork_loader,
        game,
        &[
            ArtworkType::Marquee,
            ArtworkType::Title,
            ArtworkType::Screenshot,
        ],
        &app.config,
    );
    artwork_box(ui, "Artwork", egui::vec2(width, 230.0), texture);
}

fn show_media_rail(
    ui: &mut egui::Ui,
    app: &MameApp,
    state: &mut RedesignState,
    game: &Game,
    width: f32,
) {
    let snapshot = load_game_artwork(
        ui.ctx(),
        &mut state.artwork_loader,
        game,
        &[ArtworkType::Screenshot],
        &app.config,
    );
    artwork_box(ui, "Snapshot", egui::vec2(width, width * 0.75), snapshot);
    ui.add_space(THUMBNAIL_GAP);
    ui.horizontal(|ui| {
        let gap = THUMBNAIL_GAP.min(width / 2.0).max(0.0);
        ui.spacing_mut().item_spacing.x = gap;
        let thumbnail_width = ((width - 2.0 * gap) / 3.0).max(0.0);
        for (label, artwork_type) in [
            ("Flyer", ArtworkType::Flyer),
            ("Cabinet", ArtworkType::Cabinet),
            ("PCB", ArtworkType::Pcb),
        ] {
            let texture = load_game_artwork(
                ui.ctx(),
                &mut state.artwork_loader,
                game,
                &[artwork_type],
                &app.config,
            );
            artwork_box(
                ui,
                label,
                egui::vec2(thumbnail_width, thumbnail_width * 0.75),
                texture,
            );
        }
    });
}

fn load_game_artwork(
    ctx: &egui::Context,
    loader: &mut ArtworkLoader,
    game: &Game,
    artwork_types: &[ArtworkType],
    config: &AppConfig,
) -> Option<egui::TextureHandle> {
    for rom_name in game_artwork_candidates(game) {
        for &artwork_type in artwork_types {
            if let Some(texture) = loader.load_artwork(ctx, rom_name, artwork_type, config) {
                return Some(texture);
            }
        }
    }
    None
}

fn game_artwork_candidates(game: &Game) -> Vec<&str> {
    let mut candidates = vec![game.name.as_str()];
    if let Some(parent) = game.parent.as_deref()
        && parent != game.name
    {
        candidates.push(parent);
    }
    candidates
}

fn artwork_box(
    ui: &mut egui::Ui,
    label: &str,
    size: egui::Vec2,
    texture: Option<egui::TextureHandle>,
) {
    let size = egui::vec2(size.x.max(0.0), size.y.max(0.0));
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(RedesignTokens::RADIUS_LG),
        RedesignTokens::BG_SURFACE,
    );
    if let Some(texture) = texture {
        paint_contained_image(ui, rect, &texture);
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            fonts::regular(12.0),
            RedesignTokens::TEXT_FAINT,
        );
    }
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(RedesignTokens::RADIUS_LG),
        egui::Stroke::new(1.0, RedesignTokens::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );
}

fn paint_contained_image(ui: &egui::Ui, rect: egui::Rect, texture: &egui::TextureHandle) {
    let texture_size = texture.size_vec2();
    if texture_size.x <= 0.0 || texture_size.y <= 0.0 || rect.width() <= 0.0 || rect.height() <= 0.0
    {
        return;
    }

    let scale = (rect.width() / texture_size.x).min(rect.height() / texture_size.y);
    let image_size = texture_size * scale;
    let image_rect = egui::Rect::from_center_size(rect.center(), image_size);
    let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
    ui.painter()
        .image(texture.id(), image_rect, uv, egui::Color32::WHITE);
}
