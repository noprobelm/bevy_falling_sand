use crate::brush::BrushModeSpawnState;
use bevy::{
    input::common_conditions::input_just_released, platform::collections::HashSet, prelude::*,
};
use bevy_falling_sand::prelude::*;
use std::collections::VecDeque;

pub(crate) struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_dynamic_rigid_bodies
                .run_if(in_state(BrushModeSpawnState::DynamicRigidBodies))
                .run_if(input_just_released(MouseButton::Left)),
        )
        .add_observer(on_despawn_rigid_bodies);
    }
}

#[derive(Default, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug, Component)]
pub struct DynamicRigidBodyParticle;

#[derive(Event)]
pub enum DespawnRigidBodies {
    Dynamic,
    Static,
    All,
}

fn spawn_dynamic_rigid_bodies(
    mut commands: Commands,
    dynamic_rigid_body_particles_query: Query<(Entity, &Transform), With<DynamicRigidBodyParticle>>,
) {
    let positions: Vec<(Entity, IVec2)> = dynamic_rigid_body_particles_query
        .iter()
        .map(|(entity, pos)| {
            (
                entity,
                IVec2::new(pos.translation.x as i32, pos.translation.y as i32),
            )
        })
        .collect();

    if positions.is_empty() {
        return;
    }

    for (entity, _) in &positions {
        commands
            .entity(*entity)
            .remove::<DynamicRigidBodyParticle>();
    }

    let mut unvisited: HashSet<IVec2> = positions.iter().map(|(_, pos)| *pos).collect();
    let position_to_data: bevy::platform::collections::HashMap<IVec2, Entity> = positions
        .into_iter()
        .map(|(entity, pos)| (pos, entity))
        .collect();

    while let Some(&start) = unvisited.iter().next() {
        let mut group = Vec::new();
        let mut queue = VecDeque::new();
        let mut entities_to_despawn = Vec::new();

        queue.push_back(start);
        unvisited.remove(&start);

        while let Some(current) = queue.pop_front() {
            group.push(current);

            if let Some(entity) = position_to_data.get(&current) {
                entities_to_despawn.push(*entity);
            }

            for dir in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let neighbor = current + dir;
                if unvisited.remove(&neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }

        if group.len() < 3 {
            for entity in entities_to_despawn {
                commands.entity(entity).despawn();
            }
            continue;
        }

        let min = group
            .iter()
            .copied()
            .fold(IVec2::splat(i32::MAX), IVec2::min);
        let max = group
            .iter()
            .copied()
            .fold(IVec2::splat(i32::MIN), IVec2::max);
        let mut grid = Grid::new(min, max);
        for position in &group {
            grid.set(*position);
        }

        let loop_vertices = extract_ordered_perimeter_loop(&grid);
        if loop_vertices.len() < 3 {
            for entity in entities_to_despawn {
                commands.entity(entity).despawn();
            }
            continue;
        }

        if loop_vertices.len() >= 3 {
            use earcutr::earcut;

            let vertices: Vec<Vec2> = loop_vertices;

            let center_of_mass =
                vertices.iter().fold(Vec2::ZERO, |acc, v| acc + *v) / vertices.len() as f32;

            let relative_vertices: Vec<Vec2> =
                vertices.iter().map(|v| *v - center_of_mass).collect();

            let flattened: Vec<f64> = relative_vertices
                .iter()
                .flat_map(|v| vec![v.x as f64, v.y as f64])
                .collect();

            if let Ok(indices_raw) = earcut(&flattened, &[], 2) {
                let triangle_indices: Vec<[u32; 3]> = indices_raw
                    .chunks(3)
                    .map(|c| [c[0] as u32, c[1] as u32, c[2] as u32])
                    .collect();

                for entity in entities_to_despawn {
                    commands.entity(entity).despawn();
                }

                let all_particle_positions: Vec<Vec2> = group
                    .iter()
                    .map(|pos| pos.as_vec2() - center_of_mass)
                    .collect();

                let rigid_body_entity = commands
                    .spawn((
                        RigidBody::Dynamic,
                        Collider::trimesh(relative_vertices, triangle_indices),
                        Transform::from_xyz(center_of_mass.x, center_of_mass.y, 0.0),
                        TransformInterpolation,
                        InheritedVisibility::VISIBLE,
                        LinearVelocity::default(),
                    ))
                    .id();

                for particle_pos in all_particle_positions {
                    commands.spawn((
                        Transform::from_xyz(particle_pos.x, particle_pos.y, 0.0),
                        Sprite {
                            color: Color::srgba(0.5, 0.3, 0.2, 1.0),
                            custom_size: Some(Vec2::ONE),
                            ..Default::default()
                        },
                        Visibility::default(),
                        ChildOf(rigid_body_entity),
                    ));
                }
            } else {
                for entity in entities_to_despawn {
                    commands.entity(entity).despawn();
                }
            }
        } else {
            for entity in entities_to_despawn {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn on_despawn_rigid_bodies(
    trigger: Trigger<DespawnRigidBodies>,
    mut commands: Commands,
    rigid_body_query: Query<(Entity, &RigidBody)>,
) {
    match trigger.event() {
        DespawnRigidBodies::Dynamic => rigid_body_query.iter().for_each(|(entity, rigid_body)| {
            if rigid_body == &RigidBody::Dynamic {
                commands.entity(entity).despawn();
            }
        }),
        DespawnRigidBodies::Static => rigid_body_query.iter().for_each(|(entity, rigid_body)| {
            if rigid_body == &RigidBody::Static {
                commands.entity(entity).despawn();
            }
        }),
        DespawnRigidBodies::All => rigid_body_query.iter().for_each(|(entity, _)| {
            commands.entity(entity).despawn();
        }),
    }
}

#[derive(Debug)]
struct Grid {
    min: IVec2,
    size: IVec2,
    data: Vec<bool>,
}

impl Grid {
    fn new(min: IVec2, max: IVec2) -> Self {
        let size = max - min + IVec2::ONE;
        let data = vec![false; (size.x * size.y) as usize];
        Self { min, size, data }
    }

    fn index(&self, position: IVec2) -> usize {
        let local = position - self.min;
        (local.y * self.size.x + local.x) as usize
    }

    fn set(&mut self, position: IVec2) {
        let idx = self.index(position);
        self.data[idx] = true;
    }

    fn get(&self, position: IVec2) -> bool {
        if position.x < self.min.x
            || position.y < self.min.y
            || position.x > self.min.x + self.size.x - 1
            || position.y > self.min.y + self.size.y - 1
        {
            return false;
        }
        let idx = self.index(position);
        self.data[idx]
    }

    fn iter_occupied(&self) -> impl Iterator<Item = IVec2> + '_ {
        self.data.iter().enumerate().filter_map(move |(i, &b)| {
            if b {
                let x = i as i32 % self.size.x;
                let y = i as i32 / self.size.x;
                Some(self.min + IVec2::new(x, y))
            } else {
                None
            }
        })
    }
}

fn extract_ordered_perimeter_loop(grid: &Grid) -> Vec<Vec2> {
    let edges = extract_perimeter_edges(grid);
    if edges.is_empty() {
        return Vec::new();
    }

    let mut ordered = Vec::new();
    let mut remaining = edges;

    let [current_start, mut current_end] = remaining.swap_remove(0);
    ordered.push(current_start);
    ordered.push(current_end);

    while !remaining.is_empty() {
        let mut found = false;
        for i in 0..remaining.len() {
            let [start, end] = remaining[i];
            if start == current_end {
                ordered.push(end);
                current_end = end;
                remaining.swap_remove(i);
                found = true;
                break;
            } else if end == current_end {
                ordered.push(start);
                current_end = start;
                remaining.swap_remove(i);
                found = true;
                break;
            }
        }

        if !found {
            warn!("Could not form closed perimeter loop; perimeter might be disjoint or broken.");
            break;
        }

        if ordered[0] == current_end {
            break;
        }
    }

    if ordered.len() > 1 && ordered[0] == *ordered.last().unwrap() {
        ordered.pop();
    }

    ordered
}

fn extract_perimeter_edges(grid: &Grid) -> Vec<[Vec2; 2]> {
    let mut edges = Vec::new();

    let directions = [
        (IVec2::new(1, 0), Vec2::new(0.5, 0.5), Vec2::new(0.5, -0.5)),
        (
            IVec2::new(-1, 0),
            Vec2::new(-0.5, -0.5),
            Vec2::new(-0.5, 0.5),
        ),
        (IVec2::new(0, 1), Vec2::new(-0.5, 0.5), Vec2::new(0.5, 0.5)),
        (
            IVec2::new(0, -1),
            Vec2::new(0.5, -0.5),
            Vec2::new(-0.5, -0.5),
        ),
    ];

    for position in grid.iter_occupied() {
        let base = position.as_vec2();
        for (offset, v0, v1) in directions {
            if !grid.get(position + offset) {
                edges.push([base + v0, base + v1]);
            }
        }
    }

    edges
}
