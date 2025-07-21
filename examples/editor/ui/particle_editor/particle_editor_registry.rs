use bevy::{
    platform::{
        collections::{hash_map::Entry, HashMap},
        hash::FixedHasher,
    },
    prelude::*,
};
use bevy_falling_sand::prelude::*;
use std::time::Duration;
use crate::particles::SelectedParticle;
use crate::app_state::InitializationState;

/// Event to load an existing particle type into the editor
#[derive(Event, Debug, Clone)]
pub struct LoadParticleIntoEditor {
    pub particle_name: String,
}

/// Event to create a new particle in the editor
#[derive(Event, Debug, Clone)]
pub struct CreateNewParticle;

/// Event to save the currently edited particle
#[derive(Event, Debug, Clone)]
pub struct SaveParticleFromEditor {
    pub editor_entity: Entity,
}

/// Event to apply editor changes to the actual particle type
#[derive(Event, Debug, Clone)]
pub struct ApplyEditorChanges {
    pub editor_entity: Entity,
    pub create_new: bool,
}

/// Event to apply editor changes and reset particle children
#[derive(Event, Debug, Clone)]
pub struct ApplyEditorChangesAndReset {
    pub editor_entity: Entity,
    pub create_new: bool,
}

/// Holds all editable data for a particle type in the editor
#[derive(Clone, Debug, Component)]
pub struct ParticleEditorData {
    /// The particle type name (mutable in editor)
    pub name: String,

    /// Material state category
    pub material_state: MaterialState,

    /// Movement properties
    pub density: u32,
    pub max_velocity: u8,
    pub has_momentum: bool,

    /// Visual properties
    pub color_palette: Vec<Color>,
    pub changes_color: Option<f64>,

    /// Liquid/Gas specific properties
    pub fluidity: Option<u8>,

    /// Burning properties
    pub burns_config: Option<BurnsConfig>,

    /// Fire spreading properties
    pub fire_config: Option<FireConfig>,

    /// Whether this is a new particle being created
    pub is_new: bool,

    /// Whether the editor data has been modified
    pub is_modified: bool,
}

/// Material state categories for particles
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaterialState {
    Wall,
    Solid,
    MovableSolid,
    Liquid,
    Gas,
    Other,
}

/// Configuration for burning behavior
#[derive(Clone, Debug)]
pub struct BurnsConfig {
    pub duration: Duration,
    pub tick_rate: Duration,
    pub chance_destroy_per_tick: Option<f64>,
    pub reaction: Option<ReactionConfig>,
    pub burning_colors: Option<Vec<Color>>,
    pub spreads_fire: Option<FireConfig>,
    pub ignites_on_spawn: bool,
}

/// Configuration for fire spread
#[derive(Clone, Debug)]
pub struct FireConfig {
    pub burn_radius: f32,
    pub chance_to_spread: f64,
    pub destroys_on_spread: bool,
}

/// Configuration for particle reactions
#[derive(Clone, Debug)]
pub struct ReactionConfig {
    pub produces: String,
    pub chance_to_produce: f64,
}

impl Default for ParticleEditorData {
    fn default() -> Self {
        Self {
            name: "New Particle".to_string(),
            material_state: MaterialState::Solid,
            density: 100,
            max_velocity: 3,
            has_momentum: false,
            color_palette: vec![Color::srgba_u8(128, 128, 128, 255)],
            changes_color: None,
            fluidity: None,
            burns_config: None,
            fire_config: None,
            is_new: true,
            is_modified: false,
        }
    }
}

impl ParticleEditorData {
    /// Create editor data from an existing particle type entity
    pub fn from_particle_type(
        name: String,
        particle_query: &Query<
            (
                Option<&Density>,
                Option<&Velocity>,
                Option<&Momentum>,
                Option<&ColorProfile>,
                Option<&ChangesColor>,
                Option<&Burns>,
                Option<&Fire>,
                Option<&Wall>,
                Option<&Solid>,
                Option<&MovableSolid>,
                Option<&Liquid>,
                Option<&Gas>,
            ),
            With<ParticleType>,
        >,
        entity: Entity,
    ) -> Option<Self> {
        let components = particle_query.get(entity).ok()?;

        let (
            density,
            velocity,
            momentum,
            color_profile,
            changes_color,
            burns,
            fire,
            wall,
            solid,
            movable_solid,
            liquid,
            gas,
        ) = components;

        // Determine material state
        let material_state = if wall.is_some() {
            MaterialState::Wall
        } else if solid.is_some() {
            MaterialState::Solid
        } else if movable_solid.is_some() {
            MaterialState::MovableSolid
        } else if liquid.is_some() {
            MaterialState::Liquid
        } else if gas.is_some() {
            MaterialState::Gas
        } else {
            MaterialState::Other
        };

        // Extract fluidity for liquids/gases
        let fluidity = if let Some(liquid) = liquid {
            Some(liquid.fluidity as u8)
        } else if let Some(gas) = gas {
            Some(gas.fluidity as u8)
        } else {
            None
        };

        // Extract burning configuration
        let burns_config = burns.map(|burns| BurnsConfig {
            duration: burns.duration,
            tick_rate: burns.tick_rate,
            chance_destroy_per_tick: burns.chance_destroy_per_tick,
            reaction: burns.reaction.as_ref().map(|r| ReactionConfig {
                produces: r.produces.name.to_string(),
                chance_to_produce: r.chance_to_produce,
            }),
            burning_colors: burns.color.as_ref().map(|cp| cp.palette.clone()),
            spreads_fire: burns.spreads.as_ref().map(|f| FireConfig {
                burn_radius: f.burn_radius,
                chance_to_spread: f.chance_to_spread,
                destroys_on_spread: f.destroys_on_spread,
            }),
            ignites_on_spawn: burns.ignites_on_spawn,
        });

        // Extract fire configuration
        let fire_config = fire.map(|f| FireConfig {
            burn_radius: f.burn_radius,
            chance_to_spread: f.chance_to_spread,
            destroys_on_spread: f.destroys_on_spread,
        });

        Some(Self {
            name,
            material_state,
            density: density.map(|d| d.0).unwrap_or(100),
            max_velocity: velocity.map(|v| v.max()).unwrap_or(3),
            has_momentum: momentum.is_some(),
            color_palette: color_profile
                .map(|cp| cp.palette.clone())
                .unwrap_or_else(|| vec![Color::srgba_u8(128, 128, 128, 255)]),
            changes_color: changes_color.map(|cc| cc.chance),
            fluidity,
            burns_config,
            fire_config,
            is_new: false,
            is_modified: false,
        })
    }

    /// Convert editor data to particle data for serialization
    pub fn to_particle_data(&self) -> ParticleData {
        let colors = self
            .color_palette
            .iter()
            .map(|color| {
                let srgba = color.to_srgba();
                format!(
                    "#{:02x}{:02x}{:02x}{:02x}",
                    (srgba.red * 255.0) as u8,
                    (srgba.green * 255.0) as u8,
                    (srgba.blue * 255.0) as u8,
                    (srgba.alpha * 255.0) as u8
                )
            })
            .collect();

        let (liquid, gas, movable_solid, solid, wall) = match self.material_state {
            MaterialState::Liquid => (self.fluidity, None, None, None, None),
            MaterialState::Gas => (None, self.fluidity, None, None, None),
            MaterialState::MovableSolid => (None, None, Some(true), None, None),
            MaterialState::Solid => (None, None, None, Some(true), None),
            MaterialState::Wall => (None, None, None, None, Some(true)),
            MaterialState::Other => (None, None, None, None, None),
        };

        ParticleData {
            name: self.name.clone(),
            density: Some(self.density),
            max_velocity: Some(self.max_velocity),
            momentum: if self.has_momentum { Some(true) } else { None },
            liquid,
            gas,
            movable_solid,
            solid,
            wall,
            colors: Some(colors),
            changes_colors: self.changes_color,
            fire: self.fire_config.as_ref().map(|fc| FireData {
                burn_radius: fc.burn_radius,
                chance_to_spread: fc.chance_to_spread,
            }),
            burning: None, // Calculated from burns
            burns: self.burns_config.as_ref().map(|bc| BurnsData {
                duration: bc.duration.as_millis() as u64,
                tick_rate: bc.tick_rate.as_millis() as u64,
                chance_destroy_per_tick: bc.chance_destroy_per_tick,
                reaction: bc.reaction.as_ref().map(|r| ReactionData {
                    produces: r.produces.clone(),
                    chance_to_produce: r.chance_to_produce,
                }),
                colors: bc.burning_colors.as_ref().map(|colors| {
                    colors
                        .iter()
                        .map(|color| {
                            let srgba = color.to_srgba();
                            format!(
                                "#{:02x}{:02x}{:02x}{:02x}",
                                (srgba.red * 255.0) as u8,
                                (srgba.green * 255.0) as u8,
                                (srgba.blue * 255.0) as u8,
                                (srgba.alpha * 255.0) as u8
                            )
                        })
                        .collect()
                }),
                spreads: bc.spreads_fire.as_ref().map(|sf| FireData {
                    burn_radius: sf.burn_radius,
                    chance_to_spread: sf.chance_to_spread,
                }),
            }),
        }
    }

    /// Mark this editor data as modified
    pub fn mark_modified(&mut self) {
        self.is_modified = true;
    }

    /// Mark this editor data as saved
    pub fn mark_saved(&mut self) {
        self.is_modified = false;
        self.is_new = false;
    }
}

/// Registry that maps particle type names to their editor entities
#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorRegistry {
    map: HashMap<String, Entity>,
}

impl ParticleEditorRegistry {
    /// Check if an editor entity exists for the given particle name
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Iterate over all name-entity pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entity)> {
        self.map.iter()
    }

    /// Get all particle names in the registry
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    /// Insert or update an editor entity for a particle name
    pub fn insert(&mut self, name: String, entity: Entity) -> Option<Entity> {
        self.map.insert(name, entity)
    }

    /// Get the entry for a particle name
    pub fn entry(&mut self, name: String) -> Entry<'_, String, Entity, FixedHasher> {
        self.map.entry(name)
    }

    /// Get the editor entity for a particle name
    pub fn get(&self, name: &str) -> Option<&Entity> {
        self.map.get(name)
    }

    /// Remove an editor entity for a particle name
    pub fn remove(&mut self, name: &str) -> Option<Entity> {
        self.map.remove(name)
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

/// System to synchronize the particle editor registry with the main particle type map
pub fn sync_particle_editor_registry(
    mut commands: Commands,
    particle_type_map: Res<ParticleTypeMap>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
    particle_query: Query<
        (
            Option<&Density>,
            Option<&Velocity>,
            Option<&Momentum>,
            Option<&ColorProfile>,
            Option<&ChangesColor>,
            Option<&Burns>,
            Option<&Fire>,
            Option<&Wall>,
            Option<&Solid>,
            Option<&MovableSolid>,
            Option<&Liquid>,
            Option<&Gas>,
        ),
        With<ParticleType>,
    >,
    _editor_data_query: Query<&ParticleEditorData>,
) {
    // Create editor entities for any particle types that don't have them yet
    for (name, &particle_entity) in particle_type_map.iter() {
        let name_string = name.to_string();

        if !particle_editor_registry.contains(&name_string) {
            // Create editor data from the particle type
            if let Some(editor_data) = ParticleEditorData::from_particle_type(
                name_string.clone(),
                &particle_query,
                particle_entity,
            ) {
                let editor_entity = commands.spawn(editor_data).id();
                particle_editor_registry.insert(name_string, editor_entity);
            }
        }
    }

    // Remove editor entities for particle types that no longer exist
    let mut to_remove = Vec::new();
    for (name, &editor_entity) in particle_editor_registry.iter() {
        if !particle_type_map.contains(name) {
            // Clean up the editor entity
            commands.entity(editor_entity).despawn();
            to_remove.push(name.clone());
        }
    }

    for name in to_remove {
        particle_editor_registry.remove(&name);
    }
}

/// System to handle loading particles into the editor
pub fn handle_load_particle_into_editor(
    mut commands: Commands,
    mut load_events: EventReader<LoadParticleIntoEditor>,
    particle_type_map: Res<ParticleTypeMap>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
    particle_query: Query<
        (
            Option<&Density>,
            Option<&Velocity>,
            Option<&Momentum>,
            Option<&ColorProfile>,
            Option<&ChangesColor>,
            Option<&Burns>,
            Option<&Fire>,
            Option<&Wall>,
            Option<&Solid>,
            Option<&MovableSolid>,
            Option<&Liquid>,
            Option<&Gas>,
        ),
        With<ParticleType>,
    >,
    mut current_editor: ResMut<CurrentEditorSelection>,
) {
    for event in load_events.read() {
        // Check if we already have an editor entity for this particle
        if let Some(&editor_entity) = particle_editor_registry.get(&event.particle_name) {
            current_editor.selected_entity = Some(editor_entity);
            continue;
        }

        // Create new editor entity if it doesn't exist
        if let Some(&particle_entity) = particle_type_map.get(&event.particle_name) {
            if let Some(editor_data) = ParticleEditorData::from_particle_type(
                event.particle_name.clone(),
                &particle_query,
                particle_entity,
            ) {
                let editor_entity = commands.spawn(editor_data).id();
                particle_editor_registry.insert(event.particle_name.clone(), editor_entity);
                current_editor.selected_entity = Some(editor_entity);
            }
        }
    }
}

/// System to handle creating new particles
pub fn handle_create_new_particle(
    mut commands: Commands,
    mut create_events: EventReader<CreateNewParticle>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
    mut current_editor: ResMut<CurrentEditorSelection>,
) {
    for _event in create_events.read() {
        let editor_data = ParticleEditorData::default();
        let unique_name = generate_unique_particle_name(&particle_editor_registry);
        let mut editor_data = editor_data;
        editor_data.name = unique_name.clone();

        let editor_entity = commands.spawn(editor_data).id();
        particle_editor_registry.insert(unique_name, editor_entity);
        current_editor.selected_entity = Some(editor_entity);
    }
}

/// System to handle applying editor changes to actual particle types
pub fn handle_apply_editor_changes(
    mut commands: Commands,
    mut apply_events: EventReader<ApplyEditorChanges>,
    mut editor_data_query: Query<&mut ParticleEditorData>,
    mut particle_type_map: ResMut<ParticleTypeMap>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
) {
    for event in apply_events.read() {
        if let Ok(mut editor_data) = editor_data_query.get_mut(event.editor_entity) {
            // Determine if this should create a new particle or update existing one
            // Use editor data's is_new flag and check if particle exists in type map
            let create_new = editor_data.is_new || !particle_type_map.contains(&editor_data.name);
            
            // Convert editor data to actual particle components
            apply_editor_data_to_particle_type(
                &mut commands,
                &editor_data,
                &mut particle_type_map,
                create_new,
            );

            // Update registry if name changed or it's a new particle
            if create_new {
                particle_editor_registry.insert(editor_data.name.clone(), event.editor_entity);
            }
            
            // Mark the editor data as saved
            editor_data.mark_saved();
        }
    }
}

/// Helper function to generate unique particle names
fn generate_unique_particle_name(registry: &ParticleEditorRegistry) -> String {
    let base_name = "New Particle";
    let mut counter = 1;
    let mut name = base_name.to_string();

    while registry.contains(&name) {
        name = format!("{} {}", base_name, counter);
        counter += 1;
    }

    name
}

/// Helper function to apply editor data to a particle type entity
fn apply_editor_data_to_particle_type(
    commands: &mut Commands,
    editor_data: &ParticleEditorData,
    particle_type_map: &mut ResMut<ParticleTypeMap>,
    create_new: bool,
) -> Entity {
    let entity = if create_new {
        // Create new particle type entity
        let static_name: &'static str = Box::leak(editor_data.name.clone().into_boxed_str());
        let entity = commands.spawn(ParticleType::new(static_name)).id();
        particle_type_map.insert(static_name, entity);
        entity
    } else {
        // Get existing particle type entity
        *particle_type_map
            .get(&editor_data.name)
            .expect("Particle type should exist")
    };

    // Clear existing components and apply new ones
    commands
        .entity(entity)
        .remove::<(
            Density,
            Velocity,
            Momentum,
            ColorProfile,
            ChangesColor,
            Burns,
            Fire,
        )>()
        .remove::<(Wall, Solid, MovableSolid, Liquid, Gas)>();

    // Apply basic properties
    commands.entity(entity).insert(Density(editor_data.density));
    commands
        .entity(entity)
        .insert(Velocity::new(1, editor_data.max_velocity));

    if editor_data.has_momentum {
        commands.entity(entity).insert(Momentum::ZERO);
    }

    // Apply colors
    if !editor_data.color_palette.is_empty() {
        commands
            .entity(entity)
            .insert(ColorProfile::new(editor_data.color_palette.clone()));
    }

    if let Some(chance) = editor_data.changes_color {
        commands.entity(entity).insert(ChangesColor::new(chance));
    }

    // Apply material state
    match editor_data.material_state {
        MaterialState::Wall => {
            commands.entity(entity).insert(Wall);
        }
        MaterialState::Solid => {
            commands.entity(entity).insert(Solid);
        }
        MaterialState::MovableSolid => {
            commands.entity(entity).insert(MovableSolid);
        }
        MaterialState::Liquid => {
            if let Some(fluidity) = editor_data.fluidity {
                commands.entity(entity).insert(Liquid::new(fluidity.into()));
            }
        }
        MaterialState::Gas => {
            if let Some(fluidity) = editor_data.fluidity {
                commands.entity(entity).insert(Gas::new(fluidity.into()));
            }
        }
        MaterialState::Other => {
            // No specific material component
        }
    }

    // Apply burning configuration
    if let Some(ref burns_config) = editor_data.burns_config {
        let reaction = burns_config.reaction.as_ref().map(|r| {
            let static_name: &'static str = Box::leak(r.produces.clone().into_boxed_str());
            Reacting::new(Particle::new(static_name), r.chance_to_produce)
        });

        let spreads = burns_config.spreads_fire.as_ref().map(|f| Fire {
            burn_radius: f.burn_radius,
            chance_to_spread: f.chance_to_spread,
            destroys_on_spread: f.destroys_on_spread,
        });

        commands.entity(entity).insert(Burns::new(
            burns_config.duration,
            burns_config.tick_rate,
            burns_config.chance_destroy_per_tick,
            reaction,
            burns_config
                .burning_colors
                .as_ref()
                .map(|colors| ColorProfile::new(colors.clone())),
            spreads,
            burns_config.ignites_on_spawn,
        ));
    }

    // Apply fire configuration
    if let Some(ref fire_config) = editor_data.fire_config {
        commands.entity(entity).insert(Fire {
            burn_radius: fire_config.burn_radius,
            chance_to_spread: fire_config.chance_to_spread,
            destroys_on_spread: fire_config.destroys_on_spread,
        });
    }

    entity
}

/// System to handle applying editor changes and resetting particle children
pub fn setup_initial_particle_selection(
    selected_particle: Res<SelectedParticle>,
    mut load_particle_events: EventWriter<LoadParticleIntoEditor>,
) {
    let particle_name = selected_particle.0.name.to_string();
    
    // Send event to load the initially selected particle into the editor
    load_particle_events.write(LoadParticleIntoEditor {
        particle_name,
    });
}

pub fn handle_apply_editor_changes_and_reset(
    mut commands: Commands,
    mut apply_events: EventReader<ApplyEditorChangesAndReset>,
    mut editor_data_query: Query<&mut ParticleEditorData>,
    mut particle_type_map: ResMut<ParticleTypeMap>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
    mut reset_particle_children_events: EventWriter<bevy_falling_sand::prelude::ResetParticleChildrenEvent>,
) {
    for event in apply_events.read() {
        if let Ok(mut editor_data) = editor_data_query.get_mut(event.editor_entity) {
            // Determine if this should create a new particle or update existing one
            // Use editor data's is_new flag and check if particle exists in type map
            let create_new = editor_data.is_new || !particle_type_map.contains(&editor_data.name);
            
            // Convert editor data to actual particle components
            let particle_entity = apply_editor_data_to_particle_type(
                &mut commands,
                &editor_data,
                &mut particle_type_map,
                create_new,
            );

            // Update registry if name changed or it's a new particle
            if create_new {
                particle_editor_registry.insert(editor_data.name.clone(), event.editor_entity);
            }
            
            // Mark the editor data as saved
            editor_data.mark_saved();
            
            // Send reset particle children event
            reset_particle_children_events.write(bevy_falling_sand::prelude::ResetParticleChildrenEvent {
                entity: particle_entity,
            });
        }
    }
}

/// Resource to track the currently selected editor entity
#[derive(Resource, Default)]
pub struct CurrentEditorSelection {
    pub selected_entity: Option<Entity>,
}
