use crate::*;

pub fn reset_chunks(
    commands: Commands,
    mut map: ResMut<ParticleMap>,
) {
    map.sleep_chunks(commands);
    map.deactivate_all_chunks();
}
