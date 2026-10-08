//! Lunacy-Grade Left Sidebar & Layers Tree with Hierarchy, Symbols & Asset Palette.
//! Provides deep nested layer hierarchy with collapsible branches, quick search filtering,
//! visibility/lock toggles, reusable Symbol components catalog, and vector asset presets.

use eframe::egui::{self, Color32, Key, RichText, ScrollArea, Stroke, Vec2};
use std::collections::HashSet;
use crate::scene::{CanvasEvent, Node, NodeType, NodeUpdate, ProjectDocument};
use crate::theme;

/// Left sidebar navigation tab mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LeftSidebarTab {
    Layers,
    Symbols,
    Assets,
}

impl Default for LeftSidebarTab {
    fn default() -> Self {
        Self::Layers
    }
}

/// Reusable design symbol definition.
#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub preview_icon: &'static str,
    pub description: &'static str,
}

/// Master catalog of built-in sovereign UI symbols.
pub fn available_symbols() -> Vec<SymbolDefinition> {
    vec![
        SymbolDefinition {
            id: "sym_kpi_card",
            name: "Metric KPI Card",
            category: "Analytics",
            preview_icon: "💳",
            description: "High-contrast revenue and telemetry metric container",
        },
        SymbolDefinition {
            id: "sym_intake_form",
            name: "Service Intake Form",
            category: "Forms",
            preview_icon: "📋",
            description: "Customer and hardware fault registration block",
        },
        SymbolDefinition {
            id: "sym_thermal_slip",
            name: "ESC/POS Receipt Slip",
            category: "Retail",
            preview_icon: "🧾",
            description: "80mm/58mm thermal receipt layout with QR placeholder",
        },
        SymbolDefinition {
            id: "sym_data_table",
            name: "Bound SQLite Grid",
            category: "Data",
            preview_icon: "⊞",
            description: "Virtual scrolling table with column sorting headers",
        },
        SymbolDefinition {
            id: "sym_navbar",
            name: "Sovereign TopBar",
            category: "Navigation",
            preview_icon: "⎍",
            description: "Role switcher, status badge, and breadcrumb bar",
        },
    ]
}

/// State tracking for tree collapsing, search filtering, and active sidebar mode.
#[derive(Debug, Clone, Default)]
pub struct LayersPanelState {
    pub active_tab: LeftSidebarTab,
    pub search_query: String,
    pub collapsed_nodes: HashSet<String>,
    pub renaming_node_id: Option<String>,
    pub rename_buffer: String,
}

impl LayersPanelState {
    pub fn toggle_collapsed(&mut self, node_id: &str) {
        if self.collapsed_nodes.contains(node_id) {
            self.collapsed_nodes.remove(node_id);
        } else {
            self.collapsed_nodes.insert(node_id.to_string());
        }
    }

    pub fn is_collapsed(&self, node_id: &str) -> bool {
        self.collapsed_nodes.contains(node_id)
    }
}

/// Instantiates a symbol into a concrete Node.
pub fn instantiate_symbol(symbol_id: &str, pos: (f32, f32)) -> Option<Node> {
    match symbol_id {
        "sym_kpi_card" => {
            let mut node = Node::new(
                format!("kpi_{}", uuid::Uuid::new_v4().simple()),
                "KPI Metric Card".into(),
                NodeType::Frame,
            );
            node.position = pos;
            node.layout.width = crate::scene::Sizing::Fixed(220.);
            node.layout.height = crate::scene::Sizing::Fixed(110.);
            node.style.bg_color = [0.12, 0.13, 0.18, 1.0];
            node.style.border_radius = 8.0;
            node.style.border_color = [0.25, 0.35, 0.55, 0.8];
            Some(node)
        }
        "sym_intake_form" => {
            let mut node = Node::new(
                format!("form_{}", uuid::Uuid::new_v4().simple()),
                "Intake Form".into(),
                NodeType::DynamicForm {
                    bound_entity: "service_tickets".into(),
                    title: "Hardware Intake".into(),
                    submit_label: "Save Ticket".into(),
                },
            );
            node.position = pos;
            node.layout.width = crate::scene::Sizing::Fixed(360.);
            node.layout.height = crate::scene::Sizing::Fixed(300.);
            Some(node)
        }
        "sym_thermal_slip" => {
            let mut node = Node::new(
                format!("slip_{}", uuid::Uuid::new_v4().simple()),
                "Thermal Receipt 80mm".into(),
                NodeType::Frame,
            );
            node.position = pos;
            node.layout.width = crate::scene::Sizing::Fixed(280.);
            node.layout.height = crate::scene::Sizing::Fixed(400.);
            node.style.bg_color = [0.98, 0.98, 0.98, 1.0];
            node.style.border_radius = 2.0;
            Some(node)
        }
        "sym_data_table" => {
            let mut node = Node::new(
                format!("tbl_{}", uuid::Uuid::new_v4().simple()),
                "SQLite Data Grid".into(),
                NodeType::Table {
                    bound_entity: Some("contacts".into()),
                    columns: vec!["ID".into(), "Name".into(), "Phone".into(), "Balance".into()],
                },
            );
            node.position = pos;
            node.layout.width = crate::scene::Sizing::Fixed(480.);
            node.layout.height = crate::scene::Sizing::Fixed(260.);
            Some(node)
        }
        "sym_navbar" => {
            let mut node = Node::new(
                format!("nav_{}", uuid::Uuid::new_v4().simple()),
                "Navigation Bar".into(),
                NodeType::Frame,
            );
            node.position = pos;
            node.layout.width = crate::scene::Sizing::Fixed(800.);
            node.layout.height = crate::scene::Sizing::Fixed(48.);
            node.style.bg_color = [0.08, 0.09, 0.12, 1.0];
            Some(node)
        }
        _ => None,
    }
}

/// Returns the characteristic Lunacy-style icon for a node kind.
pub fn node_kind_icon(node_type: &NodeType) -> &'static str {
    match node_type {
        NodeType::Page => "▦",
        NodeType::Frame => "⧉",
        NodeType::Group => "🗂",
        NodeType::Text { .. } => "T",
        NodeType::Shape { .. } => "▢",
        NodeType::Image { .. } => "◫",
        NodeType::Button { .. } => "🔘",
        NodeType::Table { .. } => "⊞",
        NodeType::DynamicForm { .. } => "📋",
        NodeType::TextInput { .. } => "✎",
        NodeType::NumberField { .. } => "#",
        NodeType::Dropdown { .. } => "▾",
        NodeType::Checkbox { .. } => "☑",
    }
}

/// Renders the complete Lunacy-grade Left Sidebar.
pub fn draw_lunacy_sidebar(
    ui: &mut egui::Ui,
    state: &mut LayersPanelState,
    doc: &ProjectDocument,
    selected_id: Option<&str>,
    events: &mut Vec<CanvasEvent>,
    on_insert_node: &mut Option<Node>,
) {
    ui.vertical(|ui| {
        // ── 1. Top Mode Selector Ribbon ──
        ui.horizontal(|ui| {
            let tabs = [
                (LeftSidebarTab::Layers, "▤ Layers"),
                (LeftSidebarTab::Symbols, "✦ Symbols"),
                (LeftSidebarTab::Assets, "⊞ Assets"),
            ];

            for (tab, label) in tabs {
                let is_active = state.active_tab == tab;
                let btn = egui::Button::new(
                    RichText::new(label)
                        .size(10.5)
                        .strong()
                        .color(if is_active { theme::ACCENT } else { Color32::GRAY }),
                )
                .fill(if is_active { Color32::from_rgba_unmultiplied(79, 140, 237, 28) } else { Color32::TRANSPARENT })
                .stroke(if is_active { Stroke::new(1., theme::ACCENT) } else { Stroke::NONE })
                .min_size(Vec2::new(64., 24.));

                if ui.add(btn).clicked() {
                    state.active_tab = tab;
                }
            }
        });

        ui.add_space(4.);
        ui.separator();
        ui.add_space(2.);

        // ── 2. Tab Content View ──
        match state.active_tab {
            LeftSidebarTab::Layers => {
                // Search bar
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(10.).color(Color32::GRAY));
                    ui.add(
                        egui::TextEdit::singleline(&mut state.search_query)
                            .hint_text("Filter layers...")
                            .desired_width(ui.available_width() - 8.),
                    );
                });
                ui.add_space(4.);

                ScrollArea::vertical().id_salt("lunacy_layers_tree").show(ui, |ui| {
                    if doc.nodes.is_empty() {
                        ui.label(RichText::new("No canvas layers").size(10.).italics().color(Color32::DARK_GRAY));
                    } else {
                        // Render hierarchical roots
                        for root_id in &doc.root_node_ids {
                            draw_layer_tree_recursive(
                                ui,
                                state,
                                doc,
                                root_id,
                                0,
                                selected_id,
                                events,
                            );
                        }
                    }
                });
            }
            LeftSidebarTab::Symbols => {
                ui.label(RichText::new("REUSABLE SYMBOL LIBRARY").size(9.).strong().color(theme::TEXT_MUTED));
                ui.add_space(4.);

                ScrollArea::vertical().id_salt("lunacy_symbols_catalog").show(ui, |ui| {
                    for sym in available_symbols() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(sym.preview_icon).size(16.));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(sym.name).size(11.).strong().color(Color32::WHITE));
                                    ui.label(RichText::new(sym.category).size(9.).color(theme::ACCENT));
                                });
                            });
                            ui.label(RichText::new(sym.description).size(9.).color(Color32::LIGHT_GRAY));
                            ui.add_space(2.);

                            if ui.add(
                                egui::Button::new("Insert on Canvas →")
                                    .min_size(Vec2::new(ui.available_width(), 20.))
                            ).clicked() {
                                if let Some(node) = instantiate_symbol(sym.id, (100., 100.)) {
                                    *on_insert_node = Some(node);
                                }
                            }
                        });
                        ui.add_space(3.);
                    }
                });
            }
            LeftSidebarTab::Assets => {
                ui.label(RichText::new("PRIMITIVES & VECTOR ASSETS").size(9.).strong().color(theme::TEXT_MUTED));
                ui.add_space(4.);

                let primitives = [
                    ("▢ Frame Box", NodeType::Frame),
                    ("T Text Element", NodeType::Text { content: "Sample Text".into(), font: Default::default() }),
                    ("🔘 Action Button", NodeType::Button { label: "Click Me".into(), style: crate::scene::ButtonStyle::Primary }),
                    ("⊞ Data Table", NodeType::Table { bound_entity: None, columns: vec!["Col 1".into(), "Col 2".into()] }),
                ];

                for (label, ntype) in primitives {
                    if ui.add(
                        egui::Button::new(label)
                            .min_size(Vec2::new(ui.available_width(), 24.))
                    ).clicked() {
                        let node = Node::new(
                            format!("node_{}", uuid::Uuid::new_v4().simple()),
                            label.to_string(),
                            ntype,
                        );
                        *on_insert_node = Some(node);
                    }
                    ui.add_space(2.);
                }
            }
        }
    });
}

/// Recursively renders a node and its nested hierarchy with indentation.
fn draw_layer_tree_recursive(
    ui: &mut egui::Ui,
    state: &mut LayersPanelState,
    doc: &ProjectDocument,
    node_id: &str,
    depth: usize,
    selected_id: Option<&str>,
    events: &mut Vec<CanvasEvent>,
) {
    let Some(node) = doc.get_node(node_id) else { return; };

    // Search query filtering
    let matches_search = state.search_query.is_empty()
        || node.name.to_lowercase().contains(&state.search_query.to_lowercase());

    let has_children = !node.children_ids.is_empty();
    let is_collapsed = state.is_collapsed(node_id);
    let is_selected = selected_id == Some(node_id);

    if matches_search {
        ui.horizontal(|ui| {
            // Hierarchy indentation padding
            if depth > 0 {
                ui.add_space((depth as f32) * 12.0);
            }

            // Expand/collapse chevron
            if has_children {
                let chevron = if is_collapsed { "▶" } else { "▼" };
                if ui.add(egui::Button::new(RichText::new(chevron).size(8.)).frame(false)).clicked() {
                    state.toggle_collapsed(node_id);
                }
            } else {
                ui.add_space(10.0);
            }

            // Node icon & name
            let icon = node_kind_icon(&node.node_type);
            let display_text = format!("{} {}", icon, node.name);
            let text_color = if !node.visible {
                Color32::from_rgb(100, 105, 115)
            } else if is_selected {
                theme::ACCENT
            } else {
                Color32::from_rgb(220, 225, 235)
            };

            // Inline rename check
            if state.renaming_node_id.as_deref() == Some(node_id) {
                let resp = ui.text_edit_singleline(&mut state.rename_buffer);
                if resp.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
                    if !state.rename_buffer.trim().is_empty() {
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.to_string(),
                            update: NodeUpdate::Rename(state.rename_buffer.clone()),
                        });
                    }
                    state.renaming_node_id = None;
                }
            } else {
                let label_btn = ui.selectable_label(is_selected, RichText::new(display_text).size(10.).color(text_color));
                if label_btn.clicked() {
                    events.push(CanvasEvent::NodeClicked {
                        id: node_id.to_string(),
                        shift_held: ui.input(|i| i.modifiers.shift),
                    });
                }
                if label_btn.double_clicked() {
                    state.renaming_node_id = Some(node_id.to_string());
                    state.rename_buffer = node.name.clone();
                }
            }

            // Right utility controls: lock, eye, delete
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new("✕").min_size(Vec2::new(12., 12.)).frame(false)).clicked() {
                    events.push(CanvasEvent::DeleteNode { id: node_id.to_string() });
                }

                let lock_icon = if node.locked { "🔒" } else { "🔓" };
                if ui.add(egui::Button::new(RichText::new(lock_icon).size(9.)).frame(false)).clicked() {
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.to_string(),
                        update: NodeUpdate::ToggleLock,
                    });
                }

                let eye_icon = if node.visible { "👁" } else { "🚫" };
                if ui.add(egui::Button::new(RichText::new(eye_icon).size(9.)).frame(false)).clicked() {
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.to_string(),
                        update: NodeUpdate::ToggleVisibility,
                    });
                }
            });
        });
    }

    // Render children if not collapsed
    if has_children && !is_collapsed {
        for child_id in &node.children_ids {
            draw_layer_tree_recursive(ui, state, doc, child_id, depth + 1, selected_id, events);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_catalog_and_instantiation() {
        let symbols = available_symbols();
        assert_eq!(symbols.len(), 5);

        let kpi = instantiate_symbol("sym_kpi_card", (50., 60.)).unwrap();
        assert_eq!(kpi.name, "KPI Metric Card");
        assert_eq!(kpi.position, (50., 60.));

        let form = instantiate_symbol("sym_intake_form", (10., 20.)).unwrap();
        assert!(matches!(form.node_type, NodeType::DynamicForm { .. }));
    }

    #[test]
    fn test_layers_panel_collapsing_state() {
        let mut state = LayersPanelState::default();
        assert!(!state.is_collapsed("frame_1"));

        state.toggle_collapsed("frame_1");
        assert!(state.is_collapsed("frame_1"));

        state.toggle_collapsed("frame_1");
        assert!(!state.is_collapsed("frame_1"));
    }

    #[test]
    fn test_node_kind_icons() {
        assert_eq!(node_kind_icon(&NodeType::Frame), "⧉");
        assert_eq!(node_kind_icon(&NodeType::Page), "▦");
        assert_eq!(node_kind_icon(&NodeType::Table { bound_entity: None, columns: vec![] }), "⊞");
    }
}
