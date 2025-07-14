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
        .init_resource::<MaxVelocitySelection>()
        .init_state::<ParticleMovementSelectionState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_camera, pan_camera))
        .add_systems(
            Update,
            (
                update_cursor_position,
                spawn_boundary.run_if(resource_not_exists::<BoundaryReady>),
                draw_brush,
                spawn_particles.run_if(input_pressed(MouseButton::Left)),
                cycle_selected_movement_state.run_if(input_just_pressed(KeyCode::F1)),
                bump_velocity.run_if(input_just_pressed(KeyCode::F2)),
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

#[derive(Default, Resource, Clone, Debug)]
struct BoundaryReady;

#[derive(Default, Resource, Clone, Debug)]
struct CursorPosition {
    pub current: Vec2,
}

#[derive(Resource, Clone, Debug)]
struct MaxVelocitySelection(u8);

impl Default for MaxVelocitySelection {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Resource, Clone, Debug)]
struct BrushRadius {
    radius: f32,
}

impl Default for BrushRadius {
    fn default() -> Self {
        Self { radius: 5.0 }
    }
}

#[derive(Component, Clone, Debug)]
struct ParticleMovementSelectionText;

#[derive(Component, Clone, Debug)]
struct MaxVelocitySelectionText;

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParticleMovementSelectionState {
    #[default]
    MooreNoMomentum,
    MooreMomentum,
    NeumannNoMomentum,
    NeumannMomentum,
    DownwardDiagonalNoMomentum,
    DownwardDiagonalMomentum,
}

impl std::fmt::Display for ParticleMovementSelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticleMovementSelectionState::MooreNoMomentum => {
                f.write_str("Moore Neighborhood Particle (no momentum)")
            }
            ParticleMovementSelectionState::MooreMomentum => {
                f.write_str("Moore Neighborhood Particle (with momentum)")
            }
            ParticleMovementSelectionState::NeumannNoMomentum => {
                f.write_str("Neumann Neighborhood Particle (no momentum)")
            }
            ParticleMovementSelectionState::NeumannMomentum => {
                f.write_str("Neumann Neighborhood Particle (with momentum)")
            }
            ParticleMovementSelectionState::DownwardDiagonalNoMomentum => {
                f.write_str("Downward diagonal (no momentum)")
            }
            ParticleMovementSelectionState::DownwardDiagonalMomentum => {
                f.write_str("Downward diagonal (with momentum)")
            }
        }
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

    let color_profile = ColorProfile::new(vec![
        Color::srgba(0.22, 0.11, 0.16, 1.0),
        Color::srgba(0.24, 0.41, 0.56, 1.0),
        Color::srgba(0.67, 0.74, 0.55, 1.0),
        Color::srgba(0.91, 0.89, 0.71, 1.0),
        Color::srgba(0.95, 0.61, 0.43, 1.0),
    ]);
    let density = Density(100);
    let velocity = Velocity::new(1, 1);

    commands.spawn((WallBundle::new(
        ParticleType::new("Dirt Wall"),
        ColorProfile::new(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ),));

    commands.spawn((
        ParticleType::new("Moore Neighborhood Particle (no momentum)"),
        Movement::from(vec![vec![
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(1, -1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
            IVec2::new(-1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ]]),
        density,
        velocity,
        color_profile.clone(),
    ));
    commands.spawn((
        ParticleType::new("Moore Neighborhood Particle (with momentum)"),
        Movement::from(vec![vec![
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(1, -1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
            IVec2::new(-1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ]]),
        density,
        velocity,
        color_profile.clone(),
        Momentum::default(),
    ));
    commands.spawn((
        ParticleType::new("Neumann Neighborhood Particle (no momentum)"),
        Movement::from(vec![vec![
            IVec2::new(0, -1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
            IVec2::new(0, 1),
        ]]),
        density,
        velocity,
        color_profile.clone(),
    ));
    commands.spawn((
        ParticleType::new("Neumann Neighborhood Particle (with momentum)"),
        Movement::from(vec![vec![
            IVec2::new(0, -1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
            IVec2::new(0, 1),
        ]]),
        density,
        velocity,
        color_profile.clone(),
        Momentum::default(),
    ));
    commands.spawn((
        ParticleType::new("Downward diagonal (no momentum)"),
        Movement::from(vec![vec![IVec2::new(-1, -1), IVec2::new(1, -1)]]),
        density,
        velocity,
        color_profile.clone(),
    ));
    commands.spawn((
        ParticleType::new("Downward diagonal (with momentum)"),
        Movement::from(vec![vec![IVec2::new(-1, -1), IVec2::new(1, -1)]]),
        density,
        velocity,
        color_profile.clone(),
        Momentum::default(),
    ));

    let instructions_text = "F1: Cycle particle movement rules\n\
        F2: Bump max velocity\n\
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
            parent.spawn((
                ParticleMovementSelectionText,
                Text::new("Selected movement type: Moore Neighborhood Particle (no momentum)"),
            ));
            parent.spawn((MaxVelocitySelectionText, Text::new("Maximum velocity: 1")));
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
    particle_movement_selection_state: Res<State<ParticleMovementSelectionState>>,
) {
    let name = particle_movement_selection_state.get().to_string();
    let center = cursor_position.current;
    let radius = brush_radius.radius as i32;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let spawn_x = center.x + dx as f32;
                let spawn_y = center.y + dy as f32;

                commands.spawn((
                    Particle::new(name.as_str()),
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

fn cycle_selected_movement_state(
    mut particle_query: Query<&mut Particle, Without<Wall>>,
    particle_movement_selection_state: Res<State<ParticleMovementSelectionState>>,
    mut next_particle_movement_selection_state: ResMut<NextState<ParticleMovementSelectionState>>,
    mut particle_movement_selection_text: Query<&mut Text, With<ParticleMovementSelectionText>>,
) {
    let new_state = match particle_movement_selection_state.get() {
        ParticleMovementSelectionState::MooreNoMomentum => {
            ParticleMovementSelectionState::MooreMomentum
        }
        ParticleMovementSelectionState::MooreMomentum => {
            ParticleMovementSelectionState::NeumannNoMomentum
        }
        ParticleMovementSelectionState::NeumannNoMomentum => {
            ParticleMovementSelectionState::NeumannMomentum
        }
        ParticleMovementSelectionState::NeumannMomentum => {
            ParticleMovementSelectionState::DownwardDiagonalNoMomentum
        }
        ParticleMovementSelectionState::DownwardDiagonalNoMomentum => {
            ParticleMovementSelectionState::DownwardDiagonalMomentum
        }
        ParticleMovementSelectionState::DownwardDiagonalMomentum => {
            ParticleMovementSelectionState::MooreNoMomentum
        }
    };
    let new_text = format!("Selected Movement Type: {new_state}");
    for mut particle_movement_selection_text in particle_movement_selection_text.iter_mut() {
        (**particle_movement_selection_text).clone_from(&new_text);
    }
    next_particle_movement_selection_state.set(new_state.clone());
    particle_query
        .iter_mut()
        .for_each(|mut particle| particle.name = format!("{new_state}"));
}

fn bump_velocity(
    mut ev_reset_particle_chidlren: EventWriter<ResetParticleChildrenEvent>,
    mut particle_type_query: Query<(Entity, &mut Velocity), With<ParticleType>>,
    mut velocity_selection: ResMut<MaxVelocitySelection>,
    mut velocity_selection_text: Query<&mut Text, With<MaxVelocitySelectionText>>,
) {
    if velocity_selection.0 < 5 {
        velocity_selection.0 += 1;
    } else {
        velocity_selection.0 = 1;
    }
    particle_type_query
        .iter_mut()
        .for_each(|(entity, mut velocity_bp)| {
            velocity_bp.set_max_velocity(velocity_selection.0);
            ev_reset_particle_chidlren.write(ResetParticleChildrenEvent { entity });
        });
    for mut velocity_selection_text in velocity_selection_text.iter_mut() {
        (**velocity_selection_text)
            .clone_from(&format!("Maximum velocity: {}", velocity_selection.0));
    }
}

fn reset(mut ev_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>) {
    ev_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
}
