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
        .add_systems(Startup, (setup_camera, setup_particles))
        .add_systems(Update, (setup_boundary, stream_particles))
        .run();
}

#[derive(Resource)]
struct SpawnProgressU {
    start_x: i32,
    end_x: i32,
    height: i32,
    duration: f32,
    elapsed: f32,
    completed: bool,
}

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: 0.11,
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
    ));
}

fn setup_particles(mut commands: Commands) {
    commands.spawn((
        WallBundle::new(
            ParticleType::new("Dirt Wall"),
            ColorProfile::new(vec![
                Color::Srgba(Srgba::hex("#916B4C").unwrap()),
                Color::Srgba(Srgba::hex("#73573D").unwrap()),
            ]),
        ),
        Name::new("Dirt Wall"),
    ));

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
        Name::new("Water"),
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
        Name::new("Sand"),
    ));

    commands.insert_resource(SpawnProgressU {
        start_x: -50,
        end_x: 50,
        height: 50,
        duration: 3.0,
        elapsed: 0.0,
        completed: false,
    });
}

fn setup_boundary(mut commands: Commands, time: Res<Time>, mut progress: ResMut<SpawnProgressU>) {
    progress.elapsed += time.delta_secs();

    let total_steps = 2 * progress.height + (progress.end_x - progress.start_x).abs();
    let fraction_done = (progress.elapsed / progress.duration).clamp(0.0, 1.0);

    let current_step = (fraction_done * total_steps as f32).round() as i32;
    let mut step_counter = 0;

    for y in 0..progress.height {
        if step_counter >= current_step {
            break;
        }
        commands.spawn((
            Particle::new("Dirt Wall"),
            Transform::from_xyz(progress.start_x as f32, -(y as f32), 0.0),
        ));
        step_counter += 1;
    }

    for x in progress.start_x..=progress.end_x {
        if step_counter >= current_step {
            break;
        }
        commands.spawn((
            Particle::new("Dirt Wall"),
            Transform::from_xyz(x as f32, -(progress.height as f32), 0.0),
        ));
        step_counter += 1;
    }

    for y in (0..progress.height).rev() {
        if step_counter >= current_step {
            break;
        }
        commands.spawn((
            Particle::new("Dirt Wall"),
            Transform::from_xyz(progress.end_x as f32, -(y as f32), 0.0),
        ));
        step_counter += 1;
    }

    if fraction_done >= 1.0 && !progress.completed {
        progress.completed = true;
    }
}

fn stream_particles(mut commands: Commands, progress: Res<SpawnProgressU>) {
    if progress.completed {
        let center_x = (progress.start_x + progress.end_x) / 2;
        let spawn_y = -(progress.height as f32) - 10.0;

        let radius = 3;

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    let base_x = center_x as f32 + dx as f32;
                    let y = spawn_y + dy as f32 + 100.0;

                    commands.spawn((
                        Particle::new("Sand"),
                        Transform::from_xyz(base_x - 10.0, y, 0.0),
                    ));

                    commands.spawn((
                        Particle::new("Water"),
                        Transform::from_xyz(base_x + 10.0, y, 0.0),
                    ));
                }
            }
        }
    }
}
