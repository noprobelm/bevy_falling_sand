use bevy::prelude::*;
use bfs_core::{Particle, ParticleRegistrationEvent, ParticleType};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::iter;
use std::slice::Iter;

use crate::rng::PhysicsRng;

pub struct ParticleDefinitionsPlugin;

impl Plugin for ParticleDefinitionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_particle_registration)
            .register_type::<Density>()
            .register_type::<Velocity>()
            .register_type::<Momentum>()
            .register_type::<MovementPriority>();
    }
}

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
pub struct DensityBlueprint(pub Density);

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
    pub val: u8,
    pub max: u8,
}

impl Velocity {
    pub fn new(val: u8, max: u8) -> Self {
        Velocity { val, max }
    }

    pub fn increment(&mut self) {
        if self.val < self.max {
            self.val += 1;
        }
    }

    pub fn decrement(&mut self) {
        if self.val > 1 {
            self.val -= 1;
        }
    }
}

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
pub struct VelocityBlueprint(pub Velocity);

#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct Momentum(pub IVec2);

impl Momentum {
    pub const ZERO: Self = Self(IVec2::splat(0));
}

#[derive(
    Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect, Serialize, Deserialize,
)]
#[reflect(Component)]
pub struct MomentumBlueprint(pub Momentum);

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
pub struct NeighborGroup {
    pub neighbor_group: SmallVec<[IVec2; 4]>,
}

impl NeighborGroup {
    pub fn new(neighbor_group: SmallVec<[IVec2; 4]>) -> NeighborGroup {
        NeighborGroup { neighbor_group }
    }

    pub fn empty() -> NeighborGroup {
        NeighborGroup {
            neighbor_group: SmallVec::new(),
        }
    }

    pub fn push(&mut self, neighbor: IVec2) {
        self.neighbor_group.push(neighbor);
    }

    pub fn swap(&mut self, index1: usize, index2: usize) -> Result<(), String> {
        if index1 < self.neighbor_group.len() && index2 < self.neighbor_group.len() {
            self.neighbor_group.swap(index1, index2);
            Ok(())
        } else {
            Err(format!(
                "Swap indices out of bounds: index1={}, index2={}, group size={}",
                index1,
                index2,
                self.neighbor_group.len()
            ))
        }
    }

    pub fn len(&self) -> usize {
        self.neighbor_group.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &IVec2> {
        self.neighbor_group.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut IVec2> {
        self.neighbor_group.iter_mut()
    }

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

        rng.shuffle(&mut self.neighbor_group);
        NeighborGroupIter::All(self.neighbor_group.iter())
    }
}

pub enum NeighborGroupIter<'a> {
    Single(iter::Once<&'a IVec2>),
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

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementPriority {
    pub neighbor_groups: SmallVec<[NeighborGroup; 8]>,
}

impl MovementPriority {
    pub fn new(neighbor_groups: SmallVec<[NeighborGroup; 8]>) -> MovementPriority {
        MovementPriority { neighbor_groups }
    }

    pub fn from(movement_priority: Vec<Vec<IVec2>>) -> MovementPriority {
        MovementPriority::new(
            movement_priority
                .into_iter()
                .map(|neighbor_group| NeighborGroup::new(SmallVec::from_vec(neighbor_group)))
                .collect::<SmallVec<[NeighborGroup; 8]>>(),
        )
    }

    pub fn len(&self) -> usize {
        self.neighbor_groups.len()
    }

    pub fn push_outer(&mut self, neighbor_group: NeighborGroup) {
        self.neighbor_groups.push(neighbor_group);
    }

    pub fn push_inner(&mut self, group_index: usize, neighbor: IVec2) -> Result<(), String> {
        if let Some(group) = self.neighbor_groups.get_mut(group_index) {
            group.push(neighbor);
            Ok(())
        } else {
            Err(format!("Group index {} out of bounds", group_index))
        }
    }

    pub fn swap_outer(&mut self, index1: usize, index2: usize) -> Result<(), String> {
        if index1 < self.neighbor_groups.len() && index2 < self.neighbor_groups.len() {
            self.neighbor_groups.swap(index1, index2);
            Ok(())
        } else {
            Err("Outer indices out of bounds".to_string())
        }
    }

    pub fn swap_inner(
        &mut self,
        group_index: usize,
        index1: usize,
        index2: usize,
    ) -> Result<(), String> {
        if let Some(group) = self.neighbor_groups.get_mut(group_index) {
            if index1 < group.len() && index2 < group.len() {
                return group.swap(index1, index2);
            } else {
                Err("Inner indices out of bounds".to_string())
            }
        } else {
            Err(format!("Group index {} out of bounds", group_index))
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &NeighborGroup> {
        self.neighbor_groups.iter()
    }

    pub fn iter_candidates<'a>(
        &'a mut self,
        rng: &'a mut PhysicsRng,
        momentum: Option<&'a Momentum>,
    ) -> impl Iterator<Item = &'a IVec2> + 'a {
        self.neighbor_groups
            .iter_mut()
            .flat_map(move |neighbor_group| neighbor_group.iter_candidates(rng, momentum))
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut NeighborGroup> {
        self.neighbor_groups.get_mut(index)
    }

    pub fn remove(&mut self, index: usize) -> Option<NeighborGroup> {
        if index < self.neighbor_groups.len() {
            Some(self.neighbor_groups.remove(index))
        } else {
            None
        }
    }
}

impl MovementPriority {
    pub const fn empty() -> MovementPriority {
        MovementPriority {
            neighbor_groups: SmallVec::new_const(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct MovementPriorityBlueprint(pub MovementPriority);

fn handle_particle_registration(
    mut commands: Commands,
    parent_query: Query<
        (
            Option<&DensityBlueprint>,
            Option<&VelocityBlueprint>,
            Option<&MovementPriorityBlueprint>,
            Option<&MomentumBlueprint>,
        ),
        With<ParticleType>,
    >,
    mut ev_particle_registered: EventReader<ParticleRegistrationEvent>,
    particle_query: Query<&Parent, With<Particle>>,
) {
    ev_particle_registered.read().for_each(|ev| {
        ev.entities.iter().for_each(|entity| {
            if let Ok(parent) = particle_query.get(*entity) {
                commands.entity(*entity).insert(PhysicsRng::default());
                if let Ok((density, velocity, movement_priority, momentum)) =
                    parent_query.get(parent.get())
                {
                    if let Some(density) = density {
                        commands.entity(*entity).insert(density.0);
                    } else {
                        commands.entity(*entity).remove::<Density>();
                    }
                    if let Some(velocity) = velocity {
                        commands.entity(*entity).insert(velocity.0.clone());
                    } else {
                        commands.entity(*entity).remove::<Velocity>();
                    }
                    if let Some(movement_priority) = movement_priority {
                        commands.entity(*entity).insert(movement_priority.0.clone());
                    } else {
                        commands.entity(*entity).remove::<MovementPriority>();
                    }
                    if let Some(momentum) = momentum {
                        commands.entity(*entity).insert(momentum.0.clone());
                    } else {
                        commands.entity(*entity).remove::<Momentum>();
                    }
                }
            }
        });
    });
}
