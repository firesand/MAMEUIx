//! Public Sans font roles used by the redesign shell.
//!
//! egui does not select a different face for `RichText::strong()`, so each
//! design weight is registered as its own named family.

use eframe::egui;
use std::sync::{Arc, LazyLock};

const DATA_REGULAR: &str = "public-sans-regular-data";
const DATA_MEDIUM: &str = "public-sans-medium-data";
const DATA_SEMIBOLD: &str = "public-sans-semibold-data";
const DATA_BOLD: &str = "public-sans-bold-data";

const FAMILY_REGULAR: &str = "Public Sans 400";
const FAMILY_MEDIUM: &str = "Public Sans 500";
const FAMILY_SEMIBOLD: &str = "Public Sans 600";
const FAMILY_BOLD: &str = "Public Sans 700";

static FAMILY_REGULAR_ID: LazyLock<egui::FontFamily> =
    LazyLock::new(|| egui::FontFamily::Name(FAMILY_REGULAR.into()));
static FAMILY_MEDIUM_ID: LazyLock<egui::FontFamily> =
    LazyLock::new(|| egui::FontFamily::Name(FAMILY_MEDIUM.into()));
static FAMILY_SEMIBOLD_ID: LazyLock<egui::FontFamily> =
    LazyLock::new(|| egui::FontFamily::Name(FAMILY_SEMIBOLD.into()));
static FAMILY_BOLD_ID: LazyLock<egui::FontFamily> =
    LazyLock::new(|| egui::FontFamily::Name(FAMILY_BOLD.into()));

const REGULAR_BYTES: &[u8] =
    include_bytes!("../../../assets/fonts/public_sans/PublicSans-Regular.ttf");
const MEDIUM_BYTES: &[u8] =
    include_bytes!("../../../assets/fonts/public_sans/PublicSans-Medium.ttf");
const SEMIBOLD_BYTES: &[u8] =
    include_bytes!("../../../assets/fonts/public_sans/PublicSans-SemiBold.ttf");
const BOLD_BYTES: &[u8] = include_bytes!("../../../assets/fonts/public_sans/PublicSans-Bold.ttf");

pub fn install(ctx: &egui::Context) {
    ctx.set_fonts(definitions());
}

pub fn regular(size: f32) -> egui::FontId {
    egui::FontId::new(size, FAMILY_REGULAR_ID.clone())
}

pub fn medium(size: f32) -> egui::FontId {
    egui::FontId::new(size, FAMILY_MEDIUM_ID.clone())
}

pub fn semibold(size: f32) -> egui::FontId {
    egui::FontId::new(size, FAMILY_SEMIBOLD_ID.clone())
}

pub fn bold(size: f32) -> egui::FontId {
    egui::FontId::new(size, FAMILY_BOLD_ID.clone())
}

fn named_family(name: &'static str) -> egui::FontFamily {
    egui::FontFamily::Name(name.into())
}

fn definitions() -> egui::FontDefinitions {
    let mut definitions = egui::FontDefinitions::default();
    let mut glyph_fallbacks = definitions
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for font_name in definitions
        .families
        .get(&egui::FontFamily::Monospace)
        .into_iter()
        .flatten()
    {
        if !glyph_fallbacks.contains(font_name) {
            glyph_fallbacks.push(font_name.clone());
        }
    }

    register_face(
        &mut definitions,
        DATA_REGULAR,
        FAMILY_REGULAR,
        REGULAR_BYTES,
        &glyph_fallbacks,
    );
    register_face(
        &mut definitions,
        DATA_MEDIUM,
        FAMILY_MEDIUM,
        MEDIUM_BYTES,
        &glyph_fallbacks,
    );
    register_face(
        &mut definitions,
        DATA_SEMIBOLD,
        FAMILY_SEMIBOLD,
        SEMIBOLD_BYTES,
        &glyph_fallbacks,
    );
    register_face(
        &mut definitions,
        DATA_BOLD,
        FAMILY_BOLD,
        BOLD_BYTES,
        &glyph_fallbacks,
    );

    definitions
}

fn register_face(
    definitions: &mut egui::FontDefinitions,
    data_name: &str,
    family_name: &'static str,
    bytes: &'static [u8],
    fallbacks: &[String],
) {
    definitions.font_data.insert(
        data_name.to_string(),
        Arc::new(egui::FontData::from_static(bytes)),
    );

    let mut family_fonts = Vec::with_capacity(fallbacks.len() + 1);
    family_fonts.push(data_name.to_string());
    family_fonts.extend(fallbacks.iter().cloned());
    definitions
        .families
        .insert(named_family(family_name), family_fonts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_public_sans_weight_has_emoji_fallbacks() {
        let definitions = definitions();

        for (family_name, data_name) in [
            (FAMILY_REGULAR, DATA_REGULAR),
            (FAMILY_MEDIUM, DATA_MEDIUM),
            (FAMILY_SEMIBOLD, DATA_SEMIBOLD),
            (FAMILY_BOLD, DATA_BOLD),
        ] {
            assert!(definitions.font_data.contains_key(data_name));
            let family = definitions
                .families
                .get(&named_family(family_name))
                .expect("named Public Sans family");
            assert_eq!(family.first().map(String::as_str), Some(data_name));
            assert!(family.len() > 1, "UI symbols need fallback glyph fonts");
            assert!(
                family.iter().any(|font| font == "Hack"),
                "arrows and chevrons need the monospace fallback"
            );
        }
    }
}
