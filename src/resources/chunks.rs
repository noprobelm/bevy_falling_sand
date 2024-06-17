use bevy::prelude::*;

/// An identifier for which chunk we're working with.
#[derive(Debug, Clone)]
pub enum ChunkGroup {
    One,
    Two,
    Three,
    Four
}

#[derive(Resource, Debug, Clone)]
pub struct Chunks {
    pub one: Vec<(IVec2, IVec2)>,
    pub two: Vec<(IVec2, IVec2)>,
    pub third: Vec<(IVec2, IVec2)>,
    pub four: Vec<(IVec2, IVec2)>,
}

/// When the particle simulation is utilizing parallelism, we have to make considerations for
/// particle movement so as to not create race between multiple particles trying to occupy the
/// same location. To mitigate this, we can assign each particle to a 64x64 "chunk", and operate
/// on them in groups.
///
/// Chunks are 64x64 by default. By updating particles over 4 rounds, we can guarantee that any
/// particle can move 32 pixels in any direction without attempting to access a location another
/// area of the ParticleMap is working with in parallel.
///
#[derive(Resource, Debug, Clone)]
pub struct ChunkGroups {
    /// The first group of min/max cooridnates for each chunk
    pub one: Vec<(IVec2, IVec2)>,
    /// The second group of min/max cooridnates for each chunk
    pub two: Vec<(IVec2, IVec2)>,
    /// The third group of min/max cooridnates for each chunk
    pub third: Vec<(IVec2, IVec2)>,
    /// The fourth group of min/max cooridnates for each chunk
    pub four: Vec<(IVec2, IVec2)>,
}

impl Default for ChunkGroups {
    fn default() -> ChunkGroups {
        let mut first: Vec<(IVec2, IVec2)> = Vec::with_capacity(64);
        let mut second: Vec<(IVec2, IVec2)> = Vec::with_capacity(64);
        let mut third: Vec<(IVec2, IVec2)> = Vec::with_capacity(64);
        let mut fourth: Vec<(IVec2, IVec2)> = Vec::with_capacity(64);

        for row in 0..16 {
            for col in 0..16 {
                let min_point = IVec2::new(col * 64, row * 64);
                let max_point = IVec2::new((col + 1) * 64 - 1, (row + 1) * 64 - 1);
                let chunk = if row % 2 == 0 {
                    if col % 2 == 0 {
                        &mut first
                    } else {
                        &mut second
                    }
                } else {
                    if col % 2 == 0 {
                        &mut third
                    } else {
                        &mut fourth
                    }
                };
                chunk.push((min_point, max_point));
            }
        }
        ChunkGroups {
            one: first,
            two: second,
            third,
            four: fourth,
        }
    }
}

impl ChunkGroups {
    pub fn member_of(&self, coords: &IVec2) -> ChunkGroup {
        for (chunk_group, chunk) in self.iter().enumerate() {
            for (min, max) in chunk {
                if coords.x >= min.x && coords.x <= max.x && coords.y >= min.y && coords.y <= max.y {
                    return match chunk_group {
                        0 => ChunkGroup::One,
                        1 => ChunkGroup::Two,
                        2 => ChunkGroup::Three,
                        3 => ChunkGroup::Four,
                        _ => unreachable!(),
                    };
                }
            }
        }
        panic!("Coordinates are out of bounds for all chunks");
    }
}

pub struct ChunksIterator<'a> {
    chunk_groups: &'a ChunkGroups,
    current_vec: usize,
}

impl<'a> Iterator for ChunksIterator<'a> {
    type Item = &'a Vec<(IVec2, IVec2)>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.current_vec {
            0 => Some(&self.chunk_groups.one),
            1 => Some(&self.chunk_groups.two),
            2 => Some(&self.chunk_groups.third),
            3 => Some(&self.chunk_groups.four),
            _ => None,
        };
        self.current_vec += 1;
        result
    }
}

impl ChunkGroups {
    pub fn iter(&self) -> ChunksIterator {
        ChunksIterator {
            chunk_groups: self,
            current_vec: 0,
        }
    }
}
