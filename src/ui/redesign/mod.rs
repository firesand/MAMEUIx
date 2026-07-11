//! Experimental Steam-inspired UI shell (design handoff).
//!
//! This module is intentionally isolated from the legacy UI (`show_classic_layout`,
//! `show_dock_layout`, dialogs). Enable it via **Preferences → UI shell → Redesign preview**.
//! Switch back to **Dock panels** or **Classic menu bar** at any time without losing data.

pub(crate) mod fonts;
mod shell;
mod state;
pub mod tokens;
mod topbar;
mod views;
mod widgets;

pub use shell::RedesignShell;
