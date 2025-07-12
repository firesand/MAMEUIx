use crate::models::Game;
use std::process::Command;
use anyhow::Result;

pub struct GameScanner {
    mame_path: String,
}

impl GameScanner {
    pub fn new(mame_path: &str) -> Self {
        Self {
            mame_path: mame_path.to_string(),
        }
    }

    pub fn scan_games(&self) -> Result<Vec<Game>> {
        let output = Command::new(&self.mame_path)
        .arg("-listxml")
        .output()?;

        let xml_str = String::from_utf8_lossy(&output.stdout);

        // Simplified XML parsing - in real implementation use quick-xml
        let mut games = Vec::new();

        for entry in xml_str.split("<machine ").skip(1) {
            if let Some(name) = Self::extract_attribute(entry, "name") {
                let description = Self::extract_tag(entry, "description").unwrap_or_default();
                let year = Self::extract_tag(entry, "year").unwrap_or_default();
                let manufacturer = Self::extract_tag(entry, "manufacturer").unwrap_or_default();
                let parent = Self::extract_attribute(entry, "cloneof");
                let is_device = entry.contains("isdevice=\"yes\"");
                let is_bios = entry.contains("isbios=\"yes\"");

                games.push(Game {
                    name: name.to_string(),
                           description,
                           manufacturer,
                           year,
                           driver: String::new(),
                           status: crate::models::RomStatus::Available,
                           parent,
                           category: String::new(),
                           play_count: 0,
                           is_clone: parent.is_some(),
                           is_device,
                           is_bios,
                           controls: String::new(),
                });
            }
        }

        Ok(games)
    }

    fn extract_attribute(text: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        text.find(&pattern)
        .and_then(|start| {
            let start = start + pattern.len();
            text[start..].find('"').map(|end| text[start..start + end].to_string())
        })
    }

    fn extract_tag(text: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);

        text.find(&start_tag)
        .and_then(|start| {
            let start = start + start_tag.len();
            text[start..].find(&end_tag).map(|end| text[start..start + end].to_string())
        })
    }
}
