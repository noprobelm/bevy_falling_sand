use bevy::prelude::*;
use bevy_egui::egui;
use bevy_falling_sand::prelude::ParticleMaterialsParam;

use super::particle_editor::LoadParticleIntoEditor;

#[derive(Resource)]
pub struct ParticleSearchState {
    pub active: bool,
    pub input: String,
    pub suggestions: Vec<String>,
    pub suggestion_index: Option<usize>,
    pub needs_initial_focus: bool,
}

impl Default for ParticleSearchState {
    fn default() -> Self {
        Self {
            active: false,
            input: String::new(),
            suggestions: Vec::new(),
            suggestion_index: None,
            needs_initial_focus: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct ParticleSearchCache {
    pub all_particles: Vec<String>,
}

impl ParticleSearchState {
    pub fn activate(&mut self) {
        self.active = true;
        self.input.clear();
        self.suggestions.clear();
        self.suggestion_index = None;
        self.needs_initial_focus = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.input.clear();
        self.suggestions.clear();
        self.suggestion_index = None;
        self.needs_initial_focus = false;
    }

    pub fn update_suggestions(&mut self, cache: &ParticleSearchCache) {
        self.suggestions.clear();
        self.suggestion_index = None;

        if !self.input.is_empty() {
            let query = self.input.to_lowercase();
            
            // Only use actual particle names, with case-insensitive matching
            let mut matches: Vec<String> = cache.all_particles
                .iter()
                .filter(|particle| {
                    let particle_lower = particle.to_lowercase();
                    particle_lower.contains(&query)
                })
                .cloned()
                .collect();
            
            // Sort by relevance: prefix matches first, then by length
            matches.sort_by(|a, b| {
                let a_lower = a.to_lowercase();
                let b_lower = b.to_lowercase();
                let a_starts = a_lower.starts_with(&query);
                let b_starts = b_lower.starts_with(&query);
                
                match (a_starts, b_starts) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.len().cmp(&b.len()),
                }
            });

            // Limit to top 10 results
            self.suggestions = matches.into_iter().take(10).collect();

            if !self.suggestions.is_empty() {
                self.suggestion_index = Some(0);
            }
        }
    }

    pub fn navigate_suggestions(&mut self, up: bool) {
        if self.suggestions.is_empty() {
            return;
        }

        match self.suggestion_index {
            Some(current) => {
                if up {
                    if current > 0 {
                        self.suggestion_index = Some(current - 1);
                    } else {
                        self.suggestion_index = Some(self.suggestions.len() - 1);
                    }
                } else if current + 1 < self.suggestions.len() {
                    self.suggestion_index = Some(current + 1);
                } else {
                    self.suggestion_index = Some(0);
                }
            }
            None => {
                self.suggestion_index = Some(if up { self.suggestions.len() - 1 } else { 0 });
            }
        }
    }

    pub fn get_selected_suggestion(&self) -> Option<&String> {
        self.suggestion_index
            .and_then(|idx| self.suggestions.get(idx))
    }

    pub fn select_current_suggestion(&mut self) -> Option<String> {
        if let Some(suggestion) = self.get_selected_suggestion() {
            let result = suggestion.clone();
            self.deactivate();
            Some(result)
        } else {
            None
        }
    }
}

pub struct ParticleSearch;

impl ParticleSearch {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        state: &mut ParticleSearchState,
        cache: &ParticleSearchCache,
        load_particle_events: &mut EventWriter<LoadParticleIntoEditor>,
    ) -> bool {
        if !state.active {
            return false;
        }

        let mut should_close = false;

        // Check for ESC key globally when search is active
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            should_close = true;
        }

        egui::Window::new("Search Particles")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 50.0))
            .show(ui.ctx(), |ui| {
                ui.set_min_width(400.0);
                
                // Search input
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.input)
                        .hint_text("Type particle name...")
                        .desired_width(ui.available_width())
                        .lock_focus(true)
                );

                // Auto-focus on activation
                if state.needs_initial_focus {
                    response.request_focus();
                    state.needs_initial_focus = false;
                }

                // Handle input changes
                if response.changed() {
                    state.update_suggestions(cache);
                }

                // Handle key presses - check for keys when input has focus
                if response.has_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                        state.navigate_suggestions(true);
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                        state.navigate_suggestions(false);
                    }

                    // Tab for auto-completion
                    if ui.input(|i| i.key_pressed(egui::Key::Tab)) && !state.suggestions.is_empty() {
                        if let Some(suggestion) = state.get_selected_suggestion() {
                            state.input = suggestion.clone();
                            state.update_suggestions(cache);
                        }
                    }
                }

                // Handle Enter key when losing focus or when explicitly pressed
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || 
                   (response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    if !state.suggestions.is_empty() {
                        if let Some(selected) = state.select_current_suggestion() {
                            load_particle_events.write(LoadParticleIntoEditor {
                                particle_name: selected,
                            });
                            should_close = true;
                        }
                    }
                }

                // Suggestions list
                if !state.suggestions.is_empty() {
                    ui.separator();
                    
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (i, particle) in state.suggestions.iter().enumerate() {
                                let is_selected = state.suggestion_index == Some(i);
                                
                                let response = ui.selectable_label(is_selected, particle);
                                
                                if response.clicked() {
                                    load_particle_events.write(LoadParticleIntoEditor {
                                        particle_name: particle.clone(),
                                    });
                                    should_close = true;
                                }

                                if response.hovered() {
                                    state.suggestion_index = Some(i);
                                }
                            }
                        });
                } else if !state.input.is_empty() {
                    ui.separator();
                    ui.label("No particles found");
                }

                // Instructions
                ui.separator();
                ui.horizontal(|ui| {
                    ui.small("↑↓ Navigate");
                    ui.small("Tab: Complete");
                    ui.small("Enter: Select");
                    ui.small("Esc: Close");
                });
            });

        if should_close {
            state.deactivate();
        }

        true
    }
}

// System to update particle cache when new particles are loaded
pub fn update_particle_search_cache(
    particle_materials: ParticleMaterialsParam,
    mut cache: ResMut<ParticleSearchCache>,
) {
    let mut all_particles = Vec::new();
    
    // Collect all particle names from all categories
    for particle_type in particle_materials.walls() {
        all_particles.push(particle_type.name.to_string());
    }
    for particle_type in particle_materials.solids() {
        all_particles.push(particle_type.name.to_string());
    }
    for particle_type in particle_materials.movable_solids() {
        all_particles.push(particle_type.name.to_string());
    }
    for particle_type in particle_materials.liquids() {
        all_particles.push(particle_type.name.to_string());
    }
    for particle_type in particle_materials.gases() {
        all_particles.push(particle_type.name.to_string());
    }
    for particle_type in particle_materials.other() {
        all_particles.push(particle_type.name.to_string());
    }

    // Remove duplicates and sort
    all_particles.sort();
    all_particles.dedup();

    // Only update if the list has changed
    if cache.all_particles != all_particles {
        cache.all_particles = all_particles;
    }
}

// System to handle keyboard shortcut to activate search
pub fn handle_particle_search_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut search_state: ResMut<ParticleSearchState>,
    mut contexts: bevy_egui::EguiContexts,
) {
    // KeyN to activate search (matching sandbox behavior)
    // Only activate if not already active and console input doesn't have keyboard focus
    let console_input_id = bevy_egui::egui::Id::new("console_input");
    let console_has_focus = contexts.ctx_mut().memory(|mem| {
        mem.focused() == Some(console_input_id)
    });
    
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        if !search_state.active && !console_has_focus {
            search_state.activate();
        }
    }
}