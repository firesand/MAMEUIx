// src/utils/enhanced_search.rs
// Enhanced search engine with fuzzy matching, full-text indexing, and regex caching

use crate::models::{Game, SearchMode};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use tantivy::schema::*;
use tantivy::{doc, collector::TopDocs, query::QueryParser, Index, IndexWriter, TantivyDocument};
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use anyhow::Result;

// Global regex cache for performance
lazy_static! {
    static ref REGEX_CACHE: Arc<Mutex<HashMap<String, Regex>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Configuration for search performance
#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub fuzzy_threshold: i64,           // Minimum fuzzy match score (0-100)
    pub max_fuzzy_results: usize,       // Maximum results for fuzzy search
    pub enable_fuzzy: bool,             // Enable/disable fuzzy search
    pub enable_fulltext: bool,          // Enable/disable full-text search
    pub enable_regex: bool,             // Enable/disable regex search
    pub fulltext_limit: usize,          // Maximum full-text search results
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            fuzzy_threshold: 30,        // 30% minimum match
            max_fuzzy_results: 100,
            enable_fuzzy: true,
            enable_fulltext: true,
            enable_regex: false,        // Disabled by default for security
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

        println!("Initializing full-text search index for {} games...", games.len());
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
        let query_parser = QueryParser::for_index(&index, vec![
            title, description, manufacturer, year, category, rom_name, controls, driver
        ]);

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
    pub fn fuzzy_search(&self, games: &[Game], query: &str, search_mode: &SearchMode) -> Vec<(usize, i64)> {
        if !self.config.enable_fuzzy || query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        
        for (idx, game) in games.iter().enumerate() {
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
                SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => &game.description,
            };

            if let Some(score) = self.fuzzy_matcher.fuzzy_match(search_text, query) {
                if score >= self.config.fuzzy_threshold {
                    results.push((idx, score));
                }
            }
        }

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Limit results
        results.truncate(self.config.max_fuzzy_results);
        
        results
    }

    /// Perform full-text search
    pub fn fulltext_search(&self, query: &str) -> Result<Vec<usize>> {
        if !self.config.enable_fulltext || query.is_empty() {
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

        let game_id_field = schema.get_field("game_id").unwrap();
        
        // Parse query and search
        let query = query_parser.parse_query(query)?;
        let reader = index.reader()?;
        
        let searcher = reader.searcher();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(self.config.fulltext_limit))?;

        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
                if let Some(game_id_value) = retrieved_doc.get_first(game_id_field) {
                    if let Some(game_id) = game_id_value.as_u64() {
                        results.push(game_id as usize);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Perform regex search with caching
    pub fn regex_search(&self, games: &[Game], pattern: &str, search_mode: &SearchMode) -> Result<Vec<usize>> {
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
        
        for (idx, game) in games.iter().enumerate() {
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
                SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => &game.description,
            };

            if regex.is_match(search_text) {
                results.push(idx);
            }
        }

        Ok(results)
    }

    /// Combined search using multiple strategies
    pub fn enhanced_search(&mut self, games: &[Game], query: &str, search_mode: &SearchMode) -> Result<Vec<usize>> {
        self.search_count += 1;
        self.last_search_time = std::time::Instant::now();

        let mut all_results = Vec::new();
        let mut result_scores: HashMap<usize, f64> = HashMap::new();

        // 1. Fuzzy search (gives scores)
        if self.config.enable_fuzzy {
            let fuzzy_results = self.fuzzy_search(games, query, search_mode);
            for (idx, score) in fuzzy_results {
                let normalized_score = (score as f64) / 100.0; // Normalize to 0-1
                result_scores.insert(idx, normalized_score * 0.4); // 40% weight for fuzzy
                if !all_results.contains(&idx) {
                    all_results.push(idx);
                }
            }
        }

        // 2. Full-text search 
        if self.config.enable_fulltext {
            if let Ok(fulltext_results) = self.fulltext_search(query) {
                for idx in fulltext_results {
                    let current_score = result_scores.get(&idx).unwrap_or(&0.0);
                    result_scores.insert(idx, current_score + 0.6); // 60% weight for full-text
                    if !all_results.contains(&idx) {
                        all_results.push(idx);
                    }
                }
            }
        }

        // 3. Regex search (if enabled)
        if self.config.enable_regex && (query.contains(".*") || query.contains("^") || query.contains("$")) {
            if let Ok(regex_results) = self.regex_search(games, query, search_mode) {
                for idx in regex_results {
                    let current_score = result_scores.get(&idx).unwrap_or(&0.0);
                    result_scores.insert(idx, current_score + 0.8); // High weight for regex matches
                    if !all_results.contains(&idx) {
                        all_results.push(idx);
                    }
                }
            }
        }

        // Sort by combined score (highest first)
        all_results.sort_by(|a, b| {
            let score_a = result_scores.get(a).unwrap_or(&0.0);
            let score_b = result_scores.get(b).unwrap_or(&0.0);
            score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_results)
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