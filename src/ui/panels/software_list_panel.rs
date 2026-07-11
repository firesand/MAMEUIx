use crate::mame::{SoftwareEntry, SoftwareListLoader, SoftwareListSummary};
use crate::models::AppConfig;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use walkdir::WalkDir;

struct PreparedSoftwareListData {
    lists: Vec<SoftwareListSummary>,
    entries: Vec<SoftwareEntry>,
    warnings: Vec<String>,
    normalized_search_text: Vec<String>,
    media_paths: MediaPathIndex,
    path_found_entry_count: usize,
}

enum SoftwareListWorkerMessage {
    Progress { completed: usize, total: usize },
    Stage(&'static str),
    Finished(Result<PreparedSoftwareListData, String>),
}

#[derive(Default)]
struct MediaPathIndex {
    keys: HashSet<String>,
    checked_root_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaPathPresence {
    PathFound,
    NotFound,
    NotChecked,
}

pub struct SoftwareListPanel {
    lists: Vec<SoftwareListSummary>,
    entries: Vec<SoftwareEntry>,
    warnings: Vec<String>,
    error: Option<String>,
    loaded_hash_path: Option<PathBuf>,
    loaded_software_rom_paths: Vec<PathBuf>,
    load_receiver: Option<mpsc::Receiver<SoftwareListWorkerMessage>>,
    loading_stage: Option<&'static str>,
    loading_progress: Option<(usize, usize)>,
    media_paths: MediaPathIndex,
    path_found_entry_count: usize,
    normalized_search_text: Vec<String>,
    filtered_indices: Vec<usize>,
    cached_query: String,
    cached_selected_list: Option<String>,
    filter_cache_valid: bool,
    search_text: String,
    selected_list: Option<String>,
}

impl SoftwareListPanel {
    pub fn new() -> Self {
        Self {
            lists: Vec::new(),
            entries: Vec::new(),
            warnings: Vec::new(),
            error: None,
            loaded_hash_path: None,
            loaded_software_rom_paths: Vec::new(),
            load_receiver: None,
            loading_stage: None,
            loading_progress: None,
            media_paths: MediaPathIndex::default(),
            path_found_entry_count: 0,
            normalized_search_text: Vec::new(),
            filtered_indices: Vec::new(),
            cached_query: String::new(),
            cached_selected_list: None,
            filter_cache_valid: false,
            search_text: String::new(),
            selected_list: None,
        }
    }

    pub fn invalidate(&mut self) {
        self.loaded_hash_path = None;
        self.loaded_software_rom_paths.clear();
        self.load_receiver = None;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, config: &AppConfig) {
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Software Lists")
                    .heading()
                    .color(egui::Color32::from_rgb(64, 156, 255)),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Refresh").clicked() {
                    self.invalidate();
                }
            });
        });

        ui.add_space(8.0);

        let Some(hash_path) = config.hash_path.as_ref() else {
            self.clear_loaded_data();
            self.show_empty_state(
                ui,
                "No software-list path configured",
                "Set the Hash XML Directory in Options -> Directories & Paths -> Software Lists.",
            );
            return;
        };

        self.ensure_loaded(hash_path, &config.software_rom_paths, ui.ctx());
        self.poll_worker();

        ui.horizontal_wrapped(|ui| {
            ui.label("Hash:");
            ui.monospace(hash_path.display().to_string());
            if self.load_receiver.is_some() {
                ui.separator();
                ui.label(self.loading_stage.unwrap_or("Loading software lists"));
            } else {
                ui.separator();
                ui.label(format!("{} lists", self.lists.len()));
                ui.separator();
                ui.label(format!("{} entries", self.entries.len()));
                if !config.software_rom_paths.is_empty() {
                    ui.separator();
                    if self.media_paths.checked_root_count == 0 {
                        ui.label("media paths not checked")
                            .on_hover_text("No configured software-list ROM root could be read.");
                    } else {
                        ui.label(format!(
                            "{} path matches (best effort)",
                            self.path_found_entry_count
                        ))
                        .on_hover_text(Self::presence_disclaimer());
                    }
                }
                if !self.warnings.is_empty() {
                    ui.separator();
                    ui.colored_label(
                        egui::Color32::from_rgb(222, 181, 86),
                        format!("{} parse warnings", self.warnings.len()),
                    );
                }
            }
        });

        ui.add_space(10.0);

        if self.load_receiver.is_some() {
            self.show_loading_state(ui);
            return;
        }

        if let Some(error) = &self.error {
            ui.colored_label(egui::Color32::from_rgb(224, 92, 92), error);
            return;
        }

        if self.entries.is_empty() {
            self.show_empty_state(
                ui,
                "No software entries found",
                "Check that the selected hash path contains MAME software-list XML files.",
            );
            return;
        }

        self.show_filters(ui);
        ui.add_space(8.0);
        self.show_warnings(ui);
        self.show_table(ui);
    }

    fn ensure_loaded(
        &mut self,
        hash_path: &Path,
        software_rom_paths: &[PathBuf],
        ctx: &egui::Context,
    ) {
        if self.loaded_hash_path.as_deref() == Some(hash_path)
            && self.loaded_software_rom_paths == software_rom_paths
        {
            return;
        }

        self.clear_results_for_load();
        self.loaded_hash_path = Some(hash_path.to_path_buf());
        self.loaded_software_rom_paths = software_rom_paths.to_vec();
        self.loading_stage = Some("Reading MAME hash XML");

        let hash_path = hash_path.to_path_buf();
        let software_rom_paths = software_rom_paths.to_vec();
        let repaint_ctx = ctx.clone();
        let (sender, receiver) = mpsc::channel();
        self.load_receiver = Some(receiver);

        thread::spawn(move || {
            let mut last_reported = 0;
            let load_result = SoftwareListLoader::load_from_hash_path_with_progress(
                &hash_path,
                |completed, total| {
                    if completed == 0
                        || completed == total
                        || completed.saturating_sub(last_reported) >= 8
                    {
                        last_reported = completed;
                        let _ =
                            sender.send(SoftwareListWorkerMessage::Progress { completed, total });
                        repaint_ctx.request_repaint();
                    }
                },
            );

            let prepared = match load_result {
                Ok(result) => {
                    let _ = sender.send(SoftwareListWorkerMessage::Stage(
                        "Preparing the search index",
                    ));
                    repaint_ctx.request_repaint();
                    let normalized_search_text = result
                        .entries
                        .iter()
                        .map(normalized_entry_search_text)
                        .collect();

                    let _ = sender.send(SoftwareListWorkerMessage::Stage(
                        "Indexing configured media paths",
                    ));
                    repaint_ctx.request_repaint();
                    let media_paths = build_media_path_index(&software_rom_paths);
                    let path_found_entry_count = result
                        .entries
                        .iter()
                        .filter(|entry| {
                            media_path_presence(entry, &media_paths) == MediaPathPresence::PathFound
                        })
                        .count();

                    Ok(PreparedSoftwareListData {
                        lists: result.lists,
                        entries: result.entries,
                        warnings: result.errors,
                        normalized_search_text,
                        media_paths,
                        path_found_entry_count,
                    })
                }
                Err(error) => Err(error.to_string()),
            };

            let _ = sender.send(SoftwareListWorkerMessage::Finished(prepared));
            repaint_ctx.request_repaint();
        });
    }

    fn poll_worker(&mut self) {
        let Some(receiver) = self.load_receiver.take() else {
            return;
        };

        let mut keep_receiver = true;
        loop {
            match receiver.try_recv() {
                Ok(SoftwareListWorkerMessage::Progress { completed, total }) => {
                    self.loading_stage = Some("Reading MAME hash XML");
                    self.loading_progress = Some((completed, total));
                }
                Ok(SoftwareListWorkerMessage::Stage(stage)) => {
                    self.loading_stage = Some(stage);
                    self.loading_progress = None;
                }
                Ok(SoftwareListWorkerMessage::Finished(result)) => {
                    keep_receiver = false;
                    self.loading_stage = None;
                    self.loading_progress = None;
                    match result {
                        Ok(data) => self.apply_prepared_data(data),
                        Err(error) => self.error = Some(error),
                    }
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    keep_receiver = false;
                    self.loading_stage = None;
                    self.loading_progress = None;
                    self.error = Some("Software-list loader stopped unexpectedly".to_string());
                    break;
                }
            }
        }

        if keep_receiver {
            self.load_receiver = Some(receiver);
        }
    }

    fn apply_prepared_data(&mut self, data: PreparedSoftwareListData) {
        self.lists = data.lists;
        self.entries = data.entries;
        self.warnings = data.warnings;
        self.normalized_search_text = data.normalized_search_text;
        self.media_paths = data.media_paths;
        self.path_found_entry_count = data.path_found_entry_count;
        self.selected_list = self
            .selected_list
            .take()
            .filter(|selected| self.lists.iter().any(|list| &list.name == selected));
        self.invalidate_filter_cache();
    }

    fn clear_results_for_load(&mut self) {
        self.lists.clear();
        self.entries.clear();
        self.warnings.clear();
        self.error = None;
        self.load_receiver = None;
        self.loading_progress = None;
        self.media_paths = MediaPathIndex::default();
        self.path_found_entry_count = 0;
        self.normalized_search_text.clear();
        self.filtered_indices.clear();
        self.invalidate_filter_cache();
    }

    fn clear_loaded_data(&mut self) {
        self.clear_results_for_load();
        self.loaded_hash_path = None;
        self.loaded_software_rom_paths.clear();
        self.selected_list = None;
    }

    fn show_loading_state(&self, ui: &mut egui::Ui) {
        let available_height = ui.available_height();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), available_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space((available_height * 0.3).max(24.0));
                ui.spinner();
                ui.add_space(10.0);
                ui.heading(self.loading_stage.unwrap_or("Loading software lists"));
                ui.add_space(8.0);
                if let Some((completed, total)) = self.loading_progress {
                    if total > 0 {
                        ui.add(
                            egui::ProgressBar::new(completed as f32 / total as f32)
                                .desired_width(320.0)
                                .text(format!("{completed} of {total} XML files")),
                        );
                    } else {
                        ui.label(egui::RichText::new("No XML files discovered yet").weak());
                    }
                }
            },
        );
    }

    fn show_empty_state(&self, ui: &mut egui::Ui, title: &str, detail: &str) {
        let available_height = ui.available_height();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), available_height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space((available_height * 0.35).max(24.0));
                ui.heading(title);
                ui.add_space(8.0);
                ui.label(egui::RichText::new(detail).weak());
            },
        );
    }

    fn show_filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search");
            ui.add(
                egui::TextEdit::singleline(&mut self.search_text)
                    .desired_width((ui.available_width() * 0.55).max(220.0))
                    .hint_text("Software name, description, publisher, year, or list"),
            );

            ui.separator();

            let selected_text = self
                .selected_list
                .as_deref()
                .map(Self::list_label)
                .unwrap_or_else(|| "All lists".to_string());

            egui::ComboBox::from_id_salt("software_list_filter")
                .selected_text(selected_text)
                .width(220.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(self.selected_list.is_none(), "All lists")
                        .clicked()
                    {
                        self.selected_list = None;
                    }

                    ui.separator();

                    for list in &self.lists {
                        let selected = self.selected_list.as_deref() == Some(list.name.as_str());
                        if ui
                            .selectable_label(selected, Self::list_label(&list.name))
                            .on_hover_text(&list.description)
                            .clicked()
                        {
                            self.selected_list = Some(list.name.clone());
                        }
                    }
                });
        });
    }

    fn show_warnings(&self, ui: &mut egui::Ui) {
        if self.warnings.is_empty() {
            return;
        }

        egui::CollapsingHeader::new("Parse warnings")
            .default_open(false)
            .show(ui, |ui| {
                for warning in self.warnings.iter().take(12) {
                    ui.colored_label(egui::Color32::from_rgb(222, 181, 86), warning);
                }
                if self.warnings.len() > 12 {
                    ui.label(format!(
                        "{} more warnings omitted",
                        self.warnings.len() - 12
                    ));
                }
            });
        ui.add_space(8.0);
    }

    fn show_table(&mut self, ui: &mut egui::Ui) {
        self.refresh_filter_cache_if_needed();
        ui.label(format!("Showing {} entries", self.filtered_indices.len()));
        ui.add_space(6.0);

        let table_height = ui.available_height().max(160.0);
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .max_scroll_height(table_height)
            .column(Column::initial(110.0).at_least(80.0).resizable(true))
            .column(Column::initial(130.0).at_least(90.0).resizable(true))
            .column(Column::remainder().at_least(220.0))
            .column(Column::initial(64.0).at_least(52.0).resizable(true))
            .column(Column::initial(150.0).at_least(100.0).resizable(true))
            .column(Column::initial(86.0).at_least(70.0).resizable(true))
            .column(Column::initial(92.0).at_least(76.0).resizable(true))
            .column(Column::initial(120.0).at_least(84.0).resizable(true))
            .header(30.0, |mut header| {
                header.col(|ui| {
                    ui.strong("List");
                });
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Description");
                });
                header.col(|ui| {
                    ui.strong("Year");
                });
                header.col(|ui| {
                    ui.strong("Publisher");
                });
                header.col(|ui| {
                    ui.strong("Status");
                });
                header.col(|ui| {
                    ui.strong("Media path")
                        .on_hover_text(Self::presence_disclaimer());
                });
                header.col(|ui| {
                    ui.strong("Parts");
                });
            })
            .body(|body| {
                body.rows(28.0, self.filtered_indices.len(), |mut row| {
                    let entry = &self.entries[self.filtered_indices[row.index()]];
                    row.col(|ui| {
                        ui.label(&entry.list_name)
                            .on_hover_text(&entry.list_description);
                    });
                    row.col(|ui| {
                        let response = ui.label(&entry.name);
                        if let Some(clone_of) = &entry.clone_of {
                            response.on_hover_text(format!("Clone of {}", clone_of));
                        } else if let Some(parent) = &entry.parent {
                            response.on_hover_text(format!("ROM of {}", parent));
                        }
                    });
                    row.col(|ui| {
                        ui.label(&entry.description)
                            .on_hover_text(entry.source_file.display().to_string());
                    });
                    row.col(|ui| {
                        ui.label(&entry.year);
                    });
                    row.col(|ui| {
                        ui.label(&entry.publisher);
                    });
                    row.col(|ui| {
                        ui.label(Self::support_label(&entry.supported));
                    });
                    row.col(|ui| {
                        match media_path_presence(entry, &self.media_paths) {
                            MediaPathPresence::PathFound => {
                                ui.colored_label(
                                    egui::Color32::from_rgb(94, 206, 118),
                                    "Path found",
                                )
                                .on_hover_text(Self::presence_disclaimer());
                            }
                            MediaPathPresence::NotFound => {
                                ui.colored_label(
                                    egui::Color32::from_rgb(222, 181, 86),
                                    "Not found",
                                )
                                .on_hover_text(
                                    "No matching archive or non-empty directory was found in the readable configured roots. This is not a MAME audit, and merged set layouts may differ.",
                                );
                            }
                            MediaPathPresence::NotChecked => {
                                ui.label("Not checked").on_hover_text(
                                    "Configure a readable software-list ROM root to run the best-effort path check.",
                                );
                            }
                        }
                    });
                    row.col(|ui| {
                        let interfaces = if entry.interfaces.is_empty() {
                            "-".to_string()
                        } else {
                            entry.interfaces.join(", ")
                        };
                        ui.label(format!("{} {}", entry.part_count, interfaces));
                    });
                });
            });
    }

    fn refresh_filter_cache_if_needed(&mut self) {
        let query = normalize_search_query(&self.search_text);
        if self.filter_cache_valid
            && self.cached_query == query
            && self.cached_selected_list == self.selected_list
        {
            return;
        }

        self.filtered_indices = filter_entry_indices(
            &self.entries,
            &self.normalized_search_text,
            &query,
            self.selected_list.as_deref(),
        );
        self.cached_query = query;
        self.cached_selected_list = self.selected_list.clone();
        self.filter_cache_valid = true;
    }

    fn invalidate_filter_cache(&mut self) {
        self.filter_cache_valid = false;
    }

    fn presence_disclaimer() -> &'static str {
        "Best-effort path presence only. Archive and CHD contents are not audited, and merged set layouts may differ."
    }

    fn list_label(name: &str) -> String {
        name.to_string()
    }

    fn support_label(supported: &str) -> &'static str {
        match supported {
            "no" => "No",
            "partial" => "Partial",
            _ => "Yes",
        }
    }
}

fn normalize_search_query(query: &str) -> String {
    query.trim().to_lowercase()
}

fn normalized_entry_search_text(entry: &SoftwareEntry) -> String {
    [
        entry.name.as_str(),
        entry.description.as_str(),
        entry.publisher.as_str(),
        entry.year.as_str(),
        entry.list_name.as_str(),
        entry.list_description.as_str(),
    ]
    .join("\u{1f}")
    .to_lowercase()
}

fn filter_entry_indices(
    entries: &[SoftwareEntry],
    normalized_search_text: &[String],
    normalized_query: &str,
    selected_list: Option<&str>,
) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            if selected_list.is_some_and(|selected| entry.list_name != selected) {
                return None;
            }

            if normalized_query.is_empty()
                || normalized_search_text
                    .get(index)
                    .is_some_and(|text| text.contains(normalized_query))
            {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}

fn build_media_path_index(paths: &[PathBuf]) -> MediaPathIndex {
    let mut index = MediaPathIndex::default();

    for root in paths {
        let Ok(list_dirs) = std::fs::read_dir(root) else {
            continue;
        };
        index.checked_root_count += 1;

        for list_dir in list_dirs.filter_map(|entry| entry.ok()) {
            let list_path = list_dir.path();
            if !list_path.is_dir() {
                continue;
            }

            let Some(list_name) = list_path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            let Ok(items) = std::fs::read_dir(&list_path) else {
                continue;
            };

            for item in items.filter_map(|entry| entry.ok()) {
                let item_path = item.path();
                if item_path.is_dir() {
                    if directory_contains_regular_file(&item_path)
                        && let Some(name) = item_path.file_name().and_then(|name| name.to_str())
                    {
                        index.keys.insert(media_path_key(list_name, name));
                    }
                    continue;
                }

                let has_supported_extension = item_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| {
                        ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("7z")
                    });
                if !has_supported_extension {
                    continue;
                }

                if let Some(name) = item_path.file_stem().and_then(|name| name.to_str()) {
                    index.keys.insert(media_path_key(list_name, name));
                }
            }
        }
    }

    index
}

fn directory_contains_regular_file(path: &Path) -> bool {
    WalkDir::new(path)
        .follow_links(false)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .any(|entry| entry.file_type().is_file())
}

fn media_path_presence(entry: &SoftwareEntry, index: &MediaPathIndex) -> MediaPathPresence {
    if index.checked_root_count == 0 {
        return MediaPathPresence::NotChecked;
    }

    if index
        .keys
        .contains(&media_path_key(&entry.list_name, &entry.name))
    {
        MediaPathPresence::PathFound
    } else {
        MediaPathPresence::NotFound
    }
}

fn media_path_key(list_name: &str, software_name: &str) -> String {
    format!(
        "{}:{}",
        list_name.to_lowercase(),
        software_name.to_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        MediaPathIndex, MediaPathPresence, SoftwareListPanel, build_media_path_index,
        filter_entry_indices, media_path_presence, normalize_search_query,
        normalized_entry_search_text,
    };
    use crate::mame::SoftwareEntry;
    use std::fs;

    fn entry(list: &str, name: &str, description: &str, publisher: &str) -> SoftwareEntry {
        SoftwareEntry {
            list_name: list.to_string(),
            list_description: format!("{list} software"),
            name: name.to_string(),
            description: description.to_string(),
            publisher: publisher.to_string(),
            year: "1982".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn cached_search_text_filters_without_normalizing_entries_again() {
        let entries = vec![
            entry("a2600", "combat", "Combat", "Atari"),
            entry("nes", "dkong", "Donkey Kong", "Nintendo"),
        ];
        let normalized = entries
            .iter()
            .map(normalized_entry_search_text)
            .collect::<Vec<_>>();

        assert_eq!(
            filter_entry_indices(
                &entries,
                &normalized,
                &normalize_search_query(" NINTENDO "),
                None,
            ),
            vec![1]
        );
        assert_eq!(
            filter_entry_indices(&entries, &normalized, "", Some("a2600")),
            vec![0]
        );
        assert!(filter_entry_indices(&entries, &normalized, "combat", Some("nes")).is_empty());
    }

    #[test]
    fn panel_reuses_filter_results_until_the_filter_key_changes() {
        let mut panel = SoftwareListPanel::new();
        panel.entries = vec![
            entry("a2600", "combat", "Combat", "Atari"),
            entry("nes", "dkong", "Donkey Kong", "Nintendo"),
        ];
        panel.normalized_search_text = panel
            .entries
            .iter()
            .map(normalized_entry_search_text)
            .collect();
        panel.search_text = "nintendo".to_string();

        panel.refresh_filter_cache_if_needed();
        assert_eq!(panel.filtered_indices, vec![1]);

        // A stable query/list key leaves the cached result untouched.
        panel.normalized_search_text[1].clear();
        panel.refresh_filter_cache_if_needed();
        assert_eq!(panel.filtered_indices, vec![1]);

        panel.search_text = "combat".to_string();
        panel.refresh_filter_cache_if_needed();
        assert_eq!(panel.filtered_indices, vec![0]);
    }

    #[test]
    fn path_presence_requires_a_readable_root_and_a_plausible_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("software");
        let list = root.join("a2600");
        fs::create_dir_all(&list).unwrap();
        fs::write(list.join("combat.ZIP"), []).unwrap();
        fs::create_dir(list.join("emptydir")).unwrap();
        fs::create_dir(list.join("chdgame")).unwrap();
        fs::write(list.join("chdgame").join("disk.chd"), []).unwrap();

        let index = build_media_path_index(std::slice::from_ref(&root));
        assert_eq!(index.checked_root_count, 1);
        assert_eq!(
            media_path_presence(&entry("a2600", "combat", "", ""), &index),
            MediaPathPresence::PathFound
        );
        assert_eq!(
            media_path_presence(&entry("a2600", "chdgame", "", ""), &index),
            MediaPathPresence::PathFound
        );
        assert_eq!(
            media_path_presence(&entry("a2600", "emptydir", "", ""), &index),
            MediaPathPresence::NotFound
        );
        assert_eq!(
            media_path_presence(&entry("a2600", "unknown", "", ""), &index),
            MediaPathPresence::NotFound
        );
    }

    #[test]
    fn path_presence_is_not_checked_without_a_readable_root() {
        let index = MediaPathIndex::default();
        assert_eq!(
            media_path_presence(&entry("a2600", "combat", "", ""), &index),
            MediaPathPresence::NotChecked
        );
    }
}
