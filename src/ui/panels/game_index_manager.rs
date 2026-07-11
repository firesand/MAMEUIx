// src/ui/game_index_manager.rs
// Game indexing, filtering, and search management module

use crate::models::*;
use crate::utils::enhanced_search::{EnhancedSearchEngine, SearchConfig, SearchStats};
use crate::utils::hardware_filter::HardwareFilter;
use rayon::prelude::*;
use std::collections::HashSet;
use std::time::{Duration, Instant};

pub struct GameIndexManager {
    // Core indexing
    pub game_index: Option<GameIndex>,
    pub filtered_games_cache: Vec<usize>,
    pub filter_cache_dirty: bool,
    pub last_filter_update: Instant,

    // Search management
    pub search_debounce_timer: Option<Instant>,
    pub pending_search: Option<String>,

    // Enhanced search engine
    pub enhanced_search: Option<EnhancedSearchEngine>,

    // Category management - REMOVED

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
            enhanced_search: Some(EnhancedSearchEngine::new(SearchConfig::default())),
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

    /// Build game index for fast lookup - CRITICAL for performance!
    pub fn build_game_index(&mut self, games: &[Game], favorites: &HashSet<String>) {
        println!("Building optimized game index for {} games...", games.len());
        let start = Instant::now();

        self.game_index = Some(GameIndex::build(games.to_vec(), favorites.clone()));

        let elapsed = start.elapsed();
        println!("Game index built in {:.2}s", elapsed.as_secs_f32());

        // Initialize enhanced search engine with games data
        if let Some(ref mut search_engine) = self.enhanced_search
            && let Err(e) = search_engine.initialize_fulltext_index(games)
        {
            eprintln!("Warning: Failed to initialize full-text search: {}", e);
        }

        // Force filter update with new index
        self.filter_cache_dirty = true;
    }

    /// OPTIMIZED: Update filtered games cache with new multi-selection filters
    pub fn update_filtered_games_cache(
        &mut self,
        games: &[Game],
        _selected_filter: FilterCategory, // Deprecated parameter, kept for compatibility
        filter_settings: &FilterSettings,
        _hidden_categories: &HashSet<String>,
        hardware_filter: Option<&HardwareFilter>,
    ) {
        if !self.filter_cache_dirty {
            return;
        }

        let start = Instant::now();

        // Start with all games
        self.filtered_games_cache = (0..games.len()).collect();

        // Apply new multi-selection filters
        let favorites = if let Some(idx) = &self.game_index {
            idx.favorites.clone()
        } else {
            HashSet::new()
        };
        self.apply_categorized_filters_with_favorites(
            games,
            filter_settings,
            &favorites,
            hardware_filter,
        );

        // Apply search filter only if there's text. Cache entries are scoped to
        // the complete filter state: a query cached for one manufacturer or
        // decade must never replace the result for another selection.
        if !filter_settings.search_text.is_empty() {
            let search_key = Self::filtered_search_cache_key(filter_settings);

            if let Some(ref index) = self.game_index {
                if let Some(cached) = index.get_cached_search(&search_key) {
                    // This entry already represents this exact filter/search
                    // combination, so it is safe to use as the final result.
                    self.filtered_games_cache = cached.to_vec();
                } else {
                    // Cache miss - search and cache the result
                    self.apply_search_filter_optimized(
                        games,
                        &filter_settings.search_text,
                        &filter_settings.search_mode,
                        hardware_filter,
                    );

                    // Store in cache for next time
                    if let Some(index) = &mut self.game_index {
                        index.cache_search(search_key.clone(), self.filtered_games_cache.clone());
                    }
                }
            } else {
                // No index available, do regular search
                self.apply_search_filter_optimized(
                    games,
                    &filter_settings.search_text,
                    &filter_settings.search_mode,
                    hardware_filter,
                );
            }
        }

        self.filter_cache_dirty = false;
        self.last_filter_update = Instant::now();

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 50 {
            println!(
                "Warning: Filter update took {}ms for {} results",
                elapsed.as_millis(),
                self.filtered_games_cache.len()
            );
        }
    }

    fn filtered_search_cache_key(filters: &FilterSettings) -> String {
        let mut manufacturers: Vec<_> = filters.selected_manufacturers.iter().collect();
        manufacturers.sort_unstable();

        // Prefix keeps these final-result entries separate from the legacy list
        // widgets, which still cache plain text queries in the same GameIndex.
        format!(
            "__manager_v2__|q={:?}|mode={:?}|availability={}:{}|status={}:{}|other={}:{}:{}|mfr={:?}|year={:?}:{:?}|hardware={:?}:{:?}:{:?}",
            filters.search_text,
            filters.search_mode,
            filters.availability_filters.show_available,
            filters.availability_filters.show_unavailable,
            filters.status_filters.show_working,
            filters.status_filters.show_not_working,
            filters.other_filters.show_favorites,
            filters.other_filters.show_parents_only,
            filters.other_filters.show_chd_games,
            manufacturers,
            filters.year_from,
            filters.year_to,
            filters.cpu_filter,
            filters.device_filter,
            filters.sound_filter,
        )
    }

    /// Apply the new categorized multi-selection filters
    fn apply_categorized_filters_with_favorites(
        &mut self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        hardware_filter: Option<&HardwareFilter>,
    ) {
        let hardware_fields_active = !filters.cpu_filter.is_empty()
            || !filters.device_filter.is_empty()
            || !filters.sound_filter.is_empty();

        self.filtered_games_cache.retain(|&idx| {
            if let Some(game) = games.get(idx) {
                // AVAILABILITY check (OR within category)
                let availability_match = {
                    let avail = &filters.availability_filters;
                    // If no filters selected, show all
                    if !avail.show_available && !avail.show_unavailable {
                        true
                    } else {
                        (avail.show_available && matches!(game.status, RomStatus::Available))
                            || (avail.show_unavailable
                                && !matches!(game.status, RomStatus::Available))
                    }
                };

                // STATUS check (OR within category)
                let status_match = {
                    let status = &filters.status_filters;
                    // If no filters selected, show all
                    if !status.show_working && !status.show_not_working {
                        true
                    } else {
                        let is_working =
                            matches!(game.driver_status.as_str(), "good" | "imperfect");
                        (status.show_working && is_working)
                            || (status.show_not_working && !is_working)
                    }
                };

                // OTHERS check (OR within category)
                let others_match = {
                    let others = &filters.other_filters;
                    // If no filters selected, show all
                    if !others.show_favorites && !others.show_parents_only && !others.show_chd_games
                    {
                        true
                    } else {
                        (others.show_favorites && favorites.contains(&game.name))
                            || (others.show_parents_only && !game.is_clone)
                            || (others.show_chd_games && game.requires_chd)
                    }
                };

                let hardware_match = if !hardware_fields_active {
                    true
                } else if let Some(hw) = hardware_filter {
                    hw.matches_hardware_filters(
                        &game.name,
                        &filters.cpu_filter,
                        &filters.device_filter,
                        &filters.sound_filter,
                    )
                } else {
                    false
                };

                // AND logic between categories
                availability_match
                    && status_match
                    && others_match
                    && hardware_match
                    && filters.manufacturer_matches(&game.manufacturer)
                    && filters.year_matches(&game.year)
            } else {
                false
            }
        });
    }

    fn hardware_search_match(
        game: &Game,
        search_lower: &str,
        search_mode: &SearchMode,
        hardware_filter: Option<&HardwareFilter>,
    ) -> bool {
        match search_mode {
            SearchMode::Cpu => hardware_filter
                .map(|hw| hw.game_uses_cpu(&game.name, search_lower))
                .unwrap_or(false),
            SearchMode::Device => hardware_filter
                .map(|hw| hw.game_uses_device(&game.name, search_lower))
                .unwrap_or(false),
            SearchMode::Sound => hardware_filter
                .map(|hw| hw.game_uses_sound(&game.name, search_lower))
                .unwrap_or(false),
            _ => false,
        }
    }

    /// ENHANCED: Apply search filter with multiple search strategies
    fn apply_search_filter_optimized(
        &mut self,
        games: &[Game],
        search_text: &str,
        search_mode: &SearchMode,
        hardware_filter: Option<&HardwareFilter>,
    ) {
        // Ranked enhanced search has a result cap. Once another predicate has
        // narrowed the candidates (for example manufacturer or decade), search
        // those candidates directly so globally-ranked games cannot crowd out a
        // valid result from the selected facet.
        if self.filtered_games_cache.len() == games.len() {
            match search_mode {
                SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => {
                    if let Some(ref mut search_engine) = self.enhanced_search {
                        match search_engine.enhanced_search(games, search_text, search_mode) {
                            Ok(results) => {
                                // Filter current cache to only include enhanced search results
                                let result_set: HashSet<usize> = results.into_iter().collect();
                                self.filtered_games_cache
                                    .retain(|&idx| result_set.contains(&idx));
                                return;
                            }
                            Err(e) => {
                                eprintln!(
                                    "Enhanced search failed: {}, falling back to basic search",
                                    e
                                );
                                // Fall through to basic search
                            }
                        }
                    }
                }
                _ => {
                    // For regular search modes, try enhanced search first as it's often better
                    if let Some(ref mut search_engine) = self.enhanced_search {
                        match search_engine.enhanced_search(games, search_text, search_mode) {
                            Ok(results) => {
                                let result_set: HashSet<usize> = results.into_iter().collect();
                                self.filtered_games_cache
                                    .retain(|&idx| result_set.contains(&idx));
                                return;
                            }
                            Err(_) => {
                                // Fall through to basic search if enhanced search fails
                            }
                        }
                    }
                }
            }
        }

        // Fallback: Original basic search implementation
        let search_lower = search_text.to_lowercase();

        // Use parallel processing for large datasets (huge speedup!)
        if self.filtered_games_cache.len() > 1000 {
            self.filtered_games_cache = self
                .filtered_games_cache
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
                            SearchMode::Year => game.year.to_lowercase().contains(&search_lower),
                            SearchMode::Status => game
                                .status
                                .description()
                                .to_lowercase()
                                .contains(&search_lower),
                            SearchMode::Cpu => Self::hardware_search_match(
                                game,
                                &search_lower,
                                search_mode,
                                hardware_filter,
                            ),
                            SearchMode::Device => Self::hardware_search_match(
                                game,
                                &search_lower,
                                search_mode,
                                hardware_filter,
                            ),
                            SearchMode::Sound => Self::hardware_search_match(
                                game,
                                &search_lower,
                                search_mode,
                                hardware_filter,
                            ),
                            // Enhanced search modes shouldn't reach here, but just in case
                            SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => {
                                game.description.to_lowercase().contains(&search_lower)
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
                        SearchMode::RomFileName => game.name.to_lowercase().contains(&search_lower),
                        SearchMode::Year => game.year.to_lowercase().contains(&search_lower),
                        SearchMode::Status => game
                            .status
                            .description()
                            .to_lowercase()
                            .contains(&search_lower),
                        SearchMode::Cpu => Self::hardware_search_match(
                            game,
                            &search_lower,
                            search_mode,
                            hardware_filter,
                        ),
                        SearchMode::Device => Self::hardware_search_match(
                            game,
                            &search_lower,
                            search_mode,
                            hardware_filter,
                        ),
                        SearchMode::Sound => Self::hardware_search_match(
                            game,
                            &search_lower,
                            search_mode,
                            hardware_filter,
                        ),
                        // Enhanced search modes shouldn't reach here, but just in case
                        SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => {
                            game.description.to_lowercase().contains(&search_lower)
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
        if let Some(pending) = &self.pending_search
            && let Some(timer) = self.search_debounce_timer
        {
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
            index.clear_cache();
        }
        self.filter_cache_dirty = true;
    }

    /// Configure enhanced search settings
    pub fn configure_enhanced_search(&mut self, config: SearchConfig) {
        if let Some(ref mut search_engine) = self.enhanced_search {
            search_engine.update_config(config);
        }
        if let Some(index) = &mut self.game_index {
            index.clear_cache();
        }
        self.filter_cache_dirty = true;
    }

    /// Get enhanced search statistics
    pub fn get_enhanced_search_stats(&self) -> Option<SearchStats> {
        self.enhanced_search
            .as_ref()
            .map(|engine| engine.get_stats())
    }

    /// Clear regex cache to free memory
    pub fn clear_regex_cache() {
        EnhancedSearchEngine::clear_regex_cache();
    }

    /// Check if enhanced search is available
    pub fn has_enhanced_search(&self) -> bool {
        self.enhanced_search.is_some()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn game(name: &str, title: &str, manufacturer: &str, year: &str) -> Game {
        Game {
            name: name.to_string(),
            description: title.to_string(),
            manufacturer: manufacturer.to_string(),
            year: year.to_string(),
            driver: "test".to_string(),
            driver_status: "good".to_string(),
            status: RomStatus::Available,
            parent: None,
            category: "Test".to_string(),
            play_count: 0,
            is_clone: false,
            is_device: false,
            is_bios: false,
            controls: String::new(),
            requires_chd: false,
            chd_name: None,
            verification_status: None,
        }
    }

    fn manager_with_index(games: &[Game]) -> GameIndexManager {
        let mut manager = GameIndexManager::new();
        manager.enhanced_search = None;
        manager.game_index = Some(GameIndex::build(games.to_vec(), HashSet::new()));
        manager
    }

    fn update(manager: &mut GameIndexManager, games: &[Game], filters: &FilterSettings) {
        manager.update_filtered_games_cache(
            games,
            FilterCategory::All,
            filters,
            &HashSet::new(),
            None,
        );
    }

    #[test]
    fn manufacturer_and_year_filters_are_anded() {
        let games = vec![
            game("capcom89", "Capcom 1989", "Capcom", "1989"),
            game("capcom91", "Capcom 1991", "Capcom", "1991"),
            game("sega89", "Sega 1989", "Sega", "1989"),
            game("unknown", "Unknown year", "Capcom", "????"),
        ];
        let mut manager = manager_with_index(&games);
        let mut filters = FilterSettings {
            year_from: "1980".to_string(),
            year_to: "1989".to_string(),
            ..FilterSettings::default()
        };
        filters.selected_manufacturers.insert("Capcom".to_string());

        update(&mut manager, &games, &filters);

        assert_eq!(manager.get_filtered_games(), &[0]);
    }

    #[test]
    fn cached_search_is_scoped_to_manufacturer_and_year_filters() {
        let games = vec![
            game("sf", "Street Fighter", "Capcom", "1987"),
            game("streetsm", "Street Smart", "SNK", "1989"),
            game("sf2", "Street Fighter II", "Capcom", "1991"),
        ];
        let mut manager = manager_with_index(&games);
        let mut filters = FilterSettings {
            search_text: "Street".to_string(),
            ..FilterSettings::default()
        };

        update(&mut manager, &games, &filters);
        assert_eq!(manager.get_filtered_games(), &[0, 1, 2]);

        filters.selected_manufacturers.insert("Capcom".to_string());
        filters.year_from = "1980".to_string();
        filters.year_to = "1989".to_string();
        manager.mark_cache_dirty();
        update(&mut manager, &games, &filters);
        assert_eq!(manager.get_filtered_games(), &[0]);

        filters.year_from = "1990".to_string();
        filters.year_to = "1999".to_string();
        manager.mark_cache_dirty();
        update(&mut manager, &games, &filters);
        assert_eq!(manager.get_filtered_games(), &[2]);

        // Same manufacturer and query, earlier decade: this must hit the
        // decade-specific entry rather than reuse the 1990s result.
        filters.year_from = "1980".to_string();
        filters.year_to = "1989".to_string();
        manager.mark_cache_dirty();
        update(&mut manager, &games, &filters);
        assert_eq!(manager.get_filtered_games(), &[0]);

        filters.selected_manufacturers.clear();
        filters.selected_manufacturers.insert("SNK".to_string());
        manager.mark_cache_dirty();
        update(&mut manager, &games, &filters);
        assert_eq!(manager.get_filtered_games(), &[1]);

        // Return to an earlier combination to exercise a cache hit.
        filters.selected_manufacturers.clear();
        filters.selected_manufacturers.insert("Capcom".to_string());
        manager.mark_cache_dirty();
        update(&mut manager, &games, &filters);
        assert_eq!(manager.get_filtered_games(), &[0]);
    }

    #[test]
    fn narrowed_manufacturer_search_is_not_lost_to_global_rank_limit() {
        let mut games: Vec<_> = (0..120)
            .map(|index| game(&format!("other{index}"), "Street", "Other", "1987"))
            .collect();
        let target_index = games.len();
        games.push(game("target", "zzzz Street zzzz", "Capcom", "1987"));

        let mut manager = GameIndexManager::new();
        manager.game_index = Some(GameIndex::build(games.clone(), HashSet::new()));
        let mut filters = FilterSettings {
            search_text: "Street".to_string(),
            ..FilterSettings::default()
        };
        filters.selected_manufacturers.insert("Capcom".to_string());

        update(&mut manager, &games, &filters);

        assert_eq!(manager.get_filtered_games(), &[target_index]);
    }
}
