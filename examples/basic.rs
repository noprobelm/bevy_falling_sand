use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandMinimalPlugin,
            FallingSandMovementPlugin,
            FallingSandColorPlugin,
            FallingSandDebugPlugin,
        ))
        .init_resource::<TotalParticleCount>()
        .init_resource::<SpawnParticles>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                setup_boundary.run_if(resource_not_exists::<BoundaryReady>),
                stream_particles.run_if(
                    resource_exists::<BoundaryReady>.and(resource_exists::<SpawnParticles>),
                ),
                update_total_particle_count_text.run_if(resource_exists::<TotalParticleCount>),
                toggle_spawn_particles.run_if(input_just_pressed(KeyCode::F1)),
                toggle_debug_map.run_if(input_just_pressed(KeyCode::F2)),
                toggle_debug_dirty_rects.run_if(input_just_pressed(KeyCode::F3)),
                reset.run_if(input_just_pressed(KeyCode::KeyR)),
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -100;
const BOUNDARY_END_X: i32 = 100;
const BOUNDARY_HEIGHT: i32 = 100;

fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}

#[derive(Resource)]
struct BoundaryReady;

#[derive(Default, Resource)]
struct SpawnParticles;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct TotalParticleCountText;

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

    // The instructions and modes are rendered on the left-hand side in a column.
    let instructions_text = "F1: Toggle particle spawning\n\
        F2: Show/Hide particle chunk map\n\
        F3: Show/Hide \"dirty rectangles\"\n\
        R: Reset\n";
    let style = TextFont::default();

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((Text::new(instructions_text), style.clone()));
            parent.spawn((
                TotalParticleCountText,
                Text::new("Total Particles: "),
                style.clone(),
            ));
        });
}

fn setup_boundary(mut commands: Commands, particle_type_map: Res<ParticleTypeMap>) {
    if particle_type_map.contains("Dirt Wall") {
        for y in 0..BOUNDARY_HEIGHT {
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(BOUNDARY_START_X as f32, -(y as f32), 0.0),
            ));
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(BOUNDARY_END_X as f32, -(y as f32), 0.0),
            ));
        }

        for x in BOUNDARY_START_X..=BOUNDARY_END_X {
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(x as f32, -(BOUNDARY_HEIGHT as f32), 0.0),
            ));
        }
        commands.insert_resource(BoundaryReady);
    }
}

fn stream_particles(mut commands: Commands) {
    let center_x = (BOUNDARY_START_X + BOUNDARY_END_X) / 2;
    let spawn_y = -(BOUNDARY_HEIGHT as f32) - 10.0;

    let radius = 3;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let base_x = center_x as f32 + dx as f32;
                let y = spawn_y + dy as f32 + 200.0;

                commands.spawn((
                    Particle::new("Sand"),
                    Transform::from_xyz(base_x - 75.0, y, 0.0),
                ));

                commands.spawn((
                    Particle::new("Water"),
                    Transform::from_xyz(base_x + 75.0, y, 0.0),
                ));
            }
        }
    }
}

fn toggle_spawn_particles(mut commands: Commands, debug_map: Option<Res<SpawnParticles>>) {
    if debug_map.is_some() {
        commands.remove_resource::<SpawnParticles>();
    } else {
        commands.init_resource::<SpawnParticles>();
    }
}

fn toggle_debug_map(mut commands: Commands, debug_map: Option<Res<DebugParticleMap>>) {
    if debug_map.is_some() {
        commands.remove_resource::<DebugParticleMap>();
    } else {
        commands.init_resource::<DebugParticleMap>();
    }
}

fn toggle_debug_dirty_rects(
    mut commands: Commands,
    debug_dirty_rects: Option<Res<DebugDirtyRects>>,
) {
    if debug_dirty_rects.is_some() {
        commands.remove_resource::<DebugDirtyRects>();
    } else {
        commands.init_resource::<DebugDirtyRects>();
    }
}

fn update_total_particle_count_text(
    debug_total_particle_count: Res<TotalParticleCount>,
    mut total_particle_count_text: Query<&mut Text, With<TotalParticleCountText>>,
) -> Result {
    let new_text = format!("Total Particles: {:?}", debug_total_particle_count.0);
    for mut total_particle_count_text in total_particle_count_text.iter_mut() {
        (**total_particle_count_text).clone_from(&new_text);
    }
    Ok(())
}

fn reset(mut commands: Commands) {
    commands.trigger(ClearDynamicParticlesEvent);
}
