use super::super::fonts;
use super::super::state::SettingsSection;
use super::super::tokens::RedesignTokens;
use super::super::topbar::legacy_shell_switcher;
use super::super::widgets::{card_frame, secondary_button, section_header, sidebar_row};
use crate::app::MameApp;
use crate::models::{MameExecutable, Theme, UiShellMode, VideoMode};
use crate::ui::components::mame_finder::MameFinderDialog;
use eframe::egui;
use std::path::{Path, PathBuf};

pub struct SettingsAction {
    pub style_changed: bool,
    pub save_config: bool,
    pub rescan_needed: bool,
}

const COMPACT_SETTINGS_WIDTH: f32 = 680.0;

pub fn show(ui: &mut egui::Ui, app: &mut MameApp, section: &mut SettingsSection) -> SettingsAction {
    let mut action = SettingsAction {
        style_changed: false,
        save_config: false,
        rescan_needed: false,
    };

    if uses_compact_settings_layout(ui.available_width()) {
        egui::Frame::new()
            .fill(RedesignTokens::BG_ROOT)
            .inner_margin(egui::Margin::same(RedesignTokens::PAGE_PADDING as i8))
            .show(ui, |ui| {
                show_compact_section_switcher(ui, section);
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                show_scrollable_section(ui, app, *section, &mut action);
            });
    } else {
        ui.horizontal_top(|ui| {
            let available_height = ui.available_height();
            let sidebar_w = RedesignTokens::SIDEBAR_WIDTH;
            ui.allocate_ui_with_layout(
                egui::vec2(sidebar_w, available_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    let rect = ui.max_rect();
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius::ZERO,
                        RedesignTokens::BG_PANEL,
                    );
                    ui.painter().line_segment(
                        [rect.right_top(), rect.right_bottom()],
                        egui::Stroke::new(1.0, RedesignTokens::BORDER),
                    );
                    ui.set_min_height(available_height);
                    ui.set_width(sidebar_w);
                    show_desktop_sidebar(ui, section);
                },
            );

            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), available_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    egui::Frame::new()
                        .inner_margin(egui::Margin::same(RedesignTokens::PAGE_PADDING as i8))
                        .show(ui, |ui| {
                            show_scrollable_section(ui, app, *section, &mut action);
                        });
                },
            );
        });
    }

    action
}

fn uses_compact_settings_layout(available_width: f32) -> bool {
    available_width < COMPACT_SETTINGS_WIDTH
}

fn show_scrollable_section(
    ui: &mut egui::Ui,
    app: &mut MameApp,
    section: SettingsSection,
    action: &mut SettingsAction,
) {
    let viewport_height = ui.available_height().max(0.0);
    egui::ScrollArea::vertical()
        .id_salt(settings_scroll_id(section))
        .max_height(viewport_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            show_section_content(ui, app, section, action);
            ui.add_space(RedesignTokens::PAGE_PADDING);
        });
}

fn settings_scroll_id(section: SettingsSection) -> &'static str {
    match section {
        SettingsSection::Directories => "redesign_settings_directories",
        SettingsSection::Appearance => "redesign_settings_appearance",
        SettingsSection::Performance => "redesign_settings_performance",
        SettingsSection::Shaders => "redesign_settings_shaders",
    }
}

const SETTINGS_SECTIONS: [(SettingsSection, &str); 4] = [
    (SettingsSection::Directories, "Directories"),
    (SettingsSection::Appearance, "Appearance"),
    (SettingsSection::Performance, "Performance"),
    (SettingsSection::Shaders, "Shaders"),
];

fn show_desktop_sidebar(ui: &mut egui::Ui, section: &mut SettingsSection) {
    ui.add_space(12.0);
    ui.horizontal_top(|ui| {
        ui.add_space(8.0);
        ui.allocate_ui_with_layout(
            egui::vec2((RedesignTokens::SIDEBAR_WIDTH - 16.0).max(0.0), 0.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| show_sidebar_rows(ui, section),
        );
    });
}

fn show_sidebar_rows(ui: &mut egui::Ui, section: &mut SettingsSection) {
    for (sec, label) in SETTINGS_SECTIONS {
        let selected = *section == sec;
        if sidebar_row(ui, selected, |ui| {
            ui.label(
                egui::RichText::new(label)
                    .font(fonts::medium(12.5))
                    .color(if selected {
                        RedesignTokens::TEXT_BRIGHT
                    } else {
                        RedesignTokens::TEXT_SECONDARY
                    }),
            );
        })
        .clicked()
        {
            *section = sec;
        }
    }
}

fn show_compact_section_switcher(ui: &mut egui::Ui, section: &mut SettingsSection) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
        for (sec, label) in SETTINGS_SECTIONS {
            let selected = *section == sec;
            let mut button = egui::Button::new(
                egui::RichText::new(label)
                    .font(fonts::semibold(12.0))
                    .color(if selected {
                        RedesignTokens::TEXT_BRIGHT
                    } else {
                        RedesignTokens::TEXT_SECONDARY
                    }),
            )
            .selected(selected);
            if selected {
                button = button.stroke(egui::Stroke::new(1.0, RedesignTokens::ACCENT));
            }
            if ui.add(button).clicked() {
                *section = sec;
            }
        }
    });
}

fn show_section_content(
    ui: &mut egui::Ui,
    app: &mut MameApp,
    section: SettingsSection,
    action: &mut SettingsAction,
) {
    let max_width = match section {
        SettingsSection::Directories => 720.0,
        SettingsSection::Appearance => 700.0,
        SettingsSection::Performance => 640.0,
        SettingsSection::Shaders => 760.0,
    };
    ui.set_max_width(ui.available_width().min(max_width));

    match section {
        SettingsSection::Directories => show_directories(ui, app, action),
        SettingsSection::Appearance => show_appearance(ui, app, action),
        SettingsSection::Performance => show_performance(ui, app, action),
        SettingsSection::Shaders => show_shaders(ui, app, action),
    }
}

fn show_directories(ui: &mut egui::Ui, app: &mut MameApp, action: &mut SettingsAction) {
    ui.label(
        egui::RichText::new("Directories")
            .font(fonts::bold(18.0))
            .color(RedesignTokens::TEXT_BRIGHT),
    );
    ui.label(
        egui::RichText::new(
            "Paths are validated on save. A rescan runs automatically when ROM paths change.",
        )
        .size(12.0)
        .color(RedesignTokens::TEXT_MUTED),
    );
    ui.add_space(16.0);

    section_header(ui, "MAME");
    ui.add_space(8.0);
    mame_executable_row(ui, app, action);
    ui.add_space(10.0);

    path_list_row(
        ui,
        "ROMs",
        &mut app.config.rom_paths,
        Some(&mut app.config.rom_dirs),
        "Select ROM Directory",
        action,
        true,
    );
    ui.add_space(10.0);

    path_list_row(
        ui,
        "CHDs",
        &mut app.config.extra_rom_dirs,
        None,
        "Select CHD Directory",
        action,
        true,
    );
    ui.add_space(18.0);

    section_header(ui, "ARTWORK");
    ui.add_space(8.0);
    optional_folder_row(
        ui,
        "Artwork root",
        &mut app.config.artwork_path,
        "Select Artwork Directory",
        action,
    );
    ui.add_space(10.0);
    optional_folder_row(
        ui,
        "Snapshots",
        &mut app.config.snap_path,
        "Select Snapshot Directory",
        action,
    );
    ui.add_space(10.0);
    optional_folder_row(
        ui,
        "Marquees",
        &mut app.config.marquee_path,
        "Select Marquee Directory",
        action,
    );
    ui.add_space(10.0);
    optional_folder_row(
        ui,
        "Title screens",
        &mut app.config.title_path,
        "Select Title Screen Directory",
        action,
    );
    ui.add_space(10.0);
    optional_folder_row(
        ui,
        "Flyers",
        &mut app.config.flyer_path,
        "Select Flyer Directory",
        action,
    );
    ui.add_space(10.0);
    optional_folder_row(
        ui,
        "Cabinets",
        &mut app.config.cabinet_path,
        "Select Cabinet Directory",
        action,
    );
    ui.add_space(10.0);
    optional_folder_row(
        ui,
        "PCB",
        &mut app.config.pcb_path,
        "Select PCB Directory",
        action,
    );
    ui.add_space(18.0);

    section_header(ui, "DATA FILES");
    ui.add_space(8.0);
    optional_file_row(
        ui,
        "catver.ini",
        &mut app.config.catver_ini_path,
        "Select catver.ini",
        &[("INI files", &["ini"])],
        action,
        true,
    );
    ui.add_space(10.0);
    optional_file_row(
        ui,
        "history.xml",
        &mut app.config.history_path,
        "Select history.xml",
        &[("XML files", &["xml"])],
        action,
        false,
    );
    ui.add_space(10.0);
    optional_file_row(
        ui,
        "mameinfo.dat",
        &mut app.config.mameinfo_dat_path,
        "Select mameinfo.dat",
        &[("DAT files", &["dat"])],
        action,
        false,
    );
    ui.add_space(10.0);
    optional_file_row(
        ui,
        "command.dat",
        &mut app.config.command_dat_path,
        "Select command.dat",
        &[("DAT files", &["dat"])],
        action,
        false,
    );
    ui.add_space(10.0);
    optional_file_row(
        ui,
        "hiscore.dat",
        &mut app.config.hiscore_dat_path,
        "Select hiscore.dat",
        &[("DAT files", &["dat"])],
        action,
        false,
    );
    ui.add_space(10.0);
    optional_file_row(
        ui,
        "gameinit.dat",
        &mut app.config.gameinit_dat_path,
        "Select gameinit.dat",
        &[("DAT files", &["dat"])],
        action,
        false,
    );
}

fn mame_executable_row(ui: &mut egui::Ui, app: &mut MameApp, action: &mut SettingsAction) {
    let current_path = app
        .config
        .mame_executables
        .get(app.config.selected_mame_index)
        .map(|m| m.path.as_str())
        .unwrap_or("not set");

    if path_row(ui, "MAME executable", current_path, "Browse...") {
        let mut dialog = rfd::FileDialog::new().set_title("Select MAME Executable");
        if cfg!(target_os = "windows") {
            dialog = dialog
                .add_filter("Executable files", &["exe", "EXE"])
                .add_filter("All files", &["*"]);
        } else {
            dialog = dialog.add_filter("All files", &["*"]);
        }
        if current_path != "not set" {
            if let Some(parent) = Path::new(current_path).parent() {
                dialog = dialog.set_directory(parent);
            }
        } else if cfg!(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd"
        )) {
            dialog = dialog.set_directory(MameFinderDialog::unix_browse_directory());
        }

        if let Some(path) = dialog.pick_file() {
            let resolved = MameFinderDialog::resolve_executable_path(&path.display().to_string());
            if app.config.mame_executables.is_empty() {
                app.config.mame_executables.push(MameExecutable {
                    name: "MAME".to_string(),
                    path: resolved,
                    version: "Not validated".to_string(),
                    total_games: 0,
                    working_games: 0,
                });
                app.config.selected_mame_index = 0;
            } else {
                let idx = app
                    .config
                    .selected_mame_index
                    .min(app.config.mame_executables.len() - 1);
                app.config.selected_mame_index = idx;
                app.config.mame_executables[idx].path = resolved;
            }
            mark_config_saved(action, true);
        }
    }
}

fn path_list_row(
    ui: &mut egui::Ui,
    label: &str,
    paths: &mut Vec<PathBuf>,
    mirror: Option<&mut Vec<PathBuf>>,
    title: &str,
    action: &mut SettingsAction,
    rescan_needed: bool,
) {
    let display = paths
        .first()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "not set".to_string());
    if path_row(ui, label, &display, "Browse...")
        && let Some(folder) = pick_folder(title, paths.first())
    {
        if paths.is_empty() {
            paths.push(folder.clone());
        } else {
            paths[0] = folder.clone();
        }
        if let Some(mirror) = mirror {
            if mirror.is_empty() {
                mirror.push(folder);
            } else {
                mirror[0] = folder;
            }
        }
        mark_config_saved(action, rescan_needed);
    }
}

fn optional_folder_row(
    ui: &mut egui::Ui,
    label: &str,
    path: &mut Option<PathBuf>,
    title: &str,
    action: &mut SettingsAction,
) {
    let display = optional_path_text(path, "not set");
    if path_row(ui, label, &display, "Browse...")
        && let Some(folder) = pick_folder(title, path.as_ref())
    {
        *path = Some(folder);
        mark_config_saved(action, false);
    }
}

fn optional_file_row(
    ui: &mut egui::Ui,
    label: &str,
    path: &mut Option<PathBuf>,
    title: &str,
    filters: &[(&str, &[&str])],
    action: &mut SettingsAction,
    rescan_needed: bool,
) {
    let display = optional_path_text(path, "not set");
    if path_row(ui, label, &display, "Browse...")
        && let Some(file) = pick_file(title, path.as_ref(), filters)
    {
        *path = Some(file);
        mark_config_saved(action, rescan_needed);
    }
}

fn optional_path_text(path: &Option<PathBuf>, fallback: &str) -> String {
    path.as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| fallback.to_string())
}

fn mark_config_saved(action: &mut SettingsAction, rescan_needed: bool) {
    action.save_config = true;
    action.rescan_needed |= rescan_needed;
}

fn pick_folder(title: &str, current: Option<&PathBuf>) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title(title);
    if let Some(path) = current {
        let start = if path.is_dir() {
            Some(path.as_path())
        } else {
            path.parent()
        };
        if let Some(start) = start {
            dialog = dialog.set_directory(start);
        }
    }
    dialog.pick_folder()
}

fn pick_file(
    title: &str,
    current: Option<&PathBuf>,
    filters: &[(&str, &[&str])],
) -> Option<PathBuf> {
    let mut dialog = rfd::FileDialog::new().set_title(title);
    for (name, extensions) in filters {
        dialog = dialog.add_filter(*name, extensions);
    }
    dialog = dialog.add_filter("All files", &["*"]);
    if let Some(path) = current {
        let start = if path.is_dir() {
            Some(path.as_path())
        } else {
            path.parent()
        };
        if let Some(start) = start {
            dialog = dialog.set_directory(start);
        }
    }
    dialog.pick_file()
}

fn path_row(ui: &mut egui::Ui, label: &str, path: &str, button_label: &str) -> bool {
    let label = egui::RichText::new(label)
        .size(13.0)
        .color(RedesignTokens::TEXT_SECONDARY);
    let available = ui.available_width();
    let mut clicked = false;

    if available < 340.0 {
        ui.label(label);
        path_field(ui, path, ui.available_width());
        clicked = secondary_button(ui, button_label).clicked();
    } else if available < 560.0 {
        ui.label(label);
        ui.horizontal(|ui| {
            let field_width = (ui.available_width() - 100.0).max(120.0);
            path_field(ui, path, field_width);
            clicked = secondary_button(ui, button_label).clicked();
        });
    } else {
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(150.0, 28.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(label);
                },
            );
            let field_width = (ui.available_width() - 100.0).max(120.0);
            path_field(ui, path, field_width);
            clicked = secondary_button(ui, button_label).clicked();
        });
    }
    clicked
}

fn path_field(ui: &mut egui::Ui, path: &str, width: f32) {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width.max(0.0), 28.0), egui::Sense::hover());
    ui.painter().rect(
        rect,
        egui::CornerRadius::same(RedesignTokens::RADIUS_MD),
        RedesignTokens::BG_SURFACE,
        egui::Stroke::new(1.0, RedesignTokens::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );

    let text_color = if path.starts_with("not set") || path.starts_with("same as") {
        RedesignTokens::TEXT_MUTED
    } else {
        RedesignTokens::TEXT_SECONDARY
    };
    let text_rect = rect.shrink2(egui::vec2(12.0, 0.0));
    let painter = ui.painter().with_clip_rect(text_rect);
    let galley = truncated_single_line(
        &painter,
        path,
        egui::FontId::new(12.0, egui::FontFamily::Monospace),
        text_color,
        text_rect.width().max(0.0),
    );
    let elided = galley.elided;
    let text_position = egui::Align2::LEFT_CENTER
        .anchor_size(text_rect.left_center(), galley.size())
        .min;
    painter.galley(text_position, galley, text_color);
    if elided {
        response.on_hover_text(path);
    }
}

fn truncated_single_line(
    painter: &egui::Painter,
    text: &str,
    font_id: egui::FontId,
    color: egui::Color32,
    max_width: f32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = egui::text::LayoutJob::simple_singleline(text.to_string(), font_id, color);
    job.wrap = egui::text::TextWrapping::truncate_at_width(max_width.max(0.0));
    painter.layout_job(job)
}

fn show_appearance(ui: &mut egui::Ui, app: &mut MameApp, action: &mut SettingsAction) {
    ui.label(
        egui::RichText::new("Appearance")
            .font(fonts::bold(18.0))
            .color(RedesignTokens::TEXT_BRIGHT),
    );
    ui.label(
        egui::RichText::new(
            "Themes apply when using a legacy UI shell. The redesign preview keeps its fixed palette.",
        )
            .size(12.0)
            .color(RedesignTokens::TEXT_MUTED),
    );
    ui.add_space(16.0);

    let column_count = adaptive_column_count(ui.available_width(), 156.0, 4);
    ui.columns(column_count, |cols| {
        let themes = [
            Theme::DarkGrey,
            Theme::DarkBlue,
            Theme::NeonGreen,
            Theme::ArcadePurple,
            Theme::LightClassic,
            Theme::SunsetOrange,
            Theme::OceanBlue,
            Theme::MidnightBlack,
            Theme::ForestGreen,
            Theme::RetroAmber,
            Theme::ModernSpacious,
        ];
        for (i, theme) in themes.iter().enumerate() {
            let col = i % column_count;
            show_theme_card(&mut cols[col], app, theme, action);
            cols[col].add_space(12.0);
        }
    });

    ui.add_space(16.0);
    section_header(ui, "UI SHELL (EXPERIMENTAL)");
    ui.add_space(4.0);
    legacy_shell_switcher(
        ui,
        &mut app.config.preferences.ui_shell,
        &mut action.style_changed,
    );
    if action.style_changed && app.config.preferences.ui_shell != UiShellMode::RedesignPreview {
        action.save_config = true;
    }
}

fn show_theme_card(
    ui: &mut egui::Ui,
    app: &mut MameApp,
    theme: &Theme,
    action: &mut SettingsAction,
) {
    let selected = app.config.theme == *theme;
    let stroke = if selected {
        egui::Stroke::new(1.0, RedesignTokens::ACCENT)
    } else {
        egui::Stroke::new(1.0, RedesignTokens::BORDER)
    };
    let background = ui.painter().add(egui::Shape::Noop);
    let response = egui::Frame::new()
        .fill(egui::Color32::TRANSPARENT)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(RedesignTokens::RADIUS_LG))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_height(76.0);
            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::hover());
            ui.painter().rect(
                rect,
                egui::CornerRadius::same(RedesignTokens::RADIUS_MD),
                theme_swatch_color(theme),
                egui::Stroke::new(1.0, RedesignTokens::BORDER_STRONG),
                egui::StrokeKind::Inside,
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(theme.display_name())
                    .font(fonts::semibold(12.0))
                    .color(RedesignTokens::TEXT_PRIMARY),
            );
            ui.label(
                egui::RichText::new(if selected { "current" } else { "built-in" })
                    .size(11.0)
                    .color(RedesignTokens::TEXT_MUTED),
            );
        })
        .response
        .interact(egui::Sense::click());

    ui.painter().set(
        background,
        egui::Shape::rect_filled(
            response.rect,
            egui::CornerRadius::same(RedesignTokens::RADIUS_LG),
            if response.hovered() {
                RedesignTokens::BG_ROW_HOVER
            } else {
                RedesignTokens::BG_PANEL
            },
        ),
    );

    if response.clicked() {
        app.config.theme = theme.clone();
        action.style_changed = true;
        action.save_config = true;
    }
}

fn theme_swatch_color(theme: &Theme) -> egui::Color32 {
    match theme {
        Theme::DarkBlue => egui::Color32::from_rgb(25, 35, 60),
        Theme::DarkGrey => RedesignTokens::BG_SURFACE,
        Theme::ArcadePurple => egui::Color32::from_rgb(42, 29, 58),
        Theme::LightClassic => egui::Color32::from_rgb(226, 229, 235),
        Theme::NeonGreen => egui::Color32::from_rgb(18, 48, 31),
        Theme::SunsetOrange => egui::Color32::from_rgb(70, 38, 24),
        Theme::OceanBlue => egui::Color32::from_rgb(21, 47, 70),
        Theme::MidnightBlack => egui::Color32::from_rgb(8, 9, 11),
        Theme::ForestGreen => egui::Color32::from_rgb(28, 58, 36),
        Theme::RetroAmber => egui::Color32::from_rgb(74, 54, 22),
        Theme::ModernSpacious => egui::Color32::from_rgb(45, 49, 60),
    }
}

fn show_performance(ui: &mut egui::Ui, app: &mut MameApp, action: &mut SettingsAction) {
    ui.label(
        egui::RichText::new("Performance")
            .font(fonts::bold(18.0))
            .color(RedesignTokens::TEXT_BRIGHT),
    );
    ui.label(
        egui::RichText::new("Passed to MAME as command-line options at launch.")
            .size(12.0)
            .color(RedesignTokens::TEXT_MUTED),
    );
    ui.add_space(16.0);

    if perf_checkbox_row(
        ui,
        "Auto frameskip",
        "Skip frames automatically to keep full speed",
        &mut app.config.default_game_properties.screen.auto_frameskip,
    ) {
        action.save_config = true;
    }
    if perf_u8_row(
        ui,
        "Frameskip value",
        "Manual frameskip when auto is off (0-10)",
        &mut app.config.default_game_properties.screen.frameskip_value,
        0..=10,
    ) {
        action.save_config = true;
    }
    if perf_f32_row(
        ui,
        "Emulation speed",
        "Global speed multiplier (0.10x - 2.00x)",
        &mut app.config.default_game_properties.screen.emulation_speed,
        0.10..=2.00,
    ) {
        action.save_config = true;
    }
    if perf_checkbox_row(
        ui,
        "Sleep when idle",
        "Yield CPU time when emulation is ahead",
        &mut app.config.default_game_properties.screen.sleep_when_idle,
    ) {
        action.save_config = true;
    }
    if perf_checkbox_row(
        ui,
        "Low latency",
        "Reduce input lag, may cost performance",
        &mut app.config.default_game_properties.screen.low_latency,
    ) {
        action.save_config = true;
    }
    if perf_checkbox_row(
        ui,
        "Lazy icon loading",
        "Defer icon decoding while scrolling large lists",
        &mut app.config.preferences.performance.enable_lazy_icons,
    ) {
        action.save_config = true;
    }
    if perf_checkbox_row(
        ui,
        "Virtual scrolling",
        "Render only visible game rows",
        &mut app.config.preferences.performance.enable_virtual_scrolling,
    ) {
        action.save_config = true;
    }
}

fn perf_checkbox_row(ui: &mut egui::Ui, name: &str, hint: &str, value: &mut bool) -> bool {
    let before = *value;
    perf_control_row(ui, name, hint, |ui| {
        ui.checkbox(value, if *value { "On" } else { "Off" });
    });
    before != *value
}

fn perf_u8_row(
    ui: &mut egui::Ui,
    name: &str,
    hint: &str,
    value: &mut u8,
    range: std::ops::RangeInclusive<u8>,
) -> bool {
    let before = *value;
    perf_control_row(ui, name, hint, |ui| {
        ui.add(
            egui::DragValue::new(value)
                .range(range)
                .speed(1.0)
                .custom_formatter(|n, _| format!("{n:.0}")),
        );
    });
    before != *value
}

fn perf_f32_row(
    ui: &mut egui::Ui,
    name: &str,
    hint: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) -> bool {
    let before = *value;
    perf_control_row(ui, name, hint, |ui| {
        ui.add(
            egui::DragValue::new(value)
                .range(range)
                .speed(0.05)
                .suffix("x"),
        );
    });
    (*value - before).abs() > f32::EPSILON
}

fn perf_control_row(
    ui: &mut egui::Ui,
    name: &str,
    hint: &str,
    control: impl FnOnce(&mut egui::Ui),
) {
    if ui.available_width() < 360.0 {
        ui.label(
            egui::RichText::new(name)
                .font(fonts::semibold(13.0))
                .color(RedesignTokens::TEXT_PRIMARY),
        );
        ui.label(
            egui::RichText::new(hint)
                .size(12.0)
                .color(RedesignTokens::TEXT_MUTED),
        );
        control(ui);
    } else {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(name)
                        .font(fonts::semibold(13.0))
                        .color(RedesignTokens::TEXT_PRIMARY),
                );
                ui.label(
                    egui::RichText::new(hint)
                        .size(12.0)
                        .color(RedesignTokens::TEXT_MUTED),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                control(ui);
            });
        });
    }
    ui.separator();
}

fn show_shaders(ui: &mut egui::Ui, app: &mut MameApp, action: &mut SettingsAction) {
    ui.label(
        egui::RichText::new("Shaders")
            .font(fonts::bold(18.0))
            .color(RedesignTokens::TEXT_BRIGHT),
    );
    ui.label(
        egui::RichText::new("Default launch profile · experimental redesign preview")
            .size(12.0)
            .color(RedesignTokens::TEXT_MUTED),
    );
    ui.add_space(16.0);

    let video_mode = app.config.default_game_properties.display.video_mode;
    let screen_chain = app
        .config
        .default_game_properties
        .advanced
        .bgfx_settings
        .screen_chains
        .as_str();
    let custom_chain = custom_bgfx_chain(screen_chain);
    let embedded_shader = app.config.graphics_config.current_shader.as_deref();
    let reset_available = has_shader_override(video_mode, screen_chain, embedded_shader);

    section_header(ui, "MAME BGFX SCREEN CHAIN");
    ui.add_space(8.0);
    card_frame().show(ui, |ui| {
        ui.label(
            egui::RichText::new("Configured launch setting")
                .font(fonts::semibold(13.0))
                .color(RedesignTokens::TEXT_PRIMARY),
        );
        let mode_text = if matches!(video_mode, VideoMode::BGFX) {
            "Video mode: BGFX"
        } else {
            "Video mode: not BGFX"
        };
        ui.label(
            egui::RichText::new(mode_text)
                .size(12.0)
                .color(RedesignTokens::TEXT_MUTED),
        );
        match custom_chain {
            Some(chain) => {
                ui.label(
                    egui::RichText::new(format!("Screen chain: {chain}"))
                        .monospace()
                        .size(12.0)
                        .color(if matches!(video_mode, VideoMode::BGFX) {
                            RedesignTokens::STATUS_OK
                        } else {
                            RedesignTokens::STATUS_WARN
                        }),
                );
                if !matches!(video_mode, VideoMode::BGFX) {
                    ui.label(
                        egui::RichText::new(
                            "The saved chain is inactive until BGFX video mode is selected.",
                        )
                        .size(12.0)
                        .color(RedesignTokens::TEXT_MUTED),
                    );
                }
            }
            None => {
                ui.label(
                    egui::RichText::new("Screen chain: MAME default")
                        .monospace()
                        .size(12.0)
                        .color(RedesignTokens::TEXT_SECONDARY),
                );
            }
        }
    });

    ui.add_space(12.0);
    card_frame().show(ui, |ui| {
        ui.label(
            egui::RichText::new("Preset selection is unavailable in this preview")
                .font(fonts::semibold(13.0))
                .color(RedesignTokens::STATUS_WARN),
        );
        ui.label(
            egui::RichText::new(
                "MAMEUIx's bundled GLSL effect names are not MAME BGFX screen-chain names. Applying them here could make MAME fail at launch.",
            )
            .size(12.0)
            .color(RedesignTokens::TEXT_SECONDARY),
        );
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(
                "Use the classic Game Properties editor to enter a chain installed by your MAME package. Presets will return here after installed chains can be discovered and validated.",
            )
            .size(12.0)
            .color(RedesignTokens::TEXT_MUTED),
        );
    });

    ui.add_space(12.0);
    let reset_clicked = ui
        .add_enabled_ui(reset_available, |ui| {
            secondary_button(ui, "Reset BGFX override")
        })
        .inner
        .clicked();
    if reset_clicked {
        app.config.graphics_config.current_shader = None;
        if matches!(
            app.config.default_game_properties.display.video_mode,
            VideoMode::BGFX
        ) {
            app.config.default_game_properties.display.video_mode = VideoMode::Auto;
        }
        app.config
            .default_game_properties
            .advanced
            .bgfx_settings
            .screen_chains = "default".to_string();
        action.save_config = true;
    }
}

fn custom_bgfx_chain(screen_chain: &str) -> Option<&str> {
    let chain = screen_chain.trim();
    (!chain.is_empty() && !chain.eq_ignore_ascii_case("default")).then_some(chain)
}

fn has_shader_override(
    video_mode: VideoMode,
    screen_chain: &str,
    embedded_shader: Option<&str>,
) -> bool {
    matches!(video_mode, VideoMode::BGFX)
        || custom_bgfx_chain(screen_chain).is_some()
        || embedded_shader.is_some()
}

fn adaptive_column_count(available_width: f32, min_column_width: f32, maximum: usize) -> usize {
    (((available_width + 12.0) / (min_column_width + 12.0)).floor() as usize).clamp(1, maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_settings_layout_uses_expected_breakpoint() {
        assert!(uses_compact_settings_layout(679.9));
        assert!(!uses_compact_settings_layout(680.0));
        assert!(!uses_compact_settings_layout(1200.0));
    }

    #[test]
    fn custom_bgfx_chain_ignores_empty_and_default_values() {
        assert_eq!(custom_bgfx_chain(""), None);
        assert_eq!(custom_bgfx_chain("   "), None);
        assert_eq!(custom_bgfx_chain("default"), None);
        assert_eq!(custom_bgfx_chain(" DEFAULT "), None);
        assert_eq!(custom_bgfx_chain(" crt-geom "), Some("crt-geom"));
    }

    #[test]
    fn shader_reset_is_available_for_each_kind_of_override() {
        assert!(!has_shader_override(VideoMode::Auto, "default", None));
        assert!(has_shader_override(VideoMode::BGFX, "default", None));
        assert!(has_shader_override(VideoMode::Auto, "crt-geom", None));
        assert!(has_shader_override(
            VideoMode::Auto,
            "default",
            Some("embedded-effect")
        ));
    }

    #[test]
    fn path_text_uses_real_single_line_ellipsis() {
        let ctx = egui::Context::default();
        let mut narrow_elided = false;
        let mut wide_elided = true;
        let mut narrow_width = f32::INFINITY;

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let text = "/games/steam-library/Torrent/MAME Extras/artwork";
                let narrow = truncated_single_line(
                    ui.painter(),
                    text,
                    egui::FontId::monospace(12.0),
                    egui::Color32::WHITE,
                    80.0,
                );
                let wide = truncated_single_line(
                    ui.painter(),
                    text,
                    egui::FontId::monospace(12.0),
                    egui::Color32::WHITE,
                    800.0,
                );
                narrow_elided = narrow.elided;
                wide_elided = wide.elided;
                narrow_width = narrow.size().x;
            });
        });

        assert!(narrow_elided);
        assert!(!wide_elided);
        assert!(narrow_width <= 81.0);
    }
}
