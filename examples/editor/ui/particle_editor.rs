use bevy_egui::egui;

pub struct ParticleEditor;

impl ParticleEditor {
    pub fn render(&self, ui: &mut egui::Ui) {
        let text_color = egui::Color32::from_rgb(204, 204, 204);
        ui.visuals_mut().override_text_color = Some(text_color);

        ui.heading("Particle Editor");
        ui.separator();

        // Use the original columns approach - it works fine with adequate width
        ui.columns(2, |columns| {
            columns[0].set_min_width(columns[0].available_width());
            columns[0].set_max_width(columns[0].available_width());
            self.render_particle_list(&mut columns[0]);
            
            columns[1].set_min_width(columns[1].available_width());
            columns[1].set_max_width(columns[1].available_width());
            self.render_particle_properties(&mut columns[1]);
        });
    }

    fn render_particle_list(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("particle_list_scroll")
            .show(ui, |ui| {
                const CATEGORIES: [&str; 5] =
                    ["Walls", "Solids", "Movable Solids", "Liquids", "Gases"];

                for &category in &CATEGORIES {
                    // Temporarily increase spacing for collapsing headers
                    let original_indent = ui.spacing().indent;
                    ui.spacing_mut().indent = 16.0;
                    
                    egui::CollapsingHeader::new(category)
                        .default_open(false)
                        .show(ui, |ui| {
                            // Restore original spacing inside the collapsing content
                            ui.spacing_mut().indent = original_indent;
                            let examples = match category {
                                "Walls" => vec!["Stone Wall", "Dirt Wall", "Wood Wall"],
                                "Solids" => vec!["Sand", "Dirt", "Snow"],
                                "Movable Solids" => vec!["Powder", "Salt", "Sugar"],
                                "Liquids" => vec!["Water", "Oil", "Acid"],
                                "Gases" => vec!["Steam", "Smoke", "Vapor"],
                                _ => vec![],
                            };

                            for particle_name in examples {
                                if ui.button(particle_name).clicked() {}
                            }
                        });
                    
                    // Restore original spacing after the collapsing header
                    ui.spacing_mut().indent = original_indent;
                }

                ui.add_space(8.0);
                if ui.button("New Particle").clicked() {}
                if ui.button("Save Particle").clicked() {}
            });
    }

    fn render_particle_properties(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("particle_properties_scroll")
            .show(ui, |ui| {
                let mut name = "Sand".to_string();
                ui.text_edit_singleline(&mut name);

                ui.horizontal(|ui| {
                    ui.label("State:");
                    egui::ComboBox::from_label("")
                        .selected_text("Solid")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut true, true, "Wall");
                            ui.selectable_value(&mut false, false, "Solid");
                            ui.selectable_value(&mut false, false, "Movable Solid");
                            ui.selectable_value(&mut false, false, "Liquid");
                            ui.selectable_value(&mut false, false, "Gas");
                        });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Colors");
                    if ui.button("➕").clicked() {}
                });

                for i in 0..3 {
                    ui.horizontal(|ui| {
                        ui.label(format!("R: 194 G: 178 B: {}", 128 + i * 20));
                        let mut color = egui::Color32::from_rgb(194, 178, 128);
                        ui.color_edit_button_srgba(&mut color);
                        if ui.button("❌").clicked() {}
                    });
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Density:");
                    let mut density = 100.0;
                    ui.add(egui::Slider::new(&mut density, 1.0..=1000.0).step_by(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Max Velocity:");
                    let mut velocity = 3.0;
                    ui.add(egui::Slider::new(&mut velocity, 1.0..=5.0).step_by(1.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Momentum");
                    let mut momentum = false;
                    ui.checkbox(&mut momentum, "");
                });

                ui.separator();

                let mut flammable = false;
                ui.checkbox(&mut flammable, "Flammable");

                if flammable {
                    ui.horizontal(|ui| {
                        ui.label("Duration (ms):");
                        let mut duration = "1000".to_string();
                        ui.add(egui::TextEdit::singleline(&mut duration).desired_width(60.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Tick Rate (ms):");
                        let mut tick_rate = "100".to_string();
                        ui.add(egui::TextEdit::singleline(&mut tick_rate).desired_width(60.0));
                    });

                    let mut changes_color = false;
                    ui.checkbox(&mut changes_color, "Change colors while burning");

                    let mut spreads = false;
                    ui.checkbox(&mut spreads, "Fire Spreads");

                    if spreads {
                        ui.horizontal(|ui| {
                            ui.label("Burn Radius:");
                            let mut radius = 2.0;
                            ui.add(egui::Slider::new(&mut radius, 1.0..=100.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Chance to spread:");
                            let mut chance = 0.01;
                            ui.add(egui::Slider::new(&mut chance, 0.0..=1.0));
                        });
                    }
                }
            });
    }
}
