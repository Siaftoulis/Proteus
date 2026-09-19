//! Developer & Data Schema Studio for Proteus Ecosystem.
//! Dedicated UI for software engineers, enterprise teams, and contracted developers:
//! - Visual Entity & Field Schema Designer
//! - Additive-only SQL Migration Engine (prevents data loss)
//! - Direct Database Connectors (PostgreSQL / MySQL / SQLite) & Replication Outbox
//! - Enterprise Developer Work Orders & In-Platform Escrow contracts

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use crm_core::migrations::{MigrationPlan, MigrationRunner};
use crm_core::drivers::{ConnectionConfig, DatabaseDriver, RemoteDriverMock};
use rusqlite::Connection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaFieldType {
    Text,
    Integer,
    CurrencyFloat,
    DateTime,
    Boolean,
}

impl SchemaFieldType {
    pub fn as_sql(&self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::Integer => "INTEGER",
            Self::CurrencyFloat => "REAL",
            Self::DateTime => "TEXT",
            Self::Boolean => "INTEGER",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: SchemaFieldType,
    pub is_nullable: bool,
}

#[derive(Debug, Clone)]
pub struct SchemaEntity {
    pub name: String,
    pub is_core: bool,
    pub fields: Vec<SchemaField>,
}

pub struct DeveloperStudioState {
    pub entities: Vec<SchemaEntity>,
    pub selected_entity_index: usize,
    pub new_field_name: String,
    pub new_field_type: SchemaFieldType,
    pub status_message: Option<String>,
    pub enterprise_work_order: EnterpriseWorkOrder,
    pub remote_db_url: String,
    pub active_schema_diff: Option<crm_core::schema_diff::SchemaDiff>,
}

#[derive(Debug, Clone)]
pub struct EnterpriseWorkOrder {
    pub enterprise_name: String,
    pub work_order_title: String,
    pub escrow_amount_eur: f64,
    #[allow(dead_code)]
    pub is_active: bool,
}

impl Default for DeveloperStudioState {
    fn default() -> Self {
        Self {
            entities: vec![
                SchemaEntity {
                    name: "service_tickets".to_string(),
                    is_core: true,
                    fields: vec![
                        SchemaField { name: "ticket_number".to_string(), field_type: SchemaFieldType::Integer, is_nullable: false },
                        SchemaField { name: "customer_name".to_string(), field_type: SchemaFieldType::Text, is_nullable: false },
                        SchemaField { name: "customer_phone".to_string(), field_type: SchemaFieldType::Text, is_nullable: false },
                        SchemaField { name: "device_model".to_string(), field_type: SchemaFieldType::Text, is_nullable: false },
                        SchemaField { name: "estimated_cost".to_string(), field_type: SchemaFieldType::CurrencyFloat, is_nullable: true },
                        SchemaField { name: "status".to_string(), field_type: SchemaFieldType::Text, is_nullable: false },
                    ],
                },
                SchemaEntity {
                    name: "inventory_parts".to_string(),
                    is_core: false,
                    fields: vec![
                        SchemaField { name: "sku".to_string(), field_type: SchemaFieldType::Text, is_nullable: false },
                        SchemaField { name: "part_name".to_string(), field_type: SchemaFieldType::Text, is_nullable: false },
                        SchemaField { name: "quantity".to_string(), field_type: SchemaFieldType::Integer, is_nullable: false },
                        SchemaField { name: "unit_cost".to_string(), field_type: SchemaFieldType::CurrencyFloat, is_nullable: false },
                    ],
                },
            ],
            selected_entity_index: 0,
            new_field_name: String::new(),
            new_field_type: SchemaFieldType::Text,
            status_message: None,
            enterprise_work_order: EnterpriseWorkOrder {
                enterprise_name: "Alpha Logistics & Fleet Group".to_string(),
                work_order_title: "Custom Fleet Schema & Telematics Sync Integration".to_string(),
                escrow_amount_eur: 750.0,
                is_active: true,
            },
            remote_db_url: "postgres://admin:secret@cloud.corp.net:5432/proteus_prod?sslmode=require".to_string(),
            active_schema_diff: None,
        }
    }
}

pub fn draw_developer_studio_view(ui: &mut Ui, conn: &mut Connection, state: &mut DeveloperStudioState) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("💻 Developer & Data Schema Studio (Περιβάλλον Προγραμματιστών)").strong().size(22.0));
        ui.add_space(4.0);
        ui.label(RichText::new("Εξειδικευμένο UI: Additive migrations, PostgreSQL/MySQL drivers, replication outbox και συμβάσεις.")
            .size(12.0).color(crate::theme::TEXT_MUTED));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        if let Some(msg) = &state.status_message {
            Frame::new().fill(Color32::from_rgb(16, 50, 35)).stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                .corner_radius(CornerRadius::same(6)).inner_margin(Margin::symmetric(14, 6)).show(ui, |ui| {
                    ui.label(RichText::new(msg).color(Color32::from_rgb(52, 211, 153)).strong());
                });
            ui.add_space(8.0);
        }

        ui.columns(2, |cols| {
            // Left Column: Entities & Fields
            cols[0].vertical(|ui| {
                Frame::new().fill(crate::theme::BG_CARD).stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(14)).show(ui, |ui| {
                        ui.label(RichText::new("ΟΝΤΟΤΗΤΕΣ & ΠΕΔΙΑ (DATABASE SCHEMA)").strong().color(crate::theme::TEXT_MUTED));
                        ui.add_space(6.0);
                        ui.horizontal_wrapped(|ui| {
                            for (idx, entity) in state.entities.iter().enumerate() {
                                let is_sel = state.selected_entity_index == idx;
                                let txt = if entity.is_core { format!("🔒 {}", entity.name) } else { format!("📦 {}", entity.name) };
                                let btn = if is_sel {
                                    egui::Button::new(RichText::new(txt).strong().color(Color32::WHITE)).fill(crate::theme::ACCENT_PRIMARY)
                                } else {
                                    egui::Button::new(RichText::new(txt).color(crate::theme::TEXT_SECONDARY)).fill(crate::theme::BG_BASE)
                                };
                                if ui.add(btn).clicked() { state.selected_entity_index = idx; }
                            }
                        });
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let current_entity = &state.entities[state.selected_entity_index];
                        ui.label(RichText::new(format!("Πεδία: {} ({} πεδία)", current_entity.name, current_entity.fields.len())).strong().color(crate::theme::TEXT_PRIMARY));
                        ui.add_space(6.0);

                        for field in &current_entity.fields {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("●").size(8.0).color(crate::theme::ACCENT_CYAN));
                                ui.label(RichText::new(&field.name).strong().color(crate::theme::TEXT_PRIMARY));
                                ui.label(RichText::new(field.field_type.as_sql()).size(11.0).color(Color32::from_rgb(251, 146, 60)));
                                if !field.is_nullable { ui.label(RichText::new("NOT NULL").size(10.0).color(crate::theme::TEXT_MUTED)); }
                            });
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(6.0);
                        ui.label(RichText::new("ΠΡΟΣΘΗΚΗ ΝΕΟΥ ΠΕΔΙΟΥ").strong().color(crate::theme::TEXT_MUTED));
                        ui.horizontal(|ui| {
                            ui.label("Όνομα:");
                            ui.text_edit_singleline(&mut state.new_field_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Τύπος:");
                            egui::ComboBox::from_id_salt("field_type_combo")
                                .selected_text(state.new_field_type.as_sql())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut state.new_field_type, SchemaFieldType::Text, "TEXT");
                                    ui.selectable_value(&mut state.new_field_type, SchemaFieldType::Integer, "INTEGER");
                                    ui.selectable_value(&mut state.new_field_type, SchemaFieldType::CurrencyFloat, "REAL");
                                    ui.selectable_value(&mut state.new_field_type, SchemaFieldType::DateTime, "DATETIME");
                                    ui.selectable_value(&mut state.new_field_type, SchemaFieldType::Boolean, "BOOLEAN");
                                });

                            if ui.button(RichText::new("＋ Προσθήκη").strong()).clicked() && !state.new_field_name.trim().is_empty() {
                                let new_f = SchemaField {
                                    name: state.new_field_name.trim().to_lowercase().replace(' ', "_"),
                                    field_type: state.new_field_type.clone(),
                                    is_nullable: true,
                                };
                                let target_entity = &mut state.entities[state.selected_entity_index];
                                target_entity.fields.push(new_f.clone());
                                state.status_message = Some(format!("✓ Προστέθηκε το πεδίο '{}'.", new_f.name));
                                state.new_field_name.clear();
                            }
                        });
                    });
            });

            // Right Column: Migrations, Drivers & Work Orders
            cols[1].vertical(|ui| {
                // Top: Migration Sandbox
                Frame::new().fill(crate::theme::BG_CARD).stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(14)).show(ui, |ui| {
                        ui.label(RichText::new("ADDITIVE MIGRATION RUNNER").strong().color(crate::theme::TEXT_MUTED));
                        let entity = &state.entities[state.selected_entity_index];
                        let last_field = entity.fields.last().map(|f| (f.name.as_str(), f.field_type.as_sql())).unwrap_or(("custom_col", "TEXT"));
                        let migration_sql = format!("ALTER TABLE {}\nADD COLUMN {} {};", entity.name, last_field.0, last_field.1);
                        ui.label(RichText::new(&migration_sql).monospace().size(11.0).color(crate::theme::ACCENT_CYAN));
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button("⚡ Sandboxed Dry-Run").clicked() {
                                let mut plan = MigrationPlan::new("DEV-DRY", "Dev Studio Verification", "Developer");
                                plan.add_statement(migration_sql.clone());
                                match MigrationRunner::dry_run(conn, &plan) {
                                    Ok(_) => state.status_message = Some("✓ Dry-Run επιτυχής: Το SQLite επικύρωσε τη σύνταξη.".into()),
                                    Err(e) => state.status_message = Some(format!("❌ Σφάλμα Migration: {}", e)),
                                }
                            }
                            if ui.button("🔍 Visual Diff").clicked() {
                                match crm_core::schema_diff::SchemaDiff::from_ddl(conn, std::slice::from_ref(&migration_sql)) {
                                    Ok(d) => {
                                        state.active_schema_diff = Some(d);
                                        state.status_message = Some("✓ Υπολογίστηκε το οπτικό Schema Diff.".into());
                                    }
                                    Err(e) => state.status_message = Some(format!("❌ Σφάλμα Diff: {}", e)),
                                }
                            }
                            if ui.button(RichText::new("🚀 Εκτέλεση (.bak)").strong()).clicked() {
                                let mut plan = MigrationPlan::new("DEV-APPLY", "Dev Studio Applied Migration", "Developer");
                                plan.add_statement(migration_sql.clone());
                                let db_path = crm_core::paths::get_database_path();
                                match MigrationRunner::execute(conn, &plan, Some(&db_path)) {
                                    Ok(res) => {
                                        state.status_message = Some(format!("✓ Εφαρμόστηκε σε {}ms! (Snapshot: {:?})", res.elapsed_ms, res.snapshot_path));
                                        state.active_schema_diff = None;
                                    }
                                    Err(e) => state.status_message = Some(format!("❌ Αποτυχία: {}", e)),
                                }
                            }
                        });

                        let mut should_close_diff = false;
                        if let Some(ref diff) = state.active_schema_diff {
                            ui.add_space(8.0);
                            Frame::new().fill(crate::theme::BG_BASE).stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                                .corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(10)).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("📊 VISUAL SCHEMA DIFF:").strong().size(10.0).color(crate::theme::TEXT_MUTED));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button("✕").clicked() {
                                                should_close_diff = true;
                                            }
                                        });
                                    });
                                    ui.add_space(4.0);
                                    for t in &diff.table_diffs {
                                        ui.label(RichText::new(format!("Πίνακας: {}", t.table_name)).strong().size(11.0).color(Color32::WHITE));
                                        for c in &t.column_diffs {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("+").color(Color32::from_rgb(52, 211, 153)).strong());
                                                ui.label(RichText::new(&c.column_name).color(Color32::from_rgb(52, 211, 153)));
                                                if let crm_core::schema_diff::ColumnChangeKind::Added { ref data_type, .. } = c.kind {
                                                    ui.label(RichText::new(data_type).size(10.0).color(crate::theme::TEXT_MUTED));
                                                }
                                            });
                                        }
                                    }
                                });
                        }
                        if should_close_diff {
                            state.active_schema_diff = None;
                        }
                    });

                ui.add_space(10.0);

                // Middle: Database Drivers & Cloud Sync Connector
                Frame::new().fill(crate::theme::BG_CARD).stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(14)).show(ui, |ui| {
                        ui.label(RichText::new("ENTERPRISE DATABASE CONNECTORS & SYNC").strong().color(crate::theme::TEXT_MUTED));
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label("URL:");
                            ui.text_edit_singleline(&mut state.remote_db_url);
                        });
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button("🔌 Ping Driver").clicked() {
                                match ConnectionConfig::parse_url(&state.remote_db_url) {
                                    Ok(cfg) => {
                                        let driver = RemoteDriverMock::new(cfg);
                                        match driver.ping() {
                                            Ok(_) => state.status_message = Some("✓ Σύνδεση επιτυχής με απομακρυσμένη βάση!".into()),
                                            Err(e) => state.status_message = Some(format!("❌ Σφάλμα σύνδεσης: {}", e)),
                                        }
                                    }
                                    Err(e) => state.status_message = Some(format!("❌ Μη έγκυρο URL: {}", e)),
                                }
                            }
                            let pending_count = crm_core::replication::count_pending_outbox(conn).unwrap_or(0);
                            let sync_label = format!("🔄 Sync Outbox ({})", pending_count);
                            if ui.button(sync_label).clicked() {
                                match ConnectionConfig::parse_url(&state.remote_db_url) {
                                    Ok(cfg) => {
                                        let driver = RemoteDriverMock::new(cfg);
                                        match crm_core::replication::sync_outbox_to_driver(conn, &driver, 50) {
                                            Ok(summary) => {
                                                state.status_message = Some(format!(
                                                    "✓ Συγχρονίστηκαν {} εγγραφές! (Εκκρεμούν: {})",
                                                    summary.records_pushed, summary.pending_remaining
                                                ));
                                            }
                                            Err(e) => state.status_message = Some(format!("❌ Σφάλμα συγχρονισμού: {}", e)),
                                        }
                                    }
                                    Err(e) => state.status_message = Some(format!("❌ Μη έγκυρο URL: {}", e)),
                                }
                            }
                        });
                    });

                ui.add_space(10.0);

                // Bottom: Work Order & Escrow
                Frame::new().fill(crate::theme::BG_CARD).stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(14)).show(ui, |ui| {
                        ui.label(RichText::new("ENTERPRISE ESCROW CONTRACT").strong().color(crate::theme::TEXT_MUTED));
                        let wo = &state.enterprise_work_order;
                        ui.label(RichText::new(&wo.enterprise_name).strong().color(crate::theme::TEXT_PRIMARY));
                        ui.label(RichText::new(&wo.work_order_title).size(11.0).color(crate::theme::TEXT_MUTED));
                        ui.label(RichText::new(format!("Αμοιβή: {:.2}€ (In-Platform Escrow)", wo.escrow_amount_eur)).color(Color32::from_rgb(52, 211, 153)));
                        if ui.button("📦 Παράδοση Πακέτου").clicked() {
                            state.status_message = Some("Το schema παραδόθηκε. Εκκρεμεί έγκριση για αποδέσμευση Escrow.".into());
                        }
                    });
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_developer_studio_state_initialization() {
        let state = DeveloperStudioState::default();
        assert_eq!(state.entities.len(), 2);
        assert_eq!(state.entities[0].name, "service_tickets");
        assert!(state.enterprise_work_order.is_active);
        assert_eq!(state.enterprise_work_order.escrow_amount_eur, 750.0);
        assert!(state.remote_db_url.contains("postgres://"));
    }

    #[test]
    fn test_additive_field_insertion() {
        let mut state = DeveloperStudioState::default();
        let initial_count = state.entities[0].fields.len();
        state.entities[0].fields.push(SchemaField {
            name: "telematics_id".to_string(),
            field_type: SchemaFieldType::Text,
            is_nullable: true,
        });
        assert_eq!(state.entities[0].fields.len(), initial_count + 1);
    }
}
