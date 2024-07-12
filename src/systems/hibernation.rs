use crate::*;

pub fn reset_chunks(
    commands: Commands,
    mut map: ResMut<ChunkMap>,
) {
    map.reset_chunks(commands);
}
