mod utils;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;
use bevy_turborand::prelude::*;
use utils::{
    boundary::SetupBoundary,
    status_ui::{FpsText, MovementSourceText, StatusUIPlugin},
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
            StatusUIPlugin,
        ))
        .init_state::<ParticleTypeOneMutationState>()
        .init_state::<ParticleTypeTwoMutationState>()
        .add_systems(Startup, (setup, utils::camera::setup_camera))
        .add_systems(
            Update,
            (
                utils::particles::toggle_debug_map.run_if(input_just_pressed(KeyCode::F1)),
                utils::particles::toggle_debug_dirty_rects.run_if(input_just_pressed(KeyCode::F2)),
                utils::particles::change_movement_source.run_if(input_just_pressed(KeyCode::F3)),
                utils::camera::zoom_camera,
                utils::camera::pan_camera,
                spawn_particles.before(ParticleSimulationSet),
                utils::instructions::toggle_standalone_instructions,
                update_movement_source_text,
            ),
        )
        .add_systems(
            Update,
            (
                mutate_particle_type_one.run_if(input_just_pressed(MouseButton::Left)),
                mutate_particle_type_two.run_if(input_just_pressed(MouseButton::Right)),
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_START_Y: i32 = -150;
const BOUNDARY_END_Y: i32 = 150;

#[derive(Component)]
struct ParticleTypeOneText;

#[derive(Component)]
struct ParticleTypeTwoText;

#[derive(Component)]
struct MutationParticleOne;

#[derive(Component)]
struct MutationParticleTwo;

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParticleTypeOneMutationState {
    Smoke,
    #[default]
    Water,
    Sand,
    DirtWall,
}

impl std::fmt::Display for ParticleTypeOneMutationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticleTypeOneMutationState::Smoke => f.write_str("Smoke"),
            ParticleTypeOneMutationState::Water => f.write_str("Water"),
            ParticleTypeOneMutationState::Sand => f.write_str("Sand"),
            ParticleTypeOneMutationState::DirtWall => f.write_str("Dirt Wall"),
        }
    }
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParticleTypeTwoMutationState {
    Smoke,
    #[default]
    Water,
    Sand,
    DirtWall,
}

impl std::fmt::Display for ParticleTypeTwoMutationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticleTypeTwoMutationState::Smoke => f.write_str("Smoke"),
            ParticleTypeTwoMutationState::Water => f.write_str("Water"),
            ParticleTypeTwoMutationState::Sand => f.write_str("Sand"),
            ParticleTypeTwoMutationState::DirtWall => f.write_str("Dirt Wall"),
        }
    }
}

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

    commands.spawn((
        GasBundle::new(
            ParticleType::new("Smoke"),
            Density(275),
            Velocity::new(1, 1),
            1,
            ColorProfile::new(vec![
                Color::Srgba(Srgba::hex("#706966").unwrap()),
                Color::Srgba(Srgba::hex("#858073").unwrap()),
            ]),
        ),
        ChangesColor::new(0.1),
    ));

    let setup_boundary = SetupBoundary::from_corners(
        IVec2::new(BOUNDARY_START_X, BOUNDARY_START_Y),
        IVec2::new(BOUNDARY_END_X, BOUNDARY_END_Y),
        ParticleType::new("Dirt Wall"),
    )
    .with_thickness(2);
    commands.queue(setup_boundary);

    let instructions_text = "Left mouse: Mutate particle type one\n\
        Right Mouse: Mutate particle type two\n\
        F1: Show/hide particle chunk map\n\
        F2: Show/hide dirty rectangles\n\
        F3: Change movement logic (Particles vs. Chunks)\n\
        H: Hide/Show this help\n\
        R: Reset";

    let panel_id = utils::instructions::setup_standalone_instructions(
        &mut commands,
        instructions_text,
        KeyCode::KeyH,
    );
    commands.entity(panel_id).with_children(|parent| {
        let style = TextFont::default();
        parent.spawn((FpsText, Text::new("FPS: --"), style.clone()));
        parent.spawn((
            MovementSourceText,
            Text::new("Movement Source: Particles"),
            style.clone(),
        ));
    });
}

fn spawn_particles(mut commands: Commands, time: Res<Time>, mut rng: ResMut<GlobalRng>) {
    if time.elapsed_secs() < 0.5 {
        let x_range = ((BOUNDARY_END_X - BOUNDARY_START_X) as f32 * 0.5) as i32;
        let y_range = ((BOUNDARY_END_Y - BOUNDARY_START_Y) as f32 * 0.5) as i32;

        for x in BOUNDARY_START_X + 50..BOUNDARY_START_X + 50 + x_range {
            for y in BOUNDARY_START_Y + 50..BOUNDARY_START_Y + 50 + y_range {
                if rng.chance(0.5) {
                    commands.spawn((
                        Particle::new("Water"),
                        Transform::from_xyz(x as f32, -(y as f32), 0.0),
                        MutationParticleOne,
                    ));
                } else if rng.chance(0.5) {
                    commands.spawn((
                        Particle::new("Sand"),
                        Transform::from_xyz(x as f32, -(y as f32), 0.0),
                        MutationParticleTwo,
                    ));
                }
            }
        }
    }
}

fn mutate_particle_type_one(
    mut mutate_particle_query: Query<&mut Particle, With<MutationParticleOne>>,
    state: Res<State<ParticleTypeOneMutationState>>,
    mut next_state: ResMut<NextState<ParticleTypeOneMutationState>>,
    mut particle_type_text_query: Query<&mut Text, With<ParticleTypeOneText>>,
) {
    let new_state = match state.get() {
        ParticleTypeOneMutationState::Smoke => ParticleTypeOneMutationState::DirtWall,
        ParticleTypeOneMutationState::DirtWall => ParticleTypeOneMutationState::Sand,
        ParticleTypeOneMutationState::Sand => ParticleTypeOneMutationState::Water,
        ParticleTypeOneMutationState::Water => ParticleTypeOneMutationState::Smoke,
    };
    mutate_particle_query.iter_mut().for_each(|mut particle| {
        particle.name = format!("{new_state}").into();
    });
    next_state.set(new_state.clone());
    let new_text = format!("Particle Type: {}", new_state.clone());
    for mut particle_type_text in particle_type_text_query.iter_mut() {
        (**particle_type_text).clone_from(&new_text);
    }
}

fn mutate_particle_type_two(
    mut mutate_particle_query: Query<&mut Particle, With<MutationParticleTwo>>,
    state: Res<State<ParticleTypeTwoMutationState>>,
    mut next_state: ResMut<NextState<ParticleTypeTwoMutationState>>,
    mut particle_type_text_query: Query<&mut Text, With<ParticleTypeTwoText>>,
) {
    let new_state = match state.get() {
        ParticleTypeTwoMutationState::Smoke => ParticleTypeTwoMutationState::DirtWall,
        ParticleTypeTwoMutationState::DirtWall => ParticleTypeTwoMutationState::Sand,
        ParticleTypeTwoMutationState::Sand => ParticleTypeTwoMutationState::Water,
        ParticleTypeTwoMutationState::Water => ParticleTypeTwoMutationState::Smoke,
    };
    mutate_particle_query.iter_mut().for_each(|mut particle| {
        particle.name = format!("{new_state}").into();
    });
    next_state.set(new_state.clone());
    let new_text = format!("Particle Type: {}", new_state.clone());
    for mut particle_type_text in particle_type_text_query.iter_mut() {
        (**particle_type_text).clone_from(&new_text);
    }
}

fn update_movement_source_text(
    movement_source: Res<State<MovementSource>>,
    mut movement_source_text: Query<&mut Text, With<MovementSourceText>>,
) {
    let source_text = format!("Movement Source: {:?}", movement_source.get());

    for mut text in movement_source_text.iter_mut() {
        **text = source_text.clone();
    }
}
