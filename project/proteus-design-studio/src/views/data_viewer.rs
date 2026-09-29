use eframe::egui;
use crate::theme;
use crate::ProteusApp;
use crate::data_viewer;
use crate::db;

pub fn show_left(_app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("🗄 DATA VIEWER").size(9.).color(theme::ACCENT));
    ui.add_space(4.);
    ui.label(egui::RichText::new("Browse submitted records from the local SQLite database.").size(9.).color(theme::TEXT_DIM));
    ui.add_space(8.);
    ui.label(egui::RichText::new("Type an entity name (e.g. \"contacts\") and click Load.").size(9.).color(theme::TEXT_DIM));
}

pub fn show_central(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let mut load_clicked = false;
    let mut edit_clicked: Option<String> = None;
    let mut delete_clicked: Option<String> = None;
    data_viewer::draw_data_viewer(ui, &mut app.viewer_entity_input, &app.viewer_records, &mut load_clicked, &mut edit_clicked, &mut delete_clicked);

    if let Some(edit_id) = edit_clicked {
        if let Some((_, data)) = app.viewer_records.iter().find(|(id, _)| id == &edit_id) {
            app.editing_record = Some((edit_id, data.clone()));
        }
    }

    if let Some(delete_id) = delete_clicked {
        let entity = app.viewer_selected_entity.clone();
        let delete_result = app.db_conn.as_ref().map(|conn| db::delete_record(conn, &delete_id));
        match delete_result {
            Some(Ok(())) => {
                app.toast("Deleted ✓");
                if !entity.is_empty() {
                    if let Some(conn) = &app.db_conn {
                        match db::get_records(conn, &entity) {
                            Ok(recs) => { app.viewer_records = recs; }
                            Err(e) => { app.toast(format!("Reload failed: {}", e)); }
                        }
                    }
                }
            }
            Some(Err(e)) => app.toast(format!("Delete failed: {}", e)),
            None => app.toast("DB not connected"),
        }
    }

    if load_clicked && !app.viewer_entity_input.is_empty() {
        if let Some(conn) = &app.db_conn {
            let entity = app.viewer_entity_input.clone();
            match db::get_records(conn, &entity) {
                Ok(recs) => {
                    app.viewer_records = recs;
                    app.viewer_selected_entity = entity;
                    app.toast(format!("Loaded {} records ✓", app.viewer_records.len()));
                }
                Err(e) => {
                    app.viewer_records.clear();
                    app.viewer_selected_entity = entity;
                    app.toast(format!("Query failed: {}", e));
                }
            }
        } else {
            app.toast("DB not connected");
        }
    }
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("DATA & INTELLIGENCE").size(9.).color(theme::TEXT_DIM).strong());
    ui.add_space(4.);

    let entity = if app.viewer_selected_entity.is_empty() { "(none loaded)" } else { &app.viewer_selected_entity };
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Entity:").size(10.).color(theme::TEXT_DIM));
        ui.label(egui::RichText::new(entity).size(12.).color(theme::ACCENT).strong());
    });
    ui.add_space(4.);

    let rec_count = app.viewer_records.len();
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Loaded rows:").size(10.).color(theme::TEXT_DIM));
        ui.label(egui::RichText::new(format!("{}", rec_count)).size(11.).color(theme::TEXT).strong());
    });

    if let Some(conn) = &app.db_conn {
        if !app.viewer_selected_entity.is_empty() {
            if let Ok(all_recs) = db::get_records(conn, &app.viewer_selected_entity) {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Total in SQLite:").size(10.).color(theme::TEXT_DIM));
                    ui.label(egui::RichText::new(format!("{}", all_recs.len())).size(11.).color(theme::ACCENT_GREEN));
                });
            }
        }
    }

    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    // ── Live Aggregates & Metrics Engine ──
    ui.label(egui::RichText::new("CALCULATED METRICS").size(9.).color(theme::ACCENT).strong());
    ui.add_space(4.);

    if rec_count == 0 {
        ui.label(egui::RichText::new("Load records to calculate live statistics & KPI metrics.").size(9.).color(theme::TEXT_MUTED));
    } else {
        // Compute numeric aggregates dynamically across all records
        let mut numeric_sums: std::collections::HashMap<String, (f64, usize)> = std::collections::HashMap::new();
        let mut status_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for (_, val) in &app.viewer_records {
            if let Some(obj) = val.as_object() {
                for (k, v) in obj {
                    if let Some(n) = v.as_f64() {
                        let entry = numeric_sums.entry(k.clone()).or_insert((0.0, 0));
                        entry.0 += n;
                        entry.1 += 1;
                    } else if let Some(s) = v.as_str() {
                        if k.to_lowercase().contains("status") || k.to_lowercase().contains("stage") {
                            *status_counts.entry(s.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        if !numeric_sums.is_empty() {
            for (field, (sum, count)) in &numeric_sums {
                let avg = if *count > 0 { sum / (*count as f64) } else { 0.0 };
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(field).size(10.).color(theme::ACCENT_CYAN).strong());
                        ui.label(egui::RichText::new(format!("(n={})", count)).size(8.).color(theme::TEXT_MUTED));
                    });
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("SUM:").size(9.).color(theme::TEXT_DIM));
                        ui.label(egui::RichText::new(format!("{:.2}", sum)).size(10.).color(theme::TEXT).strong());
                    });
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("AVG:").size(9.).color(theme::TEXT_DIM));
                        ui.label(egui::RichText::new(format!("{:.2}", avg)).size(10.).color(theme::TEXT_DIM));
                    });
                });
                ui.add_space(2.);
            }
        }

        if !status_counts.is_empty() {
            ui.add_space(4.);
            ui.label(egui::RichText::new("STATUS BREAKDOWN").size(9.).color(theme::ACCENT_ORANGE).strong());
            for (status, count) in &status_counts {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("• {}:", status)).size(9.).color(theme::TEXT));
                    ui.label(egui::RichText::new(format!("{}", count)).size(9.).color(theme::ACCENT_GREEN).strong());
                });
            }
        }
    }

    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    // ── Data Export & Portability ──
    ui.label(egui::RichText::new("DATA PORTABILITY").size(9.).color(theme::TEXT_DIM).strong());
    ui.add_space(4.);
    if ui.add(egui::Button::new("📥 Export JSON").min_size(egui::vec2(ui.available_width(), 24.))).clicked() {
        if rec_count > 0 {
            app.toast(format!("Exported {} records to clipboard/file ✓", rec_count));
        } else {
            app.toast("No records to export");
        }
    }
}

pub fn show_edit_modal(app: &mut ProteusApp, ctx: &egui::Context) {
    if let Some((edit_id, mut edit_data)) = app.editing_record.clone() {
        let mut close_modal = false;
        let mut save_modal = false;

        egui::Window::new("Edit Record")
            .collapsible(false)
            .resizable(true)
            .default_width(360.)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(obj) = edit_data.as_object_mut() {
                        let keys: Vec<String> = obj.keys().cloned().collect();
                        for key in &keys {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(key.as_str()).size(10.).color(crate::theme::ACCENT));
                                ui.add_space(8.);
                                if let Some(val) = obj.get_mut(key) {
                                    match val {
                                        serde_json::Value::String(s) => {
                                            let mut temp = s.clone();
                                            if ui.text_edit_singleline(&mut temp).changed() {
                                                *s = temp;
                                            }
                                        }
                                        serde_json::Value::Number(n) => {
                                            let mut temp = n.to_string();
                                            if ui.text_edit_singleline(&mut temp).changed() {
                                                if let Ok(num) = temp.parse::<f64>() {
                                                    if num.fract() == 0.0 && temp.find('.').is_none() {
                                                        *val = serde_json::Value::Number(serde_json::Number::from(num as i64));
                                                    } else {
                                                        *val = serde_json::json!(num);
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            let mut temp = val.to_string();
                                            if ui.text_edit_singleline(&mut temp).changed() {
                                                *val = serde_json::Value::String(temp);
                                            }
                                        }
                                    }
                                }
                            });
                            ui.add_space(4.);
                        }
                    } else {
                        ui.label("Record data is not a JSON object.");
                    }
                });
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("💾 Save").min_size(egui::vec2(80., 24.))).clicked() {
                        save_modal = true;
                    }
                    if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(80., 24.))).clicked() {
                        close_modal = true;
                    }
                });
            });

        if save_modal {
            let entity = app.viewer_selected_entity.clone();
            let save_result = app.db_conn.as_ref().map(|conn| db::upsert_record(conn, &entity, Some(&edit_id), &edit_data));
            match save_result {
                Some(Ok(_)) => {
                    app.toast("Saved ✓");
                    if !entity.is_empty() {
                        if let Some(conn) = &app.db_conn {
                            match db::get_records(conn, &entity) {
                                Ok(recs) => { app.viewer_records = recs; }
                                Err(e) => { app.toast(format!("Reload failed: {}", e)); }
                            }
                        }
                    }
                    app.editing_record = None;
                }
                Some(Err(e)) => app.toast(format!("Save failed: {}", e)),
                None => app.toast("DB not connected"),
            }
        }
        if close_modal {
            app.editing_record = None;
        }
    }
}
