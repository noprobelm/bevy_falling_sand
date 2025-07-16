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

    // Left side windows - styled like shaderlab
    egui::Area::new(egui::Id::new("left_panels"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let left_panel_width = 300.0;
            let margin = 4.0;
            let window_spacing = 8.0;
            let topbar_height = 30.0; // Height of the top panel

            let available_rect = ctx.available_rect();
            let left_area = egui::Rect::from_min_size(
                egui::pos2(margin, topbar_height + margin),
                egui::vec2(
                    left_panel_width,
                    available_rect.height() - topbar_height - margin * 2.0,
                ),
            );

            let window_height = (left_area.height() - window_spacing) / 2.0;

            // Define panel colors similar to shaderlab
            let panel_bg = egui::Color32::from_rgb(46, 46, 46);
            let panel_header_bg = egui::Color32::from_rgb(60, 60, 60);
            let text_color = egui::Color32::from_rgb(204, 204, 204);

            // Top window - Properties
            let top_window_rect = egui::Rect::from_min_size(
                left_area.min,
                egui::vec2(left_panel_width, window_height),
            );

            ui.allocate_rect(top_window_rect, egui::Sense::hover());
            ui.painter().rect_filled(top_window_rect, 4.0, panel_bg);

            let mut top_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(top_window_rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );
            top_ui.visuals_mut().override_text_color = Some(text_color);

            egui::Frame::NONE
                .inner_margin(egui::Margin::same(8))
                .show(&mut top_ui, |ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Properties").size(14.0).strong());
                    });
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.label("Brush Settings");
                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                ui.label("Size:");
                                ui.add(egui::Slider::new(&mut 5.0_f32, 1.0..=50.0));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Shape:");
                                egui::ComboBox::from_label("")
                                    .selected_text("Circle")
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut true, true, "Circle");
                                        ui.selectable_value(&mut false, false, "Square");
                                        ui.selectable_value(&mut false, false, "Diamond");
                                    });
                            });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            ui.label("Material");
                            ui.horizontal(|ui| {
                                if ui.button("Sand").clicked() {}
                                if ui.button("Water").clicked() {}
                                if ui.button("Stone").clicked() {}
                            });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            ui.checkbox(&mut true, "Enable physics");
                            ui.checkbox(&mut false, "Show grid");
                            ui.checkbox(&mut true, "Auto-save");

                            ui.add_space(8.0);

                            // Add more content to demonstrate scrolling
                            for i in 0..10 {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Option {}", i));
                                    ui.add(egui::Slider::new(&mut 0.5_f32, 0.0..=1.0));
                                });
                            }
                        });
                });

            // Bottom window - Layers
            let bottom_window_rect = egui::Rect::from_min_size(
                egui::pos2(
                    left_area.min.x,
                    left_area.min.y + window_height + window_spacing,
                ),
                egui::vec2(left_panel_width, window_height),
            );

            ui.allocate_rect(bottom_window_rect, egui::Sense::hover());
            ui.painter().rect_filled(bottom_window_rect, 4.0, panel_bg);

            let mut bottom_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(bottom_window_rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );
            bottom_ui.visuals_mut().override_text_color = Some(text_color);

            egui::Frame::NONE
                .inner_margin(egui::Margin::same(8))
                .show(&mut bottom_ui, |ui| {
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
                        .auto_shrink([false; 2])
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
