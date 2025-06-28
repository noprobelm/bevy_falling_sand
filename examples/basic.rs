use bevy::prelude::*;
use bevy_falling_sand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandMinimalPlugin,
            FallingSandMovementPlugin,
            FallingSandColorPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            setup_boundary.run_if(resource_not_exists::<BoundaryReady>),
        )
        .add_systems(
            Update,
            stream_particles.run_if(resource_exists::<BoundaryReady>),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -50;
const BOUNDARY_END_X: i32 = 50;
const BOUNDARY_HEIGHT: i32 = 50;

fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}

#[derive(Resource)]
struct BoundaryReady;

#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: 0.11,
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
    ));

    commands.spawn((WallBundle::new(
        ParticleType::new("Dirt Wall"),
        ColorProfile::new(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ),));

    commands.spawn((
        LiquidBundle::new(
            ParticleType::new("Water"),
            Density(750),
            Velocity::new(1, 3),
            5,
            ColorProfile::new(vec![Color::Srgba(Srgba::hex("#0B80AB80").unwrap())]),
        ),
        // If momentum effects are desired, insert the marker component.
        MomentumBlueprint::default(),
    ));

    commands.spawn((
        MovableSolidBundle::new(
            ParticleType::new("Sand"),
            Density(1250),
            Velocity::new(1, 3),
            ColorProfile::new(vec![
                Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
                Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
            ]),
        ),
        MomentumBlueprint::default(),
    ));
}

fn setup_boundary(mut commands: Commands, particle_type_map: Res<ParticleTypeMap>) {
    if particle_type_map.contains("Dirt Wall") {
        let start_x = -50;
        let end_x = 50;
        let height = 50;

        for y in 0..height {
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(start_x as f32, -(y as f32), 0.0),
            ));
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(end_x as f32, -(y as f32), 0.0),
            ));
        }

        for x in start_x..=end_x {
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(x as f32, -(height as f32), 0.0),
            ));
        }
        commands.insert_resource(BoundaryReady);
    }
}

fn stream_particles(mut commands: Commands, time: Res<Time>) {
    if time.elapsed_secs() > 30.0 {
        return;
    }
    let center_x = (BOUNDARY_START_X + BOUNDARY_END_X) / 2;
    let spawn_y = -(BOUNDARY_HEIGHT as f32) - 10.0;

    let radius = 3;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let base_x = center_x as f32 + dx as f32;
                let y = spawn_y + dy as f32 + 150.0;

                commands.spawn((
                    Particle::new("Sand"),
                    Transform::from_xyz(base_x - 30.0, y, 0.0),
                ));

                commands.spawn((
                    Particle::new("Water"),
                    Transform::from_xyz(base_x + 30.0, y, 0.0),
                ));
            }
        }
    }
}
