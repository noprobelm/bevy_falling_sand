//! BFS format (Binary Format without color) for compressed particle scenes.
//!
//! ## Format specification
//! - Magic: "BFS\0" (4 bytes)
//! - Particle type count: u16
//! - For each particle type:
//!   - Name length: u8
//!   - Name: [u8; `name_len`]
//!   - Entry count: u32
//!   - Entries (type byte + i32 coordinates):
//!     - 0: Point (x, y) = 9 bytes
//!     - 1: `HorizontalRun` (x1, x2, y) = 13 bytes
//!     - 2: `VerticalRun` (x, y1, y2) = 13 bytes
//!     - 3: Rectangle (x1, x2, y1, y2) = 17 bytes

use super::io_reader;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use std::io::{Read, Write};

/// Magic bytes for BFS format (without color).
pub const MAGIC: &[u8; 4] = b"BFS\0";

/// Represents a compressed entry for BFS format.
pub enum CompressedEntry {
    /// Single point.
    Point(IVec2),
    /// Horizontal run of points.
    HorizontalRun {
        /// Start x coordinate.
        x1: i32,
        /// End x coordinate.
        x2: i32,
        /// Y coordinate.
        y: i32,
    },
    /// Vertical run of points.
    VerticalRun {
        /// X coordinate.
        x: i32,
        /// Start y coordinate.
        y1: i32,
        /// End y coordinate.
        y2: i32,
    },
    /// Rectangle of points.
    Rectangle(IRect),
}

impl CompressedEntry {
    /// Write the compressed entry to a writer.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the underlying writer fails.
    pub fn write(&self, w: &mut impl Write) -> std::io::Result<()> {
        match self {
            Self::Point(p) => {
                w.write_all(&[0u8])?;
                w.write_all(&p.x.to_le_bytes())?;
                w.write_all(&p.y.to_le_bytes())?;
            }
            Self::HorizontalRun { x1, x2, y } => {
                w.write_all(&[1u8])?;
                w.write_all(&x1.to_le_bytes())?;
                w.write_all(&x2.to_le_bytes())?;
                w.write_all(&y.to_le_bytes())?;
            }
            Self::VerticalRun { x, y1, y2 } => {
                w.write_all(&[2u8])?;
                w.write_all(&x.to_le_bytes())?;
                w.write_all(&y1.to_le_bytes())?;
                w.write_all(&y2.to_le_bytes())?;
            }
            Self::Rectangle(rect) => {
                w.write_all(&[3u8])?;
                w.write_all(&rect.min.x.to_le_bytes())?;
                w.write_all(&rect.max.x.to_le_bytes())?;
                w.write_all(&rect.min.y.to_le_bytes())?;
                w.write_all(&rect.max.y.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

/// Compress a list of positions into run-length encoded entries.
#[must_use]
pub fn compress_positions(positions: &[IVec2]) -> Vec<CompressedEntry> {
    let mut remaining: HashSet<IVec2> = positions.iter().copied().collect();
    let mut entries = Vec::new();

    // First pass: find rectangles
    let mut sorted: Vec<IVec2> = remaining.iter().copied().collect();
    sorted.sort_by_key(|p| (p.x, p.y));

    for start in &sorted {
        if !remaining.contains(start) {
            continue;
        }

        let mut end_x = start.x;
        let mut end_y = start.y;

        while remaining.contains(&IVec2::new(end_x + 1, start.y)) {
            end_x += 1;
        }

        'expand_down: loop {
            let next_y = end_y + 1;
            for x in start.x..=end_x {
                if !remaining.contains(&IVec2::new(x, next_y)) {
                    break 'expand_down;
                }
            }
            end_y = next_y;
        }

        if end_x > start.x && end_y > start.y {
            for x in start.x..=end_x {
                for y in start.y..=end_y {
                    remaining.remove(&IVec2::new(x, y));
                }
            }
            entries.push(CompressedEntry::Rectangle(IRect::new(
                start.x, start.y, end_x, end_y,
            )));
        }
    }

    // Second pass: horizontal runs
    let mut by_y: HashMap<i32, Vec<i32>> = HashMap::default();
    for pos in &remaining {
        by_y.entry(pos.y).or_default().push(pos.x);
    }

    for (y, mut xs) in by_y {
        xs.sort_unstable();
        let mut i = 0;
        while i < xs.len() {
            let start_x = xs[i];
            let mut end_x = start_x;

            while i + 1 < xs.len() && xs[i + 1] == end_x + 1 {
                i += 1;
                end_x = xs[i];
            }

            if end_x > start_x {
                for x in start_x..=end_x {
                    remaining.remove(&IVec2::new(x, y));
                }
                entries.push(CompressedEntry::HorizontalRun {
                    x1: start_x,
                    x2: end_x,
                    y,
                });
            }

            i += 1;
        }
    }

    // Third pass: vertical runs
    let mut by_x: HashMap<i32, Vec<i32>> = HashMap::default();
    for pos in &remaining {
        by_x.entry(pos.x).or_default().push(pos.y);
    }

    for (x, mut ys) in by_x {
        ys.sort_unstable();
        let mut i = 0;
        while i < ys.len() {
            let start_y = ys[i];
            let mut end_y = start_y;

            while i + 1 < ys.len() && ys[i + 1] == end_y + 1 {
                i += 1;
                end_y = ys[i];
            }

            if end_y > start_y {
                for y in start_y..=end_y {
                    remaining.remove(&IVec2::new(x, y));
                }
                entries.push(CompressedEntry::VerticalRun {
                    x,
                    y1: start_y,
                    y2: end_y,
                });
            }

            i += 1;
        }
    }

    // Fourth pass: remaining points
    for pos in remaining {
        entries.push(CompressedEntry::Point(pos));
    }

    entries
}

/// Parsed particle data from a BFS file.
///
/// This is the common structure returned by BFS deserialization functions.
/// Unlike BFC, BFS does not store color information.
#[derive(Debug, Clone)]
pub struct ParticleData {
    /// Particle type name.
    pub name: String,
    /// World position of the particle.
    pub position: IVec2,
}

/// Serialize BFS format to a writer.
///
/// Groups particles by name and writes them using run-length compression.
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
    let mut grouped: HashMap<&str, Vec<IVec2>> = HashMap::default();
    for particle in particles {
        grouped
            .entry(&particle.name)
            .or_default()
            .push(particle.position);
    }

    // Magic header
    writer.write_all(MAGIC)?;

    // Particle type count
    #[allow(clippy::cast_possible_truncation)]
    writer.write_all(&(grouped.len() as u16).to_le_bytes())?;

    for (name, positions) in &grouped {
        // Name length and name bytes
        #[allow(clippy::cast_possible_truncation)]
        writer.write_all(&[name.len() as u8])?;
        writer.write_all(name.as_bytes())?;

        // Compress and write entries
        let entries = compress_positions(positions);
        #[allow(clippy::cast_possible_truncation)]
        writer.write_all(&(entries.len() as u32).to_le_bytes())?;

        for entry in &entries {
            entry.write(writer)?;
        }
    }

    Ok(())
}

/// Serialize BFS format to a byte vector.
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

/// Deserialize BFS format from a byte slice.
///
/// Returns a vector of particle data parsed from the BFS file.
/// This is a convenience wrapper around [`deserialize_from_reader`].
///
/// # Errors
///
/// Returns an error if the magic header is missing or the data is malformed.
pub fn deserialize_from_slice(bytes: &[u8]) -> Result<Vec<ParticleData>, String> {
    deserialize_from_reader(&mut std::io::Cursor::new(bytes))
}

/// Deserialize BFS format from a reader.
///
/// Returns a vector of particle data parsed from the BFS file.
/// Compressed entries (runs, rectangles) are expanded into individual positions.
///
/// # Errors
///
/// Returns an error if the magic header is missing or the data is malformed.
pub fn deserialize_from_reader(reader: &mut impl Read) -> Result<Vec<ParticleData>, String> {
    // Check magic header
    let mut magic = [0u8; 4];
    reader
        .read_exact(&mut magic)
        .map_err(|e| format!("Failed to read magic: {e}"))?;

    if &magic != MAGIC {
        return Err(format!("Invalid magic header: expected BFS, got {magic:?}"));
    }

    // Read particle type count
    let type_count =
        io_reader::read_u16(reader).map_err(|e| format!("Failed to read type count: {e}"))?;

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

        // Read and expand each entry
        for _ in 0..entry_count {
            let mut entry_type = [0u8; 1];
            reader
                .read_exact(&mut entry_type)
                .map_err(|e| format!("Failed to read entry type: {e}"))?;

            match entry_type[0] {
                0 => {
                    // Point
                    let x = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read point x: {e}"))?;
                    let y = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read point y: {e}"))?;
                    particles.push(ParticleData {
                        name: name.clone(),
                        position: IVec2::new(x, y),
                    });
                }
                1 => {
                    // Horizontal run
                    let x1 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read hrun x1: {e}"))?;
                    let x2 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read hrun x2: {e}"))?;
                    let y = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read hrun y: {e}"))?;
                    for x in x1..=x2 {
                        particles.push(ParticleData {
                            name: name.clone(),
                            position: IVec2::new(x, y),
                        });
                    }
                }
                2 => {
                    // Vertical run
                    let x = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read vrun x: {e}"))?;
                    let y1 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read vrun y1: {e}"))?;
                    let y2 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read vrun y2: {e}"))?;
                    for y in y1..=y2 {
                        particles.push(ParticleData {
                            name: name.clone(),
                            position: IVec2::new(x, y),
                        });
                    }
                }
                3 => {
                    // Rectangle
                    let x1 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read rect x1: {e}"))?;
                    let x2 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read rect x2: {e}"))?;
                    let y1 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read rect y1: {e}"))?;
                    let y2 = io_reader::read_i32(reader)
                        .map_err(|e| format!("Failed to read rect y2: {e}"))?;
                    for x in x1..=x2 {
                        for y in y1..=y2 {
                            particles.push(ParticleData {
                                name: name.clone(),
                                position: IVec2::new(x, y),
                            });
                        }
                    }
                }
                t => return Err(format!("Unknown entry type: {t}")),
            }
        }
    }

    Ok(particles)
}
