use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseWheel},
    prelude::*,
};
use bevy_falling_sand::prelude::*;
use bevy_turborand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandMinimalPlugin,
            FallingSandMovementPlugin,
            FallingSandColorPlugin,
        ))
        .init_state::<ParticleTypeOneMutationState>()
        .init_state::<ParticleTypeTwoMutationState>()
        .add_systems(Startup, setup)
        .add_systems(Update, zoom_camera)
        .add_systems(
            Update,
            (spawn_boundary.run_if(resource_not_exists::<BoundaryReady>),),
        )
        .add_systems(Update, spawn_particles)
        .add_systems(
            Update,
            mutate_particle_type_one.run_if(input_just_pressed(KeyCode::F1)),
        )
        .add_systems(
            Update,
            mutate_particle_type_two.run_if(input_just_pressed(KeyCode::F2)),
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
        ChangesColorBlueprint(ChangesColor::new(0.1)),
    ));

    // The instructions and modes are rendered on the left-hand side in a column.
    let instructions_text = "F1: Change particle type one\n\
        F2: Change particle type two\n";
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
                ParticleTypeOneText,
                Text::new("Particle Type: Water"),
                style.clone(),
            ));
            parent.spawn((
                ParticleTypeTwoText,
                Text::new("Particle Type: Sand"),
                style.clone(),
            ));
        });
}

fn spawn_boundary(mut commands: Commands, particle_type_map: Res<ParticleTypeMap>) {
    if particle_type_map.contains("Dirt Wall") && particle_type_map.contains("Smoke") {
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
        particle.name = format!("{new_state}");
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
        particle.name = format!("{new_state}");
    });
    next_state.set(new_state.clone());
    let new_text = format!("Particle Type: {}", new_state.clone());
    for mut particle_type_text in particle_type_text_query.iter_mut() {
        (**particle_type_text).clone_from(&new_text);
    }
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
