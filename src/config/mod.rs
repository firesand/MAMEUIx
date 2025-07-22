use crate::models::AppConfig;
// use std::path::PathBuf;
use anyhow::Result;

pub fn load_config() -> Result<AppConfig> {
    let config_dir = dirs::config_dir()
    .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
    .join("mameuix");

    let config_file = config_dir.join("config.toml");

    if config_file.exists() {
        let contents = std::fs::read_to_string(&config_file)?;
        Ok(toml::from_str(&contents)?)
    } else {
        Ok(AppConfig::default())
    }
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
