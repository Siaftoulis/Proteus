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

    // ponytail: flat table via horizontal-groups inside scroll, avoids egui_extras dependency
    egui::ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
        // Header
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(egui::RichText::new("ID").size(10.).strong().color(crate::theme::ACCENT));
                    ui.add_space(8.);
                    for key in &columns {
                        ui.label(egui::RichText::new(key.as_str()).size(10.).strong().color(crate::theme::ACCENT));
                        ui.add_space(8.);
                    }
                    ui.label(egui::RichText::new("Actions").size(10.).strong().color(crate::theme::ACCENT));
                }
            );
        });
        ui.separator();

        // Rows
        for (id, data) in records {
            let map = match data {
                serde_json::Value::Object(m) => m,
                _ => continue,
            };
            let bg = crate::theme::WIDGET_BG;
            egui::Frame::default().fill(bg).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&id[..8.min(id.len())]).size(9.).color(crate::theme::TEXT_DIM));
                    ui.add_space(8.);
                    for key in &columns {
                        let val: String = map.get(key.as_str())
                            .map(|v| match v {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Null => String::new(),
                                other => other.to_string(),
                            })
                            .unwrap_or_else(|| String::from("N/A"));
                        ui.label(egui::RichText::new(val).size(10.).color(crate::theme::TEXT));
                        ui.add_space(8.);
                    }
                    if ui.add(egui::Button::new("✎ Edit").min_size(egui::vec2(40., 16.))).clicked() {
                        *on_edit = Some(id.clone());
                    }
                    ui.add_space(2.);
                    if ui.add(egui::Button::new("🗑 Delete").min_size(egui::vec2(50., 16.))).clicked() {
                        *on_delete = Some(id.clone());
                    }
                });
            });
            ui.separator();
        }
    });
}
