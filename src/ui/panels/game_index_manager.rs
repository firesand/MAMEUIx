// src/ui/game_index_manager.rs
// Game indexing, filtering, and search management module

use crate::models::*;
use crate::mame::CategoryLoader;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use rayon::prelude::*;

pub struct GameIndexManager {
    // Core indexing
    pub game_index: Option<GameIndex>,
    pub filtered_games_cache: Vec<usize>,
    pub filter_cache_dirty: bool,
    pub last_filter_update: Instant,
    
    // Search management
    pub search_debounce_timer: Option<Instant>,
    pub pending_search: Option<String>,
    
    // Category management
    pub category_loader: Option<CategoryLoader>,
    pub category_manager: Option<filters::CategoryManager>,
    
    // Performance settings
    pub search_debounce_ms: u64,
    pub max_cache_size: usize,
}

impl GameIndexManager {
    pub fn new() -> Self {
        Self {
            game_index: None,
            filtered_games_cache: Vec::new(),
            filter_cache_dirty: true,
            last_filter_update: Instant::now(),
            search_debounce_timer: None,
            pending_search: None,
            category_loader: None,
            category_manager: None,
            search_debounce_ms: 300, // Default 300ms debounce
            max_cache_size: 100,
        }
    }

    /// Initialize with performance settings
    pub fn with_settings(mut self, search_debounce_ms: u64, max_cache_size: usize) -> Self {
        self.search_debounce_ms = search_debounce_ms;
        self.max_cache_size = max_cache_size;
        self
    }

    /// Set category loader
    pub fn set_category_loader(&mut self, loader: Option<CategoryLoader>) {
        self.category_loader = loader;
        self.filter_cache_dirty = true;
    }

    /// Set category manager
    pub fn set_category_manager(&mut self, manager: Option<filters::CategoryManager>) {
        self.category_manager = manager;
        self.filter_cache_dirty = true;
    }

    /// Build game index for fast lookup - CRITICAL for performance!
    pub fn build_game_index(&mut self, games: &[Game], favorites: &HashSet<String>) {
        println!("Building optimized game index for {} games...", games.len());
        let start = Instant::now();

        self.game_index = Some(GameIndex::build(games.to_vec(), favorites.clone()));

        let elapsed = start.elapsed();
        println!("Game index built in {:.2}s", elapsed.as_secs_f32());

        // Force filter update with new index
        self.filter_cache_dirty = true;
    }

    /// OPTIMIZED: Update filtered games cache with GameIndex
    pub fn update_filtered_games_cache(
        &mut self,
        games: &[Game],
        selected_filter: FilterCategory,
        filter_settings: &FilterSettings,
        hidden_categories: &HashSet<String>,
    ) {
        if !self.filter_cache_dirty {
            return;
        }

        let start = Instant::now();

        // CRITICAL: Use pre-computed index lists when available
        if let Some(index) = &self.game_index {
            // Fast path with O(1) access to pre-filtered lists
            self.filtered_games_cache = match selected_filter {
                FilterCategory::All => {
                    // Use sorted indices if available for better performance
                    if let Some(sorted) = index.get_sorted("name", true) {
                        sorted.to_vec()
                    } else {
                        (0..games.len()).collect()
                    }
                }
                FilterCategory::Available => index.available_games.clone(),
                FilterCategory::Missing => index.missing_games.clone(),
                FilterCategory::Favorites => index.favorite_games.clone(),
                FilterCategory::Parents => index.parent_games.clone(),
                FilterCategory::Clones => index.clone_games.clone(),
                FilterCategory::Working => index.working_games.clone(),
                FilterCategory::NotWorking => index.missing_games.clone(),
                FilterCategory::NonClones => index.parent_games.clone(),
                FilterCategory::ChdGames => index.chd_games.clone(),
                FilterCategory::NonMerged => index.parent_games.clone(), // Same as Parents for now
            };

            // Apply catver.ini category filter if set
            if let Some(ref category_name) = filter_settings.catver_category {
                if let Some(ref category_loader) = self.category_loader {
                    let category_name_lower = category_name.to_lowercase();
                    self.filtered_games_cache.retain(|&idx| {
                        if let Some(game) = games.get(idx) {
                            // Try exact match first
                            if let Some(game_category) = category_loader.get_category(&game.name) {
                                // Case-insensitive category comparison
                                game_category.to_lowercase() == category_name_lower
                            } else {
                                // Try case-insensitive match on game name
                                let game_name_lower = game.name.to_lowercase();
                                category_loader.categories.iter().any(|(catver_name, catver_category)| {
                                    catver_name.to_lowercase() == game_name_lower &&
                                    catver_category.to_lowercase() == category_name_lower
                                })
                            }
                        } else {
                            false
                        }
                    });
                }
            }

            // Apply hidden categories filter if enabled
            if filter_settings.apply_hidden_categories && !hidden_categories.is_empty() {
                if let Some(ref category_loader) = self.category_loader {
                    self.filtered_games_cache.retain(|&idx| {
                        if let Some(game) = games.get(idx) {
                            // Check if the game's category is in the hidden list
                            if let Some(game_category) = category_loader.get_category(&game.name) {
                                !hidden_categories.contains(game_category)
                            } else {
                                true // If we can't determine the category, show the game
                            }
                        } else {
                            false
                        }
                    });
                }
            }

            // Apply search filter only if there's text
            if !filter_settings.search_text.is_empty() {
                // Check cache first for instant results!
                let search_key = filter_settings.search_text.clone();
 
                if let Some(cached) = index.get_cached_search(&search_key) {
                    // Cache hit! No need to search
                    self.filtered_games_cache = cached.to_vec();
                } else {
                    // Cache miss - search and cache the result
                    self.apply_search_filter_optimized(games, &filter_settings.search_text, &filter_settings.search_mode);
 
                    // Store in cache for next time
                    if let Some(index) = &mut self.game_index {
                        index.cache_search(
                            search_key.clone(),
                            self.filtered_games_cache.clone()
                        );
                    }
                }
            }
        } else {
            // Fallback without index (should rarely happen)
            self.filtered_games_cache = self.filter_games_manual(
                games,
                selected_filter,
                filter_settings,
                hidden_categories,
            );
            if !filter_settings.search_text.is_empty() {
                self.apply_search_filter_optimized(games, &filter_settings.search_text, &filter_settings.search_mode);
            }
        }

        self.filter_cache_dirty = false;
        self.last_filter_update = Instant::now();

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 50 {
            println!("Warning: Filter update took {}ms for {} results",
                     elapsed.as_millis(), self.filtered_games_cache.len());
        }
    }

    /// Manual filtering fallback (rarely used)
    fn filter_games_manual(
        &self,
        games: &[Game],
        selected_filter: FilterCategory,
        filter_settings: &FilterSettings,
        hidden_categories: &HashSet<String>,
    ) -> Vec<usize> {
        games.iter()
        .enumerate()
        .filter(|(_, game)| {
            // First apply the main filter category
            let main_filter_passed = match selected_filter {
                FilterCategory::All => true,
                FilterCategory::Available => matches!(game.status, RomStatus::Available),
                FilterCategory::Missing => matches!(game.status, RomStatus::Missing),
                FilterCategory::Favorites => false, // Will be handled by index
                FilterCategory::Parents => !game.is_clone,
                FilterCategory::Clones => game.is_clone,
                FilterCategory::Working => matches!(game.status, RomStatus::Available),
                FilterCategory::NotWorking => !matches!(game.status, RomStatus::Available),
                FilterCategory::NonClones => !game.is_clone,
                FilterCategory::ChdGames => game.requires_chd,
                FilterCategory::NonMerged => !game.is_clone, // Same as Parents for now
            };
            
            if !main_filter_passed {
                return false;
            }
            
            // Then apply catver.ini category filter if set
            if let Some(ref category_name) = filter_settings.catver_category {
                if let Some(ref category_loader) = self.category_loader {
                    let category_name_lower = category_name.to_lowercase();
                    if let Some(game_category) = category_loader.get_category(&game.name) {
                        // Case-insensitive category comparison
                        return game_category.to_lowercase() == category_name_lower;
                    } else {
                        return false; // Game not found in catver.ini
                    }
                } else {
                    return false; // No category loader available
                }
            }
            
            true // No category filter applied
        })
        .map(|(idx, _)| idx)
        .collect()
    }

    /// OPTIMIZED: Apply search filter with parallel processing
    fn apply_search_filter_optimized(
        &mut self,
        games: &[Game],
        search_text: &str,
        search_mode: &SearchMode,
    ) {
        let search_lower = search_text.to_lowercase();

        // Use parallel processing for large datasets (huge speedup!)
        if self.filtered_games_cache.len() > 1000 {
            self.filtered_games_cache = self.filtered_games_cache
            .par_iter() // Parallel iterator from rayon
            .filter(|&&idx| {
                if let Some(game) = games.get(idx) {
                    match search_mode {
                        SearchMode::GameTitle => {
                            game.description.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Manufacturer => {
                            game.manufacturer.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::RomFileName => {
                            game.name.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Year => {
                            game.year.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Status => {
                            game.status.description().to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Cpu => {
                            game.driver.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Device => {
                            game.controls.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Sound => {
                            game.category.to_lowercase().contains(&search_lower)
                        }
                    }
                } else {
                    false
                }
            })
            .copied()
            .collect();
        } else {
            // Sequential processing for smaller datasets
            self.filtered_games_cache.retain(|&idx| {
                if let Some(game) = games.get(idx) {
                    match search_mode {
                        SearchMode::GameTitle => {
                            game.description.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Manufacturer => {
                            game.manufacturer.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::RomFileName => {
                            game.name.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Year => {
                            game.year.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Status => {
                            game.status.description().to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Cpu => {
                            game.driver.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Device => {
                            game.controls.to_lowercase().contains(&search_lower)
                        }
                        SearchMode::Sound => {
                            game.category.to_lowercase().contains(&search_lower)
                        }
                    }
                } else {
                    false
                }
            });
        }
    }

    /// Handle search input with debouncing
    pub fn handle_search_input(&mut self, new_text: String) {
        self.pending_search = Some(new_text);
        self.search_debounce_timer = Some(Instant::now());
    }

    /// Process pending search after debounce delay
    pub fn process_pending_search(&mut self) -> Option<String> {
        if let Some(pending) = &self.pending_search {
            if let Some(timer) = self.search_debounce_timer {
                let delay = Duration::from_millis(self.search_debounce_ms);

                if timer.elapsed() >= delay {
                    // Return the pending search text for application
                    let result = pending.clone();
                    self.pending_search = None;
                    self.search_debounce_timer = None;
                    self.filter_cache_dirty = true;
                    return Some(result);
                }
            }
        }
        None
    }

    /// Check if there's a pending search
    pub fn has_pending_search(&self) -> bool {
        self.pending_search.is_some()
    }

    /// Check if pending search should be processed based on debounce timer
    pub fn should_process_pending_search(&self, debounce_ms: u64) -> bool {
        if let Some(timer) = self.search_debounce_timer {
            let delay = Duration::from_millis(debounce_ms);
            timer.elapsed() >= delay
        } else {
            false
        }
    }

    /// Get filtered games cache
    pub fn get_filtered_games(&self) -> &[usize] {
        &self.filtered_games_cache
    }

    /// Check if filter cache is dirty
    pub fn is_cache_dirty(&self) -> bool {
        self.filter_cache_dirty
    }

    /// Mark cache as dirty
    pub fn mark_cache_dirty(&mut self) {
        self.filter_cache_dirty = true;
    }

    /// Clear search cache
    pub fn clear_search_cache(&mut self) {
        if let Some(index) = &mut self.game_index {
            index.clear_cache();
            println!("Cleared search cache");
        }
    }

    /// Get search cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        if let Some(index) = &self.game_index {
            (index.search_cache.len(), index.max_cache_size)
        } else {
            (0, self.max_cache_size)
        }
    }

    /// Update favorites in the index
    pub fn update_favorites(&mut self, games: &[Game], favorites: &HashSet<String>) {
        if let Some(index) = &mut self.game_index {
            index.update_favorites(games, favorites);
        }
        self.filter_cache_dirty = true;
    }

    /// Jump to game starting with character
    pub fn jump_to_game_starting_with(
        &self,
        games: &[Game],
        character: char,
        expanded_rows_cache: &[crate::ui::panels::game_list::RowData],
    ) -> Option<usize> {
        let search_char = character.to_lowercase().to_string();
        
        // Search through the expanded rows cache (which includes the current filter and sort)
        if let Some(row_index) = expanded_rows_cache.iter().position(|row| {
            if let Some(game) = games.get(row.game_idx) {
                // Jump based on game description (what's shown in the Game column)
                game.description.to_lowercase().starts_with(&search_char)
            } else {
                false
            }
        }) {
            // Found a game - get the actual game index
            if let Some(row_data) = expanded_rows_cache.get(row_index) {
                return Some(row_data.game_idx);
            }
        }
        None
    }

    /// Reset the manager
    pub fn reset(&mut self) {
        self.game_index = None;
        self.filtered_games_cache.clear();
        self.filter_cache_dirty = true;
        self.search_debounce_timer = None;
        self.pending_search = None;
        self.last_filter_update = Instant::now();
    }

    /// Get performance statistics
    pub fn get_stats(&self) -> GameIndexStats {
        GameIndexStats {
            has_index: self.game_index.is_some(),
            filtered_count: self.filtered_games_cache.len(),
            cache_dirty: self.filter_cache_dirty,
            search_cache_size: self.get_cache_stats().0,
            max_cache_size: self.get_cache_stats().1,
            last_update: self.last_filter_update,
        }
    }
}

/// Game index statistics
#[derive(Debug, Clone)]
pub struct GameIndexStats {
    pub has_index: bool,
    pub filtered_count: usize,
    pub cache_dirty: bool,
    pub search_cache_size: usize,
    pub max_cache_size: usize,
    pub last_update: Instant,
} 