use bevy_egui::egui;

pub struct LayersPanel;

impl LayersPanel {
    pub fn render(&self, ui: &mut egui::Ui) {
        let text_color = egui::Color32::from_rgb(204, 204, 204);
        ui.visuals_mut().override_text_color = Some(text_color);

        ui.heading("Layers");
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("➕ Add").clicked() {}
            if ui.button("🗑 Delete").clicked() {}
            if ui.button("👁 Toggle").clicked() {}
        });

        ui.separator();
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .id_salt("layers_scroll")
            .show(ui, |ui| {
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
    }
}