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
        app.add_systems(Update, spawn_colliders);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct TestGizmos {}

#[derive(Component, Debug)]
struct MyTestRigidBody;

#[derive(Resource, Default, Debug)]
struct PerimeterPositions((Vec<Vec<Vec2>>, Vec<Vec<[u32; 2]>>));

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
    for entity in colliders.0.drain(..) {
        commands.entity(entity).despawn();
    }

    for (i, vertices) in perimeter_positions.0.0.iter().enumerate() {
        let entity = commands
            .spawn((
                RigidBody::Static,
                Collider::polyline(vertices.clone(), Some(perimeter_positions.0.1[i].clone())),
            ))
            .id();

        colliders.0.push(entity);
    }
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

    let edges = extract_perimeter_edges(&occupied);

    let mut components = Vec::new();
    let mut perimeters = Vec::new();

    let mut vertices = Vec::new();
    for edge in &edges {
        vertices.push(edge[0]);
        vertices.push(edge[1]);
    }

    let indices: Vec<[u32; 2]> = (0..vertices.len() as u32)
        .step_by(2)
        .map(|i| [i, i + 1])
        .collect();

    components.push(vertices);
    perimeters.push(indices);

    wall_positions.0 = (components, perimeters);
}

fn extract_perimeter_edges(occupied: &HashSet<Coordinates>) -> Vec<[Vec2; 2]> {
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

    for &cell in occupied {
        let base = cell.0.as_vec2();
        for (offset, v0, v1) in directions {
            let neighbor = Coordinates(cell.0 + offset);
            if !occupied.contains(&neighbor) {
                edges.push([base + v0, base + v1]);
            }
        }
    }

    edges
}
