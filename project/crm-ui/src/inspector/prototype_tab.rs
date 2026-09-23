//! Penpot-inspired Prototype & Data Binding Tab for the Proteus Inspector.
//! Covers screen navigation triggers, SQLite submit targets, and data table bindings.

use eframe::egui::{self, Color32, RichText};
use crate::scene::{CanvasEvent, Node, NodeType, ProjectDocument};
use crate::theme;
use super::controls::info_button;
use super::mutations::*;

pub fn draw_prototype_tab(
    ui: &mut egui::Ui,
    node: &Node,
    doc: &ProjectDocument,
    events: &mut Vec<CanvasEvent>,
) {
    ui.add_space(2.);
    ui.horizontal(|ui| {
        ui.label(RichText::new("PROTOTYPE & FLOW TRIGGERS").size(9.).color(theme::ACCENT).strong());
        info_button(ui, "Ορίστε διαδραστικές ενέργειες (κλικ/υποβολή) και διασύνδεση με τοπική βάση SQLite.");
    });

    // 1. Button Actions & Navigation
    if let NodeType::Button { .. } = &node.node_type {
        draw_button_action_flow(ui, node, doc, events);
    }

    // 2. Data-Bound Table Configuration
    if let NodeType::Table { bound_entity, .. } = &node.node_type {
        draw_table_data_binding(ui, node, bound_entity.as_deref(), events);
    }

    // 3. Input Data Field Mapping
    if let NodeType::TextInput { bound_entity, bound_field, .. } = &node.node_type {
        draw_input_data_binding(ui, node, bound_entity.as_deref(), bound_field.as_deref(), events);
    }

    // 4. W3C Design Token Association
    ui.add_space(8.);
    ui.separator();
    draw_token_associations(ui, node);
}

fn draw_button_action_flow(
    ui: &mut egui::Ui,
    node: &Node,
    doc: &ProjectDocument,
    events: &mut Vec<CanvasEvent>,
) {
    ui.add_space(4.);
    ui.label(RichText::new("ON-CLICK TRIGGER:").size(8.5).color(Color32::GRAY));

    let (cur_nav, cur_submit) = doc.get_button_action(&node.id);
    let available_pages: Vec<(&String, &str)> = doc.root_node_ids.iter().filter_map(|pid| {
        doc.nodes.get(pid).map(|pn| (pid, pn.name.as_str()))
    }).collect();

    let current_nav_page = cur_nav.clone().unwrap_or_default();
    let current_nav_label = if current_nav_page.is_empty() {
        "— None (Stay on Screen) —".to_string()
    } else {
        available_pages.iter()
            .find(|(id, _)| **id == current_nav_page)
            .map(|(id, name)| format!("{} ({})", name, id))
            .unwrap_or_else(|| current_nav_page.clone())
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new("Navigate To:").size(10.));
        egui::ComboBox::from_id_salt(format!("proto_btn_nav_{}", node.id))
            .selected_text(current_nav_label)
            .width(150.)
            .show_ui(ui, |ui| {
                if ui.selectable_label(current_nav_page.is_empty(), "— None —").clicked() {
                    events.push(set_button_action(node.id.clone(), None, cur_submit.clone()));
                }
                for (pid, pname) in &available_pages {
                    let is_sel = **pid == current_nav_page;
                    if ui.selectable_label(is_sel, format!("{} ({})", pname, pid)).clicked() {
                        events.push(set_button_action(node.id.clone(), Some((*pid).clone()), cur_submit.clone()));
                    }
                }
            });
    });

    let mut submit_val = cur_submit.clone().unwrap_or_default();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Submit Entity:").size(10.));
        let resp = ui.add(egui::TextEdit::singleline(&mut submit_val).hint_text("e.g. contacts").desired_width(110.));
        if resp.lost_focus() {
            let new_sub = if submit_val.trim().is_empty() { None } else { Some(submit_val.trim().to_string()) };
            if new_sub != cur_submit {
                events.push(set_button_action(node.id.clone(), cur_nav.clone(), new_sub));
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("Quick Entities:").size(8.5).color(theme::TEXT_MUTED));
        for ent_name in &["contacts", "deals", "tasks", "tickets"] {
            let is_active = cur_submit.as_deref() == Some(*ent_name);
            if ui.selectable_label(is_active, *ent_name).clicked() {
                let new_sub = if is_active { None } else { Some((*ent_name).to_string()) };
                events.push(set_button_action(node.id.clone(), cur_nav.clone(), new_sub));
            }
        }
    });
}

fn draw_table_data_binding(
    ui: &mut egui::Ui,
    node: &Node,
    bound_entity: Option<&str>,
    events: &mut Vec<CanvasEvent>,
) {
    ui.add_space(4.);
    ui.label(RichText::new("SQLITE ENTITY BINDING:").size(8.5).color(Color32::GRAY));

    let mut ent = bound_entity.unwrap_or_default().to_string();
    ui.horizontal(|ui| {
        ui.label(RichText::new("Entity Table:").size(10.));
        let resp = ui.add(egui::TextEdit::singleline(&mut ent).hint_text("e.g. contacts").desired_width(120.));
        if resp.lost_focus() && ent != bound_entity.unwrap_or_default() {
            events.push(bind_data(node.id.clone(), if ent.is_empty() { None } else { Some(ent.clone()) }, None));
        }
    });

    ui.horizontal(|ui| {
        for preset_ent in &["contacts", "deals", "tasks", "tickets", "inventory"] {
            let is_curr = ent == *preset_ent;
            if ui.selectable_label(is_curr, *preset_ent).clicked() && !is_curr {
                events.push(bind_data(node.id.clone(), Some((*preset_ent).to_string()), None));
            }
        }
    });
}

fn draw_input_data_binding(
    ui: &mut egui::Ui,
    node: &Node,
    bound_entity: Option<&str>,
    bound_field: Option<&str>,
    events: &mut Vec<CanvasEvent>,
) {
    ui.add_space(4.);
    ui.label(RichText::new("FIELD DATA BINDING:").size(8.5).color(Color32::GRAY));

    let mut ent = bound_entity.unwrap_or_default().to_string();
    let mut fld = bound_field.unwrap_or_default().to_string();

    ui.horizontal(|ui| {
        ui.label(RichText::new("Entity:").size(10.));
        let r1 = ui.add(egui::TextEdit::singleline(&mut ent).hint_text("contacts").desired_width(80.));
        ui.label(RichText::new("Field:").size(10.));
        let r2 = ui.add(egui::TextEdit::singleline(&mut fld).hint_text("email").desired_width(80.));
        if r1.lost_focus() || r2.lost_focus() {
            let e_opt = if ent.is_empty() { None } else { Some(ent) };
            let f_opt = if fld.is_empty() { None } else { Some(fld) };
            events.push(bind_data(node.id.clone(), e_opt, f_opt));
        }
    });
}

fn draw_token_associations(ui: &mut egui::Ui, _node: &Node) {
    ui.label(RichText::new("W3C DESIGN TOKENS LINK:").size(8.5).color(theme::TEXT_MUTED));
    ui.horizontal(|ui| {
        ui.label(RichText::new("Tokens:").size(9.).color(Color32::GRAY));
        ui.label(RichText::new("surface.card &bull; radius.md").size(9.).color(theme::ACCENT));
    });
}
