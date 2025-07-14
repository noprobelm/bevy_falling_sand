use std::time::Duration;

use bevy::{
    input::{
        common_conditions::{input_just_pressed, input_pressed},
        mouse::MouseWheel,
    },
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContextPass, EguiContexts, EguiPlugin};
use bevy_falling_sand::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandPlugin::default().with_spatial_refresh_frequency(Duration::from_millis(40)),
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
        ))
        .init_state::<AppState>()
        .init_resource::<SpawnFlammableGasParticles>()
        .init_resource::<CursorPosition>()
        .init_resource::<DefaultFire>()
        .init_resource::<DefaultFlammableGas>()
        .add_systems(Startup, setup)
        .add_systems(Update, (zoom_camera, pan_camera))
        .add_systems(
            EguiContextPass,
            render_fire_settings_gui.run_if(resource_exists::<RenderGUI>),
        )
        .add_systems(
            Update,
            (
                update_app_state,
                update_cursor_position,
                spawn_boundary.run_if(resource_not_exists::<BoundaryReady>),
                spawn_fire
                    .run_if(input_pressed(MouseButton::Left))
                    .run_if(in_state(AppState::Canvas)),
                spawn_flammable_gas_particles.run_if(
                    resource_exists::<BoundaryReady>
                        .and(resource_exists::<SpawnFlammableGasParticles>),
                ),
                toggle_spawn_flamable_gas_particles
                    .run_if(input_just_pressed(KeyCode::F1))
                    .run_if(in_state(AppState::Canvas)),
                toggle_render_gui.run_if(input_just_pressed(KeyCode::KeyH)),
                reset
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

fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}

#[derive(States, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Canvas,
    Ui,
}

#[derive(Clone, Resource)]
struct DefaultFire(GasBundle, ChangesColor, Burns, Name);

impl Default for DefaultFire {
    fn default() -> Self {
        DefaultFire(
            GasBundle::new(
                ParticleTypeId::new("FIRE"),
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
                ParticleTypeId::new("Flammable Gas"),
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

#[derive(Resource)]
struct BoundaryReady;

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
        ParticleTypeId::new("Dirt Wall"),
        ColorProfile::new(vec![
            Color::Srgba(Srgba::hex("#916B4C").unwrap()),
            Color::Srgba(Srgba::hex("#73573D").unwrap()),
        ]),
    ),));

    commands.spawn((
        GasBundle::new(
            ParticleTypeId::new("Smoke"),
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

    let instructions_text = "F1: Toggle flammable gas stream\n\
        Left Mouse: Spawn fire at cursor\n\
        H: Show/hide settings GUI\n\
        R: Reset";
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

fn spawn_boundary(mut commands: Commands, particle_type_map: Res<ParticleTypeMap>) {
    if particle_type_map.contains("Dirt Wall") {
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

fn spawn_fire(mut commands: Commands, cursor_position: Res<CursorPosition>) {
    let center = cursor_position.current;
    let radius = 3;

    for dx in -radius..=radius {
        for dy in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let spawn_x = center.x + dx as f32;
                let spawn_y = center.y + dy as f32;

                commands.spawn((
                    Particle::new("FIRE"),
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

fn update_cursor_position(
    mut cursor_position: ResMut<CursorPosition>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Result {
    let (camera, camera_transform) = q_camera.single()?;

    let window = q_window.single()?;
    if let Some(world_position) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.unwrap().origin.truncate())
    {
        cursor_position.current = world_position;
    }
    Ok(())
}

fn zoom_camera(
    mut ev_scroll: EventReader<MouseWheel>,
    mut camera_query: Query<&mut Projection, With<MainCamera>>,
) {
    const ZOOM_IN_FACTOR: f32 = 0.98;
    const ZOOM_OUT_FACTOR: f32 = 1.02;

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

fn pan_camera(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
) -> Result {
    let mut transform = camera_query.single_mut()?;
    if keys.pressed(KeyCode::KeyW) {
        transform.translation.y += 2.;
    }

    if keys.pressed(KeyCode::KeyA) {
        transform.translation.x -= 2.;
    }

    if keys.pressed(KeyCode::KeyS) {
        transform.translation.y -= 2.;
    }

    if keys.pressed(KeyCode::KeyD) {
        transform.translation.x += 2.;
    }
    Ok(())
}

pub fn update_app_state(
    mut contexts: EguiContexts,
    app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    match app_state.get() {
        AppState::Ui => {
            let ctx = contexts.ctx_mut();
            if !ctx.is_pointer_over_area() {
                next_app_state.set(AppState::Canvas);
            }
        }
        AppState::Canvas => {
            let ctx = contexts.ctx_mut();
            if ctx.is_pointer_over_area() {
                next_app_state.set(AppState::Ui);
            }
        }
    }
}

fn reset(mut ev_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>) {
    ev_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
}

fn render_fire_settings_gui(
    mut contexts: EguiContexts,
    mut ev_reset_particle_children: EventWriter<ResetParticleChildrenEvent>,
    particle_type_map: Res<ParticleTypeMap>,
    mut burns_query: Query<&mut Burns, With<ParticleTypeId>>,
    mut color_profile_query: Query<&mut ColorProfile, With<ParticleTypeId>>,
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
