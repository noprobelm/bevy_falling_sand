mod utils;

use avian2d::prelude::*;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use utils::{
    boundary::{SetupBoundary, Sides},
    brush::{BrushInput, BrushKeybindings, ParticleSpawnList, SelectedBrushParticle},
    cursor::Cursor,
    states::AppState,
    status_ui::{BrushStateText, BrushTypeText, FpsText, MovementSourceText, SelectedParticleText},
};

fn main() {
    let brush_bindings = BrushKeybindings {
        sample_button: BrushInput::Key(KeyCode::ControlLeft),
        ..default()
    };
    App::new()
        .add_plugins((
            DefaultPlugins,
            FramepacePlugin,
            FallingSandPlugin::default()
                .with_length_unit(8.0)
                .with_gravity(Vec2::NEG_Y * 50.0),
            FallingSandDebugPlugin,
            utils::states::StatesPlugin,
            utils::brush::BrushPlugin::default().with_keybindings(brush_bindings),
            utils::cursor::CursorPlugin,
            utils::instructions::InstructionsPlugin::default(),
            utils::status_ui::StatusUIPlugin,
        ))
        .register_particle_sync_component::<Liquid>()
        .add_systems(
            Startup,
            (setup, utils::camera::setup_camera, setup_framepace),
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
                utils::brush::handle_alt_release_without_egui,
                utils::particles::ev_clear_dynamic_particles
                    .run_if(input_just_pressed(KeyCode::KeyR)),
                float_rigid_bodies,
                spawn_ball.run_if(input_just_pressed(KeyCode::Space)),
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_START_Y: i32 = -150;
const BOUNDARY_END_Y: i32 = 150;
const RIGID_BODY_SIZE: f32 = 2.5;

#[derive(Component, Clone, PartialEq, PartialOrd, Debug, Default)]
struct DemoRigidBody {
    pub size: f32,
}

#[derive(Component, Clone, PartialEq, PartialOrd, Debug, Default)]
struct Liquid;

fn setup(mut commands: Commands) {
    commands.spawn((
        ParticleType::new("Dirt Wall"),
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
        // Mark this particle type for inclusion in static rigid body mesh generation
        StaticRigidBodyParticle,
    ));
    commands.spawn((
        ParticleType::new("Sand"),
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
        // Mark this particle type for inclusion in static rigid body mesh generation
        StaticRigidBodyParticle,
    ));
    {
        commands.spawn((
            ParticleType::new("Water"),
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
            Momentum::default(),
            ParticleResistor(0.75),
            Liquid,
        ));
    }

    let setup_boundary = SetupBoundary::from_corners(
        IVec2::new(BOUNDARY_START_X, BOUNDARY_START_Y),
        IVec2::new(BOUNDARY_END_X, BOUNDARY_END_Y),
        ParticleType::new("Dirt Wall"),
    )
    .without_sides(vec![Sides::Top]);
    commands.queue(setup_boundary);

    commands.insert_resource(ParticleSpawnList::new(vec![
        Particle::new("Dirt Wall"),
        Particle::new("Sand"),
        Particle::new("Water"),
    ]));
    commands.insert_resource(SelectedBrushParticle(Particle::new("Dirt Wall")));

    let instructions_text = "Left mouse: Spawn/despawn particles\n\
        Right mouse: Cycle particle type\n\
        Middle Mouse: Cycle brush type\n\
        TAB: Toggle brush spawn/despawn\n\
        SPACE: Spawn dynamic rigid body\n\
        LALT + mouse wheel: Change brush size\n\
        LCTRL: Sample particle under cursor\n\
        H: Hide/Show this help\n\
        F1: Show/hide particle chunk map\n\
        F2: Show/hide \"dirty rectangles\"\n\
        F3: Change movement logic (Particles vs. Chunks)\n\
        R: Reset\n";

    let panel_id = utils::instructions::spawn_instructions_panel(&mut commands, instructions_text);

    commands.entity(panel_id).with_children(|parent| {
        let style = TextFont::default();
        parent.spawn((FpsText, Text::new("FPS: --"), style.clone()));
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
        parent.spawn((
            BrushTypeText,
            Text::new("Brush Type: Circle"),
            style.clone(),
        ));
        parent.spawn((
            MovementSourceText,
            Text::new("Movement Source: Particles"),
            style.clone(),
        ));
    });
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cursor_position: Res<Cursor>,
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
                if let Ok(Some(entity)) = chunk_map.get_copied(IVec2::new(
                    transform.translation.x as i32,
                    transform.translation.y as i32,
                )) {
                    if liquid_query.contains(entity) {
                        linear_velocity.y *= damping_factor;
                        if linear_velocity.y.abs() < 0.001 {
                            linear_velocity.y = 0.0;
                        }
                        gravity_scale.0 = -1.0;
                    }
                } else {
                    gravity_scale.set_if_neq(GravityScale(1.0));
                }
            }
        },
    );
}

fn setup_framepace(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(60.0);
}
