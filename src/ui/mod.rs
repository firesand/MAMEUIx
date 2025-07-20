// src/ui/mod.rs
// User interface components

pub mod panels;
pub mod components;
pub mod themes;

// Re-export commonly used components
pub use panels::{GameList, Sidebar, ArtworkPanel, HistoryPanel, IconManager, PerformanceManager, GameIndexManager};
pub use components::{DialogManager, DialogType, DialogAction}; 