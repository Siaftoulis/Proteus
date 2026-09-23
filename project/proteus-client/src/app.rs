use crate::views::analyst_studio::{render_analyst_studio, AnalystStudioState};
use crate::views::appointments::{draw_appointments_view, AppointmentsViewState};
use crate::views::audit_log::{draw_audit_log_view, AuditLogViewState};
use crate::views::dashboards::{draw_specialist_dashboards_view, SpecialistDashboardState};
use crate::views::developer::{draw_developer_studio_view, DeveloperStudioState};
use crate::views::enterprise_hq::{draw_enterprise_hq_view, EnterpriseHqState};
use crate::views::intake::{draw_intake_view, IntakeFormState};
use crate::views::navigation::{available_workspaces, NavTab, RoleWorkspace};
use crate::views::pipeline::draw_pipeline_view;
use crate::views::settings::{draw_settings_view, SettingsViewState};
use crate::views::store_director::{draw_store_director_view, StoreDirectorState};
use crate::views::support::{draw_support_view, SupportViewState};
use crate::views::ticket_detail::{draw_ticket_detail_modal, TicketDetailState};
use crate::views::top_bar::{render_sub_bar, render_top_bar, TopBarState};
use crm_core::audit::init_audit_schema;
use crm_core::enterprise::{init_enterprise_schema, seed_default_enterprise_if_empty};
use crm_core::paths::{ensure_database_dir_exists, get_database_path};
use crm_core::printer::ShopReceiptConfig;
use crm_core::roles::UserRole;
use egui::{Frame, Margin};
use rusqlite::Connection;

pub struct ProteusClientApp {
    conn: Connection,
    active_workspace: RoleWorkspace,
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
    enterprise_hq_state: EnterpriseHqState,
    store_director_state: StoreDirectorState,
    lan_receiver: Option<crate::lan_receiver::LanPackageReceiver>,
    lan_beacon: Option<crm_core::lan::LanDiscoveryDaemon>,
    replication_daemon: Option<crate::replication_daemon::ReplicationDaemon>,
    device_label_ref: std::sync::Arc<std::sync::Mutex<String>>,
    pending_migration: Option<crate::views::schema_diff_modal::PendingMigrationReview>,
    pub auth_state: crate::views::auth::ClientAuthState,
    pub mounted_bundle_title: String,
}

impl ProteusClientApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let db_path = get_database_path();
        let _ = ensure_database_dir_exists(&db_path);

        let conn = Connection::open(&db_path).unwrap_or_else(|_| {
            Connection::open_in_memory().expect("Critical: Failed to open SQLite")
        });

        let _ = crm_core::apply_storage_tuning(&conn);
        let _ = crm_core::tickets::init_tickets_schema(&conn);
        let _ = init_audit_schema(&conn);
        let _ = crm_core::merkle::init_merkle_schema(&conn);
        let _ = init_enterprise_schema(&conn);
        let _ = seed_default_enterprise_if_empty(&conn);
        let _ = crm_core::replication::init_outbox_schema(&conn);
        crate::views::audit_log::seed_initial_audit_events_if_empty(&conn);

        Self {
            conn,
            active_workspace: RoleWorkspace::ShopCounter,
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
            enterprise_hq_state: EnterpriseHqState::default(),
            store_director_state: StoreDirectorState::default(),
            lan_receiver: crate::lan_receiver::LanPackageReceiver::start(7443).ok(),
            lan_beacon: {
                let init_label = format!("Proteus Terminal ({})", UserRole::Ceo.display_name());
                let label_ref = std::sync::Arc::new(std::sync::Mutex::new(init_label));
                let node_id = format!(
                    "client-{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0)
                );
                crm_core::lan::LanDiscoveryDaemon::start(
                    node_id,
                    "ProteusClient".to_string(),
                    label_ref.clone(),
                    7443,
                    0,
                    true,
                )
                .ok()
            },
            replication_daemon: crate::replication_daemon::ReplicationDaemon::start(db_path.clone(), 5000).ok(),
            device_label_ref: std::sync::Arc::new(std::sync::Mutex::new(format!(
                "Proteus Terminal ({})",
                UserRole::Ceo.display_name()
            ))),
            pending_migration: None,
            auth_state: crate::views::auth::ClientAuthState::default(),
            mounted_bundle_title: "Automotive Service & Repair BOS".to_string(),
        }
    }

    fn poll_lan_packages(&mut self) {
        if let Some(ref rx) = self.lan_receiver {
            while let Some(pkg) = rx.try_recv() {
                if !pkg.schema.ddl_statements.is_empty() {
                    match crate::views::schema_diff_modal::PendingMigrationReview::from_package(&self.conn, pkg) {
                        Ok(rev) => self.pending_migration = Some(rev),
                        Err(e) => {
                            self.settings_state.package_mount_msg =
                                Some((format!("❌ Σφάλμα ανάλυσης σχήματος: {}", e), false));
                        }
                    }
                } else {
                    let db_path = get_database_path();
                    match pkg.mount(&mut self.conn, Some(&db_path), "Designer-LAN") {
                        Ok(summary) => {
                            self.settings_state.package_mount_msg = Some((
                                format!(
                                    "✓ Νέο πακέτο παραδόθηκε από τον Designer: '{}' ({} Views)",
                                    summary.package_name, summary.loaded_views_count
                                ),
                                true,
                            ));
                        }
                        Err(e) => {
                            self.settings_state.package_mount_msg =
                                Some((format!("❌ Σφάλμα εγκατάστασης πακέτου: {}", e), false));
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for ProteusClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_lan_packages();
        crate::theme::apply_theme(ctx);

        // Two-Stage Authentication Modal
        if let Some(session) = crate::views::auth::render_auth_modal(&mut self.auth_state, ctx) {
            self.active_role = session.role;
            self.operator_name = session.operator_name;
            self.mounted_bundle_title = session.mounted_bundle_title;
            if let Ok(mut label) = self.device_label_ref.lock() {
                *label = format!("Proteus Terminal ({})", session.role.display_name());
            }
        }

        if !matches!(self.auth_state.stage, crate::views::auth::AuthStage::Authenticated(_)) {
            return;
        }

        // RBAC Permissions & Dynamic Workspace / Tab Filtering
        let permissions = self.active_role.permissions();
        let workspaces = available_workspaces(self.active_role);

        // Auto-switch to first available workspace if current becomes unauthorized
        if !workspaces.contains(&self.active_workspace) {
            if let Some(first_ws) = workspaces.first() {
                self.active_workspace = *first_ws;
                self.active_tab = self.active_workspace.default_tab(&permissions, self.active_role);
            }
        }

        // Auto-switch to first available tab within the active workspace
        let current_sub_tabs = self.active_workspace.sub_tabs(&permissions, self.active_role);
        if !current_sub_tabs.iter().any(|(t, _)| *t == self.active_tab) {
            self.active_tab = self.active_workspace.default_tab(&permissions, self.active_role);
        }

        // Top Navigation Bar
        let mut top_bar_state = TopBarState {
            mounted_bundle_title: &self.mounted_bundle_title,
            active_role: &mut self.active_role,
            operator_name: &mut self.operator_name,
            device_label_ref: &self.device_label_ref,
            active_workspace: &mut self.active_workspace,
            active_tab: &mut self.active_tab,
            workspaces: &workspaces,
            permissions: &permissions,
            auth_state: &mut self.auth_state,
            conn: &self.conn,
            lan_beacon_active: self.lan_beacon.is_some(),
            replication_synced: self.replication_daemon.as_ref().map(|d| d.total_pushed()),
        };
        render_top_bar(ctx, &mut top_bar_state);

        // Contextual Sub-Navigation Bar
        render_sub_bar(
            ctx,
            &current_sub_tabs,
            &mut self.active_tab,
            self.active_workspace,
            &self.operator_name,
            &self.conn,
        );

        // Central View Area
        egui::CentralPanel::default()
            .frame(Frame::new().fill(crate::theme::BG_BASE).inner_margin(Margin::same(16)))
            .show(ctx, |ui| match self.active_tab {
                NavTab::Intake => draw_intake_view(
                    ui,
                    &self.conn,
                    &mut self.intake_state,
                    &self.receipt_config,
                    &self.settings_state.printer_name,
                ),
                NavTab::Pipeline => {
                    draw_pipeline_view(ui, &self.conn, &mut self.pipeline_search, &mut self.selected_ticket_id)
                }
                NavTab::Appointments => draw_appointments_view(
                    ui,
                    &self.conn,
                    &mut self.appointments_state,
                    &self.operator_name,
                    self.active_role.short_code(),
                ),
                NavTab::AuditLog => draw_audit_log_view(ui, &self.conn, &mut self.audit_state),
                NavTab::Settings => {
                    draw_settings_view(ui, &mut self.conn, &mut self.receipt_config, &mut self.settings_state)
                }
                NavTab::Support => {
                    let discovered = self.lan_beacon.as_ref().map(|b| b.get_live_peers()).unwrap_or_default();
                    draw_support_view(
                        ui,
                        &self.conn,
                        &mut self.support_state,
                        &self.settings_state.printer_name,
                        &discovered,
                    );
                }
                NavTab::Specialist => draw_specialist_dashboards_view(ui, &mut self.specialist_state),
                NavTab::Developer => draw_developer_studio_view(ui, &mut self.conn, &mut self.developer_state),
                NavTab::AnalystStudio => render_analyst_studio(ui, &mut self.analyst_state),
                NavTab::EnterpriseHQ => draw_enterprise_hq_view(ui, &self.conn, &mut self.enterprise_hq_state),
                NavTab::StoreDirector => draw_store_director_view(ui, &self.conn, &mut self.store_director_state),
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
