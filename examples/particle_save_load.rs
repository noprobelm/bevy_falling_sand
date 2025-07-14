use bevy::{
    input::{common_conditions::input_just_pressed, mouse::MouseWheel},
    prelude::*,
};
use bfs_assets::{FallingSandAssetsPlugin, ParticleDefinitionsAsset, ParticleDefinitionsHandle};
use bfs_color::FallingSandColorPlugin;
use bfs_core::{Particle, ParticleTypeMap};
use bfs_movement::FallingSandMovementPlugin;
use bfs_reactions::FallingSandReactionsPlugin;
use bevy_falling_sand::prelude::*;
use ron::ser::{to_string_pretty, PrettyConfig};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FallingSandMinimalPlugin,
            FallingSandColorPlugin,
            FallingSandMovementPlugin,
            FallingSandSpatialPlugin { frequency: std::time::Duration::from_millis(50) },
            FallingSandReactionsPlugin,
            FallingSandAssetsPlugin,
            FallingSandDebugPlugin,
        ))
        .init_resource::<SpawnParticles>()
        .init_resource::<CurrentParticleType>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                zoom_camera,
                pan_camera,
                setup_boundary.run_if(resource_not_exists::<BoundaryReady>),
                stream_particles.run_if(
                    resource_exists::<BoundaryReady>.and(resource_exists::<SpawnParticles>),
                ),
                cycle_particle_type.run_if(input_just_pressed(KeyCode::Tab)),
                toggle_spawn_particles.run_if(input_just_pressed(KeyCode::F1)),
                toggle_debug_map.run_if(input_just_pressed(KeyCode::F2)),
                save_particles.run_if(input_just_pressed(KeyCode::F3)),
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

#[derive(Default, Resource)]
struct SpawnParticles;

#[derive(Resource)]
struct CurrentParticleType {
    index: usize,
    types: Vec<String>,
}

impl Default for CurrentParticleType {
    fn default() -> Self {
        Self {
            index: 0,
            types: vec!["Water".to_string(), "Sand".to_string(), "Oil".to_string(), "FIRE".to_string()],
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

    // Load the particle definitions asset
    let particles_handle: Handle<ParticleDefinitionsAsset> = 
        asset_server.load("particles/demo_particles.ron");
    
    commands.spawn(ParticleDefinitionsHandle::new(particles_handle));

    let instructions_text = "TAB: Cycle particle type\n\
        F1: Toggle particle spawning\n\
        F2: Show/Hide particle chunk map\n\
        F3: Save current particles to RON file\n\
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
                // Update available particle types from loaded asset
                current_particle_type.types = asset.definitions().keys().cloned().collect();
                current_particle_type.types.sort(); // Keep consistent ordering
                
                // Update UI text
                if let Ok(mut text) = particle_type_text.single_mut() {
                    if let Some(current_type) = current_particle_type.types.get(current_particle_type.index) {
                        **text = format!("Current Particle: {} ({}/{})", 
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

fn stream_particles(
    mut commands: Commands, 
    current_particle_type: Res<CurrentParticleType>,
    particle_type_map: Res<ParticleTypeMap>,
) {
    if let Some(particle_name) = current_particle_type.types.get(current_particle_type.index) {
        // Only spawn if the particle type exists in the map
        if particle_type_map.contains(particle_name) {
            let center_x = (BOUNDARY_START_X + BOUNDARY_END_X) / 2;
            let spawn_y = -(BOUNDARY_END_Y as f32) - 10.0;

            let radius = 2;

            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    if dx * dx + dy * dy <= radius * radius {
                        let base_x = center_x as f32 + dx as f32;
                        let y = spawn_y + dy as f32 + 200.0;

                        commands.spawn((
                            Particle::new(particle_name),
                            Transform::from_xyz(base_x, y, 0.0),
                        ));
                    }
                }
            }
        }
    }
}

fn cycle_particle_type(
    mut current_particle_type: ResMut<CurrentParticleType>,
    mut particle_type_text: Query<&mut Text, With<ParticleTypeText>>,
) {
    if !current_particle_type.types.is_empty() {
        current_particle_type.index = (current_particle_type.index + 1) % current_particle_type.types.len();
        
        if let Ok(mut text) = particle_type_text.single_mut() {
            if let Some(current_type) = current_particle_type.types.get(current_particle_type.index) {
                **text = format!("Current Particle: {} ({}/{})", 
                    current_type, 
                    current_particle_type.index + 1, 
                    current_particle_type.types.len()
                );
            }
        }
    }
}

fn toggle_spawn_particles(mut commands: Commands, spawn_particles: Option<Res<SpawnParticles>>) {
    if spawn_particles.is_some() {
        commands.remove_resource::<SpawnParticles>();
    } else {
        commands.init_resource::<SpawnParticles>();
    }
}

fn toggle_debug_map(mut commands: Commands, debug_map: Option<Res<DebugParticleMap>>) {
    if debug_map.is_some() {
        commands.remove_resource::<DebugParticleMap>();
    } else {
        commands.init_resource::<DebugParticleMap>();
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