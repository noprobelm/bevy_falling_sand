mod utils;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;
use utils::{
    boundary::{SetupBoundary, Sides},
    brush::{BrushState, ParticleSpawnList, SelectedBrushParticle},
    instructions::spawn_instructions_panel,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandMinimalPlugin,
            FallingSandMovementPlugin,
            FallingSandColorPlugin,
            FallingSandDebugPlugin,
            utils::states::StatesPlugin,
            utils::brush::BrushPlugin::default(),
            utils::cursor::CursorPlugin,
            utils::instructions::InstructionsPlugin::new(),
        ))
        .init_resource::<TotalParticleCount>()
        .init_resource::<SpawnParticles>()
        .add_systems(Startup, (setup, utils::camera::setup_camera))
        .add_systems(
            Update,
            (
                update_total_particle_count_text.run_if(resource_exists::<TotalParticleCount>),
                update_brush_state_text,
                update_selected_particle_text,
                toggle_debug_map.run_if(input_just_pressed(KeyCode::F1)),
                toggle_debug_dirty_rects.run_if(input_just_pressed(KeyCode::F2)),
                utils::camera::zoom_camera.run_if(in_state(utils::states::AppState::Canvas)),
                utils::camera::pan_camera,
                utils::particles::reset_dynamic_particles.run_if(input_just_pressed(KeyCode::KeyR)),
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_START_Y: i32 = -100;
const BOUNDARY_END_Y: i32 = 50;

#[derive(Default, Resource)]
struct SpawnParticles;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct TotalParticleCountText;

#[derive(Component)]
struct BrushStateText;

#[derive(Component)]
struct SelectedParticleText;

fn setup(mut commands: Commands) {
    commands.remove_resource::<DebugParticleMap>();
    commands.remove_resource::<DebugDirtyRects>();

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
        Momentum::default(),
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
        Momentum::default(),
    ));

    commands.insert_resource(ParticleSpawnList::new(vec![
        Particle::new("Dirt Wall"),
        Particle::new("Sand"),
        Particle::new("Water"),
    ]));
    commands.insert_resource(SelectedBrushParticle(Particle::new("Dirt Wall")));

    let setup_boundary = SetupBoundary::from_corners(
        IVec2::new(BOUNDARY_START_X, BOUNDARY_START_Y),
        IVec2::new(BOUNDARY_END_X, BOUNDARY_END_Y),
        ParticleType::new("Dirt Wall"),
    )
    .without_sides(vec![Sides::Top]);
    commands.queue(setup_boundary);

    let instructions_text = "LMB: Spawn/despawn particles\n\
        RMB: Cycle particle type\n\
        TAB: Toggle brush spawn/despawn\n\
        H: Hide/Show this help\n\
        F1: Show/Hide particle chunk map\n\
        F2: Show/Hide \"dirty rectangles\"\n\
        R: Reset\n";

    let panel_id = spawn_instructions_panel(&mut commands, instructions_text);

    commands.entity(panel_id).with_children(|parent| {
        let style = TextFont::default();
        parent.spawn((
            TotalParticleCountText,
            Text::new("Total Particles: "),
            style.clone(),
        ));
        parent.spawn((
            BrushStateText,
            Text::new("Brush Mode: Spawn"),
            style.clone(),
        ));
        parent.spawn((
            SelectedParticleText,
            Text::new("Selected Particle: Sand"),
            style.clone(),
        ));
    });
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

fn update_brush_state_text(
    brush_state: Res<State<BrushState>>,
    mut brush_state_text: Query<&mut Text, With<BrushStateText>>,
) {
    let state_text = match brush_state.get() {
        BrushState::Spawn => "Brush Mode: Spawn",
        BrushState::Despawn => "Brush Mode: Despawn",
    };

    for mut text in brush_state_text.iter_mut() {
        **text = state_text.to_string();
    }
}

fn update_selected_particle_text(
    selected_particle: Res<SelectedBrushParticle>,
    mut selected_particle_text: Query<&mut Text, With<SelectedParticleText>>,
) {
    let particle_text = format!("Selected Particle: {}", selected_particle.0.name);

    for mut text in selected_particle_text.iter_mut() {
        **text = particle_text.clone();
    }
}
