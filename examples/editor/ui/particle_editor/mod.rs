mod particle_editor_registry;

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::{
    ParticleTypeMap, ParticleTypeMaterialsParam, ResetParticleChildrenEvent,
};

use particle_editor_registry::*;

pub use particle_editor_registry::{
    ApplyEditorChanges, ApplyEditorChangesAndReset, BurnsConfig, CreateNewParticle,
    CurrentEditorSelection, FireConfig, LoadParticleIntoEditor, MaterialState, ParticleEditorData,
};

pub struct ParticleEditorPlugin;

impl Plugin for ParticleEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleEditorRegistry>()
            .init_resource::<CurrentEditorSelection>()
            .add_event::<LoadParticleIntoEditor>()
            .add_event::<CreateNewParticle>()
            .add_event::<SaveParticleFromEditor>()
            .add_event::<ApplyEditorChanges>()
            .add_event::<ApplyEditorChangesAndReset>()
            .add_systems(
                OnEnter(crate::app_state::InitializationState::Finished),
                setup_initial_particle_selection,
            )
            .add_systems(
                Update,
                (
                    sync_particle_editor_registry,
                    handle_load_particle_into_editor,
                    handle_create_new_particle,
                    handle_apply_editor_changes,
                    handle_apply_editor_changes_and_reset,
                ),
            );
    }
}
pub struct ParticleEditor;

impl ParticleEditor {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        particle_materials: &ParticleTypeMaterialsParam,
        current_editor: &CurrentEditorSelection,
        editor_data_query: &mut Query<&mut ParticleEditorData>,
        load_particle_events: &mut EventWriter<LoadParticleIntoEditor>,
        create_particle_events: &mut EventWriter<CreateNewParticle>,
        apply_editor_events: &mut EventWriter<ApplyEditorChanges>,
        apply_editor_and_reset_events: &mut EventWriter<ApplyEditorChangesAndReset>,
        reset_particle_children_events: &mut EventWriter<ResetParticleChildrenEvent>,
        particle_type_map: &ParticleTypeMap,
    ) {
        let text_color = egui::Color32::from_rgb(204, 204, 204);
        ui.visuals_mut().override_text_color = Some(text_color);

        ui.heading("Particle Editor");
        ui.separator();

        let current_particle_name = if let Some(editor_entity) = current_editor.selected_entity {
            if let Ok(editor_data) = editor_data_query.get(editor_entity) {
                Some(editor_data.name.clone())
            } else {
                None
            }
        } else {
            None
        };

        ui.columns(2, |columns| {
            columns[0].set_min_width(columns[0].available_width());
            columns[0].set_max_width(columns[0].available_width());
            self.render_particle_list(
                &mut columns[0],
                particle_materials,
                load_particle_events,
                create_particle_events,
                current_editor,
                apply_editor_events,
                apply_editor_and_reset_events,
                reset_particle_children_events,
                particle_type_map,
                current_particle_name,
            );

            columns[1].set_min_width(columns[1].available_width());
            columns[1].set_max_width(columns[1].available_width());
            self.render_particle_properties(&mut columns[1], current_editor, editor_data_query);
        });
    }

    fn render_particle_list(
        &self,
        ui: &mut egui::Ui,
        particle_materials: &ParticleTypeMaterialsParam,
        load_particle_events: &mut EventWriter<LoadParticleIntoEditor>,
        create_particle_events: &mut EventWriter<CreateNewParticle>,
        current_editor: &CurrentEditorSelection,
        apply_editor_events: &mut EventWriter<ApplyEditorChanges>,
        apply_editor_and_reset_events: &mut EventWriter<ApplyEditorChangesAndReset>,
        _reset_particle_children_events: &mut EventWriter<ResetParticleChildrenEvent>,
        _particle_type_map: &ParticleTypeMap,
        current_particle_name: Option<String>,
    ) {
        egui::ScrollArea::vertical()
            .id_salt("particle_list_scroll")
            .show(ui, |ui| {
                const CATEGORIES: [&str; 6] = [
                    "Walls",
                    "Solids",
                    "Movable Solids",
                    "Liquids",
                    "Gases",
                    "Other",
                ];

                for &category in &CATEGORIES {
                    let original_indent = ui.spacing().indent;
                    ui.spacing_mut().indent = 16.0;

                    egui::CollapsingHeader::new(category)
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.spacing_mut().indent = original_indent;
                            let examples: Vec<&str> = match category {
                                "Walls" => particle_materials
                                    .walls()
                                    .iter()
                                    .map(|particle_type| particle_type.name.as_ref())
                                    .collect(),
                                "Solids" => particle_materials
                                    .solids()
                                    .iter()
                                    .map(|particle_type| particle_type.name.as_ref())
                                    .collect(),
                                "Movable Solids" => particle_materials
                                    .movable_solids()
                                    .iter()
                                    .map(|particle_type| particle_type.name.as_ref())
                                    .collect(),
                                "Liquids" => particle_materials
                                    .liquids()
                                    .iter()
                                    .map(|particle_type| particle_type.name.as_ref())
                                    .collect(),
                                "Gases" => particle_materials
                                    .gases()
                                    .iter()
                                    .map(|particle_type| particle_type.name.as_ref())
                                    .collect(),
                                "Other" => particle_materials
                                    .other()
                                    .iter()
                                    .map(|particle_type| particle_type.name.as_ref())
                                    .collect(),
                                _ => vec![],
                            };

                            for particle_name in examples {
                                if ui.button(particle_name).clicked() {
                                    load_particle_events.write(LoadParticleIntoEditor {
                                        particle_name: particle_name.to_string(),
                                    });
                                }
                            }
                        });

                    ui.spacing_mut().indent = original_indent;
                }

                ui.add_space(8.0);
                ui.vertical(|ui| {
                    if ui.button("New Particle").clicked() {
                        create_particle_events.write(CreateNewParticle {
                            duplicate_from: current_particle_name.clone(),
                        });
                    }
                    if ui.button("Save Particle").clicked() {
                        if let Some(editor_entity) = current_editor.selected_entity {
                            apply_editor_events.write(ApplyEditorChanges { editor_entity });
                        }
                    }
                    if ui.button("Save Particle & Reset Children").clicked() {
                        if let Some(editor_entity) = current_editor.selected_entity {
                            apply_editor_and_reset_events
                                .write(ApplyEditorChangesAndReset { editor_entity });
                        }
                    }
                });
            });
    }

    fn render_particle_properties(
        &self,
        ui: &mut egui::Ui,
        current_editor: &CurrentEditorSelection,
        editor_data_query: &mut Query<&mut ParticleEditorData>,
    ) {
        egui::ScrollArea::vertical()
            .id_salt("particle_properties_scroll")
            .show(ui, |ui| {
                if let Some(editor_entity) = current_editor.selected_entity {
                    if let Ok(mut editor_data) = editor_data_query.get_mut(editor_entity) {
                        self.render_editor_data(ui, &mut editor_data);
                    } else {
                        ui.label("Selected editor entity not found.");
                    }
                } else {
                    ui.label("No particle selected for editing.");
                    ui.label("Select a particle from the list on the left, or create a new one.");
                }
            });
    }

    fn render_editor_data(&self, ui: &mut egui::Ui, editor_data: &mut ParticleEditorData) {
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut editor_data.name);
        });

        ui.horizontal(|ui| {
            ui.label("State:");
            let current_state_text = match editor_data.material_state {
                MaterialState::Wall => "Wall",
                MaterialState::Solid => "Solid",
                MaterialState::MovableSolid => "Movable Solid",
                MaterialState::Liquid => "Liquid",
                MaterialState::Gas => "Gas",
                MaterialState::Other => "Other",
            };

            egui::ComboBox::from_label("")
                .selected_text(current_state_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut editor_data.material_state,
                        MaterialState::Wall,
                        "Wall",
                    );
                    ui.selectable_value(
                        &mut editor_data.material_state,
                        MaterialState::Solid,
                        "Solid",
                    );
                    ui.selectable_value(
                        &mut editor_data.material_state,
                        MaterialState::MovableSolid,
                        "Movable Solid",
                    );
                    ui.selectable_value(
                        &mut editor_data.material_state,
                        MaterialState::Liquid,
                        "Liquid",
                    );
                    ui.selectable_value(&mut editor_data.material_state, MaterialState::Gas, "Gas");
                    ui.selectable_value(
                        &mut editor_data.material_state,
                        MaterialState::Other,
                        "Other",
                    );
                });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Density:");
            let mut density_f32 = editor_data.density as f32;
            if ui
                .add(egui::Slider::new(&mut density_f32, 1.0..=1000.0).step_by(1.0))
                .changed()
            {
                editor_data.density = density_f32 as u32;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Max Velocity:");
            let mut velocity_f32 = editor_data.max_velocity as f32;
            if ui
                .add(egui::Slider::new(&mut velocity_f32, 1.0..=5.0).step_by(1.0))
                .changed()
            {
                editor_data.max_velocity = velocity_f32 as u8;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Momentum");
            ui.checkbox(&mut editor_data.has_momentum, "");
        });

        if matches!(
            editor_data.material_state,
            MaterialState::Liquid | MaterialState::Gas
        ) {
            ui.horizontal(|ui| {
                ui.label("Fluidity:");
                let fluidity_value = editor_data.fluidity.unwrap_or(3);
                let mut fluidity_f32 = fluidity_value as f32;
                if ui
                    .add(egui::Slider::new(&mut fluidity_f32, 1.0..=5.0).step_by(1.0))
                    .changed()
                {
                    editor_data.fluidity = Some(fluidity_f32 as u8);
                }
            });
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Colors");
            if ui.button("➕").clicked() {
                editor_data
                    .color_palette
                    .push(Color::srgba_u8(255, 255, 255, 255));
            }
        });

        let mut to_remove: Option<usize> = None;
        for (i, color) in editor_data.color_palette.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let srgba = color.to_srgba();
                ui.label(format!(
                    "R: {:.0} G: {:.0} B: {:.0} A: {:.0}",
                    srgba.red * 255.0,
                    srgba.green * 255.0,
                    srgba.blue * 255.0,
                    srgba.alpha * 255.0
                ));

                let mut color32 = egui::Color32::from_rgba_unmultiplied(
                    (srgba.red * 255.0) as u8,
                    (srgba.green * 255.0) as u8,
                    (srgba.blue * 255.0) as u8,
                    (srgba.alpha * 255.0) as u8,
                );

                if ui.color_edit_button_srgba(&mut color32).changed() {
                    *color = Color::srgba_u8(color32.r(), color32.g(), color32.b(), color32.a());
                }

                if ui.button("❌").clicked() {
                    to_remove = Some(i);
                }
            });
        }

        if let Some(remove_index) = to_remove {
            editor_data.color_palette.remove(remove_index);
        }

        ui.horizontal(|ui| {
            ui.label("Changes Color Chance:");
            let mut has_changes_color = editor_data.changes_color.is_some();
            if ui.checkbox(&mut has_changes_color, "").changed() {
                if has_changes_color {
                    editor_data.changes_color = Some(0.1);
                } else {
                    editor_data.changes_color = None;
                }
            }

            if let Some(ref mut chance) = editor_data.changes_color {
                ui.add(egui::Slider::new(chance, 0.0..=1.0));
            }
        });

        ui.separator();

        let mut has_burns = editor_data.burns_config.is_some();
        if ui.checkbox(&mut has_burns, "Flammable").changed() {
            if has_burns {
                editor_data.burns_config = Some(BurnsConfig {
                    duration: std::time::Duration::from_millis(1000),
                    tick_rate: std::time::Duration::from_millis(100),
                    chance_destroy_per_tick: None,
                    reaction: None,
                    burning_colors: None,
                    spreads_fire: None,
                    ignites_on_spawn: false,
                });
            } else {
                editor_data.burns_config = None;
            }
        }

        if let Some(ref mut burns_config) = editor_data.burns_config {
            ui.horizontal(|ui| {
                ui.label("Duration (ms):");
                let duration_ms = burns_config.duration.as_millis() as u64;
                let mut duration_str = duration_ms.to_string();
                if ui.text_edit_singleline(&mut duration_str).lost_focus() {
                    if let Ok(new_duration) = duration_str.parse::<u64>() {
                        burns_config.duration = std::time::Duration::from_millis(new_duration);
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Tick Rate (ms):");
                let tick_rate_ms = burns_config.tick_rate.as_millis() as u64;
                let mut tick_rate_str = tick_rate_ms.to_string();
                if ui.text_edit_singleline(&mut tick_rate_str).lost_focus() {
                    if let Ok(new_tick_rate) = tick_rate_str.parse::<u64>() {
                        burns_config.tick_rate = std::time::Duration::from_millis(new_tick_rate);
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Destroy Chance:");
                let mut has_destroy_chance = burns_config.chance_destroy_per_tick.is_some();
                if ui.checkbox(&mut has_destroy_chance, "").changed() {
                    if has_destroy_chance {
                        burns_config.chance_destroy_per_tick = Some(0.1);
                    } else {
                        burns_config.chance_destroy_per_tick = None;
                    }
                }

                if let Some(ref mut chance) = burns_config.chance_destroy_per_tick {
                    ui.add(egui::Slider::new(chance, 0.0..=1.0));
                }
            });

            let mut has_reaction = burns_config.reaction.is_some();
            if ui
                .checkbox(&mut has_reaction, "Produces particle when burning")
                .changed()
            {
                if has_reaction {
                    burns_config.reaction = Some(ReactionConfig {
                        produces: "Smoke".to_string(),
                        chance_to_produce: 0.1,
                    });
                } else {
                    burns_config.reaction = None;
                }
            }

            if let Some(ref mut reaction) = burns_config.reaction {
                ui.horizontal(|ui| {
                    ui.label("Produces:");
                    ui.text_edit_singleline(&mut reaction.produces);
                });

                ui.horizontal(|ui| {
                    ui.label("Chance:");
                    ui.add(egui::Slider::new(
                        &mut reaction.chance_to_produce,
                        0.0..=1.0,
                    ));
                });
            }

            let mut has_burning_colors = burns_config.burning_colors.is_some();
            if ui
                .checkbox(&mut has_burning_colors, "Custom burning colors")
                .changed()
            {
                if has_burning_colors {
                    burns_config.burning_colors = Some(vec![
                        Color::srgba_u8(255, 89, 0, 255),
                        Color::srgba_u8(255, 153, 0, 255),
                        Color::srgba_u8(255, 207, 0, 255),
                    ]);
                } else {
                    burns_config.burning_colors = None;
                }
            }

            if let Some(ref mut burning_colors) = burns_config.burning_colors {
                ui.label("Burning Colors:");

                if ui.button("➕ Add Color").clicked() {
                    burning_colors.push(Color::srgba_u8(255, 128, 0, 255));
                }

                let mut to_remove = None;
                for (i, color) in burning_colors.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        let srgba = color.to_srgba();
                        let mut color32 = egui::Color32::from_rgba_unmultiplied(
                            (srgba.red * 255.0) as u8,
                            (srgba.green * 255.0) as u8,
                            (srgba.blue * 255.0) as u8,
                            (srgba.alpha * 255.0) as u8,
                        );

                        if ui.color_edit_button_srgba(&mut color32).changed() {
                            *color =
                                Color::srgba_u8(color32.r(), color32.g(), color32.b(), color32.a());
                        }

                        if ui.button("❌").clicked() {
                            to_remove = Some(i);
                        }
                    });
                }

                if let Some(remove_index) = to_remove {
                    burning_colors.remove(remove_index);
                }
            }

            let mut has_fire_spreads = burns_config.spreads_fire.is_some();
            if ui.checkbox(&mut has_fire_spreads, "Fire Spreads").changed() {
                if has_fire_spreads {
                    burns_config.spreads_fire = Some(FireConfig {
                        burn_radius: 2.0,
                        chance_to_spread: 0.01,
                        destroys_on_spread: false,
                    });
                } else {
                    burns_config.spreads_fire = None;
                }
            }

            if let Some(ref mut fire_config) = burns_config.spreads_fire {
                ui.horizontal(|ui| {
                    ui.label("Burn Radius:");
                    ui.add(egui::Slider::new(&mut fire_config.burn_radius, 1.0..=100.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Chance to spread:");
                    ui.add(egui::Slider::new(
                        &mut fire_config.chance_to_spread,
                        0.0..=1.0,
                    ));
                });

                ui.checkbox(&mut fire_config.destroys_on_spread, "Destroys on spread");
            }

            ui.checkbox(&mut burns_config.ignites_on_spawn, "Ignites on spawn");
        }

        ui.separator();

        let mut has_fire = editor_data.fire_config.is_some();
        if ui.checkbox(&mut has_fire, "Emits Fire").changed() {
            if has_fire {
                editor_data.fire_config = Some(FireConfig {
                    burn_radius: 2.0,
                    chance_to_spread: 0.1,
                    destroys_on_spread: false,
                });
            } else {
                editor_data.fire_config = None;
            }
        }

        if let Some(ref mut fire_config) = editor_data.fire_config {
            ui.horizontal(|ui| {
                ui.label("Burn Radius:");
                ui.add(egui::Slider::new(&mut fire_config.burn_radius, 1.0..=100.0));
            });

            ui.horizontal(|ui| {
                ui.label("Chance to spread:");
                ui.add(egui::Slider::new(
                    &mut fire_config.chance_to_spread,
                    0.0..=1.0,
                ));
            });

            ui.checkbox(&mut fire_config.destroys_on_spread, "Destroys on spread");
        }
    }
}
