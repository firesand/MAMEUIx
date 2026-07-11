// src/ui/mod.rs
// User interface components

pub mod components;
pub mod dock;
pub mod notifications;
pub mod panels;
pub mod redesign;
pub mod themes;

// Re-export commonly used components
pub use components::{DialogAction, DialogManager, DialogType};
