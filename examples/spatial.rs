use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_pressed},
        mouse::MouseWheel,
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_falling_sand::prelude::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default().with_spatial_refresh_frequency(Duration::from_millis(40)),
        ))
        .init_resource::<CursorPosition>()
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_camera, pan_camera))
        .add_systems(
            Update,
            (
                update_cursor_position,
                spawn_boundary.run_if(resource_not_exists::<BoundaryReady>),
                spawn_particles,
                mutate_particles.run_if(input_just_pressed(MouseButton::Left)),
                reset.run_if(input_just_pressed(KeyCode::KeyR)),
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

#[derive(Default, Resource, Clone, Debug)]
pub struct CursorPosition {
    pub current: Vec2,
}

#[derive(Component)]
struct MainCamera;

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

    let instructions_text = "Left Mouse: Mutate water into sand within a radius of 10\n\
        R: Reset";
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
    if particle_type_map.contains("Dirt Wall") && particle_type_map.contains("Water") {
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

fn spawn_particles(mut commands: Commands, time: Res<Time>) {
    if time.elapsed_secs() < 0.5 {
        let x_range = ((BOUNDARY_END_X - BOUNDARY_START_X) as f32 * 0.5) as i32;
        let y_range = ((BOUNDARY_END_Y - BOUNDARY_START_Y) as f32 * 0.5) as i32;

        for x in BOUNDARY_START_X + 50..BOUNDARY_START_X + 50 + x_range {
            for y in BOUNDARY_START_Y + 50..BOUNDARY_START_Y + 50 + y_range {
                if x % 2 == 0 && y % 2 == 0 {
                    commands.spawn((
                        Particle::new("Water"),
                        Transform::from_xyz(x as f32, -(y as f32), 0.0),
                    ));
                }
            }
        }
    }
}

fn mutate_particles(
    cursor_position: Res<CursorPosition>,
    mut particle_query: Query<&mut Particle>,
    particle_tree: Res<ParticleTree>,
) {
    particle_tree
        .within_distance(cursor_position.current, 10.0)
        .iter()
        .for_each(|(_, entity)| {
            if let Some(entity) = entity {
                if let Ok(mut particle) = particle_query.get_mut(*entity) {
                    // Example mutation: Change the particle type to "Sand" when clicked
                    *particle = Particle::new("Sand");
                }
            }
        });
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
    const ZOOM_IN_FACTOR: f32 = 0.98;
    const ZOOM_OUT_FACTOR: f32 = 1.02;

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

fn pan_camera(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    let mut transform = camera_query.single_mut()?;
    if keys.pressed(KeyCode::KeyW) {
        transform.translation.y += 2.;
    }

    if keys.pressed(KeyCode::KeyA) {
        transform.translation.x -= 2.;
    }

    if keys.pressed(KeyCode::KeyS) {
        transform.translation.y -= 2.;
    }

    if keys.pressed(KeyCode::KeyD) {
        transform.translation.x += 2.;
    }
    Ok(())
}

fn reset(mut particle_query: Query<&mut Particle>) {
    particle_query.iter_mut().for_each(|mut particle| {
        if particle.name == "Sand" {
            *particle = Particle::new("Water");
        }
    });
}
