pub fn draw_data_viewer(
    ui: &mut egui::Ui,
    entity_input: &mut String,
    records: &[(String, serde_json::Value)],
    on_load: &mut bool,
    on_edit: &mut Option<String>,
    on_delete: &mut Option<String>,
) {
    ui.add_space(8.);
    ui.horizontal(|ui| {
        ui.label("Entity:");
        let resp = ui.add(egui::TextEdit::singleline(entity_input).hint_text("e.g. contacts").desired_width(180.));
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            *on_load = true;
        }
        if ui.button("Load").clicked() {
            *on_load = true;
        }
    });

    if entity_input.is_empty() {
        ui.add_space(20.);
        ui.label("Type an entity name and hit Enter or click Load.");
        return;
    }

    ui.add_space(8.);
    ui.separator();

    if records.is_empty() {
        ui.add_space(20.);
        ui.label(format!("No records found for \"{}\".", entity_input));
        return;
    }

    let columns = match &records[0].1 {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            keys
        }
        _ => { ui.label("Record data is not a JSON object."); return; }
    };

    if columns.is_empty() {
        ui.label("Record has no fields.");
        return;
    }

    // Clean grid layout ensuring strict vertical alignment between headers and row values
    egui::ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
        egui::Grid::new("data_viewer_grid")
            .striped(true)
            .spacing(egui::vec2(16.0, 8.0))
            .min_col_width(70.0)
            .show(ui, |ui| {
                // Header row
                ui.label(egui::RichText::new("ID").size(10.5).strong().color(crate::theme::ACCENT));
                for key in &columns {
                    ui.label(egui::RichText::new(key.as_str()).size(10.5).strong().color(crate::theme::ACCENT));
                }
                ui.label(egui::RichText::new("Actions").size(10.5).strong().color(crate::theme::ACCENT));
                ui.end_row();

                // Data rows
                for (id, data) in records {
                    let map = match data {
                        serde_json::Value::Object(m) => m,
                        _ => continue,
                    };

                    ui.label(egui::RichText::new(&id[..8.min(id.len())]).size(9.5).color(crate::theme::TEXT_DIM));
                    for key in &columns {
                        let val: String = map.get(key.as_str())
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Null => String::new(),
                                other => other.to_string(),
                            })
                            .unwrap_or_else(|| String::from("—"));
                        ui.label(egui::RichText::new(val).size(10.0).color(crate::theme::TEXT));
                    }
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new(egui::RichText::new("✎ Edit").size(9.5))).clicked() {
                            *on_edit = Some(id.clone());
                        }
                        if ui.add(egui::Button::new(egui::RichText::new("🗑").size(9.5).color(crate::theme::ACCENT_RED))).clicked() {
                            *on_delete = Some(id.clone());
                        }
                    });
                    ui.end_row();
                }
            });
    });
}
