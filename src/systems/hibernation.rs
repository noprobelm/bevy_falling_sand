use crate::*;

pub fn reset_chunks(
    commands: Commands,
    mut map: ResMut<ChunkMap>,
) {
    map.sleep_chunks(commands);
    map.deactivate_all_chunks();
}
