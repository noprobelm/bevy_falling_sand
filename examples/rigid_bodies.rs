mod utils;

use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;
use utils::{
    boundary::{SetupBoundary, Sides},
    brush::{ParticleSpawnList, SelectedBrushParticle},
    cursor::CursorPosition,
    states::AppState,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default()
                .with_length_unit(8.0)
                .with_spatial_refresh_frequency(Duration::from_millis(50))
                .with_gravity(Vec2::NEG_Y * 50.0),
            FallingSandDebugPlugin,
            PhysicsDebugPlugin::default(),
            utils::states::StatesPlugin,
            utils::brush::BrushPlugin::default(),
            utils::cursor::CursorPlugin,
            utils::instructions::InstructionsPlugin::default(),
            utils::status_ui::StatusUIPlugin,
        ))
        .add_systems(Startup, (setup, utils::camera::setup_camera))
        .add_systems(
            Update,
            (
                utils::particles::toggle_debug_map.run_if(input_just_pressed(KeyCode::F1)),
                utils::particles::toggle_debug_dirty_rects.run_if(input_just_pressed(KeyCode::F2)),
                utils::particles::change_movement_source.run_if(input_just_pressed(KeyCode::F3)),
                utils::camera::zoom_camera.run_if(in_state(AppState::Canvas)),
                utils::camera::pan_camera,
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

#[derive(Clone, PartialEq, PartialOrd, Debug, Default, Component)]
pub struct DemoRigidBody {
    pub size: f32,
}

fn setup(mut commands: Commands) {
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

    let instructions_text = "F1: Toggle sand spawn\n\
        F2: Toggle water spawn\n\
        Left Mouse: Spawn ball at cursor\"\n\
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
        });
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cursor_position: Res<CursorPosition>,
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
                if let Some(entity) = chunk_map.get(&IVec2::new(
                    transform.translation.x as i32,
                    transform.translation.y as i32,
                )) {
                    if liquid_query.contains(*entity) {
                        linear_velocity.y *= damping_factor;
                        if linear_velocity.y.abs() < 0.001 {
                            linear_velocity.y = 0.0;
                        }
                        gravity_scale.0 = -1.0;
                    }
                } else {
                    gravity_scale.0 = 1.0;
                }
            }
        },
    );
}
