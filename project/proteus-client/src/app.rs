use crate::views::analyst_studio::{render_analyst_studio, AnalystStudioState};
use crate::views::appointments::{draw_appointments_view, AppointmentsViewState};
use crate::views::audit_log::{draw_audit_log_view, AuditLogViewState};
use crate::views::dashboards::{draw_specialist_dashboards_view, SpecialistDashboardState};
use crate::views::developer::{draw_developer_studio_view, DeveloperStudioState};
use crate::views::intake::{draw_intake_view, IntakeFormState};
use crate::views::pipeline::draw_pipeline_view;
use crate::views::settings::{draw_settings_view, SettingsViewState};
use crate::views::support::{draw_support_view, SupportViewState};
use crate::views::ticket_detail::{draw_ticket_detail_modal, TicketDetailState};
use crm_core::audit::init_audit_schema;
use crm_core::paths::{ensure_database_dir_exists, get_database_path};
use crm_core::printer::ShopReceiptConfig;
use crm_core::roles::UserRole;
use crm_core::tickets::{init_tickets_schema, list_tickets};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use rusqlite::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Intake,
    Pipeline,
    Appointments,
    AuditLog,
    Settings,
    Support,
    Specialist,
    Developer,
    AnalystStudio,
}

pub struct ProteusClientApp {
    conn: Connection,
    active_tab: NavTab,
    active_role: UserRole,
    operator_name: String,
    intake_state: IntakeFormState,
    pipeline_search: String,
    selected_ticket_id: Option<String>,
    ticket_detail_state: TicketDetailState,
    receipt_config: ShopReceiptConfig,
    settings_state: SettingsViewState,
    support_state: SupportViewState,
    specialist_state: SpecialistDashboardState,
    developer_state: DeveloperStudioState,
    audit_state: AuditLogViewState,
    appointments_state: AppointmentsViewState,
    analyst_state: AnalystStudioState,
    lan_receiver: Option<crate::lan_receiver::LanPackageReceiver>,
    lan_beacon: Option<crm_core::lan::LanDiscoveryDaemon>,
    device_label_ref: std::sync::Arc<std::sync::Mutex<String>>,
    pending_migration: Option<crate::views::schema_diff_modal::PendingMigrationReview>,
}

impl ProteusClientApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize SQLite at standard path
        let db_path = get_database_path();
        let _ = ensure_database_dir_exists(&db_path);

        let conn = Connection::open(&db_path).unwrap_or_else(|_| {
            // In-memory fallback if disk error
            Connection::open_in_memory().expect("Critical: Failed to open SQLite")
        });

        let _ = init_tickets_schema(&conn);
        let _ = init_audit_schema(&conn);
        crate::views::audit_log::seed_initial_audit_events_if_empty(&conn);

        Self {
            conn,
            active_tab: NavTab::Intake,
            active_role: UserRole::Ceo,
            operator_name: "Admin (CEO)".to_string(),
            intake_state: IntakeFormState::default(),
            pipeline_search: String::new(),
            selected_ticket_id: None,
            ticket_detail_state: TicketDetailState::default(),
            receipt_config: ShopReceiptConfig::default(),
            settings_state: SettingsViewState::default(),
            support_state: SupportViewState::default(),
            specialist_state: SpecialistDashboardState::default(),
            developer_state: DeveloperStudioState::default(),
            audit_state: AuditLogViewState::default(),
            appointments_state: AppointmentsViewState::default(),
            analyst_state: AnalystStudioState::default(),
            lan_receiver: crate::lan_receiver::LanPackageReceiver::start(7443).ok(),
            lan_beacon: {
                let init_label = format!("Proteus Terminal ({})", UserRole::Ceo.display_name());
                let label_ref = std::sync::Arc::new(std::sync::Mutex::new(init_label));
                let node_id = format!("client-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0));
                crm_core::lan::LanDiscoveryDaemon::start(
                    node_id,
                    "ProteusClient".to_string(),
                    label_ref.clone(),
                    7443,
                    0,
                    true,
                ).ok()
            },
            device_label_ref: std::sync::Arc::new(std::sync::Mutex::new(format!("Proteus Terminal ({})", UserRole::Ceo.display_name()))),
            pending_migration: None,
        }
    }
}

impl eframe::App for ProteusClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for LAN packages pushed from Proteus Designer
        if let Some(ref rx) = self.lan_receiver {
            while let Some(pkg) = rx.try_recv() {
                if !pkg.schema.ddl_statements.is_empty() {
                    match crate::views::schema_diff_modal::PendingMigrationReview::from_package(&self.conn, pkg) {
                        Ok(rev) => {
                            self.pending_migration = Some(rev);
                        }
                        Err(e) => {
                            self.settings_state.package_mount_msg = Some((format!("❌ Σφάλμα ανάλυσης σχήματος: {}", e), false));
                        }
                    }
                } else {
                    let db_path = get_database_path();
                    match pkg.mount(&mut self.conn, Some(&db_path), "Designer-LAN") {
                        Ok(summary) => {
                            self.settings_state.package_mount_msg = Some((
                                format!("✓ Νέο πακέτο παραδόθηκε από τον Designer: '{}' ({} Views)", summary.package_name, summary.loaded_views_count),
                                true,
                            ));
                        }
                        Err(e) => {
                            self.settings_state.package_mount_msg = Some((format!("❌ Σφάλμα εγκατάστασης πακέτου: {}", e), false));
                        }
                    }
                }
            }
        }

        crate::theme::apply_theme(ctx);

        // RBAC Permissions & Dynamic Tab Filtering
        let permissions = self.active_role.permissions();
        let mut available_tabs = Vec::new();

        if permissions.can_intake_tickets {
            available_tabs.push((NavTab::Intake, "⚡ Νέα Παραλαβή"));
        }
        if permissions.can_manage_pipeline {
            available_tabs.push((NavTab::Pipeline, "📋 Ροή Επισκευών"));
        }
        if permissions.can_book_appointments {
            available_tabs.push((NavTab::Appointments, "📅 Ραντεβού"));
        }
        if permissions.can_view_audit_trail {
            available_tabs.push((NavTab::AuditLog, "📜 Audit Log"));
        }
        if self.active_role == UserRole::Ceo || self.active_role == UserRole::Technician {
            available_tabs.push((NavTab::Support, "🛠 IT Support"));
        }
        if self.active_role == UserRole::Ceo || self.active_role == UserRole::SalesConsultant {
            available_tabs.push((NavTab::Specialist, "📊 Ειδικά Dashboards"));
        }
        if permissions.can_edit_schema {
            available_tabs.push((NavTab::Developer, "💻 Dev & Schema"));
        }
        if permissions.can_infer_schemas || permissions.can_define_business_rules {
            available_tabs.push((NavTab::AnalystStudio, "📊 PCDA Studio"));
        }
        if permissions.can_manage_settings {
            available_tabs.push((NavTab::Settings, "⚙ Ρυθμίσεις"));
        }

        // Auto-switch to first available tab if current becomes hidden
        if !available_tabs.iter().any(|(t, _)| *t == self.active_tab) {
            if let Some((first_tab, _)) = available_tabs.first() {
                self.active_tab = *first_tab;
            }
        }

        // Top Navigation Bar
        egui::TopBottomPanel::top("top_nav_bar")
            .frame(
                Frame::new()
                    .fill(crate::theme::BG_PANEL)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(14, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo & Brand
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("PROTEUS").strong().size(18.0).color(crate::theme::ACCENT_CYAN));
                        ui.label(RichText::new("BOS").size(14.0).color(crate::theme::TEXT_MUTED));
                    });

                    ui.add_space(10.0);

                    // Active Role Dropdown Selector
                    egui::ComboBox::from_id_salt("active_role_selector")
                        .selected_text(RichText::new(self.active_role.display_name()).strong().size(12.0).color(Color32::WHITE))
                        .show_ui(ui, |ui| {
                            for role in UserRole::all() {
                                let is_selected = self.active_role == *role;
                                if ui.selectable_label(is_selected, role.display_name()).clicked() {
                                    self.active_role = *role;
                                    self.operator_name = match *role {
                                        UserRole::Ceo => "Admin (CEO)".to_string(),
                                        UserRole::CustomerService => "Μαρία (Reception)".to_string(),
                                        UserRole::Technician => "Νίκος (Τεχνικός)".to_string(),
                                        UserRole::SalesConsultant => "Κώστας (Sales)".to_string(),
                                        UserRole::Developer => "Αλέξανδρος (Dev)".to_string(),
                                        UserRole::BusinessAnalyst => "Δημήτρης (Analyst)".to_string(),
                                    };
                                    if let Ok(mut label) = self.device_label_ref.lock() {
                                        *label = format!("Proteus Terminal ({})", role.display_name());
                                    }
                                }
                            }
                        });

                    ui.add_space(12.0);

                    // Dynamic Nav Tabs according to active role
                    for (tab, label) in &available_tabs {
                        let is_active = self.active_tab == *tab;
                        let btn = if is_active {
                            egui::Button::new(RichText::new(*label).strong().size(12.0).color(Color32::WHITE))
                                .fill(crate::theme::ACCENT_PRIMARY)
                        } else {
                            egui::Button::new(RichText::new(*label).size(12.0).color(crate::theme::TEXT_SECONDARY))
                                .fill(crate::theme::BG_CARD)
                        };

                        if ui.add(btn).clicked() {
                            self.active_tab = *tab;
                        }
                        ui.add_space(2.0);
                    }

                    // Right Side: Active count badge & offline pill
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Offline-first pill
                        Frame::new()
                            .fill(Color32::from_rgb(16, 50, 35))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(RichText::new("● 100% Τοπική Λειτουργία").size(11.0).color(Color32::from_rgb(52, 211, 153)));
                            });

                        if self.lan_beacon.is_some() {
                            ui.label(RichText::new("📡 LAN Ready").size(11.0).color(Color32::from_rgb(56, 189, 248)));
                        }

                        let total_tickets = list_tickets(&self.conn).map(|t| t.len()).unwrap_or(0);
                        ui.label(RichText::new(format!("Σύνολο: {} Επισκευές", total_tickets)).size(12.0).color(crate::theme::TEXT_MUTED));
                    });
                });
            });

        // Central View Area
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(crate::theme::BG_BASE)
                    .inner_margin(Margin::same(16)),
            )
            .show(ctx, |ui| {
                match self.active_tab {
                    NavTab::Intake => {
                        draw_intake_view(
                            ui,
                            &self.conn,
                            &mut self.intake_state,
                            &self.receipt_config,
                            &self.settings_state.printer_name,
                        );
                    }
                    NavTab::Pipeline => {
                        draw_pipeline_view(
                            ui,
                            &self.conn,
                            &mut self.pipeline_search,
                            &mut self.selected_ticket_id,
                        );
                    }
                    NavTab::Appointments => {
                        draw_appointments_view(
                            ui,
                            &self.conn,
                            &mut self.appointments_state,
                            &self.operator_name,
                            self.active_role.short_code(),
                        );
                    }
                    NavTab::AuditLog => {
                        draw_audit_log_view(
                            ui,
                            &self.conn,
                            &mut self.audit_state,
                        );
                    }
                    NavTab::Settings => {
                        draw_settings_view(
                            ui,
                            &mut self.conn,
                            &mut self.receipt_config,
                            &mut self.settings_state,
                        );
                    }
                    NavTab::Support => {
                        draw_support_view(
                            ui,
                            &self.conn,
                            &mut self.support_state,
                            &self.settings_state.printer_name,
                        );
                    }
                    NavTab::Specialist => {
                        draw_specialist_dashboards_view(
                            ui,
                            &mut self.specialist_state,
                        );
                    }
                    NavTab::Developer => {
                        draw_developer_studio_view(
                            ui,
                            &mut self.conn,
                            &mut self.developer_state,
                        );
                    }
                    NavTab::AnalystStudio => {
                        render_analyst_studio(ui, &mut self.analyst_state);
                    }
                }
            });

        // Ticket Detail Modal (if a card is clicked)
        draw_ticket_detail_modal(
            ctx,
            &self.conn,
            &mut self.selected_ticket_id,
            &mut self.ticket_detail_state,
            &self.receipt_config,
            &self.settings_state.printer_name,
        );

        // Visual Schema Diff & Migration Confirmation Modal
        crate::views::schema_diff_modal::draw_schema_diff_modal(
            ctx,
            &mut self.conn,
            &mut self.pending_migration,
            &mut self.settings_state.package_mount_msg,
        );
    }
}


