mod utils;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_falling_sand::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_turborand::{DelegatedRng, GlobalRng};
use noise::{Fbm, NoiseFn, PerlinSurflet};
use utils::{
    brush::{ParticleSpawnList, SelectedBrushParticle},
    states::AppState,
    status_ui::{
        BrushStateText, BrushTypeText, FpsText, MovementSourceText, SelectedParticleText,
        TotalParticleCountText,
    },
};

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
            utils::brush::BrushPlugin::default(),
            utils::cursor::CursorPlugin,
            utils::instructions::InstructionsPlugin::default(),
            utils::status_ui::StatusUIPlugin,
        ))
        .init_resource::<TotalParticleCount>()
        .init_resource::<SpawnParticles>()
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
                reset_noise.run_if(input_just_pressed(KeyCode::KeyR)),
                utils::brush::handle_alt_release_without_egui,
            ),
        )
        .run();
}

#[derive(Default, Resource)]
struct SpawnParticles;

fn setup(mut commands: Commands, mut rng: ResMut<GlobalRng>) {
    commands.remove_resource::<DebugParticleMap>();
    commands.remove_resource::<DebugDirtyRects>();

    commands.spawn((
        ParticleType::new("Dirt Wall"),
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ));

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
        // Makes Water resistant to displacement by other particles.
        ParticleResistor(0.75),
        // If momentum effects are desired, insert the marker component.
        Momentum::default(),
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
    ));

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
        SPACE: Sample particle under cursor\n\
        LALT + mouse wheel: Change brush size\n\
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

    spawn_noise(&mut commands, &mut rng);
}

fn reset_noise(
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    mut despawn_writer: MessageWriter<DespawnAllParticlesSignal>,
) {
    despawn_writer.write(DespawnAllParticlesSignal);
    spawn_noise(&mut commands, &mut rng);
}

fn spawn_noise(commands: &mut Commands, rng: &mut GlobalRng) {
    let seed = rng.u32(0..u32::MAX);

    let basic_multi = Fbm::<PerlinSurflet>::new(seed);

    let map = noise::utils::PlaneMapBuilder::<_, 2>::new_fn(|point| basic_multi.get(point))
        .set_size(256, 256)
        .set_x_bounds(0.0, 1.0)
        .set_y_bounds(0.0, 1.0)
        .build();

    let (grid_width, grid_height) = map.size();

    let colors = &[
        Srgba::hex("#A0674B").unwrap(),
        Srgba::hex("#B8805D").unwrap(),
        Srgba::hex("#D8D8D8").unwrap(),
        Srgba::hex("#A8A8A8").unwrap(),
        Srgba::hex("#787878").unwrap(),
        Srgba::hex("#000000").unwrap(),
        Srgba::hex("#FFFF00").unwrap(),
    ];
    for x in 0..grid_width {
        for y in 0..grid_height {
            let val = map.get_value(x, y);
            let color = if val < -0.5 {
                colors[5]
            } else if val < -0.05 {
                colors[0]
            } else if val < 0.00 {
                colors[1]
            } else if val < 0.05 {
                colors[2]
            } else if val < 0.15 {
                colors[3]
            } else if val < 0.5 {
                colors[4]
            } else {
                continue;
            };

            if val < -0.5 {
                continue;
            }
            commands.spawn((
                Transform::from_xyz(
                    (x as i32 - grid_width as i32 / 2) as f32,
                    (y as i32 - grid_height as i32 / 2) as f32,
                    0.,
                ),
                Sprite {
                    color: Color::Srgba(color),
                    ..Default::default()
                },
                Particle::new("Dirt Wall"),
                ForceColor(Color::Srgba(color)),
            ));
        }
    }
}

fn setup_framepace(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(60.0);
}
