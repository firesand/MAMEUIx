use anyhow::{Context, Result};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct SoftwareListSummary {
    pub name: String,
    pub description: String,
    pub source_file: PathBuf,
    pub software_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SoftwareEntry {
    pub list_name: String,
    pub list_description: String,
    pub name: String,
    pub description: String,
    pub year: String,
    pub publisher: String,
    pub supported: String,
    pub clone_of: Option<String>,
    pub parent: Option<String>,
    pub part_count: usize,
    pub interfaces: Vec<String>,
    pub source_file: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct SoftwareListLoadResult {
    pub lists: Vec<SoftwareListSummary>,
    pub entries: Vec<SoftwareEntry>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct CurrentList {
    name: String,
    description: String,
    source_file: PathBuf,
    software_count: usize,
}

pub struct SoftwareListLoader;

impl SoftwareListLoader {
    pub fn load_from_hash_path(path: &Path) -> Result<SoftwareListLoadResult> {
        Self::load_from_hash_path_with_progress(path, |_, _| {})
    }

    /// Load MAME hash XML while reporting completed and total file counts.
    ///
    /// The callback is intentionally lightweight so callers can forward progress
    /// to a UI thread without coupling the parser to a particular runtime.
    pub fn load_from_hash_path_with_progress<F>(
        path: &Path,
        mut on_progress: F,
    ) -> Result<SoftwareListLoadResult>
    where
        F: FnMut(usize, usize),
    {
        if path.is_file() {
            on_progress(0, 1);
            let mut result = SoftwareListLoadResult::default();
            Self::load_file_into(path, &mut result);
            on_progress(1, 1);
            return Ok(result);
        }

        if !path.is_dir() {
            return Err(anyhow::anyhow!(
                "Software-list path is not a directory or XML file: {}",
                path.display()
            ));
        }

        let mut files: Vec<PathBuf> = fs::read_dir(path)
            .with_context(|| format!("Failed to read software-list directory {}", path.display()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
            })
            .collect();
        files.sort();

        if let Some(first_file) = files.first()
            && Self::file_looks_like_rom_manager_dat(first_file).unwrap_or(false)
        {
            return Err(anyhow::anyhow!(
                "Selected directory contains ROM-manager DAT XML files, not MAME software-list hash XML files: {}",
                path.display()
            ));
        }

        let mut result = SoftwareListLoadResult::default();
        let total_files = files.len();
        on_progress(0, total_files);
        for (index, file) in files.into_iter().enumerate() {
            Self::load_file_into(&file, &mut result);
            on_progress(index + 1, total_files);
        }

        result.lists.sort_by(|a, b| a.name.cmp(&b.name));
        result.entries.sort_by(|a, b| {
            a.list_name
                .cmp(&b.list_name)
                .then_with(|| a.description.cmp(&b.description))
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(result)
    }

    fn load_file_into(path: &Path, result: &mut SoftwareListLoadResult) {
        match fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))
            .and_then(|xml| Self::parse_xml(path, &xml))
        {
            Ok(mut file_result) => {
                result.lists.append(&mut file_result.lists);
                result.entries.append(&mut file_result.entries);
                result.errors.append(&mut file_result.errors);
            }
            Err(error) => {
                result.errors.push(format!("{}: {}", path.display(), error));
            }
        }
    }

    fn parse_xml(path: &Path, xml: &str) -> Result<SoftwareListLoadResult> {
        if Self::looks_like_rom_manager_dat(xml) {
            return Err(anyhow::anyhow!(
                "{} is a ROM-manager DAT XML file, not a MAME software-list hash XML file",
                path.display()
            ));
        }

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut result = SoftwareListLoadResult::default();
        let mut buf = Vec::new();
        let mut current_list: Option<CurrentList> = None;
        let mut current_entry: Option<SoftwareEntry> = None;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(event) if event.name().as_ref() == b"softwarelist" => {
                    current_list = Some(Self::list_from_start(path, &event, &reader)?);
                }
                Event::Start(event) if event.name().as_ref() == b"software" => {
                    if let Some(list) = &current_list {
                        current_entry = Some(Self::entry_from_start(list, &event, &reader)?);
                    }
                }
                Event::Start(event) => {
                    if let Some(entry) = current_entry.as_mut() {
                        match event.name().as_ref() {
                            b"description" => {
                                entry.description =
                                    Self::read_text_unescaped(&mut reader, event.name())?;
                            }
                            b"year" => {
                                entry.year = Self::read_text_unescaped(&mut reader, event.name())?;
                            }
                            b"publisher" => {
                                entry.publisher =
                                    Self::read_text_unescaped(&mut reader, event.name())?;
                            }
                            b"part" => Self::read_part_attrs(entry, &event, &reader)?,
                            _ => {}
                        }
                    }
                }
                Event::Empty(event) => {
                    if let Some(entry) = current_entry.as_mut()
                        && event.name().as_ref() == b"part"
                    {
                        Self::read_part_attrs(entry, &event, &reader)?;
                    }
                }
                Event::End(event) if event.name().as_ref() == b"software" => {
                    if let Some(entry) = current_entry.take() {
                        if let Some(list) = current_list.as_mut() {
                            list.software_count += 1;
                        }
                        result.entries.push(entry);
                    }
                }
                Event::End(event) if event.name().as_ref() == b"softwarelist" => {
                    if let Some(list) = current_list.take() {
                        result.lists.push(SoftwareListSummary {
                            name: list.name,
                            description: list.description,
                            source_file: list.source_file,
                            software_count: list.software_count,
                        });
                    }
                }
                Event::Eof => break,
                _ => {}
            }

            buf.clear();
        }

        Ok(result)
    }

    fn file_looks_like_rom_manager_dat(path: &Path) -> Result<bool> {
        let xml = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        Ok(Self::looks_like_rom_manager_dat(&xml))
    }

    fn looks_like_rom_manager_dat(xml: &str) -> bool {
        let xml = xml.trim_start_matches('\u{feff}').trim_start();
        xml.contains("<datafile")
            && (xml.contains("Logiqx//DTD ROM Management Datafile")
                || xml.contains("Generated by SabreTools")
                || xml.contains("<machine "))
    }

    fn list_from_start(
        path: &Path,
        event: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
    ) -> Result<CurrentList> {
        let fallback_name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let name = Self::xml_attr(event, b"name", reader)?.unwrap_or(fallback_name);
        let description =
            Self::xml_attr(event, b"description", reader)?.unwrap_or_else(|| name.clone());

        Ok(CurrentList {
            name,
            description,
            source_file: path.to_path_buf(),
            software_count: 0,
        })
    }

    fn entry_from_start(
        list: &CurrentList,
        event: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
    ) -> Result<SoftwareEntry> {
        Ok(SoftwareEntry {
            list_name: list.name.clone(),
            list_description: list.description.clone(),
            name: Self::xml_attr(event, b"name", reader)?.unwrap_or_default(),
            supported: Self::xml_attr(event, b"supported", reader)?
                .unwrap_or_else(|| "yes".to_string()),
            clone_of: Self::xml_attr(event, b"cloneof", reader)?,
            parent: Self::xml_attr(event, b"romof", reader)?,
            source_file: list.source_file.clone(),
            ..Default::default()
        })
    }

    fn read_part_attrs(
        entry: &mut SoftwareEntry,
        event: &BytesStart<'_>,
        reader: &Reader<&[u8]>,
    ) -> Result<()> {
        entry.part_count += 1;
        if let Some(interface) = Self::xml_attr(event, b"interface", reader)?
            && !entry.interfaces.contains(&interface)
        {
            entry.interfaces.push(interface);
        }
        Ok(())
    }

    fn xml_attr(
        event: &BytesStart<'_>,
        key: &[u8],
        reader: &Reader<&[u8]>,
    ) -> Result<Option<String>> {
        for attr in event.attributes() {
            let attr = attr?;
            if attr.key.as_ref() == key {
                return Ok(Some(
                    attr.decode_and_unescape_value(reader.decoder())?
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
        Ok(quick_xml::escape::unescape(&text)?.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::SoftwareListLoader;
    use std::path::Path;

    #[test]
    fn parses_mame_software_list_xml() {
        let xml = r#"
            <softwarelists>
                <softwarelist name="a2600" description="Atari 2600 cartridges">
                    <software name="combat">
                        <description>Combat</description>
                        <year>1977</year>
                        <publisher>Atari</publisher>
                        <part name="cart" interface="a2600_cart">
                            <dataarea name="rom" size="2048">
                                <rom name="combat.bin" size="2048" crc="9f7fdd53"/>
                            </dataarea>
                        </part>
                    </software>
                    <software name="combatb" cloneof="combat" supported="partial">
                        <description>Combat &amp; Bonus</description>
                        <year>1978</year>
                        <publisher>Atari</publisher>
                        <part name="cart" interface="a2600_cart"/>
                    </software>
                </softwarelist>
            </softwarelists>
        "#;

        let result = SoftwareListLoader::parse_xml(Path::new("a2600.xml"), xml).unwrap();

        assert_eq!(result.lists.len(), 1);
        assert_eq!(result.lists[0].name, "a2600");
        assert_eq!(result.lists[0].software_count, 2);
        assert_eq!(result.entries.len(), 2);
        assert_eq!(result.entries[0].description, "Combat");
        assert_eq!(result.entries[1].description, "Combat & Bonus");
        assert_eq!(result.entries[1].clone_of.as_deref(), Some("combat"));
        assert_eq!(result.entries[1].supported, "partial");
    }

    #[test]
    fn rejects_rom_manager_dat_xml() {
        let xml = r#"
            <?xml version="1.0" encoding="utf-8"?>
            <!DOCTYPE datafile PUBLIC "-//Logiqx//DTD ROM Management Datafile//EN" "http://www.logiqx.com/Dats/datafile.dtd">
            <datafile>
                <header>
                    <name>a2600</name>
                    <description>Atari 2600 cartridges</description>
                    <comment>Generated by SabreTools 1.2.1</comment>
                </header>
                <machine name="combat">
                    <description>Combat</description>
                    <year>1977</year>
                    <publisher>Atari</publisher>
                </machine>
            </datafile>
        "#;

        let error = SoftwareListLoader::parse_xml(Path::new("a2600.xml"), xml).unwrap_err();

        assert!(error.to_string().contains("ROM-manager DAT XML"));
    }
}
