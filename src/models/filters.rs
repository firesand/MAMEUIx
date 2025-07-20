use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::{FilterCategory, RomSetType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterSettings {
    pub search_text: String,
    pub search_mode: SearchMode,  // New field for search mode
    pub category: Option<super::FilterCategory>,
    pub catver_category: Option<String>, // New field for catver.ini category filtering
    pub year_from: String,
    pub year_to: String,
    pub manufacturer: String,
    pub cpu_filter: String,       // Filter by CPU type
    pub device_filter: String,    // Filter by Device type
    pub sound_filter: String,     // Filter by Sound type
    pub hide_non_games: bool,
    pub show_favorites_only: bool,
    pub status_filter: StatusFilter,
    pub apply_hidden_categories: bool,  // Toggle untuk enable/disable hidden categories filter
    pub auto_expand_clones: bool,       // Auto expand clone games in parent view
    pub rom_set_type: RomSetType,       // Detected ROM set type
    pub show_clones_in_split: bool,     // Show clones when using split set
    pub show_clones_in_merged: bool,    // Show clones when using merged set (usually false)
}

/// Represents a category from catver.ini with alphabetical organization
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CatverCategory {
    pub name: String,
    pub display_name: String,
    pub letter_group: char, // A-Z grouping
    pub game_count: usize,
}



/// Manages catver.ini categories with alphabetical organization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CategoryManager {
    pub categories: HashMap<String, CatverCategory>,
    pub categories_by_letter: HashMap<char, Vec<String>>,
    pub total_games: usize,
}

impl CategoryManager {
    pub fn new() -> Self {
        Self {
            categories: HashMap::new(),
            categories_by_letter: HashMap::new(),
            total_games: 0,
        }
    }

    pub fn has_categories_for_letter(&self, letter: char) -> bool {
        self.categories_by_letter.contains_key(&letter)
    }

    pub fn get_letter_groups(&self) -> Vec<char> {
        let mut letters: Vec<char> = self.categories_by_letter.keys().cloned().collect();
        letters.sort();
        letters
    }

    pub fn get_categories_by_letter(&self) -> Vec<(char, Vec<&CatverCategory>)> {
        let mut result: Vec<(char, Vec<&CatverCategory>)> = self.categories_by_letter
            .iter()
            .map(|(letter, category_names)| {
                let categories: Vec<&CatverCategory> = category_names
                    .iter()
                    .filter_map(|name| self.categories.get(name))
                    .collect();
                (*letter, categories)
            })
            .collect();
        result.sort_by_key(|(letter, _)| *letter);
        result
    }

    pub fn load_from_catver_map(&mut self, categories: &HashMap<String, String>) {
        self.categories.clear();
        self.categories_by_letter.clear();
        self.total_games = 0;

        for (game_name, category_name) in categories {
            let letter = category_name.chars().next().unwrap_or('A').to_ascii_uppercase();
            
            let catver_category = CatverCategory {
                name: category_name.clone(),
                display_name: category_name.clone(),
                letter_group: letter,
                game_count: 1,
            };

            self.categories.insert(category_name.clone(), catver_category);
            
            self.categories_by_letter
                .entry(letter)
                .or_insert_with(Vec::new)
                .push(category_name.clone());
            
            self.total_games += 1;
        }
    }
}



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SearchMode {
    GameTitle,
    Manufacturer,
    RomFileName,
    Year,
    Status,
    Cpu,
    Device,
    Sound,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::GameTitle
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StatusFilter {
    All,
    WorkingOnly,
    ImperfectOnly,
    NotWorkingOnly,
}

impl Default for StatusFilter {
    fn default() -> Self {
        StatusFilter::All
    }
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            search_mode: SearchMode::default(),
            category: Some(super::FilterCategory::All),
            catver_category: None,
            year_from: String::new(),
            year_to: String::new(),
            manufacturer: String::new(),
            cpu_filter: String::new(),
            device_filter: String::new(),
            sound_filter: String::new(),
            hide_non_games: true,  // Hide devices and BIOS by default
            show_favorites_only: false,
            status_filter: StatusFilter::All,
            apply_hidden_categories: true,  // Enable by default
            auto_expand_clones: false,      // Don't auto expand clones by default
            rom_set_type: RomSetType::Unknown,
            show_clones_in_split: false,
            show_clones_in_merged: false,
        }
    }
}

