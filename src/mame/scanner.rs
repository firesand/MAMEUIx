// src/mame/scanner.rs
use crate::mame::CategoryLoader;
use crate::models::{Game, RomSetType};
use anyhow::{Context, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Default)]
struct ParsedMachine {
    name: Option<String>,
    description: Option<String>,
    year: Option<String>,
    manufacturer: Option<String>,
    parent: Option<String>,
    rom_parent: Option<String>,
    device_refs: Vec<String>,
    source_file: Option<String>,
    driver_name: Option<String>,
    driver_status: Option<String>,
    category: Option<String>,
    is_device: bool,
    is_bios: bool,
    disk_name: Option<String>,
    has_required_media: bool,
}

pub struct GameScanner {
    mame_path: String,
    category_loader: Option<CategoryLoader>,
}

impl GameScanner {
    pub fn new(mame_path: &str) -> Self {
        Self {
            mame_path: mame_path.to_string(),
            category_loader: None,
        }
    }

    /// Set the category loader for this scanner
    pub fn with_category_loader(mut self, loader: CategoryLoader) -> Self {
        self.category_loader = Some(loader);
        self
    }

    /// Scan games dari MAME menggunakan -listxml
    /// Versi ini dioptimasi untuk eksekusi background thread
    pub fn scan_games(&self) -> Result<Vec<Game>> {
        // Pertama, verifikasi MAME executable valid
        if !std::path::Path::new(&self.mame_path).exists() {
            return Err(anyhow::anyhow!(
                "MAME executable not found at: {}",
                self.mame_path
            ));
        }

        // Jalankan mame -listxml dan capture output
        let output = Command::new(&self.mame_path)
            .arg("-listxml")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("Failed to execute MAME")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("MAME -listxml failed: {}", stderr));
        }

        // Parse XML output
        let xml_str = String::from_utf8_lossy(&output.stdout);
        let games = self.parse_xml(&xml_str)?;

        Ok(games)
    }

    /// Parse MAME -listxml output using a real XML reader.
    fn parse_xml(&self, xml_str: &str) -> Result<Vec<Game>> {
        Ok(self.games_from_machines(Self::parse_machines(xml_str)?))
    }

    fn parse_machines(xml_str: &str) -> Result<Vec<ParsedMachine>> {
        let mut machines = Vec::new();
        let mut reader = Reader::from_str(xml_str);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_machine: Option<ParsedMachine> = None;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(event) if event.name().as_ref() == b"machine" => {
                    current_machine = Some(Self::machine_from_start(&event, &reader)?);
                }
                Event::Start(event) => {
                    if let Some(machine) = current_machine.as_mut() {
                        match event.name().as_ref() {
                            b"description" => {
                                machine.description =
                                    Some(Self::read_text_unescaped(&mut reader, event.name())?);
                            }
                            b"year" => {
                                machine.year =
                                    Some(Self::read_text_unescaped(&mut reader, event.name())?);
                            }
                            b"manufacturer" => {
                                machine.manufacturer =
                                    Some(Self::read_text_unescaped(&mut reader, event.name())?);
                            }
                            b"category" => {
                                machine.category =
                                    Some(Self::read_text_unescaped(&mut reader, event.name())?);
                            }
                            b"driver" => Self::read_driver_attrs(machine, &event, &reader)?,
                            b"rom" => Self::read_rom_attrs(machine, &event, &reader)?,
                            b"disk" => Self::read_disk_attrs(machine, &event, &reader)?,
                            b"device_ref" => Self::read_device_ref_attrs(machine, &event, &reader)?,
                            _ => {}
                        }
                    }
                }
                Event::Empty(event) if event.name().as_ref() == b"machine" => {
                    machines.push(Self::machine_from_start(&event, &reader)?);
                }
                Event::Empty(event) => {
                    if let Some(machine) = current_machine.as_mut() {
                        match event.name().as_ref() {
                            b"driver" => Self::read_driver_attrs(machine, &event, &reader)?,
                            b"rom" => Self::read_rom_attrs(machine, &event, &reader)?,
                            b"disk" => Self::read_disk_attrs(machine, &event, &reader)?,
                            b"device_ref" => Self::read_device_ref_attrs(machine, &event, &reader)?,
                            _ => {}
                        }
                    }
                }
                Event::End(event) if event.name().as_ref() == b"machine" => {
                    if let Some(machine) = current_machine.take() {
                        machines.push(machine);
                    }
                }
                Event::Eof => break,
                _ => {}
            }

            buf.clear();
        }

        Ok(machines)
    }

    fn games_from_machines(&self, machines: Vec<ParsedMachine>) -> Vec<Game> {
        let by_name: HashMap<&str, usize> = machines
            .iter()
            .enumerate()
            .filter_map(|(index, machine)| machine.name.as_deref().map(|name| (name, index)))
            .collect();
        let mut requires_roms: Vec<bool> = machines
            .iter()
            .map(|machine| machine.has_required_media)
            .collect();
        let mut dependents = vec![Vec::new(); machines.len()];
        for (index, machine) in machines.iter().enumerate() {
            for dependency in machine.rom_parent.iter().chain(machine.device_refs.iter()) {
                if let Some(&dependency_index) = by_name.get(dependency.as_str()) {
                    dependents[dependency_index].push(index);
                } else {
                    // A partial XML listing cannot prove an unresolved BIOS or
                    // device has no ROMs. Keep its dependents visible.
                    requires_roms[index] = true;
                }
            }
        }

        // Propagate requirements backwards through the dependency graph. Each
        // node is queued at most once, so forward references and cycles need no
        // recursion and the work remains O(machines + references).
        let mut pending: VecDeque<usize> = requires_roms
            .iter()
            .enumerate()
            .filter_map(|(index, &required)| required.then_some(index))
            .collect();
        while let Some(index) = pending.pop_front() {
            for &dependent in &dependents[index] {
                if !requires_roms[dependent] {
                    requires_roms[dependent] = true;
                    pending.push_back(dependent);
                }
            }
        }

        machines
            .into_iter()
            .zip(requires_roms)
            .filter_map(|(machine, required)| self.game_from_machine(machine, required))
            .collect()
    }

    fn machine_from_start(event: &BytesStart<'_>, reader: &Reader<&[u8]>) -> Result<ParsedMachine> {
        Ok(ParsedMachine {
            name: Self::xml_attr(event, b"name", reader)?,
            parent: Self::xml_attr(event, b"cloneof", reader)?,
            rom_parent: Self::xml_attr(event, b"romof", reader)?,
            source_file: Self::xml_attr(event, b"sourcefile", reader)?,
            is_device: Self::xml_attr(event, b"isdevice", reader)?.as_deref() == Some("yes"),
            is_bios: Self::xml_attr(event, b"isbios", reader)?.as_deref() == Some("yes"),
            ..Default::default()
        })
    }

    fn read_driver_attrs(
        machine: &mut ParsedMachine,
        event: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
    ) -> Result<()> {
        if let Some(status) = Self::xml_attr(event, b"status", reader)? {
            machine.driver_status = Some(status);
        }
        if let Some(name) = Self::xml_attr(event, b"name", reader)? {
            machine.driver_name = Some(name);
        }
        Ok(())
    }

    fn read_rom_attrs(
        machine: &mut ParsedMachine,
        event: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
    ) -> Result<()> {
        machine.has_required_media |= Self::is_required_media(event, reader)?;
        Ok(())
    }

    fn is_required_media(event: &BytesStart<'_>, reader: &Reader<&[u8]>) -> Result<bool> {
        Ok(
            Self::xml_attr(event, b"optional", reader)?.as_deref() != Some("yes")
                && Self::xml_attr(event, b"status", reader)?.as_deref() != Some("nodump"),
        )
    }

    fn read_device_ref_attrs(
        machine: &mut ParsedMachine,
        event: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
    ) -> Result<()> {
        if let Some(name) = Self::xml_attr(event, b"name", reader)? {
            machine.device_refs.push(name);
        } else {
            // An incomplete reference cannot establish the absence of media.
            machine.has_required_media = true;
        }
        Ok(())
    }

    fn read_disk_attrs(
        machine: &mut ParsedMachine,
        event: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
    ) -> Result<()> {
        if Self::is_required_media(event, reader)? {
            machine.has_required_media = true;
            if machine.disk_name.is_none() {
                machine.disk_name = Self::xml_attr(event, b"name", reader)?;
            }
        }
        Ok(())
    }

    fn xml_attr(
        event: &BytesStart<'_>,
        key: &[u8],
        reader: &Reader<&[u8]>,
    ) -> Result<Option<String>> {
        for attr in event.attributes().with_checks(false) {
            let attr = attr?;
            if attr.key.as_ref() == key {
                return Ok(Some(
                    attr.decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())?
                        .into_owned(),
                ));
            }
        }
        Ok(None)
    }

    fn read_text_unescaped(
        reader: &mut Reader<&[u8]>,
        end: quick_xml::name::QName<'_>,
    ) -> Result<String> {
        let text = reader.read_text(end)?;
        let decoded = text.xml10_content()?;
        Ok(quick_xml::escape::unescape(&decoded)?.into_owned())
    }

    fn game_from_machine(&self, machine: ParsedMachine, requires_roms: bool) -> Option<Game> {
        let name = machine.name?;
        let description = machine.description.unwrap_or_else(|| name.clone());
        let year = machine.year.unwrap_or_else(|| "????".to_string());
        let manufacturer = machine
            .manufacturer
            .unwrap_or_else(|| "Unknown".to_string());
        let source_file = machine.source_file.unwrap_or_else(|| "unknown".to_string());
        let parent = machine.parent;

        // Only declared, non-optional, dumped disk images imply a required CHD.
        // Device capabilities (e.g. a CD drive) do not require mounted media,
        // and names/descriptions cannot override MAME's ROM definitions.
        let chd_name = machine.disk_name;
        let requires_chd = chd_name.is_some();

        Some(Game {
            name: name.clone(),
            description: description.clone(),
            manufacturer,
            year,
            driver: machine.driver_name.unwrap_or_else(|| {
                source_file
                    .trim_end_matches(".cpp")
                    .trim_end_matches(".c")
                    .to_string()
            }),
            driver_status: machine
                .driver_status
                .unwrap_or_else(|| "unknown".to_string()),
            status: crate::models::RomStatus::Unknown,
            parent: parent.clone(),
            category: self.category_for_game(
                &name,
                parent.as_deref(),
                machine.category.as_deref(),
                &description,
            ),
            play_count: 0,
            is_clone: parent.is_some(),
            is_device: machine.is_device,
            is_bios: machine.is_bios,
            controls: String::new(),
            requires_roms,
            requires_chd,
            chd_name,
            verification_status: None,
        })
    }

    fn category_for_game(
        &self,
        game_name: &str,
        parent_name: Option<&str>,
        xml_category: Option<&str>,
        description: &str,
    ) -> String {
        if let Some(ref loader) = self.category_loader
            && let Some(category) = loader.get_category_with_parent(game_name, parent_name)
        {
            return category.to_string();
        }

        if let Some(category) = xml_category {
            return category.to_string();
        }

        Self::guess_category_from_description(description).unwrap_or_else(|| "Misc.".to_string())
    }

    fn guess_category_from_description(description: &str) -> Option<String> {
        let description = description.to_lowercase();

        if description.contains("poker") || description.contains("slot") {
            Some("Casino".to_string())
        } else if description.contains("mahjong") {
            Some("Mahjong".to_string())
        } else if description.contains("quiz") {
            Some("Quiz".to_string())
        } else if description.contains("puzzle") {
            Some("Puzzle".to_string())
        } else if description.contains("fighter") || description.contains("boxing") {
            Some("Fighter".to_string())
        } else if description.contains("shoot") || description.contains("gun") {
            Some("Shooter".to_string())
        } else if description.contains("drive") || description.contains("racing") {
            Some("Driving".to_string())
        } else {
            None
        }
    }
    /// Detect ROM set type based on collection analysis
    pub fn detect_rom_set_type(&self, games: &[Game]) -> RomSetType {
        if games.is_empty() {
            return RomSetType::Unknown;
        }

        // Count total games vs parent games
        let total_games = games.len();
        let parent_games = games.iter().filter(|g| !g.is_clone).count();
        let clone_games = total_games - parent_games;

        // Calculate clone ratio
        let clone_ratio = if total_games > 0 {
            clone_games as f64 / total_games as f64
        } else {
            0.0
        };

        // Analyze clone patterns to determine ROM set type
        let mut clone_groups = std::collections::HashMap::new();

        for game in games {
            if game.is_clone
                && let Some(parent) = &game.parent
            {
                clone_groups
                    .entry(parent.clone())
                    .or_insert_with(Vec::new)
                    .push(game.name.clone());
            }
        }

        // Count average clones per parent
        let avg_clones_per_parent = if !clone_groups.is_empty() {
            clone_groups
                .values()
                .map(|clones| clones.len())
                .sum::<usize>() as f64
                / clone_groups.len() as f64
        } else {
            0.0
        };

        // Detection logic based on RomVault documentation and clone patterns
        if clone_ratio > 0.4 && avg_clones_per_parent > 2.0 {
            // High clone ratio with many clones per parent = Non-Merged
            RomSetType::NonMerged
        } else if clone_ratio > 0.2 && avg_clones_per_parent > 1.0 {
            // Moderate clone ratio = Split
            RomSetType::Split
        } else if clone_ratio < 0.1 {
            // Low clone ratio = likely Merged
            RomSetType::Merged
        } else {
            // Default to Split if uncertain
            RomSetType::Split
        }
    }
}

/// Scanner alternatif yang menggunakan streaming untuk instalasi MAME sangat besar
pub struct StreamingGameScanner {
    mame_path: String,
    category_loader: Option<CategoryLoader>,
}

impl StreamingGameScanner {
    pub fn new(mame_path: &str) -> Self {
        Self {
            mame_path: mame_path.to_string(),
            category_loader: None,
        }
    }

    /// Set the category loader for this scanner
    pub fn with_category_loader(mut self, loader: CategoryLoader) -> Self {
        self.category_loader = Some(loader);
        self
    }

    /// Scan games menggunakan pendekatan streaming untuk minimize memory usage
    /// Ini lebih baik untuk sistem dengan RAM terbatas atau MAME set sangat besar
    pub fn scan_games_streaming(&self) -> Result<Vec<Game>> {
        let mut machines = Vec::new();

        // Start MAME process dengan piped output
        let mut child = Command::new(&self.mame_path)
            .arg("-listxml")
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to start MAME")?;

        // Baca output line by line
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut current_machine = String::new();
            let mut in_machine = false;

            for line in reader.lines() {
                let line = line.context("Failed to read MAME output")?;

                if line.contains("<machine ") {
                    in_machine = true;
                    current_machine.clear();
                }

                if in_machine {
                    current_machine.push_str(&line);
                    current_machine.push('\n');
                }

                if line.contains("</machine>") && in_machine {
                    in_machine = false;

                    // Resolve dependencies only after the complete listing has
                    // arrived; devices and BIOS entries can occur later.
                    match GameScanner::parse_machines(&current_machine) {
                        Ok(mut parsed_machines) => machines.append(&mut parsed_machines),
                        Err(err) => eprintln!("Failed to parse machine XML entry: {err}"),
                    }
                }
            }
        }

        // Wait for process to complete
        let status = child.wait().context("Failed to wait for MAME")?;
        if !status.success() {
            return Err(anyhow::anyhow!("MAME exited with error"));
        }

        let mut scanner = GameScanner::new(&self.mame_path);
        scanner.category_loader = self.category_loader.clone();
        Ok(scanner.games_from_machines(machines))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn required_by_name(xml: &str) -> HashMap<String, bool> {
        GameScanner::new("mame")
            .parse_xml(xml)
            .unwrap()
            .into_iter()
            .map(|game| (game.name, game.requires_roms))
            .collect()
    }

    #[test]
    fn media_requirements_distinguish_dumped_required_and_optional_media() {
        let required = required_by_name(
            r#"
            <mame>
                <machine name="no_media"/>
                <machine name="rom"><rom name="program.bin"/></machine>
                <machine name="disk"><disk name="diskimage"/></machine>
                <machine name="optional_rom"><rom name="bonus.bin" optional="yes"/></machine>
                <machine name="optional_disk"><disk name="bonus" optional="yes"/></machine>
                <machine name="nodump_rom"><rom name="unknown.bin" status="nodump" optional="no"/></machine>
                <machine name="nodump_disk"><disk name="unknown" status="nodump" optional="no"/></machine>
                <machine name="baddump"><rom name="best.bin" status="baddump"/></machine>
                <machine name="media_slot"><device type="harddisk" mandatory="1"><extension name="chd"/></device></machine>
            </mame>
        "#,
        );
        assert_eq!(
            required.len(),
            9,
            "The scanner must retain media-free machines"
        );
        for name in ["rom", "disk", "baddump"] {
            assert!(required[name], "{name}");
        }
        for name in [
            "no_media",
            "optional_rom",
            "optional_disk",
            "nodump_rom",
            "nodump_disk",
            "media_slot",
        ] {
            assert!(!required[name], "{name}");
        }
    }

    #[test]
    fn media_requirements_resolve_bios_and_transitive_device_forward_references() {
        let required = required_by_name(
            r#"
            <mame>
                <machine name="bios_user" romof="bios"/>
                <machine name="device_user"><device_ref name="board"/></machine>
                <machine name="empty_device_user"><device_ref name="empty_device"/></machine>
                <machine name="empty_bios_user" romof="empty_device"/>
                <machine name="clone_only" cloneof="bios"/>
                <machine name="board" isdevice="yes"><device_ref name="firmware"/></machine>
                <machine name="empty_device" isdevice="yes"/>
                <machine name="firmware" isdevice="yes"><rom name="firmware.bin"/></machine>
                <machine name="bios" isbios="yes"><rom name="boot.bin"/></machine>
            </mame>
        "#,
        );
        for name in ["bios_user", "device_user", "board", "firmware", "bios"] {
            assert!(required[name], "{name}");
        }
        for name in [
            "empty_device_user",
            "empty_bios_user",
            "empty_device",
            "clone_only",
        ] {
            assert!(!required[name], "{name}");
        }
    }

    #[test]
    fn media_requirements_handle_cycles_and_unknown_references_conservatively() {
        let required = required_by_name(
            r#"
            <mame>
                <machine name="empty_a"><device_ref name="empty_b"/></machine>
                <machine name="empty_b"><device_ref name="empty_a"/></machine>
                <machine name="required_a"><device_ref name="required_b"/></machine>
                <machine name="required_b"><device_ref name="required_a"/><disk name="disk"/></machine>
                <machine name="unknown_bios" romof="not_in_listing"/>
                <machine name="unknown_device"><device_ref name="not_in_listing"/></machine>
                <machine name="unknown_dependent"><device_ref name="unknown_device"/></machine>
                <machine name="self_cycle" romof="self_cycle"/>
            </mame>
        "#,
        );
        for name in ["empty_a", "empty_b", "self_cycle"] {
            assert!(!required[name], "{name}");
        }
        for name in [
            "required_a",
            "required_b",
            "unknown_bios",
            "unknown_device",
            "unknown_dependent",
        ] {
            assert!(required[name], "{name}");
        }
    }

    #[test]
    #[ignore = "set MAMEUIX_TEST_MAME to an installed MAME executable"]
    fn native_mame_required_roms_classification() {
        let executable = std::env::var("MAMEUIX_TEST_MAME").expect("set MAMEUIX_TEST_MAME");
        let output = Command::new(&executable)
            .args(["-noreadconfig", "-listxml"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let xml = String::from_utf8(output.stdout).unwrap();
        let games = GameScanner::new(&executable).parse_xml(&xml).unwrap();
        let required_count = games.iter().filter(|game| game.requires_roms).count();
        eprintln!(
            "Native MAME metadata: {} machines, {} require ROMs, {} require no ROMs",
            games.len(),
            required_count,
            games.len() - required_count
        );
        for (name, expected) in [
            ("pong", false),
            ("a2600", false),
            ("pacman", true),
            ("area51", true),
            ("alto2", true),
            ("fds", true),
            ("h19", true),
            ("h8", true),
        ] {
            let game = games.iter().find(|game| game.name == name).unwrap();
            assert_eq!(game.requires_roms, expected, "{name}");
        }
        assert!(games.iter().any(|game| !game.requires_roms));
    }

    #[test]
    fn rom_only_games_and_optional_media_do_not_require_chds() {
        let xml = r#"
            <mame>
                <machine name="crusnusa" sourcefile="williams/midvunit.cpp">
                    <description>Cruis'n USA</description>
                    <rom name="program.bin" size="1024"/>
                </machine>
                <machine name="mk4" sourcefile="williams/midzeus.cpp">
                    <description>Mortal Kombat 4</description>
                    <rom name="program.bin" size="1024"/>
                </machine>
                <machine name="computer">
                    <device type="harddisk" mandatory="0">
                        <extension name="chd"/>
                    </device>
                </machine>
                <machine name="optional">
                    <disk name="bonus" optional="yes"/>
                </machine>
                <machine name="begas">
                    <disk name="begas" status="nodump" optional="no"/>
                </machine>
            </mame>
        "#;
        let games = GameScanner::new("mame").parse_xml(xml).unwrap();
        assert_eq!(games.len(), 5);
        for game in games {
            assert!(!game.requires_chd, "{}", game.name);
            assert_eq!(game.chd_name, None, "{}", game.name);
        }
    }

    #[test]
    fn required_disk_after_optional_disk_keeps_its_xml_name() {
        let xml = r#"
            <mame><machine name="diskgame">
                <disk name="bonus" optional="yes"/>
                <disk name="unavailable" status="nodump" optional="no"/>
                <disk name="actual_disk" optional="no"/>
            </machine></mame>
        "#;
        let games = GameScanner::new("mame").parse_xml(xml).unwrap();
        assert!(games[0].requires_chd);
        assert_eq!(games[0].chd_name.as_deref(), Some("actual_disk"));
    }

    #[test]
    #[ignore = "set MAMEUIX_TEST_MAME to an installed MAME executable"]
    fn native_mame_nodump_disk_is_not_reported_as_missing_media() {
        let executable = std::env::var("MAMEUIX_TEST_MAME").expect("set MAMEUIX_TEST_MAME");
        let output = Command::new(&executable)
            .args(["-noreadconfig", "-listxml", "begas"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let xml = String::from_utf8(output.stdout).unwrap();
        let games = GameScanner::new(&executable).parse_xml(&xml).unwrap();
        let begas = games.iter().find(|game| game.name == "begas").unwrap();
        assert!(!begas.requires_chd);
        assert!(begas.chd_name.is_none());
    }

    #[test]
    fn parses_machine_xml_with_entities_clone_and_chd() {
        let xml = r#"
            <mame>
                <machine name="sf2ce" cloneof="sf2" sourcefile="cps1.cpp">
                    <description>Street Fighter II&apos;: Champion Edition &amp; Extra</description>
                    <year>1992</year>
                    <manufacturer>Capcom &amp; Co.</manufacturer>
                    <driver status="good" name="cps1"/>
                    <disk name="sf2ce_disk" sha1="abc"/>
                </machine>
            </mame>
        "#;

        let scanner = GameScanner::new("mame");
        let games = scanner.parse_xml(xml).unwrap();

        assert_eq!(games.len(), 1);
        let game = &games[0];
        assert_eq!(game.name, "sf2ce");
        assert_eq!(
            game.description,
            "Street Fighter II': Champion Edition & Extra"
        );
        assert_eq!(game.manufacturer, "Capcom & Co.");
        assert_eq!(game.parent.as_deref(), Some("sf2"));
        assert_eq!(game.driver, "cps1");
        assert_eq!(game.driver_status, "good");
        assert!(game.is_clone);
        assert!(game.requires_chd);
        assert_eq!(game.chd_name.as_deref(), Some("sf2ce_disk"));
    }
}
