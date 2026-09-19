//! Domain-driven Inspector module coordinator for Proteus Designer.
//! Coordinates transform controls, smart role selectors, type-specific properties,
//! and the layers hierarchy tree.

pub mod controls;
pub mod mutations;
pub mod node_props;

pub use controls::*;
pub use mutations::*;
pub use node_props::*;

use crate::scene::{CanvasEvent, EditorState, ProjectDocument};
use crate::theme;
use eframe::egui::{self, Color32, RichText, ScrollArea, Vec2};

/// Two-zone Proteus Inspector:
/// - Upper 2/3: Transform, Sizing, Smart Box Role Selector, Styling & DB Binding
/// - Lower 1/3: Layers / Hierarchy Tree with Lock & Visibility toggles
pub fn draw_inspector(
    ui: &mut egui::Ui,
    doc: &ProjectDocument,
    editor_state: &EditorState,
    copied_dimensions: &mut Option<(f32, f32)>,
) -> Vec<CanvasEvent> {
    let mut events = Vec::new();
    let total_h = ui.available_height();
    let top_h = (total_h * 0.65).max(220.0);
    let bottom_h = (total_h - top_h - 14.0).max(120.0);

    let selected_id = editor_state.selected_node_ids.first().cloned();
    let selected_node = selected_id.as_ref().and_then(|id| doc.get_node(id));

    // ══════════════════════════════════════════════════════════════
    // ZONE 1: Upper 2/3 (Properties, Smart Role, Transform, Style)
    // ══════════════════════════════════════════════════════════════
    ui.allocate_ui(Vec2::new(ui.available_width(), top_h), |ui| {
        ScrollArea::vertical().id_salt("inspector_properties").show(ui, |ui| {
            if let (Some(node_id), Some(node)) = (selected_id.as_ref(), selected_node) {
                // Header & Rename
                ui.add_space(2.);
                let mut name_buf = node.name.clone();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("NAME:").size(9.).color(theme::TEXT_DIM));
                    if ui.add(egui::TextEdit::singleline(&mut name_buf).desired_width(140.)).changed() {
                        events.push(rename_node(node_id.clone(), name_buf));
                    }
                });
                ui.label(RichText::new(format!("ID: {}", node_id)).size(8.).color(Color32::GRAY));
                ui.separator();

                // 1. Transform & Size (with alignment & copy/paste)
                draw_transform_props(ui, node, doc, copied_dimensions, &mut events);

                ui.separator();

                // 2. Smart Box Role Selector
                draw_smart_role_selector(ui, node, &mut events);

                // 3. Type-specific property editors
                draw_input_props(ui, node, &mut events);
                draw_typography_props(ui, node, &mut events);
                draw_button_props(ui, node, doc, &mut events);
                draw_table_props(ui, node, &mut events);

                // 4. Styling & Appearance
                draw_styling_props(ui, node, &mut events);
            } else {
                ui.add_space(30.);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No Element Selected").size(11.).color(theme::TEXT_DIM).strong());
                    ui.label(RichText::new("Select a layer below or click on canvas").size(9.).color(Color32::DARK_GRAY));
                });
            }
        });
    });

    ui.separator();

    // ══════════════════════════════════════════════════════════════
    // ZONE 2: Lower 1/3 (Layers & Hierarchy Tree, Lock & Visibility)
    // ══════════════════════════════════════════════════════════════
    crate::components::layers_panel::draw_layers_panel(
        ui,
        doc,
        selected_id.as_deref(),
        bottom_h,
        &mut events,
    );

    events
}
