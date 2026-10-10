//! Inspector UI renderer for visual Scratch-style Data Blocks.

use super::data_blocks::{DataBlockKind, DataPipelineQuery};
use crate::theme;
use eframe::egui::{self, RichText, Ui};

/// Renders the configuration UI inspector for an active DataBlockKind.
pub fn render_data_block_inspector(ui: &mut Ui, block: &mut DataBlockKind) {
    ui.label(RichText::new("DATA BLOCK CONFIGURATION").size(9.).color(theme::ACCENT).strong());
    ui.add_space(4.);

    match block {
        DataBlockKind::EntitySource { entity, limit } => {
            ui.label(RichText::new("Target Table / Entity:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(entity).hint_text("e.g. contacts, invoices").desired_width(f32::INFINITY));
            ui.add_space(4.);

            ui.label(RichText::new("Query Fetch Limit:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::DragValue::new(limit).range(1..=10000));
            ui.add_space(6.);

            let query = DataPipelineQuery::compile(
                &DataBlockKind::EntitySource {
                    entity: entity.clone(),
                    limit: *limit,
                },
                &[],
                &[],
                &[],
            );
            if let Ok(sql) = query {
                ui.label(RichText::new("Generated SQL:").size(9.).color(theme::TEXT_MUTED));
                ui.add_space(2.);
                ui.monospace(RichText::new(sql).size(9.).color(theme::ACCENT_GREEN));
            }
        }
        DataBlockKind::FieldExtractor { entity, field, .. } => {
            ui.label(RichText::new("Entity:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(entity).hint_text("e.g. contacts").desired_width(f32::INFINITY));
            ui.add_space(4.);

            ui.label(RichText::new("Field / Column:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(field).hint_text("e.g. email, phone").desired_width(f32::INFINITY));
        }
        DataBlockKind::IdLinker {
            from_entity,
            from_field,
            to_entity,
            to_field,
        } => {
            ui.label(RichText::new("From Entity & FK:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(from_entity).hint_text("orders").desired_width(75.));
                ui.label(".");
                ui.add(egui::TextEdit::singleline(from_field).hint_text("customer_id").desired_width(75.));
            });
            ui.add_space(4.);

            ui.label(RichText::new("To Entity & PK:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(to_entity).hint_text("customers").desired_width(75.));
                ui.label(".");
                ui.add(egui::TextEdit::singleline(to_field).hint_text("id").desired_width(75.));
            });
        }
        DataBlockKind::QueryFilter {
            field,
            operator,
            target_val,
        } => {
            ui.label(RichText::new("Filter Field:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(field).hint_text("e.g. status").desired_width(f32::INFINITY));
            ui.add_space(4.);

            ui.label(RichText::new("Operator (==, !=, >, <):").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(operator).hint_text("==").desired_width(f32::INFINITY));
            ui.add_space(4.);

            ui.label(RichText::new("Target Value:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(target_val).hint_text("active").desired_width(f32::INFINITY));
        }
        DataBlockKind::RecordMutator { entity, operation } => {
            ui.label(RichText::new("Target Database Entity:").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(entity).hint_text("e.g. invoices").desired_width(f32::INFINITY));
            ui.add_space(4.);

            ui.label(RichText::new("Mutation (INSERT, UPDATE, UPSERT):").size(10.).color(theme::TEXT));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(operation).hint_text("UPSERT").desired_width(f32::INFINITY));
        }
    }
}

/// Returns the standard set of flow node templates for the left palette.
pub fn flow_palette_kinds() -> Vec<(&'static str, super::FlowNodeKind)> {
    vec![
        ("+ Trigger Click", super::FlowNodeKind::TriggerClick { target_node_id: String::new() }),
        ("+ Navigate To", super::FlowNodeKind::NavigateTo { page_id: String::new() }),
        ("+ Save To DB", super::FlowNodeKind::SaveToDatabase { entity: String::new() }),
        ("+ Condition (IF)", super::FlowNodeKind::Condition {
            field: "status".to_string(),
            operator: "==".to_string(),
            target_value: "active".to_string(),
        }),
        ("+ Notification", super::FlowNodeKind::ShowToast { message: "Action executed".to_string() }),
        ("+ Federation Bridge", super::FlowNodeKind::FederationBridge {
            endpoint: "http://partner-db:7443".to_string(),
            source_entity: "products".to_string(),
            target_entity: "inventory".to_string(),
            bidirectional: true,
        }),
        ("+ 🗄 DB Source", super::FlowNodeKind::DataBlock(super::DataBlockKind::EntitySource {
            entity: "contacts".to_string(),
            limit: 50,
        })),
        ("+ 🔍 Field Extract", super::FlowNodeKind::DataBlock(super::DataBlockKind::FieldExtractor {
            entity: "contacts".to_string(),
            field: "name".to_string(),
            data_type: super::PortDataType::Text,
        })),
        ("+ 🔗 FK Linker", super::FlowNodeKind::DataBlock(super::DataBlockKind::IdLinker {
            from_entity: "orders".to_string(),
            from_field: "customer_id".to_string(),
            to_entity: "customers".to_string(),
            to_field: "id".to_string(),
        })),
        ("+ ⚡ Query Filter", super::FlowNodeKind::DataBlock(super::DataBlockKind::QueryFilter {
            field: "status".to_string(),
            operator: "==".to_string(),
            target_val: "active".to_string(),
        })),
        ("+ 💾 DB Mutator", super::FlowNodeKind::DataBlock(super::DataBlockKind::RecordMutator {
            entity: "contacts".to_string(),
            operation: "UPSERT".to_string(),
        })),
        ("+ Relay Actuator", super::FlowNodeKind::HardwareRelay {
            device_id: "RELAY-01".to_string(),
            channel: 1,
            action: "pulse".to_string(),
            pulse_ms: 500,
        }),
        ("+ Sensor Watchdog", super::FlowNodeKind::HardwareSensor {
            device_id: "SENSOR-01".to_string(),
            metric: "temperature_c".to_string(),
            threshold: 30.0,
            operator: ">".to_string(),
        }),
        ("+ Scale Weigh", super::FlowNodeKind::HardwareScale {
            device_id: "SCALE-01".to_string(),
            require_stable: true,
        }),
        ("+ VFD Display", super::FlowNodeKind::HardwareDisplay {
            device_id: "VFD-01".to_string(),
            line1: "TOTAL: 0.00 EUR".to_string(),
            line2: "THANK YOU".to_string(),
        }),
        ("+ 📦 RFID Chute", super::FlowNodeKind::HardwareRfidChute {
            device_id: "RFID-RING-01".to_string(),
            target_container_code: "TOTE-01".to_string(),
            auto_seal_on_count: Some(50),
        }),
    ]
}
