use bevy::{input::common_conditions::input_just_pressed, prelude::*};
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
        .init_resource::<SpawnParticles>()
        .init_state::<ParticleTypeMutationState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (spawn_boundary.run_if(resource_not_exists::<BoundaryReady>),),
        )
        .add_systems(Update, spawn_particles)
        .add_systems(
            Update,
            mutate_particle_state.run_if(input_just_pressed(KeyCode::F1)),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -100;
const BOUNDARY_END_X: i32 = 100;
const BOUNDARY_START_Y: i32 = -100;
const BOUNDARY_END_Y: i32 = 100;

fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}

#[derive(Resource)]
struct BoundaryReady;

#[derive(Default, Resource)]
struct SpawnParticles;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct TotalParticleCountText;

#[derive(Component)]
struct MutationParticle;

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParticleTypeMutationState {
    #[default]
    Smoke,
    Water,
    Sand,
    DirtWall,
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParticleStateMutationState {
    #[default]
    Gas,
    MovableSolid,
    Liquid,
    Solid,
    Wall,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: 0.11,
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
    let instructions_text = "F1: Toggle particle spawning\n\
        F2: Show/Hide particle chunk map\n\
        F3: Show/Hide \"dirty rectangles\"\n\
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
            parent.spawn((
                TotalParticleCountText,
                Text::new("Total Particles: "),
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
        for x in BOUNDARY_START_X + 50..BOUNDARY_END_X - 50 {
            for y in BOUNDARY_START_Y + 50..BOUNDARY_END_Y - 50 {
                if rng.chance(0.5) {
                    commands.spawn((
                        Particle::new("Smoke"),
                        Transform::from_xyz(x as f32, -(y as f32), 0.0),
                        MutationParticle,
                    ));
                }
            }
        }
    }
}

fn mutate_particle_state(
    mut mutate_particle_query: Query<&mut Particle, With<MutationParticle>>,
    state: Res<State<ParticleTypeMutationState>>,
    mut next_state: ResMut<NextState<ParticleTypeMutationState>>,
) {
    match state.get() {
        ParticleTypeMutationState::Smoke => {
            mutate_particle_query.iter_mut().for_each(|mut particle| {
                particle.name = String::from("Water");
            });
            next_state.set(ParticleTypeMutationState::Water);
        }
        ParticleTypeMutationState::Water => {
            mutate_particle_query.iter_mut().for_each(|mut particle| {
                particle.name = String::from("Sand");
            });
            next_state.set(ParticleTypeMutationState::Sand);
        }
        ParticleTypeMutationState::Sand => {
            mutate_particle_query.iter_mut().for_each(|mut particle| {
                particle.name = String::from("Dirt Wall");
            });
            next_state.set(ParticleTypeMutationState::DirtWall);
        }
        ParticleTypeMutationState::DirtWall => {
            mutate_particle_query.iter_mut().for_each(|mut particle| {
                particle.name = String::from("Smoke");
            });
            next_state.set(ParticleTypeMutationState::Smoke);
        }
    }
}
