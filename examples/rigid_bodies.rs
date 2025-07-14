use std::time::Duration;

use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_falling_sand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default()
                .with_length_unit(8.0)
                .with_spatial_refresh_frequency(Duration::from_millis(50))
                .with_gravity(Vec2::NEG_Y * 50.0),
            PhysicsDebugPlugin::default(),
        ))
        .init_resource::<SpawnWaterParticles>()
        .init_resource::<SpawnSandParticles>()
        .init_resource::<CursorPosition>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                pan_camera,
                zoom_camera,
                update_cursor_position,
                float_rigid_bodies,
                setup_boundary.run_if(resource_not_exists::<BoundaryReady>),
                spawn_water_particles.run_if(
                    resource_exists::<BoundaryReady>.and(resource_exists::<SpawnWaterParticles>),
                ),
                spawn_sand_particles.run_if(
                    resource_exists::<BoundaryReady>.and(resource_exists::<SpawnSandParticles>),
                ),
                toggle_spawn_sand_particles.run_if(input_just_pressed(KeyCode::F1)),
                toggle_spawn_water_particles.run_if(input_just_pressed(KeyCode::F2)),
                spawn_ball.run_if(input_just_pressed(MouseButton::Left)),
                reset.run_if(input_just_pressed(KeyCode::KeyR)),
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_END_Y: i32 = 150;
const RIGID_BODY_SIZE: f32 = 2.5;

fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}

#[derive(Resource)]
struct BoundaryReady;

#[derive(Default, Resource)]
struct SpawnWaterParticles;

#[derive(Default, Resource)]
struct SpawnSandParticles;

#[derive(Component)]
struct MainCamera;

#[derive(Clone, PartialEq, PartialOrd, Debug, Default, Component)]
pub struct DemoRigidBody {
    pub size: f32,
}

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
        ParticleTypeId::new("Dirt Wall"),
        ColorProfile::new(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ),));

    commands.spawn((
        LiquidBundle::new(
            ParticleTypeId::new("Water"),
            Density(750),
            Velocity::new(1, 3),
            5,
            ColorProfile::new(vec![Color::Srgba(Srgba::hex("#0B80AB80").unwrap())]),
        ),
        // If momentum effects are desired, insert the marker component.
        Momentum::default(),
    ));

    commands.spawn((
        MovableSolidBundle::new(
            ParticleTypeId::new("Sand"),
            Density(1250),
            Velocity::new(1, 3),
            ColorProfile::new(vec![
                Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
                Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
            ]),
        ),
        Momentum::default(),
    ));

    let instructions_text = "F1: Toggle sand spawn\n\
        F2: Toggle water spawn\n\
        Left Mouse: Spawn ball at cursor\"\n\
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

fn setup_boundary(mut commands: Commands, particle_type_map: Res<ParticleTypeMap>) {
    if particle_type_map.contains("Dirt Wall") {
        for y in 0..BOUNDARY_END_Y {
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
                Transform::from_xyz(x as f32, -(BOUNDARY_END_Y as f32), 0.0),
            ));
        }
        commands.insert_resource(BoundaryReady);
    }
}

fn spawn_water_particles(mut commands: Commands) {
    let center_x = (BOUNDARY_START_X + BOUNDARY_END_X) / 2;
    let spawn_y = -(BOUNDARY_END_Y as f32) - 10.0;

    let radius = 3;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let base_x = center_x as f32 + dx as f32;
                let y = spawn_y + dy as f32 + 200.0;

                commands.spawn((
                    Particle::new("Water"),
                    Transform::from_xyz(base_x + 75.0, y, 0.0),
                ));
            }
        }
    }
}

fn spawn_sand_particles(mut commands: Commands) {
    let center_x = (BOUNDARY_START_X + BOUNDARY_END_X) / 2;
    let spawn_y = -(BOUNDARY_END_Y as f32) - 10.0;

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
            }
        }
    }
}

fn toggle_spawn_water_particles(
    mut commands: Commands,
    debug_map: Option<Res<SpawnWaterParticles>>,
) {
    if debug_map.is_some() {
        commands.remove_resource::<SpawnWaterParticles>();
    } else {
        commands.init_resource::<SpawnWaterParticles>();
    }
}

fn toggle_spawn_sand_particles(mut commands: Commands, debug_map: Option<Res<SpawnSandParticles>>) {
    if debug_map.is_some() {
        commands.remove_resource::<SpawnSandParticles>();
    } else {
        commands.init_resource::<SpawnSandParticles>();
    }
}

fn reset(
    mut commands: Commands,
    mut ev_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>,
    demo_rigid_body_query: Query<Entity, With<DemoRigidBody>>,
) {
    ev_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
    demo_rigid_body_query.iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cursor_position: Res<CursorPosition>,
) -> Result {
    commands.spawn((
        RigidBody::Dynamic,
        Collider::circle(RIGID_BODY_SIZE),
        Transform::from_xyz(cursor_position.current.x, cursor_position.current.y, 0.),
        DemoRigidBody {
            size: RIGID_BODY_SIZE,
        },
        TransformInterpolation,
        GravityScale(1.0),
        Mesh2d(meshes.add(Circle::new(RIGID_BODY_SIZE))),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::rgba_u8(246, 174, 45, 255)))),
    ));
    Ok(())
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

fn float_rigid_bodies(
    mut rigid_body_query: Query<(
        &RigidBody,
        &Transform,
        &mut GravityScale,
        &mut LinearVelocity,
    )>,
    liquid_query: Query<&Particle, With<Liquid>>,
    chunk_map: Res<ParticleMap>,
) {
    let damping_factor = 0.95;
    rigid_body_query.iter_mut().for_each(
        |(rigid_body, transform, mut gravity_scale, mut linear_velocity)| {
            if rigid_body == &RigidBody::Dynamic {
                if let Some(entity) = chunk_map.get(&IVec2::new(
                    transform.translation.x as i32,
                    transform.translation.y as i32,
                )) {
                    if liquid_query.contains(*entity) {
                        linear_velocity.y *= damping_factor;
                        if linear_velocity.y.abs() < 0.001 {
                            linear_velocity.y = 0.0;
                        }
                        gravity_scale.0 = -1.0;
                    }
                } else {
                    gravity_scale.0 = 1.0;
                }
            }
        },
    );
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
