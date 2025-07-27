mod utils;

use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_falling_sand::prelude::*;
use bfs_assets::{FallingSandAssetsPlugin, ParticleDefinitionsAsset, ParticleDefinitionsHandle};
use bfs_core::{Particle, ParticleTypeMap};
use ron::ser::{to_string_pretty, PrettyConfig};
use std::collections::HashMap;
use utils::status_ui::{StatusUIPlugin, FpsText};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandMinimalPlugin,
            FallingSandReactionsPlugin,
            FallingSandMovementPlugin,
            FallingSandColorPlugin,
            FallingSandAssetsPlugin,
            StatusUIPlugin,
            utils::instructions::InstructionsPlugin::default(),
        ))
        .init_resource::<CurrentParticleType>()
        .init_resource::<CursorPosition>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                zoom_camera,
                pan_camera,
                update_cursor_position,
                setup_boundary.run_if(resource_not_exists::<BoundaryReady>),
                spawn_particle_at_cursor.run_if(resource_exists::<BoundaryReady>),
                cycle_particle_type.run_if(input_just_pressed(KeyCode::Tab)),
                save_particles.run_if(input_just_pressed(KeyCode::F1)),
                reset.run_if(input_just_pressed(KeyCode::KeyR)),
                check_asset_loading,
            ),
        )
        .run();
}

const BOUNDARY_START_X: i32 = -150;
const BOUNDARY_END_X: i32 = 150;
const BOUNDARY_END_Y: i32 = 150;

fn resource_not_exists<T: Resource>(world: &World) -> bool {
    !world.contains_resource::<T>()
}

#[derive(Resource)]
struct BoundaryReady;

#[derive(Resource)]
struct CurrentParticleType {
    index: usize,
    types: Vec<String>,
}

#[derive(Resource, Default)]
struct CursorPosition {
    world_pos: Vec2,
}

impl Default for CurrentParticleType {
    fn default() -> Self {
        Self {
            index: 0,
            types: vec![],
        }
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct ParticleTypeText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            near: -1000.0,
            scale: 0.2,
            ..OrthographicProjection::default_2d()
        }),
        MainCamera,
    ));
    
    // Instructions and status panel
    let instructions_text = "This example demonstrates saving and loading particles.\n\
        Click and drag to spawn particles\n\
        Tab: Cycle particle type\n\
        F1: Save particles to file\n\
        R: Reset\n\
        Mouse wheel: Zoom camera";
    let panel_id = utils::instructions::spawn_instructions_panel(&mut commands, instructions_text);
    
    commands.entity(panel_id).with_children(|parent| {
        let style = TextFont::default();
        parent.spawn((
            FpsText,
            Text::new("FPS: --"),
            style.clone(),
        ));
    });

    // Load the particle definitions asset
    let particles_handle: Handle<ParticleDefinitionsAsset> =
        asset_server.load("particles/particles.ron");

    commands.spawn(ParticleDefinitionsHandle::new(particles_handle));

    let instructions_text = "TAB: Cycle particle type\n\
        Left Click: Spawn particles at cursor\n\
        F1: Save current particles to RON file\n\
        WASD: Pan camera\n\
        Mouse Wheel: Zoom\n\
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
                ParticleTypeText,
                Text::new("Current Particle: Loading..."),
                style.clone(),
            ));
        });
}

fn check_asset_loading(
    handles: Query<&ParticleDefinitionsHandle>,
    assets: Res<Assets<ParticleDefinitionsAsset>>,
    mut current_particle_type: ResMut<CurrentParticleType>,
    mut particle_type_text: Query<&mut Text, With<ParticleTypeText>>,
) {
    for handle_component in handles.iter() {
        if handle_component.spawned {
            if let Some(asset) = assets.get(&handle_component.handle) {
                if current_particle_type.types.is_empty() {
                    current_particle_type.types = asset
                        .definitions()
                        .keys()
                        .filter(|name| !name.to_lowercase().contains("wall"))
                        .cloned()
                        .collect();
                    current_particle_type.types.sort();
                    current_particle_type.index = 0;
                }

                // Update UI text
                if let Ok(mut text) = particle_type_text.single_mut() {
                    if current_particle_type.types.is_empty() {
                        **text = "Current Particle: Loading...".to_string();
                    } else if let Some(current_type) =
                        current_particle_type.types.get(current_particle_type.index)
                    {
                        **text = format!(
                            "Current Particle: {} ({}/{})",
                            current_type,
                            current_particle_type.index + 1,
                            current_particle_type.types.len()
                        );
                    }
                }
            }
        }
    }
}

fn setup_boundary(mut commands: Commands, particle_type_map: Res<ParticleTypeMap>) {
    if particle_type_map.contains("Wall") || particle_type_map.contains("Dirt Wall") {
        let wall_type = if particle_type_map.contains("Wall") {
            "Wall"
        } else {
            "Dirt Wall"
        };

        for y in 0..BOUNDARY_END_Y {
            commands.spawn((
                Particle::new(wall_type),
                Transform::from_xyz(BOUNDARY_START_X as f32, -(y as f32), 0.0),
            ));
            commands.spawn((
                Particle::new(wall_type),
                Transform::from_xyz(BOUNDARY_END_X as f32, -(y as f32), 0.0),
            ));
        }

        for x in BOUNDARY_START_X..=BOUNDARY_END_X {
            commands.spawn((
                Particle::new(wall_type),
                Transform::from_xyz(x as f32, -(BOUNDARY_END_Y as f32), 0.0),
            ));
        }
        commands.insert_resource(BoundaryReady);
    }
}

fn cycle_particle_type(
    mut current_particle_type: ResMut<CurrentParticleType>,
    mut particle_type_text: Query<&mut Text, With<ParticleTypeText>>,
) {
    if !current_particle_type.types.is_empty() {
        current_particle_type.index =
            (current_particle_type.index + 1) % current_particle_type.types.len();

        if let Ok(mut text) = particle_type_text.single_mut() {
            if let Some(current_type) = current_particle_type.types.get(current_particle_type.index)
            {
                **text = format!(
                    "Current Particle: {} ({}/{})",
                    current_type,
                    current_particle_type.index + 1,
                    current_particle_type.types.len()
                );
            }
        }
    }
}

fn save_particles(
    handles: Query<&ParticleDefinitionsHandle>,
    assets: Res<Assets<ParticleDefinitionsAsset>>,
) {
    for handle_component in handles.iter() {
        if handle_component.spawned {
            if let Some(asset) = assets.get(&handle_component.handle) {
                // Create a subset of particles to save (just a few examples)
                let mut save_definitions = HashMap::new();

                // Select a few interesting particles to save
                let particles_to_save = ["Water", "Sand", "Oil", "FIRE", "Wall"];

                for particle_name in particles_to_save {
                    if let Some(particle_data) = asset.get(particle_name) {
                        save_definitions.insert(particle_name.to_string(), particle_data.clone());
                    }
                }

                // Convert to RON format and save
                match to_string_pretty(&save_definitions, PrettyConfig::default()) {
                    Ok(ron_content) => {
                        // In a real application, you would save this to a file
                        // For this example, we'll just print it to the console
                        info!("=== SAVED PARTICLES TO RON ===");
                        println!("{}", ron_content);
                        info!("=== END SAVED PARTICLES ===");
                        info!("Saved {} particle definitions", save_definitions.len());
                    }
                    Err(e) => {
                        error!("Failed to serialize particles to RON: {}", e);
                    }
                }

                break; // Only process the first handle
            }
        }
    }
}

fn reset(mut ev_clear_dynamic_particles: EventWriter<ClearDynamicParticlesEvent>) {
    ev_clear_dynamic_particles.write(ClearDynamicParticlesEvent);
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

fn update_cursor_position(
    mut cursor_pos: ResMut<CursorPosition>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if let (Ok(window), Ok((camera, camera_transform))) =
        (window_query.single(), camera_query.single())
    {
        if let Some(screen_pos) = window.cursor_position() {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, screen_pos) {
                cursor_pos.world_pos = world_pos;
            }
        }
    }
}

fn spawn_particle_at_cursor(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPosition>,
    current_particle_type: Res<CurrentParticleType>,
    particle_type_map: Res<ParticleTypeMap>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Some(particle_name) = current_particle_type.types.get(current_particle_type.index) {
            if particle_type_map.contains(particle_name) {
                let spawn_pos = cursor_pos.world_pos;
                let radius = 3;

                for dx in -radius..=radius {
                    for dy in -radius..=radius {
                        if dx * dx + dy * dy <= radius * radius {
                            let x = spawn_pos.x + dx as f32;
                            let y = spawn_pos.y + dy as f32;

                            commands.spawn((
                                Particle::from_string(particle_name.clone()),
                                Transform::from_xyz(x, y, 0.0),
                            ));
                        }
                    }
                }
            }
        }
    }
}
