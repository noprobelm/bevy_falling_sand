//! BFC format (Binary Format with Color) for particle scenes with color data.
//!
//! ## Format specification
//! - Magic: "BFC\0" (4 bytes)
//! - Particle type count: u16
//! - For each particle type:
//!   - Name length: u8
//!   - Name: [u8; `name_len`]
//!   - Entry count: u32
//!   - Entries: (x: i32, y: i32, `color_type`: u8, `color_data`: variable)
//!     - `color_type` 0: `ColorIndex` - `color_data` is u16 (2 bytes)
//!     - `color_type` 1: `ForceColor` - `color_data` is 4 x f32 RGBA (16 bytes)

use super::io_reader;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use std::io::{Read, Write};

/// Magic bytes for BFC format (with color).
pub const MAGIC: &[u8; 4] = b"BFC\0";

/// Parsed particle data from a BFC file.
///
/// This is the common structure returned by BFC deserialization functions.
/// Other modules can convert this to their own types as needed.
#[derive(Debug, Clone)]
pub struct ParticleData {
    /// Particle type name.
    pub name: String,
    /// World position of the particle.
    pub position: IVec2,
    /// Color index if using palette.
    pub color_index: Option<usize>,
    /// Forced color if overriding palette (RGBA).
    pub force_color: Option<[f32; 4]>,
}

/// Serialize BFC format to a writer.
///
/// Writes particle data to any `std::io::Write` implementation.
/// This is the counterpart to [`deserialize_from_reader`].
///
/// # Errors
///
/// Returns an error if writing to the writer fails.
pub fn serialize_to_writer(
    particles: &[ParticleData],
    writer: &mut impl Write,
) -> std::io::Result<()> {
    // Group particles by type
    let mut grouped: HashMap<&str, Vec<&ParticleData>> = HashMap::default();
    for particle in particles {
        grouped.entry(&particle.name).or_default().push(particle);
    }

    // Magic header
    writer.write_all(MAGIC)?;

    // Particle type count
    #[allow(clippy::cast_possible_truncation)]
    writer.write_all(&(grouped.len() as u16).to_le_bytes())?;

    for (name, entries) in &grouped {
        // Name length and name bytes
        writer.write_all(&[name.len() as u8])?;
        writer.write_all(name.as_bytes())?;

        // Entry count
        writer.write_all(&(entries.len() as u32).to_le_bytes())?;

        // Write each particle with position and color
        for particle in entries {
            writer.write_all(&particle.position.x.to_le_bytes())?;
            writer.write_all(&particle.position.y.to_le_bytes())?;

            // Write color data
            if let Some([r, g, b, a]) = particle.force_color {
                writer.write_all(&[1u8])?;
                writer.write_all(&r.to_le_bytes())?;
                writer.write_all(&g.to_le_bytes())?;
                writer.write_all(&b.to_le_bytes())?;
                writer.write_all(&a.to_le_bytes())?;
            } else {
                writer.write_all(&[0u8])?;
                let idx = particle.color_index.unwrap_or(0) as u16;
                writer.write_all(&idx.to_le_bytes())?;
            }
        }
    }

    Ok(())
}

/// Serialize BFC format to a byte vector.
///
/// Convenience wrapper around [`serialize_to_writer`].
///
/// # Panics
///
/// Panics if writing bytes to the writer fails.
#[must_use]
pub fn serialize_to_bytes(particles: &[ParticleData]) -> Vec<u8> {
    let mut bytes = Vec::new();
    serialize_to_writer(particles, &mut bytes).expect("Writing to Vec should not fail");
    bytes
}

/// Deserialize BFC format from a byte slice.
///
/// Returns a vector of particle data parsed from the BFC file.
/// This is a convenience wrapper around [`deserialize_from_reader`].
///
/// # Errors
///
/// Returns an error if the magic header is missing or the data is malformed.
pub fn deserialize_from_slice(bytes: &[u8]) -> Result<Vec<ParticleData>, String> {
    deserialize_from_reader(&mut std::io::Cursor::new(bytes))
}

/// Deserialize BFC format from a reader.
///
/// Returns a vector of particle data parsed from the BFC file.
///
/// # Errors
///
/// Returns an error if the magic header is missing or the data is malformed.
#[allow(clippy::many_single_char_names)]
pub fn deserialize_from_reader(reader: &mut impl Read) -> Result<Vec<ParticleData>, String> {
    // Check magic header
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .map_err(|e| format!("Failed to read magic: {e}"))?;

    if &magic != MAGIC {
        return Err(format!("Invalid magic header: expected BFC, got {magic:?}"));
    }

    // Read particle type count
    let mut count_buf = [0u8; 2];
    reader
        .read_exact(&mut count_buf)
        .map_err(|e| format!("Failed to read type count: {e}"))?;
    let type_count = u16::from_le_bytes(count_buf);

    let mut particles = Vec::new();

    for _ in 0..type_count {
        // Read name
        let mut name_len = [0u8; 1];
        reader
            .read_exact(&mut name_len)
            .map_err(|e| format!("Failed to read name length: {e}"))?;

        let mut name_buf = vec![0u8; name_len[0] as usize];
        reader
            .read_exact(&mut name_buf)
            .map_err(|e| format!("Failed to read name: {e}"))?;

        let name = String::from_utf8(name_buf).map_err(|e| format!("Invalid UTF-8 name: {e}"))?;

        // Read entry count
        let mut entry_count_buf = [0u8; 4];
        reader
            .read_exact(&mut entry_count_buf)
            .map_err(|e| format!("Failed to read entry count: {e}"))?;
        let entry_count = u32::from_le_bytes(entry_count_buf);

        // Read each particle
        for _ in 0..entry_count {
            let x = io_reader::read_i32(reader).map_err(|e| format!("Failed to read x: {e}"))?;
            let y = io_reader::read_i32(reader).map_err(|e| format!("Failed to read y: {e}"))?;

            let mut color_type = [0u8; 1];
            reader
                .read_exact(&mut color_type)
                .map_err(|e| format!("Failed to read color type: {e}"))?;

            let (color_index, force_color) = match color_type[0] {
                0 => {
                    // ColorIndex
                    let idx = io_reader::read_u16(reader)
                        .map_err(|e| format!("Failed to read color index: {e}"))?;
                    (Some(idx as usize), None)
                }
                1 => {
                    // ForceColor (RGBA as f32)
                    let r = io_reader::read_f32(reader)
                        .map_err(|e| format!("Failed to read r: {e}"))?;
                    let g = io_reader::read_f32(reader)
                        .map_err(|e| format!("Failed to read g: {e}"))?;
                    let b = io_reader::read_f32(reader)
                        .map_err(|e| format!("Failed to read b: {e}"))?;
                    let a = io_reader::read_f32(reader)
                        .map_err(|e| format!("Failed to read a: {e}"))?;
                    (None, Some([r, g, b, a]))
                }
                t => return Err(format!("Unknown color type: {t}")),
            };

            particles.push(ParticleData {
                name: name.clone(),
                position: IVec2::new(x, y),
                color_index,
                force_color,
            });
        }
    }

    Ok(particles)
}
