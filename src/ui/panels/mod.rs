// src/ui/panels/mod.rs
// Main UI panels and components

pub mod game_list;
pub mod sidebar;
pub mod artwork_panel;
pub mod artwork_loader;
pub mod history_panel;
pub mod icon_manager;
pub mod performance_manager;
pub mod game_index_manager;

pub use game_list::GameList;
pub use sidebar::Sidebar;
pub use artwork_panel::ArtworkPanel;
pub use history_panel::HistoryPanel;
pub use icon_manager::IconManager;
pub use performance_manager::PerformanceManager;
pub use game_index_manager::GameIndexManager;
