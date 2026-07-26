mod utils;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_rand::prelude::{GlobalRng, WyRand};
use utils::status_ui::{FpsText, MovementSourceText, StatusUIPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            FallingSandMinimalPlugin::default(),
            FallingSandMovementPlugin,
            FallingSandRenderPlugin,
            FallingSandDebugPlugin,
            utils::states::StatesPlugin,
            StatusUIPlugin,
        ))
        .init_state::<ParticleTypeOneMutationState>()
        .init_state::<ParticleTypeTwoMutationState>()
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
                utils::camera::zoom_camera,
                utils::camera::pan_camera,
                utils::camera::smooth_zoom,
                utils::instructions::toggle_standalone_instructions,
                mutate_particle_type_one.run_if(input_just_pressed(MouseButton::Left)),
                mutate_particle_type_two.run_if(input_just_pressed(MouseButton::Right)),
                update_movement_source_text,
            ),
        )
        .add_systems(
            PostUpdate,
            spawn_particles.before(ParticleSystems::Simulation),
        )
        .run();
}

const START_X: i32 = -200;
const END_X: i32 = 200;
const START_Y: i32 = -200;
const END_Y: i32 = 200;

#[derive(Copy, Clone, Resource)]
struct MutationParticleIds {
    dirt_wall: ParticleTypeId,
    water: ParticleTypeId,
    sand: ParticleTypeId,
    smoke: ParticleTypeId,
}

impl Default for MutationParticleIds {
    fn default() -> Self {
        Self {
            dirt_wall: ParticleTypeId::new(),
            water: ParticleTypeId::new(),
            sand: ParticleTypeId::new(),
            smoke: ParticleTypeId::new(),
        }
    }
}

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
    let ids = MutationParticleIds::default();
    commands.insert_resource(ids);
    commands.remove_resource::<DebugParticleMap>();
    commands.remove_resource::<DebugDirtyRects>();
    commands.spawn((
        ParticleType::from_id(ids.dirt_wall),
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ));

    commands.spawn((
        ParticleType::from_id(ids.water),
        Density(750),
        Speed::new(0, 3),
        ColorProfile::palette(vec![Color::Srgba(Srgba::hex("#0B80AB80").unwrap())]),
        Movement::from(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
            vec![IVec2::X, IVec2::NEG_X],
            vec![IVec2::new(2, 0), IVec2::new(-2, 0)],
            vec![IVec2::new(3, 0), IVec2::new(-3, 0)],
            vec![IVec2::new(4, 0), IVec2::new(-4, 0)],
        ]),
        // If momentum effects are desired, insert the marker component.
        Momentum::default(),
        ParticleResistor(0.75),
    ));
    commands.spawn((
        ParticleType::from_id(ids.sand),
        Density(1250),
        Speed::new(5, 10),
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#FFEB8A").unwrap()),
            Color::Srgba(Srgba::hex("#F2E06B").unwrap()),
        ]),
        Movement::from(vec![
            vec![IVec2::NEG_Y],
            vec![IVec2::NEG_ONE, IVec2::new(1, -1)],
        ]),
        Momentum::default(),
    ));
    commands.spawn((
        ParticleType::from_id(ids.smoke),
        Density(275),
        Speed::new(0, 1),
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#706966").unwrap()),
            Color::Srgba(Srgba::hex("#858073").unwrap()),
        ]),
        Movement::from(vec![
            vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)],
            vec![IVec2::new(0, 2), IVec2::new(0, -2)],
        ]),
    ));

    let instructions_text = "Left mouse: Mutate particle type one\n\
        Right Mouse: Mutate particle type two\n\
        F1: Show/hide particle chunk map\n\
        F2: Show/hide dirty rectangles\n\
        F3: Change movement logic (Particles vs. Chunks)\n\
        H: Hide/Show this help";

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

fn spawn_particles(
    mut spawn_writer: MessageWriter<SpawnParticleSignal>,
    time: Res<Time>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    ids: Res<MutationParticleIds>,
) {
    if time.elapsed_secs() < 0.5 {
        let x_range = ((END_X - START_X) as f32 * 0.5) as i32;
        let y_range = ((END_Y - START_Y) as f32 * 0.5) as i32;

        for x in START_X + 50..START_X + 50 + x_range {
            for y in START_Y + 50..START_Y + 50 + y_range {
                let position = IVec2::new(x, -y);
                if rng.chance(0.5) {
                    spawn_writer.write(
                        SpawnParticleSignal::new(ids.water, position).with_on_spawn(|cmd| {
                            cmd.insert(MutationParticleOne);
                        }),
                    );
                } else if rng.chance(0.5) {
                    spawn_writer.write(SpawnParticleSignal::new(ids.sand, position).with_on_spawn(
                        |cmd| {
                            cmd.insert(MutationParticleTwo);
                        },
                    ));
                }
            }
        }
    }
}

fn mutate_particle_type_one(
    mut mutate_particle_query: Query<&mut AttachedToParticleType, With<MutationParticleOne>>,
    registry: Res<ParticleTypeRegistry>,
    ids: Res<MutationParticleIds>,
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
    let new_id = match new_state {
        ParticleTypeOneMutationState::Smoke => ids.smoke,
        ParticleTypeOneMutationState::DirtWall => ids.dirt_wall,
        ParticleTypeOneMutationState::Sand => ids.sand,
        ParticleTypeOneMutationState::Water => ids.water,
    };
    if let Some(&new_parent) = registry.get(new_id) {
        mutate_particle_query.iter_mut().for_each(|mut attached| {
            attached.0 = new_parent;
        });
    }
    next_state.set(new_state.clone());
    let new_text = format!("Particle Type: {new_state}");
    for mut particle_type_text in particle_type_text_query.iter_mut() {
        (**particle_type_text).clone_from(&new_text);
    }
}

fn mutate_particle_type_two(
    mut mutate_particle_query: Query<&mut AttachedToParticleType, With<MutationParticleTwo>>,
    registry: Res<ParticleTypeRegistry>,
    ids: Res<MutationParticleIds>,
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
    let new_id = match new_state {
        ParticleTypeTwoMutationState::Smoke => ids.smoke,
        ParticleTypeTwoMutationState::DirtWall => ids.dirt_wall,
        ParticleTypeTwoMutationState::Sand => ids.sand,
        ParticleTypeTwoMutationState::Water => ids.water,
    };
    if let Some(&new_parent) = registry.get(new_id) {
        mutate_particle_query.iter_mut().for_each(|mut attached| {
            attached.0 = new_parent;
        });
    }
    next_state.set(new_state.clone());
    let new_text = format!("Particle Type: {new_state}");
    for mut particle_type_text in particle_type_text_query.iter_mut() {
        (**particle_type_text).clone_from(&new_text);
    }
}

fn update_movement_source_text(
    movement_source: Res<State<MovementSystemState>>,
    mut movement_source_text: Query<&mut Text, With<MovementSourceText>>,
) {
    let source_text = format!("Movement Source: {:?}", movement_source.get());

    for mut text in movement_source_text.iter_mut() {
        **text = source_text.clone();
    }
}

fn setup_framepace(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(60.0);
}
