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

impl CatverCategory {
    pub fn new(name: String) -> Self {
        let display_name = name.clone();
        let letter_group = name.chars().next().unwrap_or('A').to_ascii_uppercase();
        Self {
            name,
            display_name,
            letter_group,
            game_count: 0,
        }
    }
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

    /// Load categories from a HashMap of game names to category names
    pub fn load_from_catver_map(&mut self, categories_map: &HashMap<String, String>) {
        self.categories.clear();
        self.categories_by_letter.clear();
        
        // First, collect unique category names and count games per category
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for (_game_name, category_name) in categories_map {
            *category_counts.entry(category_name.clone()).or_insert(0) += 1;
        }
        
        // Create unique categories from catver.ini data
        for (category_name, game_count) in category_counts {
            let mut category = CatverCategory::new(category_name.clone());
            category.game_count = game_count;
            let letter = category.letter_group;
            
            // Add to categories map
            self.categories.insert(category_name.clone(), category);
            
            // Add to letter grouping
            self.categories_by_letter
                .entry(letter)
                .or_insert_with(Vec::new)
                .push(category_name);
        }
        
        // Sort categories within each letter group
        for categories in self.categories_by_letter.values_mut() {
            categories.sort();
        }
        
        self.total_games = categories_map.len();
    }

    /// Get all categories organized by letter (A-Z)
    pub fn get_categories_by_letter(&self) -> Vec<(char, Vec<&CatverCategory>)> {
        let mut result = Vec::new();
        
        for letter in 'A'..='Z' {
            if let Some(category_names) = self.categories_by_letter.get(&letter) {
                let mut categories: Vec<&CatverCategory> = category_names
                    .iter()
                    .filter_map(|name| self.categories.get(name))
                    .collect();
                categories.sort_by(|a, b| a.display_name.cmp(&b.display_name));
                
                if !categories.is_empty() {
                    result.push((letter, categories));
                }
            }
        }
        
        result
    }

    /// Get category by name
    pub fn get_category(&self, name: &str) -> Option<&CatverCategory> {
        self.categories.get(name)
    }

    /// Get all category names sorted alphabetically
    pub fn get_all_category_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.categories.keys().cloned().collect();
        names.sort();
        names
    }


    /// Get all letter groups (A-Z) that have categories
    pub fn get_letter_groups(&self) -> Vec<char> {
        let mut letters: Vec<char> = self.categories_by_letter.keys().cloned().collect();
        letters.sort();
        letters
    }

    /// Check if a letter has any categories
    pub fn has_categories_for_letter(&self, letter: char) -> bool {
        self.categories_by_letter.get(&letter)
            .map(|cats| !cats.is_empty())
            .unwrap_or(false)
    }

    /// Get total count of games in a specific category
    pub fn get_category_game_count(&self, category_name: &str) -> usize {
        self.categories.get(category_name)
            .map(|cat| cat.game_count)
            .unwrap_or(0)
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

