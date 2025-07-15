mod utils;

use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{egui, EguiContextPass, EguiContexts, EguiPlugin};
use bevy_falling_sand::prelude::*;
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
            FallingSandPlugin::default().with_spatial_refresh_frequency(Duration::from_millis(20)),
            FallingSandDebugPlugin,
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
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
        .add_systems(Startup, (setup, utils::camera::setup_camera))
        .add_systems(
            EguiContextPass,
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
                    .before(ParticleSimulationSet),
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
struct DefaultFire(GasBundle, ChangesColor, Burns, Name);

impl Default for DefaultFire {
    fn default() -> Self {
        DefaultFire(
            GasBundle::new(
                ParticleType::new("FIRE"),
                Density(450),
                Velocity::new(1, 3),
                1,
                ColorProfile::new(vec![
                    Color::Srgba(Srgba::hex("#FF5900FF").unwrap()),
                    Color::Srgba(Srgba::hex("#FF9100FF").unwrap()),
                    Color::Srgba(Srgba::hex("#FFCF00FF").unwrap()),
                    Color::Srgba(Srgba::hex("#C74A05FF").unwrap()),
                ]),
            ),
            ChangesColor::new(0.1),
            Burns::new(
                Duration::from_secs(1),
                Duration::from_millis(100),
                Some(0.5),
                None,
                None,
                Some(Fire {
                    burn_radius: 1.5,
                    chance_to_spread: 0.01,
                    destroys_on_spread: false,
                }),
                true,
            ),
            Name::new("FIRE"),
        )
    }
}

#[derive(Resource)]
struct DefaultFlammableGas(GasBundle, ChangesColor, Burns, Name);

impl Default for DefaultFlammableGas {
    fn default() -> Self {
        DefaultFlammableGas(
            GasBundle::new(
                ParticleType::new("Flammable Gas"),
                Density(200),
                Velocity::new(1, 1),
                1,
                ColorProfile::new(vec![
                    Color::Srgba(Srgba::hex("#40621880").unwrap()),
                    Color::Srgba(Srgba::hex("#4A731C80").unwrap()),
                ]),
            ),
            ChangesColor::new(0.1),
            Burns::new(
                Duration::from_secs(1),
                Duration::from_millis(50),
                Some(0.5),
                None,
                Some(ColorProfile::new(vec![
                    Color::Srgba(Srgba::hex("#FF5900").unwrap()),
                    Color::Srgba(Srgba::hex("#FF0000").unwrap()),
                    Color::Srgba(Srgba::hex("#FF9900").unwrap()),
                    Color::Srgba(Srgba::hex("#FFCF00").unwrap()),
                    Color::Srgba(Srgba::hex("#FFE808").unwrap()),
                ])),
                Some(Fire {
                    burn_radius: 4.,
                    chance_to_spread: 0.175,
                    destroys_on_spread: true,
                }),
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

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    default_flammable_gas: Res<DefaultFlammableGas>,
    default_fire: Res<DefaultFire>,
) {
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
        Name::new("Smoke"),
    ));

    commands.spawn((
        default_flammable_gas.0.clone(),
        default_flammable_gas.1,
        default_flammable_gas.2.clone(),
        default_flammable_gas.3.clone(),
    ));

    commands.spawn((
        default_fire.0.clone(),
        default_fire.1,
        default_fire.2.clone(),
        default_fire.3.clone(),
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
        parent.spawn((
            FpsText,
            Text::new("FPS: --"),
            style.clone(),
        ));
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
    mut ev_reset_particle_children: EventWriter<ResetParticleChildrenEvent>,
    particle_type_map: Res<ParticleTypeMap>,
    mut burns_query: Query<&mut Burns, With<ParticleType>>,
    mut color_profile_query: Query<&mut ColorProfile, With<ParticleType>>,
    mut commands: Commands,
    default_fire: Res<DefaultFire>,
    default_flammable_gas: Res<DefaultFlammableGas>,
) {
    let fire_entity = *particle_type_map.get(&"FIRE".to_string()).unwrap();
    let flammable_gas_entity = *particle_type_map.get(&"Flammable Gas".to_string()).unwrap();

    egui::Window::new("Particle Properties").show(contexts.ctx_mut(), |ui| {
        {
            ui.heading("🔥 Fire Settings");
            ui.separator();
            ui.add_space(4.0);

            let mut burns = burns_query.get_mut(fire_entity).unwrap();
            let mut fire_color = color_profile_query.get_mut(fire_entity).unwrap();
            let mut fire_updated = false;

            if let Some(fire) = burns.spreads.as_mut() {
                let mut slider_value = fire.burn_radius;
                let response =
                    ui.add(egui::Slider::new(&mut slider_value, 0.0..=50.0).text("Fire Radius"));
                if response.drag_stopped() {
                    fire.burn_radius = slider_value;
                    fire_updated = true;
                }
                let mut slider_value = fire.chance_to_spread;
                let response = ui.add(
                    egui::Slider::new(&mut slider_value, 0.0..=1.0)
                        .text("Chance to spread (per frame)"),
                );
                if response.drag_stopped() {
                    fire.chance_to_spread = slider_value;
                    fire_updated = true;
                }
                let mut checkbox_enabled = fire.destroys_on_spread;
                let response = ui.checkbox(&mut checkbox_enabled, "Destroy on Spread");
                if response.changed() {
                    fire.destroys_on_spread = checkbox_enabled;
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

            ui.add_space(8.0);

            let mut chance_destroy = burns.chance_destroy_per_tick.unwrap_or(0.0);
            if ui
                .add(
                    egui::Slider::new(&mut chance_destroy, 0.0..=1.0)
                        .text("Chance Destroy per Tick"),
                )
                .drag_stopped()
            {
                if chance_destroy > 0.0 {
                    burns.chance_destroy_per_tick = Some(chance_destroy);
                } else {
                    burns.chance_destroy_per_tick = None;
                }
                fire_updated = true;
            }

            ui.add_space(8.0);

            let mut smoke_enabled = burns.reaction.is_some();
            if ui.checkbox(&mut smoke_enabled, "Smoke").changed() {
                if smoke_enabled {
                    burns.reaction = Some(Reacting {
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
                    burns.reaction = Some(Reacting {
                        produces: Particle::new("Smoke"),
                        chance_to_produce,
                    });
                    fire_updated = true;
                }
            }

            ui.add_space(8.0);

            render_color_profile_editor(ui, "Fire Colors", &mut fire_color, &mut fire_updated);

            ui.add_space(8.0);

            if ui.button("🔄 Reset Fire to Default").clicked() {
                commands.spawn((
                    default_fire.0.clone(),
                    default_fire.1,
                    default_fire.2.clone(),
                    default_fire.3.clone(),
                ));
            }

            if fire_updated {
                ev_reset_particle_children.write(ResetParticleChildrenEvent {
                    entity: fire_entity,
                });
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

            if let Some(fire) = flammable_gas_burns.spreads.as_mut() {
                let mut slider_value = fire.burn_radius;
                let response = ui
                    .add(egui::Slider::new(&mut slider_value, 0.0..=50.0).text("Gas Fire Radius"));
                if response.drag_stopped() {
                    fire.burn_radius = slider_value;
                    flammable_gas_updated = true;
                }
                let mut slider_value = fire.chance_to_spread;
                let response = ui.add(
                    egui::Slider::new(&mut slider_value, 0.0..=1.0)
                        .text("Chance to spread (per frame)"),
                );
                if response.changed() {
                    fire.chance_to_spread = slider_value;
                    flammable_gas_updated = true;
                }
                let mut checkbox_enabled = fire.destroys_on_spread;
                let response = ui.checkbox(&mut checkbox_enabled, "Destroy on Spread");
                if response.changed() {
                    fire.destroys_on_spread = checkbox_enabled;
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

            let mut chance_destroy = flammable_gas_burns.chance_destroy_per_tick.unwrap_or(0.0);
            if ui
                .add(
                    egui::Slider::new(&mut chance_destroy, 0.0..=1.0)
                        .text("Chance Destroy per Tick"),
                )
                .drag_stopped()
            {
                if chance_destroy > 0.0 {
                    flammable_gas_burns.chance_destroy_per_tick = Some(chance_destroy);
                } else {
                    flammable_gas_burns.chance_destroy_per_tick = None;
                }
                flammable_gas_updated = true;
            }

            render_color_profile_editor(
                ui,
                "Fire Colors",
                flammable_gas_burns.color.as_mut().unwrap(),
                &mut flammable_gas_updated,
            );

            ui.add_space(8.0);

            if ui.button("🔄 Reset Flammable Gas to Default").clicked() {
                commands.spawn((
                    default_flammable_gas.0.clone(),
                    default_flammable_gas.1,
                    default_flammable_gas.2.clone(),
                    default_flammable_gas.3.clone(),
                ));
            }

            if flammable_gas_updated {
                ev_reset_particle_children.write(ResetParticleChildrenEvent {
                    entity: flammable_gas_entity,
                });
            }
        }
    });
}

pub fn render_color_profile_editor(
    ui: &mut egui::Ui,
    label: &str,
    color_profile: &mut ColorProfile,
    updated: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.button("➕").clicked() {
            color_profile.add_color(Color::srgba_u8(255, 255, 255, 255));
            *updated = true;
        }
    });

    let palette_snapshot = color_profile.palette.clone();
    let palette_len = palette_snapshot.len();

    let mut to_remove: Option<usize> = None;
    let mut to_change: Option<(usize, Color)> = None;

    let widgets_per_row = 4;

    ui.vertical(|ui| {
        for chunk in palette_snapshot.chunks(widgets_per_row) {
            ui.horizontal(|ui| {
                for color in chunk {
                    let color_index = palette_snapshot.iter().position(|c| c == color).unwrap();

                    let srgba = color.to_srgba();
                    let (r, g, b, a) = (
                        (srgba.red * 255.) as u8,
                        (srgba.green * 255.) as u8,
                        (srgba.blue * 255.) as u8,
                        (srgba.alpha * 255.) as u8,
                    );

                    let mut color32 = egui::Color32::from_rgba_unmultiplied(r, g, b, a);

                    ui.horizontal(|ui| {
                        if ui.color_edit_button_srgba(&mut color32).changed() {
                            to_change = Some((
                                color_index,
                                Color::srgba_u8(color32.r(), color32.g(), color32.b(), color32.a()),
                            ));
                        }
                        let can_remove = palette_len > 1;
                        if ui
                            .add_enabled(can_remove, egui::Button::new("❌"))
                            .clicked()
                        {
                            to_remove = Some(color_index);
                        }
                    });
                }
            });
        }
    });

    if let Some((index, new_color)) = to_change {
        color_profile.edit_color(index, new_color);
        *updated = true;
    }
    if let Some(index) = to_remove {
        color_profile.remove_color(index);
        *updated = true;
    }
}
