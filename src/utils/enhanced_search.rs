// src/utils/enhanced_search.rs
// Enhanced search engine with fuzzy matching, full-text indexing, and regex caching

use crate::models::{Game, SearchMode};
use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tantivy::schema::*;
use tantivy::{
    Index, IndexWriter, TantivyDocument, Term,
    collector::TopDocs,
    doc,
    query::{BooleanQuery, ConstScoreQuery, QueryParser, TermSetQuery},
};

// Global regex cache for performance
lazy_static! {
    static ref REGEX_CACHE: Arc<Mutex<HashMap<String, Regex>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Configuration for search performance
#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub fuzzy_threshold: i64,     // Minimum fuzzy match score (0-100)
    pub max_fuzzy_results: usize, // Maximum results for fuzzy search
    pub enable_fuzzy: bool,       // Enable/disable fuzzy search
    pub enable_fulltext: bool,    // Enable/disable full-text search
    pub enable_regex: bool,       // Enable/disable regex search
    pub fulltext_limit: usize,    // Maximum full-text search results
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            fuzzy_threshold: 30, // 30% minimum match
            max_fuzzy_results: 100,
            enable_fuzzy: true,
            enable_fulltext: true,
            enable_regex: true, // Evaluated only when the user selects Regex mode
            fulltext_limit: 500,
        }
    }
}

/// Enhanced search engine with multiple search strategies
pub struct EnhancedSearchEngine {
    // Fuzzy matching
    fuzzy_matcher: SkimMatcherV2,

    // Full-text search
    fulltext_index: Option<Index>,
    fulltext_schema: Option<Schema>,
    query_parser: Option<QueryParser>,

    // Search configuration
    config: SearchConfig,

    // Performance metrics
    last_search_time: std::time::Instant,
    search_count: usize,
}

impl EnhancedSearchEngine {
    /// Create new enhanced search engine
    pub fn new(config: SearchConfig) -> Self {
        Self {
            fuzzy_matcher: SkimMatcherV2::default(),
            fulltext_index: None,
            fulltext_schema: None,
            query_parser: None,
            config,
            last_search_time: std::time::Instant::now(),
            search_count: 0,
        }
    }

    /// Initialize full-text search index
    pub fn initialize_fulltext_index(&mut self, games: &[Game]) -> Result<()> {
        if !self.config.enable_fulltext {
            return Ok(());
        }

        println!(
            "Initializing full-text search index for {} games...",
            games.len()
        );
        let start = std::time::Instant::now();

        // Create schema
        let mut schema_builder = Schema::builder();
        let game_id = schema_builder.add_u64_field("game_id", STORED | INDEXED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let description = schema_builder.add_text_field("description", TEXT);
        let manufacturer = schema_builder.add_text_field("manufacturer", TEXT);
        let year = schema_builder.add_text_field("year", TEXT);
        let category = schema_builder.add_text_field("category", TEXT);
        let rom_name = schema_builder.add_text_field("rom_name", TEXT);
        let controls = schema_builder.add_text_field("controls", TEXT);
        let driver = schema_builder.add_text_field("driver", TEXT);

        let schema = schema_builder.build();

        // Create in-memory index for speed
        let index = Index::create_in_ram(schema.clone());

        // Create query parser for multiple fields
        let query_parser = QueryParser::for_index(
            &index,
            vec![
                title,
                description,
                manufacturer,
                year,
                category,
                rom_name,
                controls,
                driver,
            ],
        );

        // Index all games
        let mut index_writer: IndexWriter = index.writer(50_000_000)?; // 50MB heap

        for (idx, game) in games.iter().enumerate() {
            let doc = doc!(
                game_id => idx as u64,
                title => game.description.clone(),
                description => game.description.clone(),
                manufacturer => game.manufacturer.clone(),
                year => game.year.clone(),
                category => game.category.clone(),
                rom_name => game.name.clone(),
                controls => game.controls.clone(),
                driver => game.driver.clone(),
            );
            index_writer.add_document(doc)?;
        }

        index_writer.commit()?;

        // Store everything
        self.fulltext_index = Some(index);
        self.fulltext_schema = Some(schema);
        self.query_parser = Some(query_parser);

        let elapsed = start.elapsed();
        println!("Full-text index built in {:.2}s", elapsed.as_secs_f32());

        Ok(())
    }

    /// Perform fuzzy search
    pub fn fuzzy_search(
        &self,
        games: &[Game],
        query: &str,
        search_mode: &SearchMode,
    ) -> Vec<(usize, i64)> {
        self.fuzzy_search_candidates(games, 0..games.len(), query, search_mode)
    }

    fn fuzzy_search_candidates(
        &self,
        games: &[Game],
        candidates: impl IntoIterator<Item = usize>,
        query: &str,
        search_mode: &SearchMode,
    ) -> Vec<(usize, i64)> {
        if !self.config.enable_fuzzy || query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for idx in candidates {
            let Some(game) = games.get(idx) else { continue };
            let search_text = match search_mode {
                SearchMode::GameTitle => &game.description,
                SearchMode::Manufacturer => &game.manufacturer,
                SearchMode::RomFileName => &game.name,
                SearchMode::Year => &game.year,
                SearchMode::Status => game.status.description(),
                SearchMode::Cpu => &game.driver,
                SearchMode::Device => &game.controls,
                SearchMode::Sound => &game.category,
                // Enhanced search modes should use enhanced_search() instead
                SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => {
                    &game.description
                }
            };

            if let Some(score) = self.fuzzy_matcher.fuzzy_match(search_text, query)
                && score >= self.config.fuzzy_threshold
            {
                results.push((idx, score));
            }
        }

        // Sort by score (highest first)
        results.sort_by_key(|b| std::cmp::Reverse(b.1));

        // Limit results
        results.truncate(self.config.max_fuzzy_results);

        results
    }

    /// Perform full-text search
    pub fn fulltext_search(&self, query: &str) -> Result<Vec<usize>> {
        self.fulltext_search_candidates(query, None)
    }

    fn fulltext_search_candidates(
        &self,
        query: &str,
        candidates: Option<&[usize]>,
    ) -> Result<Vec<usize>> {
        if !self.config.enable_fulltext
            || query.is_empty()
            || self.config.fulltext_limit == 0
            || candidates.is_some_and(|ids| ids.is_empty())
        {
            return Ok(Vec::new());
        }

        let index = match &self.fulltext_index {
            Some(index) => index,
            None => return Ok(Vec::new()),
        };

        let query_parser = match &self.query_parser {
            Some(parser) => parser,
            None => return Ok(Vec::new()),
        };

        let schema = match &self.fulltext_schema {
            Some(schema) => schema,
            None => return Ok(Vec::new()),
        };

        let game_id_field = schema.get_field("game_id")?;

        // Restrict indexed IDs before TopDocs applies its limit. Filtering a
        // globally capped result afterwards can discard every valid candidate.
        let mut query = query_parser.parse_query(query)?;
        if let Some(candidates) = candidates {
            let terms = candidates
                .iter()
                .map(|&idx| Term::from_field_u64(game_id_field, idx as u64));
            let candidate_query = ConstScoreQuery::new(Box::new(TermSetQuery::new(terms)), 0.0);
            query = Box::new(BooleanQuery::intersection(vec![
                query,
                Box::new(candidate_query),
            ]));
        }
        let reader = index.reader()?;

        let searcher = reader.searcher();
        let top_docs = searcher.search(
            &query,
            &TopDocs::with_limit(self.config.fulltext_limit).order_by_score(),
        )?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address)
                && let Some(game_id_value) = retrieved_doc.get_first(game_id_field)
                && let Some(game_id) = game_id_value.as_u64()
            {
                results.push(game_id as usize);
            }
        }

        Ok(results)
    }

    /// Perform regex search with caching
    pub fn regex_search(
        &self,
        games: &[Game],
        pattern: &str,
        search_mode: &SearchMode,
    ) -> Result<Vec<usize>> {
        self.regex_search_candidates(games, 0..games.len(), pattern, search_mode)
    }

    fn regex_search_candidates(
        &self,
        games: &[Game],
        candidates: impl IntoIterator<Item = usize>,
        pattern: &str,
        search_mode: &SearchMode,
    ) -> Result<Vec<usize>> {
        if !self.config.enable_regex || pattern.is_empty() {
            return Ok(Vec::new());
        }

        // Get or create cached regex
        let regex = {
            let mut cache = REGEX_CACHE.lock().unwrap();
            if let Some(cached_regex) = cache.get(pattern) {
                cached_regex.clone()
            } else {
                let new_regex = Regex::new(pattern)?;
                cache.insert(pattern.to_string(), new_regex.clone());
                new_regex
            }
        };

        let mut results = Vec::new();

        for idx in candidates {
            let Some(game) = games.get(idx) else { continue };
            let search_text = match search_mode {
                SearchMode::GameTitle => &game.description,
                SearchMode::Manufacturer => &game.manufacturer,
                SearchMode::RomFileName => &game.name,
                SearchMode::Year => &game.year,
                SearchMode::Status => game.status.description(),
                SearchMode::Cpu => &game.driver,
                SearchMode::Device => &game.controls,
                SearchMode::Sound => &game.category,
                // Enhanced search modes should use enhanced_search() instead
                SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => {
                    &game.description
                }
            };

            if regex.is_match(search_text) {
                results.push(idx);
            }
        }

        Ok(results)
    }

    /// Search all games using the explicitly selected enhanced mode.
    pub fn enhanced_search(
        &mut self,
        games: &[Game],
        query: &str,
        search_mode: &SearchMode,
    ) -> Result<Vec<usize>> {
        let candidates: Vec<_> = (0..games.len()).collect();
        self.enhanced_search_candidates(games, &candidates, query, search_mode)
    }

    /// Apply the selected search semantics to current filter candidates. Keep
    /// original game IDs throughout; subset positions are never library IDs.
    pub fn enhanced_search_candidates(
        &mut self,
        games: &[Game],
        candidates: &[usize],
        query: &str,
        search_mode: &SearchMode,
    ) -> Result<Vec<usize>> {
        self.search_count += 1;
        self.last_search_time = std::time::Instant::now();
        match search_mode {
            SearchMode::FuzzySearch => Ok(self
                .fuzzy_search_candidates(games, candidates.iter().copied(), query, search_mode)
                .into_iter()
                .map(|(idx, _score)| idx)
                .collect()),
            SearchMode::FullText => self.fulltext_search_candidates(query, Some(candidates)),
            SearchMode::Regex => {
                self.regex_search_candidates(games, candidates.iter().copied(), query, search_mode)
            }
            _ => anyhow::bail!("Expected an enhanced search mode"),
        }
    }

    /// Get search performance stats
    pub fn get_stats(&self) -> SearchStats {
        SearchStats {
            total_searches: self.search_count,
            last_search_duration: self.last_search_time.elapsed(),
            fulltext_enabled: self.fulltext_index.is_some(),
            fuzzy_enabled: self.config.enable_fuzzy,
            regex_enabled: self.config.enable_regex,
            regex_cache_size: {
                let cache = REGEX_CACHE.lock().unwrap();
                cache.len()
            },
        }
    }

    /// Clear regex cache (for memory management)
    pub fn clear_regex_cache() {
        let mut cache = REGEX_CACHE.lock().unwrap();
        cache.clear();
    }

    /// Update search configuration
    pub fn update_config(&mut self, config: SearchConfig) {
        self.config = config;
    }
}

/// Search performance statistics
#[derive(Debug, Clone)]
pub struct SearchStats {
    pub total_searches: usize,
    pub last_search_duration: std::time::Duration,
    pub fulltext_enabled: bool,
    pub fuzzy_enabled: bool,
    pub regex_enabled: bool,
    pub regex_cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RomStatus;

    fn game(name: &str, title: &str) -> Game {
        Game {
            name: name.into(),
            description: title.into(),
            manufacturer: "Maker".into(),
            year: "1990".into(),
            driver: "test".into(),
            driver_status: "good".into(),
            status: RomStatus::Available,
            parent: None,
            category: "Arcade".into(),
            play_count: 0,
            is_clone: false,
            is_device: false,
            is_bios: false,
            controls: "Joystick".into(),
            requires_roms: true,
            requires_chd: false,
            chd_name: None,
            verification_status: None,
        }
    }

    #[test]
    fn fulltext_search_returns_current_game_ids_after_reindexing() {
        let mut engine = EnhancedSearchEngine::new(SearchConfig::default());
        engine
            .initialize_fulltext_index(&[
                game("alpha", "Space Fighter"),
                game("bravo", "Racing Driver"),
            ])
            .unwrap();
        assert_eq!(engine.fulltext_search("Racing").unwrap(), vec![1]);
        engine
            .initialize_fulltext_index(&[game("charlie", "Racing Legend")])
            .unwrap();
        assert_eq!(engine.fulltext_search("Racing").unwrap(), vec![0]);
        assert!(engine.fulltext_search("Space").unwrap().is_empty());
    }
    #[test]
    fn candidate_search_filters_before_limits_and_keeps_original_game_ids() {
        let games = vec![
            game("strong", "Racing"),
            game("target", "A long Racing title with extra words"),
        ];
        let mut engine = EnhancedSearchEngine::new(SearchConfig {
            max_fuzzy_results: 1,
            fulltext_limit: 1,
            ..SearchConfig::default()
        });
        engine.initialize_fulltext_index(&games).unwrap();
        for mode in [SearchMode::FuzzySearch, SearchMode::FullText] {
            assert_eq!(
                engine.enhanced_search(&games, "Racing", &mode).unwrap(),
                vec![0]
            );
            assert_eq!(
                engine
                    .enhanced_search_candidates(&games, &[1], "Racing", &mode)
                    .unwrap(),
                vec![1]
            );
            assert!(
                engine
                    .enhanced_search_candidates(&games, &[], "Racing", &mode)
                    .unwrap()
                    .is_empty()
            );
            assert!(
                engine
                    .enhanced_search_candidates(&games, &[99], "Racing", &mode)
                    .unwrap()
                    .is_empty()
            );
        }
        engine.update_config(SearchConfig {
            fulltext_limit: 0,
            ..SearchConfig::default()
        });
        assert!(engine.fulltext_search("Racing").unwrap().is_empty());
    }

    #[test]
    fn explicit_regex_supports_alternation_and_never_unions_other_search_modes() {
        let mut games = vec![
            game("sf", "Street Fighter"),
            game("pac", "Pac Man"),
            game("other", "Unrelated"),
        ];
        games[2].manufacturer = "Street Fighter".into();
        let mut engine = EnhancedSearchEngine::new(SearchConfig::default());
        engine.initialize_fulltext_index(&games).unwrap();
        assert_eq!(
            engine
                .enhanced_search(&games, "Street|Pac", &SearchMode::Regex)
                .unwrap(),
            vec![0, 1]
        );
        assert_eq!(
            engine
                .enhanced_search(&games, "Street", &SearchMode::Regex)
                .unwrap(),
            vec![0]
        );
        assert_eq!(
            engine
                .enhanced_search_candidates(&games, &[1], "Street|Pac", &SearchMode::Regex)
                .unwrap(),
            vec![1]
        );
        assert!(
            engine
                .enhanced_search_candidates(&games, &[], "Street|Pac", &SearchMode::Regex)
                .unwrap()
                .is_empty()
        );
        assert!(
            engine
                .enhanced_search_candidates(&games, &[99], "Street|Pac", &SearchMode::Regex)
                .unwrap()
                .is_empty()
        );
        assert!(
            engine
                .enhanced_search(&games, "[", &SearchMode::Regex)
                .is_err()
        );
        engine.update_config(SearchConfig {
            enable_regex: false,
            ..SearchConfig::default()
        });
        assert!(
            engine
                .enhanced_search(&games, "Street", &SearchMode::Regex)
                .unwrap()
                .is_empty()
        );
    }
}
