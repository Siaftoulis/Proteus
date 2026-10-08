//! Top Navigation and Contextual Sub-Navigation Bars for Proteus Client.
//! Renders the brand, role switcher, workspace pills, system connectivity, and active tools.

use crate::views::auth::ClientAuthState;
use crate::views::navigation::{NavTab, RoleWorkspace};
use proteus_core::roles::{RolePermissions, UserRole};
use proteus_core::tickets::list_tickets;
use egui::{Align2, Color32, CornerRadius, FontId, Frame, Margin, Pos2, Rect, RichText, Shape, Stroke, Ui};
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
    pub shop_name: &'a str,
    pub logo_icon: &'a str,
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
                // Logo & Store Brand
                ui.horizontal(|ui| {
                    crate::views::logo::render_store_logo_widget(
                        ui,
                        egui::vec2(22.0, 22.0),
                        state.logo_icon,
                        state.shop_name,
                        true,
                    );
                    ui.add_space(6.0);
                    ui.label(RichText::new(state.shop_name).strong().size(14.0).color(crate::theme::TEXT_PRIMARY));
                    ui.label(RichText::new("BOS").size(11.0).color(crate::theme::TEXT_MUTED));
                });

                ui.add_space(6.0);

                // Mounted PR Package Badge
                Frame::new()
                    .fill(crate::theme::BADGE_MOUNTED_BG)
                    .stroke(Stroke::new(1.0, crate::theme::BADGE_MOUNTED_BORDER))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(7, 3))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(state.mounted_bundle_title)
                                .size(11.0)
                                .color(crate::theme::BADGE_MOUNTED_TEXT),
                        );
                    });

                ui.add_space(8.0);

                // Active Role Dropdown Selector
                egui::ComboBox::from_id_salt("active_role_selector")
                    .selected_text(
                        RichText::new(state.active_role.display_name())
                            .strong()
                            .size(12.0)
                            .color(crate::theme::TEXT_PRIMARY),
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
                                    UserRole::BusinessAnalyst => "Δημήτρης (BA)".to_string(),
                                    UserRole::DataAnalyst => "Ελένη (Data)".to_string(),
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
                        .button(RichText::new("Έξοδος").size(11.0).color(crate::theme::BTN_TOP_TEXT))
                        .on_hover_text("Αποσύνδεση από λογαριασμό Marketplace")
                        .clicked()
                    {
                        state.auth_state.logout();
                    }
                    if ui
                        .button(RichText::new("Αλλαγή CRM").size(11.0).color(crate::theme::BTN_TOP_TEXT))
                        .on_hover_text("Επιλογή άλλου CRM project από τον Launcher")
                        .clicked()
                    {
                        state.auth_state.switch_project();
                    }
                    if ui
                        .button(RichText::new("Κλείδωμα").size(11.0).color(crate::theme::BTN_TOP_TEXT))
                        .on_hover_text("Κλείδωμα τερματικού (Απαιτεί Master / Staff PIN)")
                        .clicked()
                    {
                        state.auth_state.lock_terminal();
                    }

                    // Offline-first pill (PDS unified tokens)
                    Frame::new()
                        .fill(crate::theme::LOCAL_STATUS_BG)
                        .stroke(Stroke::new(1.0, crate::theme::LOCAL_STATUS_BORDER))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(7, 3))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new("● 100% Τοπική Λειτουργία")
                                    .size(11.0)
                                    .color(crate::theme::LOCAL_STATUS_TEXT),
                            );
                        });

                    let pending_sync = proteus_core::replication::count_pending_outbox(state.conn).unwrap_or(0);
                    if pending_sync > 0 {
                        Frame::new()
                            .fill(crate::theme::LOCAL_STATUS_BG)
                            .stroke(Stroke::new(1.0, crate::theme::BORDER_STRONG))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(7, 3))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("Sync: {} Εκκρεμή", pending_sync))
                                        .size(11.0)
                                        .color(crate::theme::TEXT_PRIMARY),
                                );
                            });
                    }

                    if let Ok(Some(rec)) = proteus_core::security::TimeIntegrityGuard::get_record(state.conn) {
                        if rec.tamper_locked {
                            ui.label(RichText::new("⚠️ TIME LOCK").size(11.0).color(Color32::from_rgb(244, 63, 94)).strong())
                                .on_hover_text("Ανιχνεύθηκε rollback ρολογιού! Το σύστημα είναι σε Read-Only mode.");
                        }
                    }

                    if state.lan_beacon_active {
                        ui.label(RichText::new("LAN Ready").size(11.0).color(crate::theme::TEXT_SECONDARY));
                    }

                    if let Some(pushed) = state.replication_synced {
                        ui.label(
                            RichText::new(format!("Sync ({})", pushed))
                                .size(11.0)
                                .color(crate::theme::TEXT_SECONDARY),
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
                .inner_margin(Margin::symmetric(14, 5)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let bar_id = ui.make_persistent_id(format!("sub_bar_{}", active_workspace.short_code()));
                let font_id = FontId::proportional(11.5);

                let mut clicked_tab: Option<NavTab> = None;

                Frame::new()
                    .fill(crate::theme::TAB_CONTAINER_BG)
                    .stroke(Stroke::new(1.0, crate::theme::TAB_CONTAINER_STROKE))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(3, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;

                            let bg_shape_idx = ui.painter().add(Shape::Noop);
                            let mut target_rect: Option<Rect> = None;

                            for (tab, label) in sub_tabs {
                                let is_active = *tab == *active_tab;

                                let text_width = ui.painter()
                                    .layout_no_wrap((*label).to_string(), font_id.clone(), Color32::WHITE)
                                    .size()
                                    .x;
                                let item_size = egui::vec2(text_width + 18.0, 24.0);

                                let (rect, resp) = ui.allocate_exact_size(item_size, egui::Sense::click());

                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                if is_active {
                                    target_rect = Some(rect);
                                }

                                if resp.clicked() && !is_active {
                                    clicked_tab = Some(*tab);
                                }

                                let text_color = if is_active {
                                    crate::theme::TAB_ACTIVE_TEXT
                                } else if resp.hovered() {
                                    crate::theme::TAB_HOVER_TEXT
                                } else {
                                    crate::theme::TAB_INACTIVE_TEXT
                                };

                                ui.painter().text(
                                    rect.center(),
                                    Align2::CENTER_CENTER,
                                    *label,
                                    font_id.clone(),
                                    text_color,
                                );
                            }

                            if let Some(target) = target_rect {
                                let anim_min_x = ui.ctx().animate_value_with_time(bar_id.with("min_x"), target.min.x, 0.18);
                                let anim_max_x = ui.ctx().animate_value_with_time(bar_id.with("max_x"), target.max.x, 0.18);
                                let anim_min_y = ui.ctx().animate_value_with_time(bar_id.with("min_y"), target.min.y, 0.18);
                                let anim_max_y = ui.ctx().animate_value_with_time(bar_id.with("max_y"), target.max.y, 0.18);

                                let anim_rect = Rect::from_min_max(
                                    Pos2::new(anim_min_x, anim_min_y),
                                    Pos2::new(anim_max_x, anim_max_y),
                                );

                                ui.painter().set(
                                    bg_shape_idx,
                                    Shape::rect_filled(
                                        anim_rect,
                                        CornerRadius::same(5),
                                        crate::theme::TAB_ACTIVE_FILL,
                                    ),
                                );
                            }
                        });
                    });

                if let Some(t) = clicked_tab {
                    *active_tab = t;
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
    let bar_id = ui.make_persistent_id("affinity_client_workspaces_bar");
    let font_id = FontId::proportional(11.5);
    let pill_fill = crate::theme::TAB_ACTIVE_FILL;
    let text_active = crate::theme::TAB_ACTIVE_TEXT;
    let text_muted = crate::theme::TAB_INACTIVE_TEXT;
    let text_hover = crate::theme::TAB_HOVER_TEXT;

    let mut next_ws: Option<RoleWorkspace> = None;

    Frame::new()
        .fill(crate::theme::TAB_CONTAINER_BG)
        .stroke(Stroke::new(1.0, crate::theme::TAB_CONTAINER_STROKE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(3, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;

                let bg_shape_idx = ui.painter().add(Shape::Noop);
                let mut target_rect: Option<Rect> = None;

                for ws in workspaces {
                    let is_active = *ws == *active_workspace;
                    let label = ws.display_label();

                    let text_width = ui.painter()
                        .layout_no_wrap(label.to_string(), font_id.clone(), Color32::WHITE)
                        .size()
                        .x;
                    let item_size = egui::vec2(text_width + 18.0, 26.0);

                    let (rect, resp) = ui.allocate_exact_size(item_size, egui::Sense::click());

                    if resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }

                    if is_active {
                        target_rect = Some(rect);
                    }

                    if resp.clicked() && !is_active {
                        next_ws = Some(*ws);
                    }

                    let text_color = if is_active { text_active } else if resp.hovered() { text_hover } else { text_muted };
                    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font_id.clone(), text_color);
                }

                if let Some(target) = target_rect {
                    let anim_min_x = ui.ctx().animate_value_with_time(bar_id.with("min_x"), target.min.x, 0.18);
                    let anim_max_x = ui.ctx().animate_value_with_time(bar_id.with("max_x"), target.max.x, 0.18);
                    let anim_min_y = ui.ctx().animate_value_with_time(bar_id.with("min_y"), target.min.y, 0.18);
                    let anim_max_y = ui.ctx().animate_value_with_time(bar_id.with("max_y"), target.max.y, 0.18);
                    let anim_rect = Rect::from_min_max(Pos2::new(anim_min_x, anim_min_y), Pos2::new(anim_max_x, anim_max_y));
                    ui.painter().set(bg_shape_idx, Shape::rect_filled(anim_rect, CornerRadius::same(5), pill_fill));
                }
            });
        });

    if let Some(ws) = next_ws {
        *active_workspace = ws;
        *active_tab = ws.default_tab(permissions, active_role);
    }
}
