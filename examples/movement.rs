mod utils;

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorOptions, PrimaryWindow},
};
use bevy_falling_sand::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use utils::{
    brush::{ParticleSpawnList, SelectedBrushParticle},
    states::AppState,
    status_ui::{FpsText, MovementSourceText},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            FallingSandPlugin::default().with_map_size(16),
            FallingSandDebugPlugin,
            utils::states::StatesPlugin,
            utils::cursor::CursorPlugin,
            utils::instructions::InstructionsPlugin::default(),
            utils::brush::BrushPlugin::default(),
            utils::status_ui::StatusUIPlugin,
        ))
        .init_resource::<MaxVelocitySelection>()
        .add_systems(
            Startup,
            (setup, utils::camera::setup_camera, setup_framepace),
        )
        .add_systems(
            PreUpdate,
            utils::particles::disable_chunk_loading
                .after(ChunkSystems::Loading)
                .run_if(run_once),
        )
        .add_systems(
            Update,
            (
                utils::particles::toggle_debug_map.run_if(input_just_pressed(KeyCode::F1)),
                utils::particles::toggle_debug_dirty_rects.run_if(input_just_pressed(KeyCode::F2)),
                utils::particles::change_movement_source.run_if(input_just_pressed(KeyCode::F3)),
                utils::camera::zoom_camera.run_if(in_state(AppState::Canvas)),
                utils::camera::pan_camera,
                utils::camera::smooth_zoom,
                utils::particles::msgw_clear_dynamic_particles
                    .run_if(input_just_pressed(KeyCode::KeyR)),
                bump_velocity.run_if(input_just_pressed(KeyCode::KeyV)),
                utils::brush::handle_alt_release_without_egui,
            ),
        )
        .run();
}

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
    mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) -> Result {
    commands.remove_resource::<DebugParticleMap>();
    commands.remove_resource::<DebugDirtyRects>();

    primary_cursor_options.visible = false;

    let color_profile = ColorProfile::palette(vec![
        Color::srgba(0.22, 0.11, 0.16, 1.0),
        Color::srgba(0.24, 0.41, 0.56, 1.0),
        Color::srgba(0.67, 0.74, 0.55, 1.0),
        Color::srgba(0.91, 0.89, 0.71, 1.0),
        Color::srgba(0.95, 0.61, 0.43, 1.0),
    ]);
    let density = Density(100);
    let speed = Speed::new(0, 1);

    commands.spawn((
        ParticleType::new("Dirt Wall"),
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ));

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
        speed,
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
        speed,
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
        speed,
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
        speed,
        color_profile.clone(),
        Momentum::default(),
    ));
    commands.spawn((
        ParticleType::new("Downward diagonal (no momentum)"),
        Movement::from(vec![vec![IVec2::new(-1, -1), IVec2::new(1, -1)]]),
        density,
        speed,
        color_profile.clone(),
    ));
    commands.spawn((
        ParticleType::new("Downward diagonal (with momentum)"),
        Movement::from(vec![vec![IVec2::new(-1, -1), IVec2::new(1, -1)]]),
        density,
        speed,
        color_profile.clone(),
        Momentum::default(),
    ));

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
        parent.spawn((FpsText, Text::new("FPS: --"), style.clone()));
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
    mut ev_reset_particle_chidlren: MessageWriter<SyncParticleTypeChildrenSignal>,
    mut particle_type_query: Query<(Entity, &mut Speed), With<ParticleType>>,
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
            velocity_bp.set_max_speed(velocity_selection.0);
            ev_reset_particle_chidlren
                .write(SyncParticleTypeChildrenSignal::from_parent_handle(entity));
        });
    for mut velocity_selection_text in velocity_selection_text.iter_mut() {
        (**velocity_selection_text)
            .clone_from(&format!("Maximum velocity: {}", velocity_selection.0));
    }
}

fn setup_framepace(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(60.0);
}
