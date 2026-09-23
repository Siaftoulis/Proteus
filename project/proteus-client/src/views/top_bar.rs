//! Top Navigation and Contextual Sub-Navigation Bars for Proteus Client.
//! Renders the brand, role switcher, workspace pills, system connectivity, and active tools.

use crate::views::auth::ClientAuthState;
use crate::views::navigation::{NavTab, RoleWorkspace};
use crm_core::roles::{RolePermissions, UserRole};
use crm_core::tickets::list_tickets;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

pub struct TopBarState<'a> {
    pub mounted_bundle_title: &'a str,
    pub active_role: &'a mut UserRole,
    pub operator_name: &'a mut String,
    pub device_label_ref: &'a std::sync::Arc<std::sync::Mutex<String>>,
    pub active_workspace: &'a mut RoleWorkspace,
    pub active_tab: &'a mut NavTab,
    pub workspaces: &'a [RoleWorkspace],
    pub permissions: &'a RolePermissions,
    pub auth_state: &'a mut ClientAuthState,
    pub conn: &'a Connection,
    pub lan_beacon_active: bool,
    pub replication_synced: Option<usize>,
}

/// Renders the top primary navigation bar (Brand, Role, Workspace Pills, Actions).
pub fn render_top_bar(ctx: &egui::Context, state: &mut TopBarState<'_>) {
    egui::TopBottomPanel::top("top_nav_bar")
        .frame(
            Frame::new()
                .fill(crate::theme::BG_PANEL)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .inner_margin(Margin::symmetric(14, 8)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Logo & Brand
                ui.horizontal(|ui| {
                    ui.label(RichText::new("PROTEUS").strong().size(17.0).color(crate::theme::ACCENT_CYAN));
                    ui.label(RichText::new("BOS").size(13.0).color(crate::theme::TEXT_MUTED));
                });

                ui.add_space(6.0);

                // Mounted PR Package Badge
                Frame::new()
                    .fill(Color32::from_rgb(18, 28, 45))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(56, 130, 220)))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(7, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("📦 {}", state.mounted_bundle_title))
                                .size(11.0)
                                .color(Color32::from_rgb(140, 195, 255)),
                        );
                    });

                ui.add_space(8.0);

                // Active Role Dropdown Selector
                egui::ComboBox::from_id_salt("active_role_selector")
                    .selected_text(
                        RichText::new(state.active_role.display_name())
                            .strong()
                            .size(12.0)
                            .color(Color32::WHITE),
                    )
                    .show_ui(ui, |ui| {
                        for role in UserRole::all() {
                            let is_selected = *state.active_role == *role;
                            if ui.selectable_label(is_selected, role.display_name()).clicked() {
                                *state.active_role = *role;
                                *state.operator_name = match *role {
                                    UserRole::Ceo => "Admin (CEO)".to_string(),
                                    UserRole::CustomerService => "Μαρία (Reception)".to_string(),
                                    UserRole::Technician => "Νίκος (Τεχνικός)".to_string(),
                                    UserRole::SalesConsultant => "Κώστας (Sales)".to_string(),
                                    UserRole::Developer => "Αλέξανδρος (Dev)".to_string(),
                                    UserRole::BusinessAnalyst => "Δημήτρης (Analyst)".to_string(),
                                };
                                if let Ok(mut label) = state.device_label_ref.lock() {
                                    *label = format!("Proteus Terminal ({})", role.display_name());
                                }
                            }
                        }
                    });

                ui.add_space(10.0);

                // Dedicated Role Workspaces Pills
                render_workspace_pills_ui(
                    ui,
                    state.workspaces,
                    state.active_workspace,
                    state.active_tab,
                    state.permissions,
                    *state.active_role,
                );

                // Right Side: Quick Action buttons & Connectivity Badges
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new("🚪 Έξοδος").size(11.0))
                        .on_hover_text("Αποσύνδεση από λογαριασμό Marketplace")
                        .clicked()
                    {
                        state.auth_state.logout();
                    }
                    if ui
                        .button(RichText::new("🔄 Αλλαγή CRM").size(11.0))
                        .on_hover_text("Επιλογή άλλου CRM project από τον Launcher")
                        .clicked()
                    {
                        state.auth_state.switch_project();
                    }
                    if ui
                        .button(RichText::new("🔒 Κλείδωμα").size(11.0))
                        .on_hover_text("Κλείδωμα τερματικού (Απαιτεί Master / Staff PIN)")
                        .clicked()
                    {
                        state.auth_state.lock_terminal();
                    }

                    // Offline-first pill
                    Frame::new()
                        .fill(Color32::from_rgb(16, 50, 35))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(7, 3))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("● 100% Τοπική Λειτουργία")
                                    .size(11.0)
                                    .color(Color32::from_rgb(52, 211, 153)),
                            );
                        });

                    let pending_sync = crm_core::replication::count_pending_outbox(state.conn).unwrap_or(0);
                    if pending_sync > 0 {
                        Frame::new()
                            .fill(Color32::from_rgb(60, 40, 10))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(251, 146, 60)))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(7, 3))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("⚡ {} Εκκρεμή", pending_sync))
                                        .size(11.0)
                                        .color(Color32::from_rgb(251, 146, 60)),
                                );
                            });
                    }

                    if state.lan_beacon_active {
                        ui.label(RichText::new("📡 LAN Ready").size(11.0).color(Color32::from_rgb(56, 189, 248)));
                    }

                    if let Some(pushed) = state.replication_synced {
                        ui.label(
                            RichText::new(format!("🔄 Sync ({})", pushed))
                                .size(11.0)
                                .color(Color32::from_rgb(52, 211, 153)),
                        );
                    }
                });
            });
        });
}

/// Renders the contextual sub-navigation toolbar for the active workspace.
pub fn render_sub_bar(
    ctx: &egui::Context,
    sub_tabs: &[(NavTab, &'static str)],
    active_tab: &mut NavTab,
    active_workspace: RoleWorkspace,
    operator_name: &str,
    conn: &Connection,
) {
    egui::TopBottomPanel::top("sub_nav_bar")
        .frame(
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .inner_margin(Margin::symmetric(14, 6)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (tab, label) in sub_tabs {
                    let is_active = *tab == *active_tab;
                    let btn = if is_active {
                        egui::Button::new(RichText::new(*label).strong().size(12.0).color(Color32::WHITE))
                            .fill(crate::theme::ACCENT_PRIMARY)
                    } else {
                        egui::Button::new(RichText::new(*label).size(12.0).color(crate::theme::TEXT_SECONDARY))
                            .fill(crate::theme::BG_CARD)
                    };

                    if ui.add(btn).clicked() {
                        *active_tab = *tab;
                    }
                    ui.add_space(2.0);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total_tickets = list_tickets(conn).map(|t| t.len()).unwrap_or(0);
                    ui.label(
                        RichText::new(format!("Σύνολο: {} Επισκευές", total_tickets))
                            .size(11.0)
                            .color(crate::theme::TEXT_MUTED),
                    );
                    ui.label(
                        RichText::new(format!("Χειριστής: {}", operator_name))
                            .size(11.0)
                            .color(crate::theme::TEXT_SECONDARY),
                    );
                    ui.label(
                        RichText::new(format!("[{}]", active_workspace.short_code()))
                            .size(11.0)
                            .strong()
                            .color(crate::theme::ACCENT_CYAN),
                    );
                });
            });
        });
}

fn render_workspace_pills_ui(
    ui: &mut Ui,
    workspaces: &[RoleWorkspace],
    active_workspace: &mut RoleWorkspace,
    active_tab: &mut NavTab,
    permissions: &RolePermissions,
    active_role: UserRole,
) {
    ui.horizontal(|ui| {
        for ws in workspaces {
            let is_active = *ws == *active_workspace;
            let (bg, border_color, text_color) = if is_active {
                (
                    Color32::from_rgb(30, 41, 59),
                    crate::theme::ACCENT_CYAN,
                    Color32::WHITE,
                )
            } else {
                (
                    Color32::from_rgb(18, 21, 27),
                    crate::theme::BORDER_SUBTLE,
                    crate::theme::TEXT_SECONDARY,
                )
            };

            let stroke = if is_active {
                Stroke::new(1.5, border_color)
            } else {
                Stroke::new(1.0, border_color)
            };

            Frame::new()
                .fill(bg)
                .stroke(stroke)
                .corner_radius(CornerRadius::same(5))
                .inner_margin(Margin::symmetric(10, 5))
                .show(ui, |ui| {
                    let resp = ui.add(
                        egui::Label::new(
                            RichText::new(ws.display_label())
                                .strong()
                                .size(12.0)
                                .color(text_color),
                        )
                        .sense(egui::Sense::click()),
                    );
                    if resp.clicked() && !is_active {
                        *active_workspace = *ws;
                        *active_tab = ws.default_tab(permissions, active_role);
                    }
                });

            ui.add_space(3.0);
        }
    });
}
