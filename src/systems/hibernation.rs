use crate::*;

/// Resets all chunks in preparation for the next frame
///
/// When this system runs, all chunks are checked to see if they should be awakened in preparation for the next frame
/// (see field `should_process_this_frame`). After this, their 'activated' status is reset (see field
/// `should_process_next_frame`)
pub fn reset_chunks(
    commands: Commands,
    mut map: ResMut<ChunkMap>,
) {
    map.reset_chunks(commands);
}
