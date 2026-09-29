//! Bespoke Data-Bound Table & Dynamic Form widgets for Proteus CRM.
//! Provides native offline-first SQLite data binding, live Designer preview,
//! pagination, search filtering, and schema-driven CRUD generation.

use eframe::egui::{self, Color32, RichText, Sense, Stroke};
use crate::scene::{CanvasEvent, Node, NodeStyle, Styling};
use crate::theme;

fn zoom_font(base: f32, z: f32) -> f32 {
    (base * z).clamp(6.0, 48.0)
}

fn style_fill(s: &NodeStyle) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (s.bg_color[0] * 255.) as u8,
        (s.bg_color[1] * 255.) as u8,
        (s.bg_color[2] * 255.) as u8,
        (s.bg_color[3] * 255.) as u8,
    )
}

fn style_border_color(s: &NodeStyle) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (s.border_color[0] * 255.) as u8,
        (s.border_color[1] * 255.) as u8,
        (s.border_color[2] * 255.) as u8,
        (s.border_color[3] * 255.) as u8,
    )
}

fn widget_frame(styling: &Styling, style: &NodeStyle, z: f32) -> egui::Frame {
    let bg = styling.background.map(|c| Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a))
        .unwrap_or_else(|| style_fill(style));
    let stroke_w = (style.border_width * z).max(0.5);
    let stroke_c = styling.border.as_ref()
        .map(|b| Color32::from_rgba_premultiplied(b.color.r, b.color.g, b.color.b, b.color.a))
        .unwrap_or_else(|| style_border_color(style));

    egui::Frame::default()
        .fill(bg)
        .corner_radius(style.border_radius * z)
        .stroke(Stroke::new(stroke_w, stroke_c))
        .inner_margin(8.0 * z)
}

/// Renders the DataBoundTable widget with live SQLite data preview, search filter, and row actions.
#[allow(clippy::too_many_arguments)]
pub fn render_data_bound_table(
    ui: &mut egui::Ui,
    node_id: &str,
    node: &Node,
    bound_entity: Option<&str>,
    columns: &[String],
    z: f32,
    play_mode: bool,
    form_state: &mut std::collections::HashMap<String, String>,
    table_cache: &std::collections::HashMap<String, Vec<(String, serde_json::Value)>>,
    events: &mut Vec<CanvasEvent>,
) {
    let frame = widget_frame(&node.styling, &node.style, z);
    frame.show(ui, |ui| {
        ui.set_min_size(ui.available_size());
        let ent = bound_entity.unwrap_or("contacts");

        // 1. Header Toolbar
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("⊞ {}", ent.to_uppercase()))
                    .size(zoom_font(11.0, z))
                    .color(theme::ACCENT)
                    .strong(),
            );

            if !play_mode {
                ui.label(
                    RichText::new("● Live SQLite")
                        .size(zoom_font(8.5, z))
                        .color(Color32::from_rgb(52, 199, 89)),
                );
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let rec_count = table_cache.get(ent).map(|v| v.len()).unwrap_or(0);
                ui.label(
                    RichText::new(format!("{} records", rec_count))
                        .size(zoom_font(8.5, z))
                        .color(theme::TEXT_MUTED),
                );

                // Search Filter input
                let search_key = format!("{}:search", node_id);
                let search_val = form_state.entry(search_key).or_default();
                if play_mode {
                    ui.add(
                        egui::TextEdit::singleline(search_val)
                            .hint_text("🔍 Filter...")
                            .desired_width(70.0 * z),
                    );
                }
            });
        });

        ui.add_space(3.0 * z);

        // 2. Column Resolution
        let default_cols = match ent {
            "deals" => vec!["ID".into(), "Title".into(), "Value".into(), "Stage".into()],
            "tickets" => vec!["ID".into(), "Customer".into(), "Phone".into(), "Status".into()],
            _ => vec!["ID".into(), "Name".into(), "Email".into(), "Phone".into(), "Company".into()],
        };
        let col_names = if columns.is_empty() { &default_cols } else { columns };

        let num_cols = col_names.len() + if play_mode { 1 } else { 0 };
        let col_w = (ui.available_width() / num_cols as f32).max(45.0 * z);

        // 3. Table Column Headers
        ui.horizontal(|ui| {
            for col in col_names {
                ui.allocate_ui_with_layout(
                    egui::vec2(col_w, 16.0 * z),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(col)
                                .size(zoom_font(9.0, z))
                                .color(theme::TEXT_DIM)
                                .strong(),
                        );
                    },
                );
            }
            if play_mode {
                ui.allocate_ui_with_layout(
                    egui::vec2(col_w * 0.6, 16.0 * z),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label(RichText::new("Act").size(zoom_font(8.5, z)).color(theme::TEXT_MUTED));
                    },
                );
            }
        });

        ui.separator();

        // 4. Data Rows from SQLite cache
        let search_key = format!("{}:search", node_id);
        let search_query = form_state.get(&search_key).cloned().unwrap_or_default().to_lowercase();
        let db_records = table_cache.get(ent);

        if let Some(recs) = db_records {
            let filtered: Vec<&(String, serde_json::Value)> = recs.iter().filter(|(id, val)| {
                if search_query.is_empty() {
                    return true;
                }
                if id.to_lowercase().contains(&search_query) {
                    return true;
                }
                if let serde_json::Value::Object(map) = val {
                    for v in map.values() {
                        if let Some(s) = v.as_str() {
                            if s.to_lowercase().contains(&search_query) {
                                return true;
                            }
                        }
                    }
                }
                false
            }).collect();

            if filtered.is_empty() {
                ui.add_space(6.0 * z);
                ui.label(
                    RichText::new(if search_query.is_empty() {
                        format!("No '{}' records found in SQLite. Submit a Dynamic Form to insert!", ent)
                    } else {
                        format!("No matches for '{}'", search_query)
                    })
                    .size(zoom_font(9.0, z))
                    .color(theme::TEXT_MUTED),
                );
            } else {
                for (row_idx, (id, data)) in filtered.iter().take(6).enumerate() {
                    let row_bg = if row_idx % 2 == 1 {
                        Color32::from_white_alpha(4)
                    } else {
                        Color32::TRANSPARENT
                    };

                    egui::Frame::default().fill(row_bg).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for (c_idx, col) in col_names.iter().enumerate() {
                                let cell_val = if c_idx == 0 && col.eq_ignore_ascii_case("id") {
                                    format!("#{}", &id[..6.min(id.len())])
                                } else {
                                    let key = col.to_lowercase().replace(' ', "_");
                                    data.get(&key)
                                        .or_else(|| data.get(col))
                                        .and_then(|v| {
                                            if let Some(s) = v.as_str() {
                                                Some(s.to_string())
                                            } else if let Some(n) = v.as_f64() {
                                                Some(format!("{:.0}", n))
                                            } else if let Some(b) = v.as_bool() {
                                                Some(if b { "Yes".into() } else { "No".into() })
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_else(|| "—".to_string())
                                };

                                ui.allocate_ui_with_layout(
                                    egui::vec2(col_w, 16.0 * z),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(cell_val)
                                                .size(zoom_font(8.8, z))
                                                .color(theme::TEXT),
                                        );
                                    },
                                );
                            }

                            // Delete Action in Play mode
                            if play_mode {
                                ui.allocate_ui_with_layout(
                                    egui::vec2(col_w * 0.6, 16.0 * z),
                                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                                    |ui| {
                                        if ui.small_button("🗑").on_hover_text("Delete record from SQLite").clicked() {
                                            events.push(CanvasEvent::DeleteRecord {
                                                entity: ent.to_string(),
                                                id: id.clone(),
                                            });
                                        }
                                    },
                                );
                            }
                        });
                    });
                }
            }
        } else {
            // Live preview fallback
            for r_idx in 1..=3 {
                ui.horizontal(|ui| {
                    for (c_idx, _) in col_names.iter().enumerate() {
                        let cell_val = match c_idx {
                            0 => format!("#{:03}", r_idx),
                            1 => format!("Live Record {}", r_idx),
                            2 => format!("data{}@acme.com", r_idx),
                            _ => "Active".into(),
                        };
                        ui.allocate_ui_with_layout(
                            egui::vec2(col_w, 16.0 * z),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.label(RichText::new(cell_val).size(zoom_font(8.8, z)).color(theme::TEXT));
                            },
                        );
                    }
                });
            }
        }
    });
}

/// Renders a schema-driven DynamicForm widget that captures inputs and saves directly to SQLite.
#[allow(clippy::too_many_arguments)]
pub fn render_dynamic_form(
    ui: &mut egui::Ui,
    node_id: &str,
    node: &Node,
    bound_entity: &str,
    title: &str,
    submit_label: &str,
    z: f32,
    play_mode: bool,
    form_state: &mut std::collections::HashMap<String, String>,
    events: &mut Vec<CanvasEvent>,
) {
    let frame = widget_frame(&node.styling, &node.style, z);
    frame.show(ui, |ui| {
        ui.set_min_size(ui.available_size());

        // Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("📋 {}", title))
                    .size(zoom_font(11.5, z))
                    .color(Color32::WHITE)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("SQLite: {}", bound_entity))
                        .size(zoom_font(8.5, z))
                        .color(theme::ACCENT),
                );
            });
        });

        ui.add_space(2.0 * z);
        ui.separator();
        ui.add_space(2.0 * z);

        // Schema-defined fields
        let fields: Vec<(&'static str, &'static str, &'static str, bool, Vec<&'static str>)> = match bound_entity {
            "deals" => vec![
                ("title", "Deal Title", "e.g. Acme Enterprise License", true, vec![]),
                ("value", "Deal Value ($)", "25000", true, vec![]),
                ("stage", "Pipeline Stage", "", true, vec!["Lead", "Proposal", "Negotiation", "Won", "Lost"]),
                ("expected_close", "Expected Close", "2026-12-31", false, vec![]),
            ],
            "tickets" => vec![
                ("customer_name", "Customer Name", "e.g. John Doe", true, vec![]),
                ("customer_phone", "Phone", "+30 210 1234567", true, vec![]),
                ("device_model", "Device Model", "POS Cash Terminal", false, vec![]),
                ("issue_summary", "Issue Summary", "Receipt printer jam", true, vec![]),
                ("status", "Status", "", false, vec!["Open", "In Progress", "Resolved"]),
            ],
            _ => vec![
                ("name", "Full Name", "e.g. Maria Smith", true, vec![]),
                ("email", "Email Address", "maria@example.com", true, vec![]),
                ("phone", "Phone Number", "+1 555-0100", false, vec![]),
                ("company", "Company", "Acme Corporation", false, vec![]),
                ("status", "Status", "", false, vec!["Lead", "Contacted", "Qualified", "Customer"]),
            ],
        };

        for (field_key, label, placeholder, is_req, options) in fields {
            let state_key = format!("{}:{}", node_id, field_key);

            ui.horizontal(|ui| {
                let req_marker = if is_req { " *" } else { "" };
                ui.allocate_ui_with_layout(
                    egui::vec2(80.0 * z, 18.0 * z),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.label(
                            RichText::new(format!("{}{}", label, req_marker))
                                .size(zoom_font(8.8, z))
                                .color(if is_req { theme::TEXT } else { theme::TEXT_DIM }),
                        );
                    },
                );

                if !options.is_empty() {
                    // Dropdown
                    if play_mode {
                        let cur_val = form_state.entry(state_key.clone()).or_insert_with(|| options[0].to_string());
                        let display_val = if cur_val.is_empty() { options[0].to_string() } else { cur_val.clone() };
                        egui::ComboBox::from_id_salt(&state_key)
                            .selected_text(RichText::new(&display_val).size(zoom_font(9.0, z)))
                            .width(ui.available_width() - 4.0 * z)
                            .show_ui(ui, |ui| {
                                for opt in options {
                                    let is_sel = *cur_val == opt;
                                    if ui.selectable_label(is_sel, opt).clicked() {
                                        *cur_val = opt.to_string();
                                    }
                                }
                            });
                    } else {
                        ui.label(
                            RichText::new(format!("{} ▾", options[0]))
                                .size(zoom_font(9.0, z))
                                .color(theme::TEXT_MUTED),
                        );
                    }
                } else {
                    // Text / Number Input
                    if play_mode {
                        let val = form_state.entry(state_key).or_default();
                        ui.add(
                            egui::TextEdit::singleline(val)
                                .hint_text(placeholder)
                                .desired_width(ui.available_width() - 4.0 * z),
                        );
                    } else {
                        ui.add_enabled_ui(false, |ui| {
                            let mut dummy = String::new();
                            ui.add(
                                egui::TextEdit::singleline(&mut dummy)
                                    .hint_text(placeholder)
                                    .desired_width(ui.available_width() - 4.0 * z),
                            );
                        });
                    }
                }
            });
            ui.add_space(1.5 * z);
        }

        ui.add_space(4.0 * z);

        // Submit Button
        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
            let btn_resp = ui.add_sized(
                egui::vec2(ui.available_width(), 26.0 * z),
                egui::Button::new(
                    RichText::new(format!("💾 {}", submit_label))
                        .size(zoom_font(10.5, z))
                        .color(Color32::WHITE)
                        .strong(),
                )
                .fill(Color32::from_rgb(40, 110, 220))
                .corner_radius(4.0 * z)
                .sense(if play_mode { Sense::click() } else { Sense::hover() }),
            );

            if play_mode && btn_resp.clicked() {
                events.push(CanvasEvent::DynamicFormSubmit {
                    node_id: node_id.to_string(),
                    entity: bound_entity.to_string(),
                });
            }
        });
    });
}
