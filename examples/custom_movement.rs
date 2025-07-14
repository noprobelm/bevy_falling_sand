mod utils;

use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow};
use bevy_falling_sand::prelude::*;
use std::time::Duration;
use utils::{
    boundary::SetupBoundary,
    brush::{ParticleSpawnList, SelectedBrushParticle},
    states::AppState,
    status_ui::MovementSourceText,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default().with_spatial_refresh_frequency(Duration::from_millis(40)),
            FallingSandDebugPlugin,
            utils::states::StatesPlugin,
            utils::cursor::CursorPlugin,
            utils::instructions::InstructionsPlugin::default(),
            utils::brush::BrushPlugin::default(),
            utils::status_ui::StatusUIPlugin,
        ))
        .init_resource::<MaxVelocitySelection>()
        .add_systems(Startup, (setup, utils::camera::setup_camera))
        .add_systems(
            Update,
            (
                utils::particles::toggle_debug_map.run_if(input_just_pressed(KeyCode::F1)),
                utils::particles::toggle_debug_dirty_rects.run_if(input_just_pressed(KeyCode::F2)),
                utils::particles::change_movement_source.run_if(input_just_pressed(KeyCode::F3)),
                utils::camera::zoom_camera.run_if(in_state(AppState::Canvas)),
                utils::camera::pan_camera,
                utils::particles::ev_clear_dynamic_particles
                    .run_if(input_just_pressed(KeyCode::KeyR)),
                bump_velocity.run_if(input_just_pressed(KeyCode::KeyV)),
                utils::brush::handle_alt_release_without_egui,
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_START_Y: i32 = -150;
const BOUNDARY_END_Y: i32 = 150;

#[derive(Resource, Clone, Debug)]
struct MaxVelocitySelection(u8);

impl Default for MaxVelocitySelection {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component, Clone, Debug)]
struct MaxVelocitySelectionText;

fn setup(
    mut commands: Commands,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) -> Result {
    commands.remove_resource::<DebugParticleMap>();
    commands.remove_resource::<DebugDirtyRects>();

    let mut window = primary_window.single_mut()?;
    window.cursor_options.visible = false;

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

    let setup_boundary = SetupBoundary::from_corners(
        IVec2::new(BOUNDARY_START_X, BOUNDARY_START_Y),
        IVec2::new(BOUNDARY_END_X, BOUNDARY_END_Y),
        ParticleType::new("Dirt Wall"),
    );
    commands.queue(setup_boundary);

    // Setup particle spawn list for brush system
    let particles = vec![
        Particle::new("Moore Neighborhood Particle (no momentum)"),
        Particle::new("Moore Neighborhood Particle (with momentum)"),
        Particle::new("Neumann Neighborhood Particle (no momentum)"),
        Particle::new("Neumann Neighborhood Particle (with momentum)"),
        Particle::new("Downward diagonal (no momentum)"),
        Particle::new("Downward diagonal (with momentum)"),
    ];
    commands.insert_resource(ParticleSpawnList::new(particles));
    commands.insert_resource(SelectedBrushParticle(Particle::new(
        "Moore Neighborhood Particle (no momentum)",
    )));

    let instructions_text = "Left mouse: Spawn/despawn particles\n\
        Right mouse: Cycle particle type\n\
        V: Bump max velocity\n\
        F1: Show/hide particle chunk map\n\
        F2: Show/hide dirty rectangles\n\
        F3: Change movement logic (Particles vs. Chunks)\n\
        R: Reset";

    let panel_id = utils::instructions::spawn_instructions_panel(&mut commands, instructions_text);
    commands.entity(panel_id).with_children(|parent| {
        let style = TextFont::default();
        parent.spawn((
            MaxVelocitySelectionText,
            Text::new("Maximum velocity: 1"),
            style.clone(),
        ));
        parent.spawn((
            MovementSourceText,
            Text::new("Movement Source: Particles"),
            style.clone(),
        ));
    });
    Ok(())
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
