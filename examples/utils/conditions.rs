use bevy::prelude::*;

pub fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}
