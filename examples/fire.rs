mod utils;

use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use bevy_falling_sand::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use utils::{
    boundary::SetupBoundary,
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
            FallingSandPlugin::default(),
            FallingSandDebugPlugin,
            EguiPlugin::default(),
            utils::states::StatesPlugin,
            utils::brush::BrushPlugin::default(),
            utils::cursor::CursorPlugin,
            utils::instructions::InstructionsPlugin::default(),
            utils::status_ui::StatusUIPlugin,
            utils::gui::GuiPlugin,
        ))
        .init_resource::<SpawnFlammableGasParticles>()
        .init_resource::<CursorPosition>()
        .init_resource::<DefaultFire>()
        .init_resource::<DefaultFlammableGas>()
        .add_systems(
            Startup,
            (setup, utils::camera::setup_camera, setup_framepace),
        )
        .add_systems(
            EguiPrimaryContextPass,
            render_fire_settings_gui.run_if(resource_exists::<RenderGUI>),
        )
        .add_systems(
            Update,
            (
                utils::particles::toggle_debug_map.run_if(input_just_pressed(KeyCode::F1)),
                utils::particles::toggle_debug_dirty_rects.run_if(input_just_pressed(KeyCode::F2)),
                utils::particles::change_movement_source.run_if(input_just_pressed(KeyCode::F3)),
                utils::camera::zoom_camera.run_if(in_state(AppState::Canvas)),
                utils::camera::pan_camera.run_if(in_state(AppState::Canvas)),
                spawn_flammable_gas_particles
                    .run_if(resource_exists::<SpawnFlammableGasParticles>)
                    .before(ParticleSystems::Simulation),
                toggle_spawn_flamable_gas_particles
                    .run_if(input_just_pressed(KeyCode::F4))
                    .run_if(in_state(AppState::Canvas)),
                toggle_render_gui.run_if(input_just_pressed(KeyCode::KeyH)),
                utils::particles::ev_clear_dynamic_particles
                    .run_if(input_just_pressed(KeyCode::KeyR))
                    .run_if(in_state(AppState::Canvas)),
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_START_Y: i32 = -150;
const BOUNDARY_END_Y: i32 = 150;

#[derive(Clone, Resource)]
struct DefaultFire(
    ParticleType,
    Density,
    Speed,
    ColorProfile,
    Movement,
    Flammable,
    Name,
);

impl Default for DefaultFire {
    fn default() -> Self {
        let mut neighbors = vec![vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]];
        for i in 0..1 {
            neighbors.push(vec![IVec2::X * (i + 2), IVec2::NEG_X * (i + 2)]);
        }
        DefaultFire(
            ParticleType::new("FIRE"),
            Density(450),
            Speed::new(0, 3),
            ColorProfile::palette(vec![
                Color::Srgba(Srgba::hex("#FF5900FF").unwrap()),
                Color::Srgba(Srgba::hex("#FF9100FF").unwrap()),
                Color::Srgba(Srgba::hex("#FFCF00FF").unwrap()),
                Color::Srgba(Srgba::hex("#C74A05FF").unwrap()),
            ]),
            Movement::from(neighbors),
            Flammable::new(
                Duration::from_secs(1),
                Duration::from_millis(100),
                0.5,
                None,
                0.01,
                true,
                1.0,
                false,
                true,
            ),
            Name::new("FIRE"),
        )
    }
}

#[derive(Resource)]
struct DefaultFlammableGas(
    ParticleType,
    Density,
    Speed,
    ColorProfile,
    Movement,
    Flammable,
    Name,
);

impl Default for DefaultFlammableGas {
    fn default() -> Self {
        let mut neighbors = vec![vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]];
        for i in 0..1 {
            neighbors.push(vec![
                IVec2::X * (i + 2) as i32,
                IVec2::NEG_X * (i + 2) as i32,
            ]);
        }
        DefaultFlammableGas(
            ParticleType::new("Flammable Gas"),
            Density(200),
            Speed::new(0, 1),
            ColorProfile::palette(vec![
                Color::Srgba(Srgba::hex("#40621880").unwrap()),
                Color::Srgba(Srgba::hex("#4A731C80").unwrap()),
            ]),
            Movement::from(neighbors),
            Flammable::new(
                Duration::from_secs(1),
                Duration::from_millis(50),
                0.5,
                None,
                0.175,
                true,
                1.0,
                true,
                false,
            ),
            Name::new("Flammable Gas"),
        )
    }
}

#[derive(Default, Resource)]
struct RenderGUI;

#[derive(Default, Resource)]
struct SpawnFlammableGasParticles;

#[derive(Default, Resource, Clone, Debug)]
pub struct CursorPosition {
    pub current: Vec2,
}

fn setup(
    mut commands: Commands,
    default_flammable_gas: Res<DefaultFlammableGas>,
    default_fire: Res<DefaultFire>,
) {
    commands.remove_resource::<DebugParticleMap>();
    commands.remove_resource::<DebugDirtyRects>();

    commands.spawn((
        ParticleType::new("Dirt Wall"),
        ColorProfile::palette(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ));

    {
        let mut neighbors = vec![vec![IVec2::Y, IVec2::new(1, 1), IVec2::new(-1, 1)]];
        for i in 0..1 {
            neighbors.push(vec![
                IVec2::X * (i + 2) as i32,
                IVec2::NEG_X * (i + 2) as i32,
            ]);
        }
        commands.spawn((
            ParticleType::new("Smoke"),
            Density(275),
            Speed::new(0, 1),
            ColorProfile::palette(vec![
                Color::Srgba(Srgba::hex("#706966").unwrap()),
                Color::Srgba(Srgba::hex("#858073").unwrap()),
            ]),
            Movement::from(neighbors),
            Name::new("Smoke"),
        ));
    }

    commands.spawn((
        default_flammable_gas.0.clone(),
        default_flammable_gas.1,
        default_flammable_gas.2,
        default_flammable_gas.3.clone(),
        default_flammable_gas.4.clone(),
        default_flammable_gas.5.clone(),
        default_flammable_gas.6.clone(),
    ));

    commands.spawn((
        default_fire.0.clone(),
        default_fire.1,
        default_fire.2,
        default_fire.3.clone(),
        default_fire.4.clone(),
        default_fire.5.clone(),
        default_fire.6.clone(),
    ));

    let setup_boundary = SetupBoundary::from_corners(
        IVec2::new(BOUNDARY_START_X, BOUNDARY_START_Y),
        IVec2::new(BOUNDARY_END_X, BOUNDARY_END_Y),
        ParticleType::new("Dirt Wall"),
    )
    .with_thickness(2);
    commands.queue(setup_boundary);

    commands.insert_resource(ParticleSpawnList::new(vec![
        Particle::new("FIRE"),
        Particle::new("Flammable Gas"),
    ]));
    commands.insert_resource(SelectedBrushParticle(Particle::new("FIRE")));

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
        F4: Toggle flammable gas stream\n\
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
}

fn spawn_flammable_gas_particles(mut commands: Commands) {
    let center_x = (BOUNDARY_START_X + BOUNDARY_END_X) / 2;
    let center_y = (BOUNDARY_START_Y + BOUNDARY_END_Y) / 2;

    let radius = 10;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let spawn_x = center_x as f32 + dx as f32;
                let spawn_y = center_y as f32 + dy as f32;

                commands.spawn((
                    Particle::new("Flammable Gas"),
                    Transform::from_xyz(spawn_x, spawn_y, 0.0),
                ));
            }
        }
    }
}

fn toggle_spawn_flamable_gas_particles(
    mut commands: Commands,
    debug_map: Option<Res<SpawnFlammableGasParticles>>,
) {
    if debug_map.is_some() {
        commands.remove_resource::<SpawnFlammableGasParticles>();
    } else {
        commands.init_resource::<SpawnFlammableGasParticles>();
    }
}

fn toggle_render_gui(mut commands: Commands, render_gui: Option<Res<RenderGUI>>) {
    if render_gui.is_some() {
        commands.remove_resource::<RenderGUI>();
    } else {
        commands.init_resource::<RenderGUI>();
    }
}

fn render_fire_settings_gui(
    mut contexts: EguiContexts,
    mut ev_reset_particle_children: MessageWriter<SyncParticleTypeChildrenSignal>,
    particle_type_map: Res<ParticleTypeRegistry>,
    mut burns_query: Query<&mut Flammable, With<ParticleType>>,
    mut commands: Commands,
    default_fire: Res<DefaultFire>,
    default_flammable_gas: Res<DefaultFlammableGas>,
) {
    let fire_entity = *particle_type_map.get(&"FIRE".to_string()).unwrap();
    let flammable_gas_entity = *particle_type_map.get(&"Flammable Gas".to_string()).unwrap();

    egui::Window::new("Particle Properties").show(contexts.ctx_mut().unwrap(), |ui| {
        {
            ui.heading("🔥 Fire Settings");
            ui.separator();
            ui.add_space(4.0);

            let mut burns = burns_query.get_mut(fire_entity).unwrap();
            let mut fire_updated = false;

            {
                let mut slider_value = burns.chance_to_ignite;
                let response = ui.add(
                    egui::Slider::new(&mut slider_value, 0.0..=1.0)
                        .text("Chance to ignite (per contact)"),
                );
                if response.drag_stopped() {
                    burns.chance_to_ignite = slider_value;
                    fire_updated = true;
                }
                let mut checkbox_enabled = burns.spreads_fire;
                let response = ui.checkbox(&mut checkbox_enabled, "Spreads Fire");
                if response.changed() {
                    burns.spreads_fire = checkbox_enabled;
                    fire_updated = true;
                }
            }

            ui.add_space(8.0);

            let mut burns_duration = burns.duration.as_secs_f32();
            if ui
                .add(egui::Slider::new(&mut burns_duration, 0.1..=60.0).text("Burn Duration (s)"))
                .drag_stopped()
            {
                burns.duration = Duration::from_secs_f32(burns_duration);
                fire_updated = true;
            }

            let max_tick_ms = burns.duration.as_millis().max(1) as f32;
            let mut tick_rate_ms = burns.tick_rate.as_millis() as f32;
            if ui
                .add(egui::Slider::new(&mut tick_rate_ms, 0.0..=max_tick_ms).text("Tick Rate (ms)"))
                .drag_stopped()
            {
                burns.tick_rate =
                    Duration::from_millis(tick_rate_ms.clamp(0.0, max_tick_ms) as u64);
                fire_updated = true;
            }
            ui.add_space(8.0);

            if ui
                .add(
                    egui::Slider::new(&mut burns.chance_despawn_per_tick, 0.0..=1.0)
                        .text("Chance Destroy per Tick"),
                )
                .drag_stopped()
            {
                fire_updated = true;
            }

            ui.add_space(8.0);

            let mut smoke_enabled = burns.reaction.is_some();
            if ui.checkbox(&mut smoke_enabled, "Smoke").changed() {
                if smoke_enabled {
                    burns.reaction = Some(BurnProduct {
                        produces: Particle::new("Smoke"),
                        chance_to_produce: 0.5,
                    });
                } else {
                    burns.reaction = None;
                }
                fire_updated = true;
            }

            if smoke_enabled {
                let chance = burns
                    .reaction
                    .as_ref()
                    .map(|r| r.chance_to_produce)
                    .unwrap_or(0.5);

                let mut chance_to_produce = chance;

                if ui
                    .add(
                        egui::Slider::new(&mut chance_to_produce, 0.0..=1.0)
                            .text("Chance to produce smoke (per frame)"),
                    )
                    .drag_stopped()
                {
                    burns.reaction = Some(BurnProduct {
                        produces: Particle::new("Smoke"),
                        chance_to_produce,
                    });
                    fire_updated = true;
                }
            }

            ui.add_space(8.0);

            if ui.button("🔄 Reset Fire to Default").clicked() {
                commands.spawn((
                    default_fire.0.clone(),
                    default_fire.1,
                    default_fire.2,
                    default_fire.3.clone(),
                    default_fire.4.clone(),
                    default_fire.5.clone(),
                    default_fire.6.clone(),
                ));
            }

            if fire_updated {
                ev_reset_particle_children.write(
                    SyncParticleTypeChildrenSignal::from_parent_handle(fire_entity),
                );
            }
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        {
            ui.heading("💨 Flammable Gas Settings");
            ui.separator();
            ui.add_space(4.0);

            let mut flammable_gas_burns = burns_query.get_mut(flammable_gas_entity).unwrap();
            let mut flammable_gas_updated = false;

            {
                let mut slider_value = flammable_gas_burns.chance_to_ignite;
                let response = ui.add(
                    egui::Slider::new(&mut slider_value, 0.0..=1.0)
                        .text("Chance to ignite (per contact)"),
                );
                if response.drag_stopped() {
                    flammable_gas_burns.chance_to_ignite = slider_value;
                    flammable_gas_updated = true;
                }
                let mut checkbox_enabled = flammable_gas_burns.spreads_fire;
                let response = ui.checkbox(&mut checkbox_enabled, "Spreads Fire");
                if response.changed() {
                    flammable_gas_burns.spreads_fire = checkbox_enabled;
                    flammable_gas_updated = true;
                }
            }

            ui.add_space(8.0);

            let mut burns_duration = flammable_gas_burns.duration.as_secs_f32();
            if ui
                .add(egui::Slider::new(&mut burns_duration, 0.1..=60.0).text("Burn Duration (s)"))
                .drag_stopped()
            {
                flammable_gas_burns.duration = Duration::from_secs_f32(burns_duration);
                flammable_gas_updated = true;
            }

            let max_tick_ms = flammable_gas_burns.duration.as_millis().max(1) as f32;
            let mut tick_rate_ms = flammable_gas_burns.tick_rate.as_millis() as f32;
            if ui
                .add(egui::Slider::new(&mut tick_rate_ms, 0.0..=max_tick_ms).text("Tick Rate (ms)"))
                .drag_stopped()
            {
                flammable_gas_burns.tick_rate =
                    Duration::from_millis(tick_rate_ms.clamp(0.0, max_tick_ms) as u64);
                flammable_gas_updated = true;
            }
            ui.add_space(8.0);

            if ui
                .add(
                    egui::Slider::new(&mut flammable_gas_burns.chance_despawn_per_tick, 0.0..=1.0)
                        .text("Chance Destroy per Tick"),
                )
                .drag_stopped()
            {
                flammable_gas_updated = true;
            }

            ui.add_space(8.0);

            if ui.button("🔄 Reset Flammable Gas to Default").clicked() {
                commands.spawn((
                    default_flammable_gas.0.clone(),
                    default_flammable_gas.1,
                    default_flammable_gas.2,
                    default_flammable_gas.3.clone(),
                    default_flammable_gas.4.clone(),
                    default_flammable_gas.5.clone(),
                    default_flammable_gas.6.clone(),
                ));
            }

            if flammable_gas_updated {
                ev_reset_particle_children.write(
                    SyncParticleTypeChildrenSignal::from_parent_handle(flammable_gas_entity),
                );
            }
        }
    });
}

fn setup_framepace(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(60.0);
}
