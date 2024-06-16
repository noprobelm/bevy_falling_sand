use bevy::prelude::{Component, States, Resource};

/// Possible particle types. Add a variant of this enum to an entity to spawn the corresponding type
/// into the world.
#[derive(Component, States, Resource, Default, Debug, Copy, Clone, Reflect, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub enum ParticleType {
    #[default]
    Water,
    Fire,
    Oil,
    Whiskey,
    Sand,
    Wall,
    DirtWall,
    RockWall,
    DenseRockWall,
    GrassWall,
    Steam,
}
