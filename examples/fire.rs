use std::time::Duration;

use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_pressed},
        mouse::MouseWheel,
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_falling_sand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default().with_spatial_refresh_frequency(Duration::from_millis(50)),
        ))
        .init_resource::<SpawnFlammableGasParticles>()
        .init_resource::<CursorPosition>()
        .add_systems(Startup, setup)
        .add_systems(Update, zoom_camera)
        .add_systems(
            Update,
            (
                update_cursor_position,
                spawn_boundary.run_if(resource_not_exists::<BoundaryReady>),
                spawn_fire.run_if(input_pressed(MouseButton::Left)),
                spawn_flammable_gas_particles.run_if(
                    resource_exists::<BoundaryReady>
                        .and(resource_exists::<SpawnFlammableGasParticles>),
                ),
                toggle_spawn_flamable_gas_particles.run_if(input_just_pressed(KeyCode::F1)),
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_START_Y: i32 = -150;
const BOUNDARY_END_Y: i32 = 150;

fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}

#[derive(Resource)]
struct BoundaryReady;

#[derive(Component)]
struct MainCamera;

#[derive(Default, Resource)]
struct SpawnFlammableGasParticles;

#[derive(Default, Resource, Clone, Debug)]
pub struct CursorPosition {
    pub current: Vec2,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: 0.2,
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
        GasBundle::new(
            ParticleType::new("Flammable Gas"),
            Density(200),
            Velocity::new(1, 1),
            1,
            ColorProfile::new(vec![
                Color::Srgba(Srgba::hex("#40621880").unwrap()),
                Color::Srgba(Srgba::hex("#4A731C80").unwrap()),
            ]),
        ),
        ChangesColorBlueprint(ChangesColor::new(0.1)),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Some(0.5),
            None,
            Some(ColorProfile::new(vec![
                Color::Srgba(Srgba::hex("#FF5900").unwrap()),
                Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                Color::Srgba(Srgba::hex("#FF9900").unwrap()),
                Color::Srgba(Srgba::hex("#FFCF00").unwrap()),
                Color::Srgba(Srgba::hex("#FFE808").unwrap()),
            ])),
            Some(Fire {
                burn_radius: 2.,
                chance_to_spread: 1.,
                destroys_on_spread: true,
            }),
        )),
        Name::new("Flammable Gas"),
    ));

    commands.spawn((
        GasBundle::new(
            ParticleType::new("FIRE"),
            Density(450),
            Velocity::new(1, 3),
            1,
            ColorProfile::new(vec![
                Color::Srgba(Srgba::hex("#FF5900FF").unwrap()),
                Color::Srgba(Srgba::hex("#FF9100FF").unwrap()),
                Color::Srgba(Srgba::hex("#FFCF00FF").unwrap()),
                Color::Srgba(Srgba::hex("#C74A05FF").unwrap()),
            ]),
        ),
        ChangesColorBlueprint(ChangesColor::new(0.1)),
        FireBlueprint(Fire {
            burn_radius: 1.5,
            chance_to_spread: 0.01,
            destroys_on_spread: false,
        }),
        BurnsBlueprint(Burns::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
            Some(0.5),
            None,
            None,
            None,
        )),
        BurningBlueprint(Burning::new(
            Duration::from_secs(1),
            Duration::from_millis(100),
        )),
        Name::new("FIRE"),
    ));

    // The instructions and modes are rendered on the left-hand side in a column.
    let instructions_text = "F1: Toggle flammable gas stream\n\
        Left Mouse: Spawn fire at cursor\n\
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
        });
}

fn spawn_boundary(mut commands: Commands, particle_type_map: Res<ParticleTypeMap>) {
    if particle_type_map.contains("Dirt Wall") {
        for y in BOUNDARY_START_Y - 1..BOUNDARY_END_Y + 1 {
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(BOUNDARY_START_X as f32, -(y as f32), 0.0),
            ));
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(BOUNDARY_START_X as f32 - 1., -(y as f32), 0.0),
            ));

            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(BOUNDARY_END_X as f32, -(y as f32), 0.0),
            ));
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(BOUNDARY_END_X as f32 + 1., -(y as f32), 0.0),
            ));
        }

        for x in BOUNDARY_START_X - 1..=BOUNDARY_END_X + 1 {
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(x as f32, -(BOUNDARY_START_Y as f32), 0.0),
            ));
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(x as f32, -(BOUNDARY_START_Y as f32 - 1.), 0.0),
            ));

            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(x as f32, -(BOUNDARY_END_Y as f32), 0.0),
            ));
            commands.spawn((
                Particle::new("Dirt Wall"),
                Transform::from_xyz(x as f32, -(BOUNDARY_END_Y as f32 + 1.), 0.0),
            ));
        }

        commands.insert_resource(BoundaryReady);
    }
}

fn spawn_flammable_gas_particles(mut commands: Commands) {
    let center_x = (BOUNDARY_START_X + BOUNDARY_END_X) / 2;
    let center_y = (BOUNDARY_START_Y + BOUNDARY_END_Y) / 2;

    let radius = 10;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let spawn_x = center_x as f32 + dx as f32;
                let spawn_y = center_y as f32 + dy as f32;

                commands.spawn((
                    Particle::new("Flammable Gas"),
                    Transform::from_xyz(spawn_x, spawn_y, 0.0),
                ));
            }
        }
    }
}

fn spawn_fire(mut commands: Commands, cursor_position: Res<CursorPosition>) {
    let center = cursor_position.current;
    let radius = 3;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let spawn_x = center.x + dx as f32;
                let spawn_y = center.y + dy as f32;

                commands.spawn((
                    Particle::new("FIRE"),
                    Transform::from_xyz(spawn_x, spawn_y, 0.0),
                ));
            }
        }
    }
}
fn toggle_spawn_flamable_gas_particles(
    mut commands: Commands,
    debug_map: Option<Res<SpawnFlammableGasParticles>>,
) {
    if debug_map.is_some() {
        commands.remove_resource::<SpawnFlammableGasParticles>();
    } else {
        commands.init_resource::<SpawnFlammableGasParticles>();
    }
}

fn update_cursor_position(
    mut cursor_position: ResMut<CursorPosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Result {
    let (camera, camera_transform) = q_camera.single()?;

    let window = q_window.single()?;
    if let Some(world_position) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        cursor_position.current = world_position;
    }
    Ok(())
}

fn zoom_camera(
    mut ev_scroll: EventReader<MouseWheel>,
    mut camera_query: Query<&mut Projection, With<MainCamera>>,
) {
    const ZOOM_IN_FACTOR: f32 = 0.9;
    const ZOOM_OUT_FACTOR: f32 = 1.1;

    if !ev_scroll.is_empty() {
        let mut projection = match camera_query.single_mut() {
            Ok(p) => p,
            Err(_) => return,
        };
        let Projection::Orthographic(orthographic) = projection.as_mut() else {
            return;
        };
        ev_scroll.read().for_each(|ev| {
            if ev.y < 0. {
                orthographic.scale *= ZOOM_OUT_FACTOR;
            } else if ev.y > 0. {
                orthographic.scale *= ZOOM_IN_FACTOR;
            }
        });
    };
}
