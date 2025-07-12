use std::process::{Command, Child};
use crate::models::AppConfig;

pub fn launch_game(rom_name: &str, config: &AppConfig) -> Result<Child, Box<dyn std::error::Error>> {
    if let Some(mame) = config.mame_executables.get(config.selected_mame_index) {
        let mut cmd = Command::new(&mame.path);

        // Add ROM paths
        if !config.rom_dirs.is_empty() {
            let rom_paths = config.rom_dirs.iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join(";");
            cmd.arg("-rompath").arg(&rom_paths);
        }

        // Apply video settings
        if config.video_settings.video_backend != "auto" {
            cmd.arg("-video").arg(&config.video_settings.video_backend);
        }

        if config.video_settings.window_mode {
            cmd.arg("-window");
        }

        if config.video_settings.wait_vsync {
            cmd.arg("-waitvsync");
        }

        if !config.video_settings.keep_aspect {
            cmd.arg("-nokeepaspect");
        }

        if !config.video_settings.filter {
            cmd.arg("-nofilter");
        }

        if config.video_settings.prescale > 0 {
            cmd.arg("-prescale").arg(config.video_settings.prescale.to_string());
        }

        // Add custom args
        if !config.video_settings.custom_args.is_empty() {
            for arg in config.video_settings.custom_args.split_whitespace() {
                cmd.arg(arg);
            }
        }

        // ROM name must be last
        cmd.arg(rom_name);

        Ok(cmd.spawn()?)
    } else {
        Err("No MAME executable configured".into())
    }
}
