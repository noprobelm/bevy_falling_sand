use std::collections::VecDeque;

use avian2d::prelude::*;
use bevy::{platform::collections::HashSet, prelude::*};
use bfs_core::Coordinates;
use bfs_movement::Wall;

pub struct FallingSandPhysicsPlugin;

impl Plugin for FallingSandPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<TestGizmos>();
        app.init_resource::<PerimeterPositions>();
        app.init_resource::<TerrainColliders>();
        app.add_event::<WallsChangedEvent>();
        app.add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()));
        app.add_systems(Startup, setup);
        app.add_systems(Update, accelerate_bodies);
        app.add_systems(Update, walls_changed);
        app.add_systems(Update, map_wall_particles);
        //app.add_systems(Update, draw_gizmos);
        app.add_systems(Update, spawn_colliders);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct TestGizmos {}

#[derive(Component, Debug)]
struct MyTestRigidBody;

#[derive(Resource, Default, Debug)]
struct PerimeterPositions((Vec<Vec<Coordinates>>, Vec<Vec<[u32; 2]>>));

#[derive(Resource, Default, Debug)]
struct TerrainColliders(Vec<Entity>);

#[derive(Event)]
struct WallsChangedEvent;

fn setup(mut commands: Commands) {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(0.5),
        Transform::from_xyz(0.0, 2.0, 0.0),
        Sprite {
            color: Color::srgba(1., 0., 0., 1.),
            ..default()
        },
        MyTestRigidBody,
    ));
    commands.spawn((RigidBody::Static, Collider::rectangle(5.0, 0.5)));
}

fn accelerate_bodies(
    mut query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<MyTestRigidBody>>,
    time: Res<Time>,
) {
    let delta_secs = time.delta_secs();
    for (mut linear_velocity, mut angular_velocity) in &mut query {
        linear_velocity.x += 2.0 * delta_secs;
        angular_velocity.0 += 0.5 * delta_secs;
    }
}

fn spawn_colliders(
    mut commands: Commands,
    mut colliders: ResMut<TerrainColliders>,
    perimeter_positions: Res<PerimeterPositions>,
) {
    if !perimeter_positions.is_changed() {
        return;
    }
    // Despawn previous colliders
    for entity in colliders.0.drain(..) {
        commands.entity(entity).despawn();
    }

    // Spawn new colliders
    for (i, vertices) in perimeter_positions.0.0.iter().enumerate() {
        let entity = commands
            .spawn((
                RigidBody::Static,
                Collider::polyline(
                    vertices.iter().map(|c| c.0.as_vec2()).collect(),
                    Some(perimeter_positions.0.1[i].clone()),
                ),
            ))
            .id();

        colliders.0.push(entity);
    }
}

fn draw_gizmos(
    mut gizmos: Gizmos,
    mut test_gizmos: Gizmos<TestGizmos>,
    wall_positions: Res<PerimeterPositions>,
) {
    wall_positions.0.0.iter().enumerate().for_each(|(i, v)| {
        v.iter().for_each(|c| {
            gizmos.circle_2d(c.0.as_vec2(), 0.25, Color::srgba(1., 0., 0., 1.));
        })
        // gizmos.linestrip_2d(
        //     v.iter().map(|c| c.0.as_vec2()),
        //     Color::srgba(1., 0., 0., 1.),
        // );
    });
}

fn trace_perimeter(perimeter: &HashSet<Coordinates>) -> Vec<Coordinates> {
    let mut output = Vec::new();

    // Pick starting point: lowest Y, then lowest X
    let &start = perimeter
        .iter()
        .min_by_key(|coord| (coord.0.y, coord.0.x))
        .unwrap();
    let mut current = start;
    let mut prev_dir = 6; // coming from left

    let offsets = [
        IVec2::new(0, 1),   // 0: up
        IVec2::new(1, 1),   // 1: up-right
        IVec2::new(1, 0),   // 2: right
        IVec2::new(1, -1),  // 3: down-right
        IVec2::new(0, -1),  // 4: down
        IVec2::new(-1, -1), // 5: down-left
        IVec2::new(-1, 0),  // 6: left
        IVec2::new(-1, 1),  // 7: up-left
    ];

    let mut visited = HashSet::new();

    loop {
        output.push(current);
        visited.insert(current);
        let mut found = false;

        for i in 0..8 {
            let dir = (prev_dir + i + 7) % 8;
            let offset = offsets[dir];
            let neighbor = Coordinates(current.0 + offset);

            if perimeter.contains(&neighbor) && !visited.contains(&neighbor) {
                current = neighbor;
                prev_dir = dir;
                found = true;
                break;
            }
        }

        if !found || current == start {
            break;
        }
    }

    output
}

fn walls_changed(
    query: Query<Entity, Changed<Wall>>,
    removed: RemovedComponents<Wall>,
    mut ev_walls_changed: EventWriter<WallsChangedEvent>,
) {
    if !query.is_empty() || !removed.is_empty() {
        ev_walls_changed.write(WallsChangedEvent);
    }
}

fn map_wall_particles(
    ev_walls_changed: EventReader<WallsChangedEvent>,
    query: Query<&Coordinates, With<Wall>>,
    mut wall_positions: ResMut<PerimeterPositions>,
) {
    if ev_walls_changed.is_empty() {
        return;
    }
    let occupied: HashSet<Coordinates> = query.iter().copied().collect();
    let mut visited: HashSet<Coordinates> = HashSet::new();

    let mut components: Vec<Vec<Coordinates>> = Vec::new();
    let mut perimeters: Vec<Vec<[u32; 2]>> = Vec::new();

    for &start in &occupied {
        if visited.contains(&start) {
            continue;
        }

        let mut component = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            component.push(current);

            for neighbor in neighbors(current) {
                if occupied.contains(&neighbor) && !visited.contains(&neighbor) {
                    queue.push_back(neighbor);
                    visited.insert(neighbor);
                }
            }
        }

        // Build perimeter set
        let perimeter_set: HashSet<Coordinates> = component
            .iter()
            .filter(|&&coord| neighbors(coord).iter().any(|n| !occupied.contains(n)))
            .copied()
            .collect();

        // Trace perimeter in correct order
        let ordered_perimeter = trace_perimeter(&perimeter_set);

        // Build vertices and index buffer
        let vertices: Vec<Coordinates> = ordered_perimeter.clone();
        let indices: Vec<[u32; 2]> = (0..vertices.len() as u32)
            .zip(1..=vertices.len() as u32)
            .map(|(a, b)| [a, b % vertices.len() as u32])
            .collect();

        components.push(vertices);
        perimeters.push(indices);
    }

    wall_positions.0 = (components, perimeters);
}

fn neighbors(p: Coordinates) -> Vec<Coordinates> {
    vec![
        Coordinates(p.0 + IVec2::new(1, 0)),
        Coordinates(p.0 + IVec2::new(-1, 0)),
        Coordinates(p.0 + IVec2::new(0, 1)),
        Coordinates(p.0 + IVec2::new(0, -1)),
        Coordinates(p.0 + IVec2::new(1, -1)),
        Coordinates(p.0 + IVec2::new(1, 1)),
        Coordinates(p.0 + IVec2::new(-1, -1)),
        Coordinates(p.0 + IVec2::new(-1, 1)),
    ]
}
