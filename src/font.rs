use anyhow::{Context, Result, anyhow};
use fontdb::{Database, Family, Query};
use fontdue::{Font, FontSettings};

/// Finds and loads a system monospace font for terminal rendering.
pub(crate) fn load_monospace_font() -> Result<Font> {
    let mut database = Database::new();
    database.load_system_fonts();

    let preferred_families = [
        Family::Name("DejaVu Sans Mono"),
        Family::Name("Liberation Mono"),
        Family::Monospace,
    ];
    let id = preferred_families
        .iter()
        .find_map(|family| {
            database.query(&Query {
                families: &[*family],
                ..Query::default()
            })
        })
        .ok_or_else(|| anyhow!("no monospace system font found"))?;
    let face = database
        .face(id)
        .ok_or_else(|| anyhow!("font face disappeared"))?;

    match &face.source {
        fontdb::Source::File(path) => {
            let bytes = std::fs::read(path)
                .with_context(|| format!("failed to read font at {}", path.display()))?;
            Font::from_bytes(bytes, FontSettings::default())
                .map_err(|error| anyhow!("failed to load font: {error}"))
        }
        fontdb::Source::Binary(bytes) => {
            Font::from_bytes(bytes.as_ref().as_ref(), FontSettings::default())
                .map_err(|error| anyhow!("failed to load font: {error}"))
        }
        _ => Err(anyhow!("unsupported font source")),
    }
}
