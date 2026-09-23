//! Domain-driven Penpot-inspired Inspector module coordinator for Proteus Designer.
//! Coordinates the Design Tab (Geometry, Styling, Roles) and Prototype Tab (Flows, DB bindings).

pub mod controls;
pub mod design_tab;
pub mod mutations;
pub mod prototype_tab;

pub use controls::*;
pub use design_tab::*;
pub use mutations::*;
pub use prototype_tab::*;

use crate::models::InspectorTab;
use crate::scene::{CanvasEvent, EditorState, ProjectDocument};
use crate::theme;
use eframe::egui::{self, Color32, RichText, ScrollArea, Vec2};

/// Full-height Penpot-inspired Inspector with Design and Prototype tabs.
pub fn draw_inspector(
    ui: &mut egui::Ui,
    doc: &ProjectDocument,
    editor_state: &EditorState,
    inspector_tab: &mut InspectorTab,
    copied_dimensions: &mut Option<(f32, f32)>,
) -> Vec<CanvasEvent> {
    let mut events = Vec::new();

    // ── Penpot Inspector Tab Header ──
    ui.horizontal(|ui| {
        let is_design = *inspector_tab == InspectorTab::Design;
        let is_proto = *inspector_tab == InspectorTab::Prototype;

        if ui.add(
            egui::Button::new(
                RichText::new("📐 DESIGN")
                    .size(10.)
                    .strong()
                    .color(if is_design { theme::ACCENT } else { Color32::GRAY }),
            )
            .fill(if is_design { Color32::from_rgba_unmultiplied(79, 140, 237, 25) } else { Color32::TRANSPARENT })
            .stroke(if is_design { egui::Stroke::new(1., theme::ACCENT) } else { egui::Stroke::NONE })
            .min_size(Vec2::new(95., 24.)),
        ).clicked() {
            *inspector_tab = InspectorTab::Design;
        }

        if ui.add(
            egui::Button::new(
                RichText::new("⚡ PROTOTYPE")
                    .size(10.)
                    .strong()
                    .color(if is_proto { theme::ACCENT } else { Color32::GRAY }),
            )
            .fill(if is_proto { Color32::from_rgba_unmultiplied(79, 140, 237, 25) } else { Color32::TRANSPARENT })
            .stroke(if is_proto { egui::Stroke::new(1., theme::ACCENT) } else { egui::Stroke::NONE })
            .min_size(Vec2::new(95., 24.)),
        ).clicked() {
            *inspector_tab = InspectorTab::Prototype;
        }
    });

    ui.add_space(4.);
    ui.separator();

    let selected_id = editor_state.selected_node_ids.first().cloned();
    let selected_node = selected_id.as_ref().and_then(|id| doc.get_node(id));

    ScrollArea::vertical().id_salt("penpot_inspector_scroll").show(ui, |ui| {
        if let (Some(node_id), Some(node)) = (selected_id.as_ref(), selected_node) {
            // Element Name Header
            ui.add_space(2.);
            let mut name_buf = node.name.clone();
            ui.horizontal(|ui| {
                ui.label(RichText::new("LAYER:").size(9.).color(theme::TEXT_DIM));
                if ui.add(egui::TextEdit::singleline(&mut name_buf).desired_width(130.)).changed() {
                    events.push(rename_node(node_id.clone(), name_buf));
                }
            });
            ui.label(RichText::new(format!("ID: {}", node_id)).size(8.).color(Color32::GRAY));
            ui.separator();

            match inspector_tab {
                InspectorTab::Design => {
                    draw_design_tab(ui, node, doc, copied_dimensions, &mut events);
                }
                InspectorTab::Prototype => {
                    draw_prototype_tab(ui, node, doc, &mut events);
                }
            }
        } else {
            ui.add_space(24.);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("No Element Selected").size(11.).color(theme::TEXT_DIM).strong());
                ui.label(RichText::new("Select a layer from the left panel or click on the canvas.").size(9.5).color(Color32::GRAY));
            });
        }
    });

    events
}
