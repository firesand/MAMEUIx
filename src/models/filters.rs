use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::{FilterCategory, RomSetType};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterSettings {
    pub search_text: String,
    pub search_mode: SearchMode,  // New field for search mode
    pub category: Option<super::FilterCategory>, // DEPRECATED - kept for migration
    pub catver_category: Option<String>, // New field for catver.ini category filtering
    pub year_from: String,
    pub year_to: String,
    pub manufacturer: String,
    pub cpu_filter: String,       // Filter by CPU type
    pub device_filter: String,    // Filter by Device type
    pub sound_filter: String,     // Filter by Sound type
    pub show_favorites_only: bool,
    pub status_filter: StatusFilter,
    pub apply_hidden_categories: bool,  // Toggle untuk enable/disable hidden categories filter
    pub auto_expand_clones: bool,       // Auto expand clone games in parent view
    pub rom_set_type: RomSetType,       // Detected ROM set type
    pub show_clones_in_split: bool,     // Show clones when using split set
    pub show_clones_in_merged: bool,    // Show clones when using merged set (usually false)
    pub select_all_mode: bool,          // Radio button state for Select/Clear All
    
    // New multi-selection filter fields
    pub availability_filters: AvailabilityFilters,
    pub status_filters: StatusFilters,
    pub other_filters: OtherFilters,
}

/// Filters for ROM availability status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AvailabilityFilters {
    pub show_available: bool,
    pub show_unavailable: bool,
}

/// Filters for game working status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusFilters {
    pub show_working: bool,
    pub show_not_working: bool,
}

/// Other miscellaneous filters
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OtherFilters {
    pub show_favorites: bool,
    pub show_parents_only: bool,
    pub show_chd_games: bool,
}

impl Default for AvailabilityFilters {
    fn default() -> Self {
        Self {
            show_available: true,
            show_unavailable: true,
        }
    }
}

impl Default for StatusFilters {
    fn default() -> Self {
        Self {
            show_working: true,
            show_not_working: true,
        }
    }
}

impl Default for OtherFilters {
    fn default() -> Self {
        Self {
            show_favorites: false,
            show_parents_only: false,
            show_chd_games: false,
        }
    }
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

        // First, count games per category to avoid duplicates
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        
        for (_game_name, category_name) in categories {
            *category_counts.entry(category_name.clone()).or_insert(0) += 1;
            self.total_games += 1;
        }

        // Now create unique category entries with correct game counts
        for (category_name, game_count) in category_counts {
            let letter = category_name.chars().next().unwrap_or('A').to_ascii_uppercase();
            
            let catver_category = CatverCategory {
                name: category_name.clone(),
                display_name: category_name.clone(),
                letter_group: letter,
                game_count,
            };

            self.categories.insert(category_name.clone(), catver_category);
            
            // Only add each category once to the letter groups
            self.categories_by_letter
                .entry(letter)
                .or_insert_with(Vec::new)
                .push(category_name);
        }
        
        // Sort categories within each letter group for consistent display
        for categories_in_letter in self.categories_by_letter.values_mut() {
            categories_in_letter.sort();
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
    // Enhanced search modes
    FuzzySearch,    // Fuzzy matching search
    FullText,       // Full-text search across all fields
    Regex,          // Regular expression search
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
            category: None, // Deprecated field
            catver_category: None,
            year_from: String::new(),
            year_to: String::new(),
            manufacturer: String::new(),
            cpu_filter: String::new(),
            device_filter: String::new(),
            sound_filter: String::new(),
            show_favorites_only: false,
            status_filter: StatusFilter::All,
            apply_hidden_categories: true,  // Enable by default
            auto_expand_clones: false,      // Don't auto expand clones by default
            rom_set_type: RomSetType::Unknown,
            show_clones_in_split: false,
            show_clones_in_merged: false,
            select_all_mode: false,      // Default to false (Clear All mode)
            availability_filters: AvailabilityFilters::default(),
            status_filters: StatusFilters::default(),
            other_filters: OtherFilters::default(),
        }
    }
}

impl FilterSettings {
    /// Migrate from legacy single-selection filters to new multi-selection
    pub fn migrate_from_legacy(&mut self) {
        if let Some(category) = &self.category {
            // Reset all filters first
            self.availability_filters = AvailabilityFilters::default();
            self.status_filters = StatusFilters::default();
            self.other_filters = OtherFilters::default();
            
            // Map old category to new filters
            match category {
                super::FilterCategory::All => {
                    // All filters enabled by default
                }
                super::FilterCategory::Available => {
                    self.availability_filters.show_available = true;
                    self.availability_filters.show_unavailable = false;
                }
                super::FilterCategory::Missing | super::FilterCategory::Unavailable => {
                    self.availability_filters.show_available = false;
                    self.availability_filters.show_unavailable = true;
                }
                super::FilterCategory::Working => {
                    self.status_filters.show_working = true;
                    self.status_filters.show_not_working = false;
                }
                super::FilterCategory::NotWorking => {
                    self.status_filters.show_working = false;
                    self.status_filters.show_not_working = true;
                }
                super::FilterCategory::Favorites => {
                    self.other_filters.show_favorites = true;
                }
                super::FilterCategory::Parents | super::FilterCategory::NonClones => {
                    self.other_filters.show_parents_only = true;
                }
                super::FilterCategory::ChdGames => {
                    self.other_filters.show_chd_games = true;
                }
                _ => {}
            }
            
            // Clear the old category field
            self.category = None;
        }
        
        // Also migrate show_favorites_only to new system
        if self.show_favorites_only {
            self.other_filters.show_favorites = true;
            self.show_favorites_only = false;
        }
    }
    
    /// Check if any filters are active
    pub fn has_active_filters(&self) -> bool {
        // Check if non-default filters are selected
        let availability_active = !(self.availability_filters.show_available &&
                                   self.availability_filters.show_unavailable);
        let status_active = !(self.status_filters.show_working &&
                             self.status_filters.show_not_working);
        let other_active = self.other_filters.show_favorites ||
                          self.other_filters.show_parents_only ||
                          self.other_filters.show_chd_games;
        
        availability_active || status_active || other_active ||
        !self.search_text.is_empty() || self.catver_category.is_some()
    }
    
    /// Count active filters
    pub fn count_active_filters(&self) -> usize {
        let mut count = 0;
        
        // Only count if not all selected in a category
        if !(self.availability_filters.show_available && self.availability_filters.show_unavailable) {
            if self.availability_filters.show_available { count += 1; }
            if self.availability_filters.show_unavailable { count += 1; }
        }
        
        if !(self.status_filters.show_working && self.status_filters.show_not_working) {
            if self.status_filters.show_working { count += 1; }
            if self.status_filters.show_not_working { count += 1; }
        }
        
        if self.other_filters.show_favorites { count += 1; }
        if self.other_filters.show_parents_only { count += 1; }
        if self.other_filters.show_chd_games { count += 1; }
        
        count
    }
}

