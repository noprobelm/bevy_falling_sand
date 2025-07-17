mod top_bar;

use top_bar::UiTopBar;

use bevy::prelude::*;
pub(super) use bevy_egui::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_systems(EguiContextPass, render);
    }
}

fn render(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    egui::TopBottomPanel::top("Top panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            UiTopBar.render(ui);
        });
    });

    egui::SidePanel::left("left_panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                ui.available_rect_before_wrap(),
                0.0,
                egui::Color32::from_rgb(30, 30, 30),
            );

            let margin = 4.0;
            let window_spacing = 8.0;
            let panel_bg = egui::Color32::from_rgb(46, 46, 46);
            let text_color = egui::Color32::from_rgb(204, 204, 204);
            let available_height = ui.available_height();
            let window_height = (available_height - margin * 2.0 - window_spacing) / 2.0;

            // Use TopBottomPanel for guaranteed 50/50 split
            egui::TopBottomPanel::top("particle_editor_window")
                .exact_height(window_height)
                .resizable(false)
                .show_inside(ui, |ui| {
                    egui::Frame::NONE
                        .fill(panel_bg)
                        .rounding(4.0)
                        .inner_margin(egui::Margin::same(8))
                        .show(ui, |ui| {
                            ui.visuals_mut().override_text_color = Some(text_color);

                            // Header
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Particle Editor").size(14.0).strong(),
                                );
                            });
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            // Visual-only particle editor replica
                            ui.horizontal(|ui| {
                                // Left side - particle list (auto-sizing)
                                ui.vertical(|ui| {
                                    egui::ScrollArea::vertical()
                                        .id_salt("particle_list_scroll")
                                        .auto_shrink([true, false])
                                        .max_height(window_height - 60.0) // Reserve space for header and margins
                                        .show(ui, |ui| {
                                            // Particle categories
                                            const CATEGORIES: [&str; 5] = [
                                                "Walls",
                                                "Solids",
                                                "Movable Solids",
                                                "Liquids",
                                                "Gases",
                                            ];

                                            for &category in &CATEGORIES {
                                                egui::CollapsingHeader::new(category)
                                                    .default_open(false)
                                                    .id_salt(category)
                                                    .show(ui, |ui| {
                                                        // Example particles
                                                        let examples = match category {
                                                            "Walls" => vec![
                                                                "Stone Wall",
                                                                "Dirt Wall",
                                                                "Wood Wall",
                                                            ],
                                                            "Solids" => {
                                                                vec!["Sand", "Dirt", "Snow"]
                                                            }
                                                            "Movable Solids" => {
                                                                vec!["Powder", "Salt", "Sugar"]
                                                            }
                                                            "Liquids" => {
                                                                vec!["Water", "Oil", "Acid"]
                                                            }
                                                            "Gases" => {
                                                                vec!["Steam", "Smoke", "Vapor"]
                                                            }
                                                            _ => vec![],
                                                        };

                                                        for particle_name in examples {
                                                            if ui.button(particle_name).clicked() {}
                                                        }
                                                    });
                                            }

                                            ui.add_space(8.0);
                                            if ui.button("New Particle").clicked() {}
                                            if ui.button("Save Particle").clicked() {}
                                        });
                                });

                                ui.separator();

                                // Right side - particle properties (auto-sizing)
                                ui.vertical(|ui| {
                                    egui::ScrollArea::vertical()
                                        .id_salt("particle_properties_scroll")
                                        .auto_shrink([true, false])
                                        .max_height(window_height - 60.0) // Reserve space for header and margins
                                        .show(ui, |ui| {
                                            // Name field
                                            let mut name = "Sand".to_string();
                                            ui.text_edit_singleline(&mut name);

                                            // State selector
                                            ui.horizontal(|ui| {
                                                ui.label("State:");
                                                egui::ComboBox::from_label("")
                                                    .selected_text("Solid")
                                                    .show_ui(ui, |ui| {
                                                        ui.selectable_value(
                                                            &mut true, true, "Wall",
                                                        );
                                                        ui.selectable_value(
                                                            &mut false, false, "Solid",
                                                        );
                                                        ui.selectable_value(
                                                            &mut false,
                                                            false,
                                                            "Movable Solid",
                                                        );
                                                        ui.selectable_value(
                                                            &mut false, false, "Liquid",
                                                        );
                                                        ui.selectable_value(
                                                            &mut false, false, "Gas",
                                                        );
                                                    });
                                            });

                                            ui.separator();

                                            // Colors section
                                            ui.horizontal(|ui| {
                                                ui.label("Colors");
                                                if ui.button("➕").clicked() {}
                                            });

                                            // Example colors
                                            for i in 0..3 {
                                                ui.horizontal(|ui| {
                                                    ui.label(format!(
                                                        "R: 194 G: 178 B: {}",
                                                        128 + i * 20
                                                    ));
                                                    let mut color =
                                                        egui::Color32::from_rgb(194, 178, 128);
                                                    ui.color_edit_button_srgba(&mut color);
                                                    if ui.button("❌").clicked() {}
                                                });
                                            }

                                            ui.separator();

                                            // Density
                                            ui.horizontal(|ui| {
                                                ui.label("Density:");
                                                let mut density = 100.0;
                                                ui.add(
                                                    egui::Slider::new(&mut density, 1.0..=1000.0)
                                                        .step_by(1.0),
                                                );
                                            });

                                            // Max velocity
                                            ui.horizontal(|ui| {
                                                ui.label("Max Velocity:");
                                                let mut velocity = 3.0;
                                                ui.add(
                                                    egui::Slider::new(&mut velocity, 1.0..=5.0)
                                                        .step_by(1.0),
                                                );
                                            });

                                            // Momentum checkbox
                                            ui.horizontal(|ui| {
                                                ui.label("Momentum");
                                                let mut momentum = false;
                                                ui.checkbox(&mut momentum, "");
                                            });

                                            ui.separator();

                                            // Flammable section
                                            let mut flammable = false;
                                            ui.checkbox(&mut flammable, "Flammable");

                                            if flammable {
                                                ui.horizontal(|ui| {
                                                    ui.label("Duration (ms):");
                                                    let mut duration = "1000".to_string();
                                                    ui.add(
                                                        egui::TextEdit::singleline(&mut duration)
                                                            .desired_width(40.0),
                                                    );
                                                });

                                                ui.horizontal(|ui| {
                                                    ui.label("Tick Rate (ms):");
                                                    let mut tick_rate = "100".to_string();
                                                    ui.add(
                                                        egui::TextEdit::singleline(&mut tick_rate)
                                                            .desired_width(40.0),
                                                    );
                                                });

                                                let mut changes_color = false;
                                                ui.checkbox(
                                                    &mut changes_color,
                                                    "Change colors while burning",
                                                );

                                                let mut spreads = false;
                                                ui.checkbox(&mut spreads, "Fire Spreads");

                                                if spreads {
                                                    ui.horizontal(|ui| {
                                                        ui.label("Burn Radius:");
                                                        let mut radius = 2.0;
                                                        ui.add(egui::Slider::new(
                                                            &mut radius,
                                                            1.0..=100.0,
                                                        ));
                                                    });

                                                    ui.horizontal(|ui| {
                                                        ui.label("Chance to spread:");
                                                        let mut chance = 0.01;
                                                        ui.add(egui::Slider::new(
                                                            &mut chance,
                                                            0.0..=1.0,
                                                        ));
                                                    });
                                                }
                                            }
                                        });
                                });
                            });
                        });
                });

            // Bottom window - Layers (takes remaining space)
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::NONE
                        .fill(panel_bg)
                        .rounding(4.0)
                        .inner_margin(egui::Margin::same(8)),
                )
                .show_inside(ui, |ui| {
                    ui.visuals_mut().override_text_color = Some(text_color);

                    // Header
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Layers").size(14.0).strong());
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Layer controls
                    ui.horizontal(|ui| {
                        if ui.button("➕ Add").clicked() {}
                        if ui.button("🗑 Delete").clicked() {}
                        if ui.button("👁 Toggle").clicked() {}
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .id_salt("layers_scroll")
                        .auto_shrink([true, false])
                        .max_height(window_height - 60.0) // Reserve space for header and margins
                        .show(ui, |ui| {
                            // Example layers
                            for i in (0..15).rev() {
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut (i == 0), "");
                                    ui.label(format!("Layer {}", i));
                                    if i == 0 {
                                        ui.label("(active)");
                                    }
                                });
                            }
                        });
                });
        });
}

