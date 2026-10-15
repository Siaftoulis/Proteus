//! Business & Data Analyst Studio for Proteus Workbench (proteus-design-studio).
//! Provides interactive client discovery, bespoke Markdown spec generation,
//! and 1-click canvas mockup scaffolding for rapid prototyping without an external designer.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use proteus_core::inference::{InferredTable, SchemaInferer};
use crate::theme;
use crate::views::analyst_spec::{self, AnalystSubTab, BusinessSector, DiscoveryInterview, UiStylePreference};
use crate::app_state::ProteusApp;
use crate::models::Mode;

pub struct AnalystState {
    pub subtab: AnalystSubTab,
    pub interview: DiscoveryInterview,
    pub generated_markdown_spec: String,
    pub table_name: String,
    pub raw_input: String,
    pub inferred_table: Option<InferredTable>,
    pub generated_ddl: String,
    pub export_summary: Option<String>,
    pub status_msg: Option<String>,
}

impl Default for AnalystState {
    fn default() -> Self {
        let sample_raw = r#"[
  {"part_sku": "SKU-9921", "part_name": "High-Pressure Hydraulic Valve", "stock_qty": 45, "unit_cost": 128.50, "is_critical": true},
  {"part_sku": "SKU-4410", "part_name": "Ceramic Brake Rotor Assembly", "stock_qty": 12, "unit_cost": 310.00, "is_critical": false},
  {"part_sku": "SKU-7732", "part_name": "CAN-Bus Telemetry Interface", "stock_qty": 88, "unit_cost": 64.25, "is_critical": true}
]"#;
        let interview = DiscoveryInterview::default();
        let md = analyst_spec::compile_markdown_specification(&interview);
        Self {
            subtab: AnalystSubTab::ClientDiscovery,
            interview,
            generated_markdown_spec: md,
            table_name: "parts_inventory".to_string(),
            raw_input: sample_raw.to_string(),
            inferred_table: None,
            generated_ddl: String::new(),
            export_summary: None,
            status_msg: None,
        }
    }
}

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(4.);
    ui.label(RichText::new("BUSINESS ANALYSIS").size(11.).strong().color(theme::TEXT));
    ui.add_space(6.);

    // Subtab pills
    let current_subtab = app.analyst_state.subtab;
    ui.horizontal(|ui| {
        if ui.selectable_label(current_subtab == AnalystSubTab::ClientDiscovery, "📋 Ανάλυση").clicked() {
            app.analyst_state.subtab = AnalystSubTab::ClientDiscovery;
        }
        if ui.selectable_label(current_subtab == AnalystSubTab::MarkdownSpec, "📑 Spec .md").clicked() {
            app.analyst_state.generated_markdown_spec = analyst_spec::compile_markdown_specification(&app.analyst_state.interview);
            app.analyst_state.subtab = AnalystSubTab::MarkdownSpec;
        }
        if ui.selectable_label(current_subtab == AnalystSubTab::SchemaInferer, "⚡ Σχήμα").clicked() {
            app.analyst_state.subtab = AnalystSubTab::SchemaInferer;
        }
    });

    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    match app.analyst_state.subtab {
        AnalystSubTab::ClientDiscovery | AnalystSubTab::MarkdownSpec => {
            ui.label(RichText::new("Πελάτης:").size(10.).color(theme::TEXT_DIM));
            ui.add(egui::TextEdit::singleline(&mut app.analyst_state.interview.client_business_name).hint_text("Επωνυμία επιχείρησης"));
            ui.add_space(4.);

            ui.label(RichText::new("Α.Φ.Μ.:").size(10.).color(theme::TEXT_DIM));
            ui.add(egui::TextEdit::singleline(&mut app.analyst_state.interview.client_tax_id).hint_text("ΑΦΜ"));
            ui.add_space(4.);

            ui.label(RichText::new("Υπεύθυνος / Email:").size(10.).color(theme::TEXT_DIM));
            ui.add(egui::TextEdit::singleline(&mut app.analyst_state.interview.client_contact_person).hint_text("Ονοματεπώνυμο"));
            ui.add(egui::TextEdit::singleline(&mut app.analyst_state.interview.client_contact_email).hint_text("Email"));
            ui.add_space(8.);

            let compile_btn = ui.add(
                egui::Button::new(RichText::new("📑 Παραγωγή Spec .md").size(11.).strong().color(Color32::WHITE))
                    .fill(theme::ACCENT)
                    .corner_radius(CornerRadius::same(4))
                    .min_size(Vec2::new(ui.available_width(), 26.)),
            );
            if compile_btn.clicked() {
                app.analyst_state.generated_markdown_spec = analyst_spec::compile_markdown_specification(&app.analyst_state.interview);
                app.analyst_state.subtab = AnalystSubTab::MarkdownSpec;
                app.toast("✓ Παράχθηκε το Markdown Spec");
            }

            ui.add_space(6.);
            let scaffold_btn = ui.add(
                egui::Button::new(RichText::new("⚡ Mockup Στο Canvas").size(11.).strong().color(Color32::WHITE))
                    .fill(Color32::from_rgb(46, 125, 50))
                    .corner_radius(CornerRadius::same(4))
                    .min_size(Vec2::new(ui.available_width(), 26.)),
            );
            if scaffold_btn.clicked() {
                let brief = app.analyst_state.interview.to_client_project_brief();
                app.scaffold_from_project_brief(&brief);
                app.mode = Mode::Designer;
                app.toast("✓ Δημιουργήθηκαν Mockup Blocks στο Canvas από την Ανάλυση!");
            }
        }
        AnalystSubTab::SchemaInferer => {
            ui.label(RichText::new("Target Table:").size(10.).color(theme::TEXT_DIM));
            ui.add(egui::TextEdit::singleline(&mut app.analyst_state.table_name).hint_text("table_name"));
            ui.add_space(6.);

            ui.label(RichText::new("Raw JSON / Tabular Data:").size(10.).color(theme::TEXT_DIM));
            ui.add(
                egui::TextEdit::multiline(&mut app.analyst_state.raw_input)
                    .desired_rows(8)
                    .font(egui::TextStyle::Monospace)
                    .hint_text("Paste JSON array here..."),
            );
            ui.add_space(6.);

            let infer_btn = ui.add(
                egui::Button::new(RichText::new("⚡ Infer Relational Schema").size(11.).strong().color(Color32::WHITE))
                    .fill(theme::ACCENT)
                    .corner_radius(CornerRadius::same(4))
                    .min_size(Vec2::new(ui.available_width(), 26.)),
            );
            if infer_btn.clicked() {
                match serde_json::from_str::<serde_json::Value>(&app.analyst_state.raw_input) {
                    Ok(val) => {
                        let tname = if app.analyst_state.table_name.trim().is_empty() { "inferred_dataset" } else { app.analyst_state.table_name.trim() };
                        match SchemaInferer::infer_from_json(tname, &val) {
                            Ok(inferred) => {
                                app.analyst_state.generated_ddl = inferred.to_sqlite_ddl();
                                app.analyst_state.inferred_table = Some(inferred);
                                app.analyst_state.status_msg = Some("Inferred relational schema successfully".to_string());
                            }
                            Err(e) => app.analyst_state.status_msg = Some(format!("Inference error: {}", e)),
                        }
                    }
                    Err(e) => app.analyst_state.status_msg = Some(format!("Invalid JSON: {}", e)),
                }
            }
        }
    }

    if let Some(ref msg) = app.analyst_state.status_msg {
        ui.add_space(8.);
        ui.label(RichText::new(msg).size(9.5).color(theme::ACCENT));
    }
}

pub fn show_central(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(8.);
    ui.horizontal(|ui| {
        ui.label(RichText::new("📋 Business Analysis & Client Discovery Suite").size(15.).strong().color(theme::TEXT));
        ui.add_space(12.);
        ui.label(RichText::new("Επικοινωνία με Πελάτη, Προδιαγραφή Markdown & Αυτόματο Mockup Canvas").size(11.).color(theme::TEXT_DIM));
    });
    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    match app.analyst_state.subtab {
        AnalystSubTab::ClientDiscovery => draw_discovery_form(app, ui),
        AnalystSubTab::MarkdownSpec => draw_markdown_spec_view(app, ui),
        AnalystSubTab::SchemaInferer => draw_schema_inferer_view(app, ui),
    }
}

fn draw_discovery_form(app: &mut ProteusApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Card 1: Business Identity & Sector
        Frame::new().fill(theme::PANEL).stroke(Stroke::new(1., theme::BORDER)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(12)).show(ui, |ui| {
            ui.label(RichText::new("1. ΣΤΟΙΧΕΙΑ ΠΕΛΑΤΗ & ΚΛΑΔΟΣ ΕΠΙΧΕΙΡΗΣΗΣ").size(11.).strong().color(theme::TEXT));
            ui.add_space(8.);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Κλάδος:").strong().color(theme::TEXT_DIM));
                let current_sector = app.analyst_state.interview.client_industry;
                for sector in &[
                    BusinessSector::Retail,
                    BusinessSector::FieldService,
                    BusinessSector::Hospitality,
                    BusinessSector::Wholesale,
                    BusinessSector::ECommerce,
                ] {
                    if ui.selectable_label(current_sector == *sector, sector.label_el()).clicked() {
                        app.analyst_state.interview.client_industry = *sector;
                    }
                }
            });
            ui.add_space(8.);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Στυλ UI:").strong().color(theme::TEXT_DIM));
                let current_style = app.analyst_state.interview.ui_style;
                for style in &[
                    UiStylePreference::MinimalistClean,
                    UiStylePreference::TouchTabletOptimized,
                    UiStylePreference::HighContrastDark,
                ] {
                    if ui.selectable_label(current_style == *style, style.label_el()).clicked() {
                        app.analyst_state.interview.ui_style = *style;
                    }
                }
            });
        });

        ui.add_space(10.);

        // Card 2: Required Proteus Modules
        Frame::new().fill(theme::PANEL).stroke(Stroke::new(1., theme::BORDER)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(12)).show(ui, |ui| {
            ui.label(RichText::new("2. ΑΠΑΙΤΟΥΜΕΝΑ ΑΥΤΟΝΟΜΑ ΥΠΟΣΥΣΤΗΜΑΤΑ (PROTEUS MODULES)").size(11.).strong().color(theme::TEXT));
            ui.add_space(8.);
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.analyst_state.interview.module_crm, "👥 CRM & Επαφές");
                ui.checkbox(&mut app.analyst_state.interview.module_pos, "🧾 POS Ταμείο & myDATA");
                ui.checkbox(&mut app.analyst_state.interview.module_field_service, "🔧 Field Service & Τεχνικοί");
            });
            ui.add_space(6.);
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.analyst_state.interview.module_wms, "📦 WMS & Αποθήκη");
                ui.checkbox(&mut app.analyst_state.interview.module_cardex, "💳 Cardex & Υπόλοιπα");
                ui.checkbox(&mut app.analyst_state.interview.module_webstore, "🌐 WebStore / E-Shop");
            });
        });

        ui.add_space(10.);

        // Card 3: Interview Notes & Desired Look/Feel
        Frame::new().fill(theme::PANEL).stroke(Stroke::new(1., theme::BORDER)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(12)).show(ui, |ui| {
            ui.label(RichText::new("3. ΣΗΜΕΙΩΣΕΙΣ ΣΥΝΕΝΤΕΥΞΗΣ ΠΕΛΑΤΗ & ΕΠΙΘΥΜΗΤΗ ΛΕΙΤΟΥΡΓΙΑ").size(11.).strong().color(theme::TEXT));
            ui.add_space(4.);
            ui.label(RichText::new("Καταγράψτε πώς θέλει να φαίνεται ο πελάτης, πώς θέλει να το χρησιμοποιεί και τι καθημερινές ανάγκες έχει:").size(10.).color(theme::TEXT_DIM));
            ui.add_space(6.);
            ui.add(
                egui::TextEdit::multiline(&mut app.analyst_state.interview.interview_notes)
                    .desired_rows(6)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Body)
                    .hint_text("Π.χ. Ο πελάτης θέλει ένα απλό CRM με γρήγορη καταχώρηση πελατών, έκδοση αποδείξεων..."),
            );
        });

        ui.add_space(12.);

        // Action CTAs
        ui.horizontal(|ui| {
            let gen_md = ui.add(
                egui::Button::new(RichText::new("📑 Παραγωγή Προδιαγραφής Markdown").size(12.).strong().color(Color32::WHITE))
                    .fill(theme::ACCENT)
                    .corner_radius(CornerRadius::same(4))
                    .min_size(Vec2::new(240., 32.)),
            );
            if gen_md.clicked() {
                app.analyst_state.generated_markdown_spec = analyst_spec::compile_markdown_specification(&app.analyst_state.interview);
                app.analyst_state.subtab = AnalystSubTab::MarkdownSpec;
                app.toast("✓ Παράχθηκε το Markdown Spec");
            }

            let scaffold = ui.add(
                egui::Button::new(RichText::new("⚡ Αυτόματη Δημιουργία Canvas Mockup Blocks").size(12.).strong().color(Color32::WHITE))
                    .fill(Color32::from_rgb(46, 125, 50))
                    .corner_radius(CornerRadius::same(4))
                    .min_size(Vec2::new(300., 32.)),
            );
            if scaffold.clicked() {
                let brief = app.analyst_state.interview.to_client_project_brief();
                app.scaffold_from_project_brief(&brief);
                app.mode = Mode::Designer;
                app.toast("✓ Δημιουργήθηκαν Mockup Blocks στο Canvas από την Ανάλυση!");
            }
        });
    });
}

fn draw_markdown_spec_view(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button(RichText::new("← Επιστροφή στην Ανάλυση").size(11.)).clicked() {
            app.analyst_state.subtab = AnalystSubTab::ClientDiscovery;
        }
        ui.add_space(10.);
        if ui.button(RichText::new("📋 Αντιγραφή στο Clipboard").size(11.).strong().color(Color32::WHITE)).clicked() {
            ui.ctx().copy_text(app.analyst_state.generated_markdown_spec.clone());
            app.toast("✓ Αντιγράφηκε στο πρόχειρο!");
        }
        ui.add_space(10.);
        let scaffold = ui.add(
            egui::Button::new(RichText::new("⚡ Mockup Στο Canvas").size(11.).strong().color(Color32::WHITE))
                .fill(Color32::from_rgb(46, 125, 50)),
        );
        if scaffold.clicked() {
            let brief = app.analyst_state.interview.to_client_project_brief();
            app.scaffold_from_project_brief(&brief);
            app.mode = Mode::Designer;
            app.toast("✓ Δημιουργήθηκαν Mockup Blocks στο Canvas!");
        }
    });

    ui.add_space(10.);

    Frame::new().fill(theme::PANEL).stroke(Stroke::new(1., theme::BORDER)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(12)).show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut app.analyst_state.generated_markdown_spec)
                    .desired_rows(22)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
        });
    });
}

fn draw_schema_inferer_view(app: &mut ProteusApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(ref table) = app.analyst_state.inferred_table {
            Frame::new().fill(theme::PANEL).stroke(Stroke::new(1., theme::BORDER)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(12)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Table: {}", table.table_name)).size(13.).strong().color(theme::TEXT));
                    ui.label(RichText::new(format!("({} columns)", table.columns.len())).size(11.).color(theme::TEXT_DIM));
                });
                ui.add_space(8.);
                egui::Grid::new("inferred_cols_grid").striped(true).min_col_width(100.).show(ui, |ui| {
                    ui.label(RichText::new("Column Name").strong().color(theme::TEXT_DIM));
                    ui.label(RichText::new("Inferred Type").strong().color(theme::TEXT_DIM));
                    ui.label(RichText::new("SQLite Type").strong().color(theme::TEXT_DIM));
                    ui.label(RichText::new("Nullable").strong().color(theme::TEXT_DIM));
                    ui.label(RichText::new("Primary Key").strong().color(theme::TEXT_DIM));
                    ui.end_row();

                    for col in &table.columns {
                        ui.label(RichText::new(&col.name).strong().color(theme::TEXT));
                        ui.label(format!("{:?}", col.col_type));
                        ui.label(col.col_type.to_sqlite_type());
                        ui.label(if col.is_nullable { "YES" } else { "NO" });
                        ui.label(if col.is_primary_key { "🔑 PK" } else { "-" });
                        ui.end_row();
                    }
                });
            });

            ui.add_space(12.);

            Frame::new().fill(theme::PANEL).stroke(Stroke::new(1., theme::BORDER)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(12)).show(ui, |ui| {
                ui.label(RichText::new("GENERATED SQLITE DDL").size(11.).strong().color(theme::TEXT));
                ui.add_space(6.);
                let mut ddl_copy = app.analyst_state.generated_ddl.clone();
                ui.add(egui::TextEdit::multiline(&mut ddl_copy).desired_rows(4).font(egui::TextStyle::Monospace).desired_width(f32::INFINITY));
            });
        } else {
            Frame::new().fill(theme::PANEL).stroke(Stroke::new(1., theme::BORDER)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(24)).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No Data Ingested Yet").size(14.).strong().color(theme::TEXT));
                    ui.add_space(6.);
                    ui.label(RichText::new("Paste a raw JSON dataset in the left panel and click 'Infer Relational Schema' to extract relational tables.").size(11.).color(theme::TEXT_DIM));
                });
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyst_state_defaults() {
        let state = AnalystState::default();
        assert_eq!(state.subtab, AnalystSubTab::ClientDiscovery);
        assert!(!state.generated_markdown_spec.is_empty());
        assert!(state.generated_markdown_spec.contains("TechFix Solutions"));
    }
}
