use crate::particles::SelectedParticle;
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_falling_sand::prelude::*;
use std::time::Duration;

#[derive(Event, Debug, Clone)]
pub struct LoadParticleIntoEditor {
    pub particle_name: String,
}

#[derive(Event, Debug, Clone)]
pub struct CreateNewParticle {
    pub duplicate_from: Option<String>,
}

#[derive(Event, Debug, Clone)]
pub struct SaveParticleFromEditor;

#[derive(Event, Debug, Clone)]
pub struct ApplyEditorChanges {
    pub editor_entity: Entity,
}

#[derive(Event, Debug, Clone)]
pub struct ApplyEditorChangesAndReset {
    pub editor_entity: Entity,
}

#[derive(Clone, Debug, Component)]
pub struct ParticleEditorData {
    pub name: String,

    pub material_state: MaterialState,

    pub density: u32,
    pub max_velocity: u8,
    pub has_momentum: bool,

    pub color_palette: Vec<Color>,
    pub changes_color: Option<f64>,

    pub fluidity: Option<u8>,

    pub burns_config: Option<BurnsConfig>,

    pub fire_config: Option<FireConfig>,

    pub is_new: bool,

    pub is_modified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaterialState {
    Wall,
    Solid,
    MovableSolid,
    Liquid,
    Gas,
    Other,
}

#[derive(Clone, Debug)]
pub struct BurnsConfig {
    pub duration: Duration,
    pub tick_rate: Duration,
    pub duration_str: String,
    pub tick_rate_str: String,
    pub chance_destroy_per_tick: Option<f64>,
    pub reaction: Option<ReactionConfig>,
    pub burning_colors: Option<Vec<Color>>,
    pub spreads_fire: Option<FireConfig>,
    pub ignites_on_spawn: bool,
}

#[derive(Clone, Debug)]
pub struct FireConfig {
    pub burn_radius: f32,
    pub chance_to_spread: f64,
    pub destroys_on_spread: bool,
}

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

        let fluidity = if let Some(liquid) = liquid {
            Some(liquid.fluidity as u8)
        } else if let Some(gas) = gas {
            Some(gas.fluidity as u8)
        } else {
            None
        };

        let burns_config = burns.map(|burns| BurnsConfig {
            duration: burns.duration,
            tick_rate: burns.tick_rate,
            duration_str: burns.duration.as_millis().to_string(),
            tick_rate_str: burns.tick_rate.as_millis().to_string(),
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

    pub fn mark_saved(&mut self) {
        self.is_modified = false;
        self.is_new = false;
    }
}

#[derive(Resource, Clone, Default, Debug)]
pub struct ParticleEditorRegistry {
    map: HashMap<String, Entity>,
}

impl ParticleEditorRegistry {
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Entity)> {
        self.map.iter()
    }

    pub fn insert(&mut self, name: String, entity: Entity) -> Option<Entity> {
        self.map.insert(name, entity)
    }

    pub fn get(&self, name: &str) -> Option<&Entity> {
        self.map.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<Entity> {
        self.map.remove(name)
    }
}

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
    for (name, &particle_entity) in particle_type_map.iter() {
        let name_string = name.to_string();

        if !particle_editor_registry.contains(&name_string) {
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

    let mut to_remove = Vec::new();
    for (name, &editor_entity) in particle_editor_registry.iter() {
        if !particle_type_map.contains(name) {
            commands.entity(editor_entity).despawn();
            to_remove.push(name.clone());
        }
    }

    for name in to_remove {
        particle_editor_registry.remove(&name);
    }
}

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
    selected_particle: Option<ResMut<SelectedParticle>>,
) {
    let mut selected_particle_mut = selected_particle;

    for event in load_events.read() {
        if let Some(ref mut selected_particle) = selected_particle_mut {
            let static_name: &'static str = Box::leak(event.particle_name.clone().into_boxed_str());
            selected_particle.0 = Particle::new(static_name);
        }

        if let Some(&editor_entity) = particle_editor_registry.get(&event.particle_name) {
            current_editor.selected_entity = Some(editor_entity);
            continue;
        }

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

pub fn handle_create_new_particle(
    mut commands: Commands,
    mut create_events: EventReader<CreateNewParticle>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
    mut current_editor: ResMut<CurrentEditorSelection>,
    mut particle_type_map: ResMut<ParticleTypeMap>,
    selected_particle: Option<ResMut<SelectedParticle>>,
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
) {
    let mut selected_particle_mut = selected_particle;

    for event in create_events.read() {
        let editor_data = if let Some(ref duplicate_from) = event.duplicate_from {
            if let Some(&particle_entity) = particle_type_map.get(duplicate_from) {
                if let Some(mut duplicated_data) = ParticleEditorData::from_particle_type(
                    duplicate_from.clone(),
                    &particle_query,
                    particle_entity,
                ) {
                    let unique_name =
                        generate_unique_particle_name_with_base(&particle_type_map, "New Particle");
                    duplicated_data.name = unique_name;
                    duplicated_data.is_new = true;
                    duplicated_data.is_modified = false;
                    duplicated_data
                } else {
                    let mut default_data = ParticleEditorData::default();
                    let unique_name =
                        generate_unique_particle_name_with_base(&particle_type_map, "New Particle");
                    default_data.name = unique_name;
                    default_data
                }
            } else {
                let mut default_data = ParticleEditorData::default();
                let unique_name =
                    generate_unique_particle_name_with_base(&particle_type_map, "New Particle");
                default_data.name = unique_name;
                default_data
            }
        } else {
            let mut editor_data = ParticleEditorData::default();
            let unique_name =
                generate_unique_particle_name_with_base(&particle_type_map, "New Particle");
            editor_data.name = unique_name;
            editor_data
        };

        apply_editor_data_to_particle_type(
            &mut commands,
            &editor_data,
            &mut particle_type_map,
            true,
        );

        let mut final_editor_data = editor_data;
        final_editor_data.mark_saved();

        let editor_entity = commands.spawn(final_editor_data.clone()).id();
        particle_editor_registry.insert(final_editor_data.name.clone(), editor_entity);
        current_editor.selected_entity = Some(editor_entity);

        if let Some(ref mut selected_particle) = selected_particle_mut {
            let static_name: &'static str =
                Box::leak(final_editor_data.name.clone().into_boxed_str());
            selected_particle.0 = Particle::new(static_name);
        }
    }
}

pub fn handle_apply_editor_changes(
    mut commands: Commands,
    mut apply_events: EventReader<ApplyEditorChanges>,
    mut editor_data_query: Query<&mut ParticleEditorData>,
    mut particle_type_map: ResMut<ParticleTypeMap>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
) {
    for event in apply_events.read() {
        if let Ok(mut editor_data) = editor_data_query.get_mut(event.editor_entity) {
            let create_new = editor_data.is_new || !particle_type_map.contains(&editor_data.name);

            apply_editor_data_to_particle_type(
                &mut commands,
                &editor_data,
                &mut particle_type_map,
                create_new,
            );

            if create_new {
                particle_editor_registry.insert(editor_data.name.clone(), event.editor_entity);
            }

            editor_data.mark_saved();
        }
    }
}

fn generate_unique_particle_name_with_base(
    particle_type_map: &ParticleTypeMap,
    base_name: &str,
) -> String {
    let mut counter = 1;
    let mut name = base_name.to_string();

    while particle_type_map.contains(&name) {
        counter += 1;
        name = format!("{} {}", base_name, counter);
    }

    name
}

fn apply_editor_data_to_particle_type(
    commands: &mut Commands,
    editor_data: &ParticleEditorData,
    particle_type_map: &mut ResMut<ParticleTypeMap>,
    create_new: bool,
) -> Entity {
    let entity = if create_new {
        let static_name: &'static str = Box::leak(editor_data.name.clone().into_boxed_str());
        let entity = commands.spawn(ParticleType::new(static_name)).id();
        particle_type_map.insert(static_name, entity);
        entity
    } else {
        *particle_type_map
            .get(&editor_data.name)
            .expect("Particle type should exist")
    };

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

    commands.entity(entity).insert(Density(editor_data.density));
    commands
        .entity(entity)
        .insert(Velocity::new(1, editor_data.max_velocity));

    if editor_data.has_momentum {
        commands.entity(entity).insert(Momentum::ZERO);
    }

    if !editor_data.color_palette.is_empty() {
        commands
            .entity(entity)
            .insert(ColorProfile::new(editor_data.color_palette.clone()));
    }

    if let Some(chance) = editor_data.changes_color {
        commands.entity(entity).insert(ChangesColor::new(chance));
    }

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
            let fluidity = editor_data.fluidity.unwrap_or(3);
            commands.entity(entity).insert(Liquid::new(fluidity.into()));
        }
        MaterialState::Gas => {
            let fluidity = editor_data.fluidity.unwrap_or(3);
            commands.entity(entity).insert(Gas::new(fluidity.into()));
        }
        MaterialState::Other => {}
    }

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

    if let Some(ref fire_config) = editor_data.fire_config {
        commands.entity(entity).insert(Fire {
            burn_radius: fire_config.burn_radius,
            chance_to_spread: fire_config.chance_to_spread,
            destroys_on_spread: fire_config.destroys_on_spread,
        });
    }

    entity
}

pub fn setup_initial_particle_selection(
    selected_particle: Res<SelectedParticle>,
    mut load_particle_events: EventWriter<LoadParticleIntoEditor>,
) {
    let particle_name = selected_particle.0.name.to_string();

    load_particle_events.write(LoadParticleIntoEditor { particle_name });
}

pub fn handle_apply_editor_changes_and_reset(
    mut commands: Commands,
    mut apply_events: EventReader<ApplyEditorChangesAndReset>,
    mut editor_data_query: Query<&mut ParticleEditorData>,
    mut particle_type_map: ResMut<ParticleTypeMap>,
    mut particle_editor_registry: ResMut<ParticleEditorRegistry>,
    mut reset_particle_children_events: EventWriter<
        bevy_falling_sand::prelude::ResetParticleChildrenEvent,
    >,
) {
    for event in apply_events.read() {
        if let Ok(mut editor_data) = editor_data_query.get_mut(event.editor_entity) {
            let create_new = editor_data.is_new || !particle_type_map.contains(&editor_data.name);

            let particle_entity = apply_editor_data_to_particle_type(
                &mut commands,
                &editor_data,
                &mut particle_type_map,
                create_new,
            );

            if create_new {
                particle_editor_registry.insert(editor_data.name.clone(), event.editor_entity);
            }

            editor_data.mark_saved();

            reset_particle_children_events.write(
                bevy_falling_sand::prelude::ResetParticleChildrenEvent {
                    entity: particle_entity,
                },
            );
        }
    }
}

#[derive(Resource, Default)]
pub struct CurrentEditorSelection {
    pub selected_entity: Option<Entity>,
}
