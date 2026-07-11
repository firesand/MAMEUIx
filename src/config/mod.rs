use crate::models::AppConfig;
use crate::models::MameExecutable;
use crate::models::UiShellMode;
use anyhow::Result;
use std::path::PathBuf;

pub fn load_config() -> Result<AppConfig> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("mameuix");

    let config_file = config_dir.join("config.toml");

    if config_file.exists() {
        let contents = std::fs::read_to_string(&config_file)?;
        let mut config = match toml::from_str::<AppConfig>(&contents) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!(
                    "Warning: config.toml parse failed ({e}). Salvaging paths and MAME settings..."
                );
                salvage_config(&contents)
            }
        };
        if !contents.contains("ui_shell") {
            config.preferences.ui_shell = if config.preferences.use_dock_layout {
                UiShellMode::LegacyDock
            } else {
                UiShellMode::LegacyClassic
            };
        }
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

/// Recover critical settings when the full config file is corrupted.
fn salvage_config(contents: &str) -> AppConfig {
    let mut config = AppConfig::default();

    // The header (before nested tables) is usually still valid after dedupe damage.
    let header_end = contents
        .find("\n[filter_settings]")
        .or_else(|| contents.find("\n[game_stats"))
        .or_else(|| contents.find("\n[default_game_properties"))
        .unwrap_or(contents.len());
    let header = &contents[..header_end];

    if let Ok(table) = toml::from_str::<toml::Table>(header) {
        if let Some(idx) = table
            .get("selected_mame_index")
            .and_then(|v| v.as_integer())
        {
            config.selected_mame_index = idx as usize;
        }
        if let Ok(paths) = table
            .get("rom_paths")
            .cloned()
            .unwrap_or(toml::Value::Array(vec![]))
            .try_into::<Vec<PathBuf>>()
        {
            config.rom_paths = paths;
            config.rom_dirs = config.rom_paths.clone();
        }
        for key in [
            "artwork_path",
            "snap_path",
            "cabinet_path",
            "title_path",
            "flyer_path",
            "marquee_path",
            "pcb_path",
            "icons_path",
            "sw_path",
            "history_path",
            "catver_ini_path",
            "mameinfo_dat_path",
            "gameinit_dat_path",
            "command_dat_path",
        ] {
            if let Some(val) = table.get(key).and_then(|v| v.as_str()) {
                let path: PathBuf = val.into();
                match key {
                    "artwork_path" => config.artwork_path = Some(path),
                    "snap_path" => config.snap_path = Some(path),
                    "cabinet_path" => config.cabinet_path = Some(path),
                    "title_path" => config.title_path = Some(path),
                    "flyer_path" => config.flyer_path = Some(path),
                    "marquee_path" => config.marquee_path = Some(path),
                    "pcb_path" => config.pcb_path = Some(path),
                    "icons_path" => config.icons_path = Some(path),
                    "sw_path" => config.sw_path = Some(path),
                    "history_path" => config.history_path = Some(path),
                    "catver_ini_path" => config.catver_ini_path = Some(path),
                    "mameinfo_dat_path" => config.mameinfo_dat_path = Some(path),
                    "gameinit_dat_path" => config.gameinit_dat_path = Some(path),
                    "command_dat_path" => config.command_dat_path = Some(path),
                    _ => {}
                }
            }
        }
        if let Ok(execs) = table
            .get("mame_executables")
            .cloned()
            .unwrap_or(toml::Value::Array(vec![]))
            .try_into::<Vec<MameExecutable>>()
        {
            config.mame_executables = execs;
        }
    }

    if contents.contains("ui_shell = \"RedesignPreview\"") {
        config.preferences.ui_shell = UiShellMode::RedesignPreview;
    } else if contents.contains("ui_shell = \"LegacyClassic\"") {
        config.preferences.ui_shell = UiShellMode::LegacyClassic;
    }

    config
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("mameuix");

    std::fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("config.toml");
    let contents = toml::to_string_pretty(config)?;
    std::fs::write(&config_file, contents)?;

    Ok(())
}
