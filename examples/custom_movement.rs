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
        .init_resource::<BrushRadius>()
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_camera, pan_camera))
        .add_systems(
            Update,
            (
                update_cursor_position,
                spawn_boundary.run_if(resource_not_exists::<BoundaryReady>),
                draw_brush,
                spawn_particles.run_if(input_pressed(MouseButton::Left)),
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

#[derive(Resource, Clone, Debug)]
pub struct BrushRadius {
    radius: f32,
}

impl Default for BrushRadius {
    fn default() -> Self {
        Self { radius: 5.0 }
    }
}

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) -> Result {
    let mut window = primary_window.single_mut()?;
    window.cursor_options.visible = false;

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
        ParticleType::new("Moore Neighborhood Particle (no momentum)"),
        MovementPriorityBlueprint(MovementPriority::from(vec![vec![
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(1, -1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
            IVec2::new(-1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ]])),
        DensityBlueprint(Density(100)),
        VelocityBlueprint(Velocity::new(1, 3)),
        ColorProfileBlueprint(ColorProfile::new(vec![
            Color::srgba(0.22, 0.11, 0.16, 1.0),
            Color::srgba(0.24, 0.41, 0.56, 1.0),
            Color::srgba(0.67, 0.74, 0.55, 1.0),
            Color::srgba(0.91, 0.89, 0.71, 1.0),
            Color::srgba(0.95, 0.61, 0.43, 1.0),
        ])),
    ));

    commands.spawn((
        ParticleType::new("Moore Neighborhood Particle (with momentum)"),
        MovementPriorityBlueprint(MovementPriority::from(vec![vec![
            IVec2::new(-1, -1),
            IVec2::new(1, -1),
        ]])),
        DensityBlueprint(Density(100)),
        VelocityBlueprint(Velocity::new(1, 3)),
        ColorProfileBlueprint(ColorProfile::new(vec![
            Color::srgba(0.22, 0.11, 0.16, 1.0),
            Color::srgba(0.24, 0.41, 0.56, 1.0),
            Color::srgba(0.67, 0.74, 0.55, 1.0),
            Color::srgba(0.91, 0.89, 0.71, 1.0),
            Color::srgba(0.95, 0.61, 0.43, 1.0),
        ])),
        MomentumBlueprint::default(),
    ));

    let instructions_text = "Left Mouse: Mutate water into sand within radius\n\
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
    Ok(())
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

fn spawn_particles(
    mut commands: Commands,
    cursor_position: Res<CursorPosition>,
    brush_radius: Res<BrushRadius>,
) {
    let center = cursor_position.current;
    let radius = brush_radius.radius as i32;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let spawn_x = center.x + dx as f32;
                let spawn_y = center.y + dy as f32;

                commands.spawn((
                    Particle::new("Moore Neighborhood Particle (with momentum)"),
                    Transform::from_xyz(spawn_x, spawn_y, 0.0),
                ));
            }
        }
    }
}

fn draw_brush(
    cursor_position: Res<CursorPosition>,
    spatial_query_radius: Res<BrushRadius>,
    mut gizmos: Gizmos,
) {
    gizmos.circle_2d(
        cursor_position.current,
        spatial_query_radius.radius,
        Color::WHITE,
    );
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

fn reset(mut particle_query: Query<&mut Particle>) {}
