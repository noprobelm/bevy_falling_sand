use std::iter;
use std::mem;
use std::slice::Iter;
use std::ops::RangeBounds;

use bevy::{prelude::*, utils::HashSet};
use bevy_turborand::{RngComponent, DelegatedRng};
use serde::{Serialize, Deserialize};
use smallvec::SmallVec;
use bfs_core::{ChunkMap, Coordinates, Hibernating, Particle, ParticleSimulationSet, SimulationRun};

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_movement
                .in_set(ParticleSimulationSet).run_if(resource_exists::<SimulationRun>)
        )
        .register_type::<Density>()
        .register_type::<Velocity>()
        .register_type::<Momentum>();
    }
}


/// RNG to use when dealing with any entity that needs random movement behaviors.
#[derive(Clone, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct PhysicsRng(pub RngComponent);

impl PhysicsRng {
    /// Shuffles a given slice.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        self.0.shuffle(slice);
    }

    /// Returns a boolean value based on a rate. rate represents the chance to return a true value, with 0.0 being no
    /// chance and 1.0 will always return true.
    pub fn chance(&mut self, rate: f64) -> bool {
        self.0.chance(rate)
    }

    /// Samples a random item from a slice of values.
    pub fn sample<'a, T>(&mut self, list: &'a [T]) -> Option<&'a T> {
        self.0.sample(&list)
    }

    /// Returns a usize value for stable indexing across different word size platforms.
    pub fn index(&mut self, bound: impl RangeBounds<usize>) -> usize {
        self.0.index(bound)
    }
}

/// The density of a particle.
#[derive(
    Copy,
    Clone,
    Hash,
    Debug,
    Default,
    Eq,
    PartialEq,
    PartialOrd,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component, Debug)]
pub struct Density(pub u32);

/// A particle's velocity.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Default,
    Component,
    Reflect,
    Serialize,
    Deserialize,
)]
#[reflect(Component)]
pub struct Velocity {
    /// The current velocity of the particle.
    pub val: u8,
    /// The maximum velocity of the particle.
    pub max: u8,
}

impl Velocity {
    /// Creates a new velocity component.
    #[inline(always)]
    pub fn new(val: u8, max: u8) -> Self {
        Velocity { val, max }
    }

    /// Increment the velocity by 1
    #[inline(always)]
    pub fn increment(&mut self) {
        if self.val < self.max {
            self.val += 1;
        }
    }

    /// Decrement the velocity by 1
    #[inline(always)]
    pub fn decrement(&mut self) {
        if self.val > 1 {
            self.val -= 1;
        }
    }
}


/// Momentum component for particles. If a particle possesses this component, it will dynamically attempt to move in the
/// same direction it moved in the previous frame.
#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Momentum(pub IVec2);

impl Momentum {
    /// Use if the particle is capable of gaining momentum, but currently has none.
    pub const ZERO: Self = Self(IVec2::splat(0));
}

/// A group of neighbors representing equally prioritized candidates for particle movement.
/// Positions are relative to the particle's position.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
pub struct NeighborGroup {
    /// The neighbor candidates.
    pub neighbor_group: SmallVec<[IVec2; 4]>,
}

impl NeighborGroup {
    /// Creates a new NeighborGroup instance.
    pub fn new(neighbor_group: SmallVec<[IVec2; 4]>) -> NeighborGroup {
        NeighborGroup { neighbor_group }
    }

    /// An iterator over neighbors.
    pub fn iter(&self) -> impl Iterator<Item = &IVec2> {
        self.neighbor_group.iter()
    }

    /// A mutable iterator over neighbors.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut IVec2> {
        self.neighbor_group.iter_mut()
    }

    /// An iterator over random neighbors.
    pub fn iter_candidates<'a>(
        &'a mut self,
        rng: &mut PhysicsRng,
        momentum: Option<&Momentum>,
    ) -> NeighborGroupIter<'a> {
        if let Some(momentum) = momentum {
            if let Some(position) = self
                .neighbor_group
                .iter()
                .position(|&candidate| momentum.0 == candidate)
            {
                return NeighborGroupIter::Single(iter::once(&self.neighbor_group[position]));
            }
        }

        // Shuffle the neighbors and return the iterator over all neighbors
        rng.shuffle(&mut self.neighbor_group);
        NeighborGroupIter::All(self.neighbor_group.iter())
    }
}

/// An iterator over neighbor groups
pub enum NeighborGroupIter<'a> {
    /// A single neighbor should be prioritized above all others.
    Single(iter::Once<&'a IVec2>),
    /// All neighbors should be iterated.
    All(Iter<'a, IVec2>),
}

impl<'a> Iterator for NeighborGroupIter<'a> {
    type Item = &'a IVec2;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            NeighborGroupIter::Single(iter) => iter.next(),
            NeighborGroupIter::All(iter) => iter.next(),
        }
    }
}

/// A collection of neighbor groups ordered by descending priority.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementPriority {
    /// The neighbor ps.
    pub neighbor_groups: SmallVec<[NeighborGroup; 8]>,
}

impl MovementPriority {
    /// Creates a new NeighborGroup instance.
    pub fn new(neighbor_groups: SmallVec<[NeighborGroup; 8]>) -> MovementPriority {
        MovementPriority { neighbor_groups }
    }

    /// Creates a new NeighborGroup instance.
    pub fn from(movement_priority: SmallVec<[SmallVec<[IVec2; 4]>; 8]>) -> MovementPriority {
        MovementPriority::new(
            movement_priority
                .into_iter()
                .map(|neighbor_group| NeighborGroup::new(neighbor_group))
                .collect::<SmallVec<[NeighborGroup; 8]>>(),
        )
    }

    /// An iterator over neighbors.
    pub fn iter(&self) -> impl Iterator<Item = &NeighborGroup> {
        self.neighbor_groups.iter()
    }

    /// Iterates over movement candidates for a particle.
    pub fn iter_candidates<'a>(
        &'a mut self,
        rng: &'a mut PhysicsRng,
        momentum: Option<&'a Momentum>,
    ) -> impl Iterator<Item = &'a IVec2> + 'a {
        self.neighbor_groups
            .iter_mut()
            .flat_map(move |neighbor_group| neighbor_group.iter_candidates(rng, momentum))
    }
}

impl MovementPriority {
    /// Returns an empty MovementPriority
    pub const fn empty() -> MovementPriority {
        MovementPriority {
            neighbor_groups: SmallVec::new_const(),
        }
    }
}

/// Moves all qualifying particles 'v' times equal to their current velocity
#[allow(unused_mut)]
pub fn handle_movement(
    mut particle_query: Query<
        (
            Entity,
            &Particle,
            &mut Coordinates,
            &mut Transform,
            &mut PhysicsRng,
            &mut Velocity,
            Option<&mut Momentum>,
            &Density,
            &mut MovementPriority,
        ),
        Without<Hibernating>,
    >,
    mut map: ResMut<ChunkMap>,
) {
    // Check visited before we perform logic on a particle (particles shouldn't move more than once)
    let mut visited: HashSet<IVec2> = HashSet::default();
    unsafe {
        particle_query.iter_unsafe().for_each(
            |(
                _,
                particle_type,
                mut coordinates,
                mut transform,
                mut rng,
                mut velocity,
                mut momentum,
                density,
                mut movement_priority,
            )| {
                // Used to determine if we should add the particle to set of visited particles.
                let mut moved = false;
                'velocity_loop: for _ in 0..velocity.val {
                    // If a particle is blocked on a certain vector, we shouldn't attempt to swap it with other particles along that
                    // same vector.
                    let mut obstructed: HashSet<IVec2> = HashSet::default();

                    for relative_coordinates in movement_priority
                        .iter_candidates(&mut rng, momentum.as_deref().cloned().as_ref())
                    {
                        let neighbor_coordinates = coordinates.0 + *relative_coordinates;

                        if visited.contains(&neighbor_coordinates)
                            || obstructed.contains(&relative_coordinates.signum())
                        {
                            continue;
                        }

                        match map.entity(&neighbor_coordinates) {
                            Some(neighbor_entity) => {
                                if let Ok((
                                    _,
                                    neighbor_particle_type,
                                    mut neighbor_coordinates,
                                    mut neighbor_transform,
                                    _,
                                    _,
                                    _,
                                    neighbor_density,
                                    _,
                                )) = particle_query.get_unchecked(*neighbor_entity)
                                {
                                    if *particle_type == *neighbor_particle_type {
                                        continue;
                                    }
                                    if density > neighbor_density {
                                        map.swap(neighbor_coordinates.0, coordinates.0);

                                        swap_particle_positions(
                                            &mut coordinates,
                                            &mut transform,
                                            &mut neighbor_coordinates,
                                            &mut neighbor_transform,
                                        );

                                        if let Some(ref mut momentum) = momentum {
                                            momentum.0 = IVec2::ZERO; // Reset momentum after a swap
                                        }

                                        velocity.decrement();
                                        moved = true;
                                        break 'velocity_loop;
                                    }
                                    // We've encountered an anchored or hibernating particle. If this is a hibernating particle, it's guaranteed to
                                    // be awoken on the next frame with the logic contained in ChunkMap.reset_chunks()
                                    else {
                                        obstructed.insert(relative_coordinates.signum());
                                        continue;
                                    }
                                }
                                // We've encountered an anchored particle
                                else {
                                    obstructed.insert(relative_coordinates.signum());
                                    continue;
                                }
                            }
                            // We've encountered a free slot for the target particle to move to
                            None => {
                                map.swap(coordinates.0, neighbor_coordinates);
                                coordinates.0 = neighbor_coordinates;

                                transform.translation.x = neighbor_coordinates.x as f32;
                                transform.translation.y = neighbor_coordinates.y as f32;

                                if let Some(ref mut momentum) = momentum {
                                    momentum.0 = *relative_coordinates; // Set momentum relative to the current position
                                }

                                velocity.increment();

                                moved = true;

                                continue 'velocity_loop;
                            }
                        };
                    }
                }

                if moved {
                    visited.insert(coordinates.0);
                } else {
                    if let Some(ref mut momentum) = momentum {
                        momentum.0 = IVec2::ZERO;
                    }
                    velocity.decrement();
                }
            },
        );
    }
}

fn swap_particle_positions(
    first_coordinates: &mut Coordinates,
    first_transform: &mut Transform,
    second_coordinates: &mut Coordinates,
    second_transform: &mut Transform,
) {
    mem::swap(
        &mut first_transform.translation,
        &mut second_transform.translation,
    );
    mem::swap(&mut first_coordinates.0, &mut second_coordinates.0);
}
