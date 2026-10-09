//! ProteusApp application state, default initialization, entity mutations, and persistence engine.

use chrono::Utc;
use proteus_core::Database;
use eframe::egui::{self, Color32};
use std::sync::{Arc, Mutex};
use crate::models::*;
use crate::scene;
use crate::views;
use crate::db;

pub struct ProteusApp {
    pub db: Option<Arc<Mutex<Database>>>,
    pub pid: String,
    pub pname: String,
    pub mode: Mode,
    pub layout: LayoutMode,
    pub project_doc: scene::ProjectDocument,
    pub editor_state: scene::EditorState,
    pub palette_drag: Option<scene::NodeType>,
    pub viewport: Viewport2D,
    pub show_login: bool,
    pub auth: Option<String>,
    pub toast: Option<String>,
    pub tt: f32,
    pub loaded: bool,
    pub _style_set: bool,
    pub db_conn: Option<rusqlite::Connection>,
    pub form_state: std::collections::HashMap<String, String>,
    pub viewer_selected_entity: String,
    pub viewer_entity_input: String,
    pub viewer_records: Vec<(String, serde_json::Value)>,
    pub table_cache: std::collections::HashMap<String, Vec<(String, serde_json::Value)>>,
    pub editing_record: Option<(String, serde_json::Value)>,
    pub flow_viewport: Viewport2D,
    pub selected_flow_node: Option<String>,
    pub flow_drag_active: bool,
    pub flow_wiring_source: Option<String>,
    pub flow_wiring_pos: Option<(f32, f32)>,
    pub flow_canvas_center: (f32, f32),
    pub active_play_page: Option<String>,
    pub play_viewport: Viewport2D,
    pub designer_selected_node: Option<String>,
    pub designer_drag_active: bool,
    pub designer_drag_offset: (f32, f32),
    pub spawn_counter: u32,
    pub device_preset: DevicePreset,
    pub show_device_toolbar: bool,
    pub viewport_profile: crate::viewport::ViewportProfile,
    pub virtual_keyboard_height: f32,
    pub active_tool: crate::models::DesignerTool,
    pub designer_left_tab: crate::models::DesignerLeftTab,
    pub inspector_tab: crate::models::InspectorTab,
    pub copied_dimensions: Option<(f32, f32)>,
    pub marquee_start: Option<egui::Pos2>,
    pub draw_start: Option<egui::Pos2>,
    pub vrr_mode: crate::models::VrrMode,
    pub theme_mode: crate::theme::ThemeMode,
    pub accent_preset: crate::theme::AccentPreset,

    // Contacts & Pipeline
    pub contacts: Vec<Contact>,
    pub sel_contact: Option<String>,
    pub search_query: String,
    pub new_note_text: String,
    pub deals: Vec<Deal>,
    pub sel_deal: Option<String>,
    pub editing_contact: Option<String>,
    pub edit_name: String,
    pub edit_email: String,
    pub edit_phone: String,
    pub edit_company: String,
    pub edit_tags: String,

    // Studio state
    pub studio_layers: Vec<StudioLayer>,
    pub studio_sel_layer: Option<usize>,
    pub studio_tool: StudioTool,
    pub studio_color: Color32,
    pub studio_color2: Color32,
    pub studio_brush_size: f32,
    pub studio_dragging: Option<(usize, egui::Pos2, egui::Pos2)>,
    pub studio_shape_start: Option<egui::Pos2>,
    pub studio_shape_current: Option<egui::Pos2>,
    pub studio_show_text_dialog: bool,
    pub studio_text_input: String,
    pub studio_show_color_popup: bool,
    pub studio_color_picker_target: u8,
    pub _studio_show_tool_options: bool,

    // Tasks state
    pub tasks: Vec<Task>,
    pub sel_task: Option<String>,
    pub task_filter_status: Option<TaskStatus>,
    pub task_filter_priority: Option<TaskPriority>,

    // LAN Autonomous Discovery & Dispatch
    pub lan_daemon: Option<proteus_core::lan::LanDiscoveryDaemon>,
    pub auto_deploy_on_save: bool,

    // Role Workspaces
    pub analyst_state: views::analyst::AnalystState,
    pub networking_state: views::networking::NetworkingState,
    pub troubleshoot_state: views::troubleshoot::TroubleshootState,
    pub connected_data_state: views::connected_data::ConnectedDataState,
    pub settings_state: views::settings::SettingsState,
    pub flasher_state: views::appliance_flasher::ApplianceFlasherState,
    pub i18n_state: views::i18n_manager::I18nManagerState,

    // Command Palette (Ctrl+K)
    pub show_command_palette: bool,
    pub command_palette_query: String,
    pub command_palette_sel_idx: usize,

    // Welcome Hub / Launcher (Affinity-style)
    pub in_welcome_hub: bool,
    pub welcome_tab: crate::models::WelcomeTab,
    pub welcome_user_name: String,
    pub welcome_user_role: String,
    pub welcome_selected_preset: usize,
    pub welcome_selected_project: usize,
    pub welcome_selected_occupation: usize,

    // Bespoke Brief Ingestion & Scaffold
    pub show_brief_modal: bool,
    pub cached_briefs: Vec<proteus_core::brief::ClientProjectBrief>,
    pub selected_brief_idx: usize,

    // ESC/POS Hardware & Spooler
    pub hardware_printer_name: String,
    pub hardware_paper_58mm: bool,

    // Smart Snapping & Guides ("Aimbot")
    pub active_snap_guides: Vec<views::designer::SnapGuideKind>,
    pub smart_snap_enabled: bool,

    // Collaborative Studio Multiplayer
    pub collab_daemon: Option<proteus_core::lan::LanCollaborationDaemon>,
    pub live_collaborators: Vec<proteus_core::lan::PeerPresencePacket>,
    pub last_cursor_broadcast_epoch: u64,

    // Universal UI Layout Import Modal
    pub show_import_schema_modal: bool,
    pub import_schema_input: String,

    // Sovereign Model Context Protocol (MCP) AI Bridge
    pub ai_bridge: crate::ai_bridge::StudioAiBridge,

    // Quality Linter Pre-Flight Modal & Certification Gate
    pub show_linter_modal: bool,
    pub active_lint_report: Option<proteus_core::package::LintReport>,
    pub candidate_package: Option<proteus_core::package::PrPackage>,

    // In-App Share Point & Sandbox Workspace (Master Problem Audit P18/P20)
    pub show_sharepoint_modal: bool,
    pub sharepoint_tab: usize,
    pub loaded_requirement: Option<proteus_core::package::RequirementsPackage>,
    pub generated_preview: Option<proteus_core::package::PreviewBundle>,
    pub active_review_session: Option<proteus_core::package::ClientReviewSession>,
    pub escrow_contract: Option<proteus_core::package::EscrowContract>,
}

pub fn app_dir() -> String {
    std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".into())
        + "/proteus"
}

impl Default for ProteusApp {
    fn default() -> Self {
        let db_path = app_dir();
        std::fs::create_dir_all(&db_path).ok();
        let db = Database::new(&format!("{}/crm.db", db_path))
            .ok()
            .map(|d| Arc::new(Mutex::new(d)));

        let mut app = Self {
            db,
            pid: "p1".into(),
            pname: "Demo Project".into(),
            mode: Mode::Designer,
            layout: LayoutMode::Free,
            project_doc: scene::ProjectDocument::new(),
            editor_state: scene::EditorState::default(),
            palette_drag: None,
            viewport: Viewport2D::new(),
            show_login: false,
            auth: None,
            toast: None,
            tt: 0.,
            contacts: vec![
                Contact {
                    id: "c1".into(),
                    name: "John Smith".into(),
                    email: "john@acme.com".into(),
                    phone: "+1 555-0100".into(),
                    company: "Acme Corp".into(),
                    tags: vec!["Enterprise".into(), "VIP".into()],
                    notes: vec![ContactNote {
                        id: "n1".into(),
                        text: "Initial call went well, interested in enterprise plan".into(),
                        created_at: Utc::now().to_rfc3339(),
                    }],
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
                Contact {
                    id: "c2".into(),
                    name: "Jane Doe".into(),
                    email: "jane@widgets.co".into(),
                    phone: "+1 555-0200".into(),
                    company: "Widgets Co".into(),
                    tags: vec!["Mid-Market".into()],
                    notes: vec![],
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
                Contact {
                    id: "c3".into(),
                    name: "Bob Wilson".into(),
                    email: "bob@example.com".into(),
                    phone: "+1 555-0300".into(),
                    company: "Example LLC".into(),
                    tags: vec!["SMB".into()],
                    notes: vec![],
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
            ],
            sel_contact: None,
            search_query: String::new(),
            new_note_text: String::new(),
            deals: vec![
                Deal {
                    id: "d1".into(),
                    title: "Acme Enterprise License".into(),
                    value: 24000.,
                    stage: "Proposal".into(),
                    contact_id: Some("c1".into()),
                    expected_close: "2026-08-15".into(),
                    notes: "Waiting on legal review".into(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
                Deal {
                    id: "d2".into(),
                    title: "Widgets Co Partnership".into(),
                    value: 5000.,
                    stage: "Qualified".into(),
                    contact_id: Some("c2".into()),
                    expected_close: "2026-09-01".into(),
                    notes: "".into(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
                Deal {
                    id: "d3".into(),
                    title: "Consulting — Example LLC".into(),
                    value: 3000.,
                    stage: "New".into(),
                    contact_id: Some("c3".into()),
                    expected_close: "2026-07-30".into(),
                    notes: "Initial inquiry".into(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
            ],
            sel_deal: None,
            editing_contact: None,
            edit_name: String::new(),
            edit_email: String::new(),
            edit_phone: String::new(),
            edit_company: String::new(),
            edit_tags: String::new(),
            loaded: false,
            _style_set: false,
            db_conn: {
                let conn = db::init_db(std::path::Path::new("data.db")).ok();
                conn
            },
            form_state: std::collections::HashMap::new(),
            viewer_selected_entity: String::new(),
            viewer_entity_input: String::new(),
            viewer_records: vec![],
            table_cache: {
                let mut map = std::collections::HashMap::new();
                if let Ok(conn) = db::init_db(std::path::Path::new("data.db")) {
                    if let Ok(recs) = db::get_records(&conn, "contacts") {
                        map.insert("contacts".into(), recs);
                    }
                }
                map
            },
            editing_record: None,
            flow_viewport: Viewport2D::new(),
            selected_flow_node: None,
            flow_drag_active: false,
            flow_wiring_source: None,
            flow_wiring_pos: None,
            flow_canvas_center: (100., 100.),
            active_play_page: None,
            play_viewport: Viewport2D::new(),
            designer_selected_node: None,
            designer_drag_active: false,
            designer_drag_offset: (0., 0.),
            spawn_counter: 0,
            device_preset: DevicePreset::Desktop,
            show_device_toolbar: false,
            viewport_profile: crate::viewport::ViewportProfile::default(),
            virtual_keyboard_height: 0.0,
            active_tool: crate::models::DesignerTool::Select,
            designer_left_tab: crate::models::DesignerLeftTab::default(),
            inspector_tab: crate::models::InspectorTab::default(),
            copied_dimensions: None,
            marquee_start: None,
            draw_start: None,
            vrr_mode: crate::models::VrrMode::default(),
            theme_mode: crate::theme::ThemeMode::default(),
            accent_preset: crate::theme::AccentPreset::default(),
            tasks: vec![
                Task {
                    id: "t1".into(),
                    title: "Follow up with John Smith — enterprise pricing".into(),
                    description: "Send updated proposal with 3-year discount".into(),
                    contact_id: Some("c1".into()),
                    deal_id: Some("d1".into()),
                    assignee: "me".into(),
                    due_date: "2026-07-10".into(),
                    status: TaskStatus::InProgress,
                    priority: TaskPriority::High,
                    created_by: "me".into(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                    completed_at: None,
                },
                Task {
                    id: "t2".into(),
                    title: "Demo for Jane Doe — Widgets Co".into(),
                    description: "Schedule Zoom demo, prepare custom deck".into(),
                    contact_id: Some("c2".into()),
                    deal_id: Some("d2".into()),
                    assignee: "me".into(),
                    due_date: "2026-07-12".into(),
                    status: TaskStatus::Open,
                    priority: TaskPriority::Medium,
                    created_by: "me".into(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                    completed_at: None,
                },
                Task {
                    id: "t3".into(),
                    title: "Send contract to Bob Wilson".into(),
                    description: "Draft and send consulting agreement".into(),
                    contact_id: Some("c3".into()),
                    deal_id: Some("d3".into()),
                    assignee: "me".into(),
                    due_date: "2026-07-15".into(),
                    status: TaskStatus::Open,
                    priority: TaskPriority::Low,
                    created_by: "me".into(),
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                    completed_at: None,
                },
            ],
            sel_task: None,
            task_filter_status: None,
            task_filter_priority: None,
            studio_layers: vec![
                StudioLayer {
                    id: "sl1".into(),
                    name: "Background".into(),
                    x: 0., y: 0., w: 800., h: 600., z: 0,
                    visible: true, opacity: 1., locked: false,
                    content: LayerContent::Shape {
                        kind: "rect".into(), x: 0., y: 0., w: 800., h: 600.,
                        fill_r: 30, fill_g: 30, fill_b: 30, fill_a: 255,
                        stroke_r: 0, stroke_g: 0, stroke_b: 0, stroke_a: 0,
                        stroke_width: 0.,
                    },
                },
                StudioLayer {
                    id: "sl2".into(),
                    name: "Circle".into(),
                    x: 340., y: 200., w: 120., h: 120., z: 1,
                    visible: true, opacity: 0.8, locked: false,
                    content: LayerContent::Shape {
                        kind: "ellipse".into(), x: 340., y: 200., w: 120., h: 120.,
                        fill_r: 52, fill_g: 152, fill_b: 219, fill_a: 200,
                        stroke_r: 255, stroke_g: 255, stroke_b: 255, stroke_a: 255,
                        stroke_width: 2.,
                    },
                },
                StudioLayer {
                    id: "sl3".into(),
                    name: "Text".into(),
                    x: 320., y: 360., w: 160., h: 40., z: 2,
                    visible: true, opacity: 1., locked: false,
                    content: LayerContent::Text {
                        content: "Hello World".into(), font_size: 24.,
                        r: 255, g: 255, b: 255, a: 255,
                    },
                },
            ],
            studio_sel_layer: None,
            studio_tool: StudioTool::Select,
            studio_color: Color32::from_rgb(52, 152, 219),
            studio_color2: Color32::from_rgb(255, 255, 255),
            studio_brush_size: 4.,
            studio_dragging: None,
            studio_shape_start: None,
            studio_shape_current: None,
            studio_show_text_dialog: false,
            studio_text_input: String::new(),
            studio_show_color_popup: false,
            studio_color_picker_target: 0,
            _studio_show_tool_options: false,
            lan_daemon: proteus_core::lan::LanDiscoveryDaemon::start(
                "designer-master".to_string(),
                "ProteusDesigner".to_string(),
                std::sync::Arc::new(std::sync::Mutex::new("Proteus Studio Master".to_string())),
                0,
                proteus_core::lan::DEFAULT_BEACON_PORT,
                false,
            ).ok(),
            auto_deploy_on_save: false,
            analyst_state: views::analyst::AnalystState::default(),
            networking_state: views::networking::NetworkingState::default(),
            troubleshoot_state: views::troubleshoot::TroubleshootState::default(),
            connected_data_state: views::connected_data::ConnectedDataState::default(),
            settings_state: views::settings::SettingsState::default(),
            flasher_state: views::appliance_flasher::ApplianceFlasherState::default(),
            i18n_state: views::i18n_manager::I18nManagerState::default(),
            show_command_palette: false,
            command_palette_query: String::new(),
            command_palette_sel_idx: 0,
            in_welcome_hub: true,
            welcome_tab: crate::models::WelcomeTab::Home,
            welcome_user_name: "Panagiotis Siaftoulis".into(),
            welcome_user_role: "UI/UX Designer (PCD)".into(),
            welcome_selected_preset: 1, // Desktop HD
            welcome_selected_project: 0,
            welcome_selected_occupation: 0,
            show_brief_modal: false,
            cached_briefs: Vec::new(),
            selected_brief_idx: 0,
            hardware_printer_name: "POS-80".to_string(),
            hardware_paper_58mm: false,
            active_snap_guides: Vec::new(),
            smart_snap_enabled: true,
            collab_daemon: proteus_core::lan::LanCollaborationDaemon::start(
                format!("designer-{}", uuid::Uuid::new_v4().simple()),
                0,
                proteus_core::lan::DEFAULT_COLLAB_PORT,
            ).ok(),
            live_collaborators: Vec::new(),
            last_cursor_broadcast_epoch: 0,
            show_import_schema_modal: false,
            import_schema_input: String::new(),
            ai_bridge: crate::ai_bridge::StudioAiBridge::new(),
            show_linter_modal: false,
            active_lint_report: None,
            candidate_package: None,
            show_sharepoint_modal: false,
            sharepoint_tab: 0,
            loaded_requirement: None,
            generated_preview: None,
            active_review_session: None,
            escrow_contract: None,
        };
        app.seed_database_if_empty();
        app.reload_table_cache("contacts");
        app.reload_table_cache("deals");
        app
    }
}

impl ProteusApp {
    /// Broadcasts local designer mouse position over LAN to collaborative peers.
    pub fn broadcast_cursor(&mut self, world_pos: (f32, f32)) {
        let now = proteus_core::lan::current_epoch_millis();
        if now.saturating_sub(self.last_cursor_broadcast_epoch) >= 40 {
            self.last_cursor_broadcast_epoch = now;
            if let Some(ref daemon) = self.collab_daemon {
                let pkt = proteus_core::lan::PeerPresencePacket {
                    client_id: daemon.local_client_id().to_string(),
                    user_name: self.welcome_user_name.clone(),
                    user_role: self.welcome_user_role.clone(),
                    color_rgb: [79, 140, 237],
                    cursor_world: world_pos,
                    selected_node_id: self.designer_selected_node.clone(),
                    current_page: None,
                    epoch_millis: now,
                };
                daemon.broadcast_presence(&pkt);
            }
        }
    }

    /// Polls live collaborative peers from LAN daemon and prunes stale peers.
    pub fn poll_collaborators(&mut self) {
        if let Some(ref daemon) = self.collab_daemon {
            self.live_collaborators = daemon.get_live_collaborators(4000);
        }
    }

    pub fn palette(&self, ctx: &egui::Context) -> crate::theme::Palette {
        let is_dark = self.theme_mode.resolve_is_dark(ctx);
        crate::theme::Palette::new(is_dark, self.accent_preset)
    }

    pub fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some(msg.into());
        self.tt = 0.;
    }

    /// Seeds entity schemas and initial contacts/deals into SQLite via proteus-core DataEngine.
    pub fn seed_database_if_empty(&mut self) {
        if let Some(db_arc) = &self.db {
            if let Ok(db) = db_arc.lock() {
                // 1. Schemas
                let contacts_schema = proteus_core::schema::EntitySchema::new("contacts", "Contacts")
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "name".into(),
                        label: "Full Name".into(),
                        field_type: proteus_core::schema::FieldType::Text,
                        required: true,
                        default_value: None,
                    })
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "email".into(),
                        label: "Email Address".into(),
                        field_type: proteus_core::schema::FieldType::Email,
                        required: true,
                        default_value: None,
                    })
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "phone".into(),
                        label: "Phone Number".into(),
                        field_type: proteus_core::schema::FieldType::Phone,
                        required: false,
                        default_value: None,
                    })
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "company".into(),
                        label: "Company".into(),
                        field_type: proteus_core::schema::FieldType::Text,
                        required: false,
                        default_value: None,
                    })
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "status".into(),
                        label: "Status".into(),
                        field_type: proteus_core::schema::FieldType::Select(vec![
                            "Lead".into(), "Contacted".into(), "Qualified".into(), "Customer".into()
                        ]),
                        required: false,
                        default_value: None,
                    });
                let _ = db.save_schema(&self.pid, &contacts_schema);

                let deals_schema = proteus_core::schema::EntitySchema::new("deals", "Deals")
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "title".into(),
                        label: "Deal Title".into(),
                        field_type: proteus_core::schema::FieldType::Text,
                        required: true,
                        default_value: None,
                    })
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "value".into(),
                        label: "Value ($)".into(),
                        field_type: proteus_core::schema::FieldType::Number,
                        required: true,
                        default_value: None,
                    })
                    .with_field(proteus_core::schema::FieldDefinition {
                        name: "stage".into(),
                        label: "Pipeline Stage".into(),
                        field_type: proteus_core::schema::FieldType::Select(vec![
                            "Lead".into(), "Proposal".into(), "Negotiation".into(), "Won".into(), "Lost".into()
                        ]),
                        required: true,
                        default_value: None,
                    });
                let _ = db.save_schema(&self.pid, &deals_schema);

                // 2. Seed contacts if empty
                if db.count_records(&self.pid, "contacts").unwrap_or(0) == 0 {
                    for c in &self.contacts {
                        let json = serde_json::json!({
                            "name": c.name,
                            "email": c.email,
                            "phone": c.phone,
                            "company": c.company,
                            "tags": c.tags,
                            "status": "Customer",
                        });
                        let _ = db.create_record(&self.pid, "contacts", &json.to_string());
                    }
                }

                // 3. Seed deals if empty
                if db.count_records(&self.pid, "deals").unwrap_or(0) == 0 {
                    for d in &self.deals {
                        let json = serde_json::json!({
                            "title": d.title,
                            "value": d.value,
                            "stage": d.stage,
                            "contact_id": d.contact_id,
                        });
                        let _ = db.create_record(&self.pid, "deals", &json.to_string());
                    }
                }
            }
        }
    }

    /// Reloads table cache from proteus-core SQLite DataEngine with fallback to legacy db.
    pub fn reload_table_cache(&mut self, entity: &str) {
        let mut recs = Vec::new();
        if let Some(db_arc) = &self.db {
            if let Ok(db) = db_arc.lock() {
                if let Ok(db_records) = db.list_records(&self.pid, entity) {
                    for r in db_records {
                        let parsed = r.parse_fields().unwrap_or(serde_json::Value::Object(Default::default()));
                        recs.push((r.id, parsed));
                    }
                }
            }
        }
        if recs.is_empty() {
            if let Some(conn) = &self.db_conn {
                if let Ok(legacy_recs) = db::get_records(conn, entity) {
                    recs = legacy_recs;
                }
            }
        }
        self.table_cache.insert(entity.to_string(), recs);
    }

    pub fn sync_contact_to_db(&mut self, contact: &Contact) {
        let data = serde_json::json!({
            "name": contact.name,
            "email": contact.email,
            "phone": contact.phone,
            "company": contact.company,
            "tags": contact.tags,
            "notes": contact.notes,
            "created_at": contact.created_at,
            "updated_at": contact.updated_at,
        });
        if let Some(db_arc) = &self.db {
            if let Ok(db) = db_arc.lock() {
                let _ = db.create_record(&self.pid, "contacts", &data.to_string());
            }
        }
        if let Some(conn) = &self.db_conn {
            let _ = db::upsert_record(conn, "contacts", Some(&contact.id), &data);
        }
        self.reload_table_cache("contacts");
    }

    pub fn delete_contact_from_db(&mut self, id: &str) {
        if let Some(db_arc) = &self.db {
            if let Ok(db) = db_arc.lock() {
                let _ = db.delete_record(id);
            }
        }
        if let Some(conn) = &self.db_conn {
            let _ = db::delete_record(conn, id);
        }
        self.reload_table_cache("contacts");
    }

    pub fn sync_deal_to_db(&mut self, deal: &Deal) {
        let data = serde_json::json!({
            "title": deal.title,
            "value": deal.value,
            "stage": deal.stage,
            "contact_id": deal.contact_id,
            "expected_close": deal.expected_close,
            "notes": deal.notes,
            "created_at": deal.created_at,
            "updated_at": deal.updated_at,
        });
        if let Some(db_arc) = &self.db {
            if let Ok(db) = db_arc.lock() {
                let _ = db.create_record(&self.pid, "deals", &data.to_string());
            }
        }
        if let Some(conn) = &self.db_conn {
            let _ = db::upsert_record(conn, "deals", Some(&deal.id), &data);
        }
        self.reload_table_cache("deals");
    }

    pub fn delete_deal_from_db(&mut self, id: &str) {
        if let Some(db_arc) = &self.db {
            if let Ok(db) = db_arc.lock() {
                let _ = db.delete_record(id);
            }
        }
        if let Some(conn) = &self.db_conn {
            let _ = db::delete_record(conn, id);
        }
        self.reload_table_cache("deals");
    }

    pub fn spawn_designer_node(&mut self, nt: scene::NodeType, world_pos: egui::Pos2) {
        let (w, h) = match nt {
            scene::NodeType::Frame => (200., 200.),
            scene::NodeType::Text { .. } => (100., 30.),
            scene::NodeType::TextInput { .. } => (200., 36.),
            scene::NodeType::Button { .. } => (120., 40.),
            scene::NodeType::Checkbox { .. } => (160., 24.),
            scene::NodeType::Dropdown { .. } => (200., 36.),
            scene::NodeType::NumberField { .. } => (120., 36.),
            scene::NodeType::Image { .. } => (120., 120.),
            scene::NodeType::Shape { .. } => (100., 100.),
            scene::NodeType::Table { .. } => (380., 220.),
            scene::NodeType::DynamicForm { .. } => (320., 290.),
            scene::NodeType::Page | scene::NodeType::Group => (200., 200.),
        };
        self.spawn_designer_node_with_size(nt, (world_pos.x - w / 2., world_pos.y - h / 2.), (w, h));
    }

    pub fn push_undo(&mut self) {
        if self.editor_state.undo_stack.len() >= 50 {
            self.editor_state.undo_stack.remove(0);
        }
        self.editor_state.undo_stack.push(self.project_doc.clone());
        self.editor_state.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(prev) = self.editor_state.undo_stack.pop() {
            self.editor_state.redo_stack.push(self.project_doc.clone());
            self.project_doc = prev;
            self.designer_selected_node = None;
            self.editor_state.selected_node_ids.clear();
            self.toast("Undo ⟲");
        }
    }

    pub fn redo(&mut self) {
        if let Some(next) = self.editor_state.redo_stack.pop() {
            self.editor_state.undo_stack.push(self.project_doc.clone());
            self.project_doc = next;
            self.designer_selected_node = None;
            self.editor_state.selected_node_ids.clear();
            self.toast("Redo ⟳");
        }
    }

    pub fn spawn_designer_node_with_size(
        &mut self,
        nt: scene::NodeType,
        world_pos: (f32, f32),
        size: (f32, f32),
    ) -> String {
        self.push_undo();
        self.spawn_counter += 1;
        let id = format!("node-{}", self.spawn_counter);
        let (name, node_type) = match nt {
            scene::NodeType::Frame => (format!("Box {}", self.spawn_counter), scene::NodeType::Frame),
            scene::NodeType::Text { .. } => (format!("Text {}", self.spawn_counter), scene::NodeType::Text {
                content: "New Text".into(),
                font: scene::FontSpec { family: "Inter".into(), size: 14., weight: 400, color: scene::Rgba { r: 215, g: 215, b: 215, a: 255 } },
            }),
            scene::NodeType::TextInput { .. } => (format!("Input {}", self.spawn_counter), scene::NodeType::TextInput {
                placeholder: "Type...".into(), field_type: scene::FieldType::Text, bound_entity: None, bound_field: None,
            }),
            scene::NodeType::Button { .. } => (format!("Button {}", self.spawn_counter), scene::NodeType::Button {
                label: "Action".into(), style: scene::ButtonStyle::Primary,
            }),
            scene::NodeType::Checkbox { .. } => (format!("Check {}", self.spawn_counter), scene::NodeType::Checkbox {
                label: "Check me".into(), bound_entity: None, bound_field: None,
            }),
            scene::NodeType::Dropdown { .. } => (format!("Dropdown {}", self.spawn_counter), scene::NodeType::Dropdown {
                options: vec!["Option 1".into()], multiple: false, bound_entity: None, bound_field: None,
            }),
            scene::NodeType::NumberField { .. } => (format!("Number {}", self.spawn_counter), scene::NodeType::NumberField {
                min: None, max: None, step: 1.0, bound_entity: None, bound_field: None,
            }),
            scene::NodeType::Image { .. } => (format!("Image {}", self.spawn_counter), scene::NodeType::Image {
                url: String::new(), fit: scene::ImageFit::Cover,
            }),
            scene::NodeType::Shape { .. } => (format!("Shape {}", self.spawn_counter), scene::NodeType::Shape {
                kind: scene::ShapeKind::Rectangle,
            }),
            scene::NodeType::Table { bound_entity, columns } => (format!("Data Table {}", self.spawn_counter), scene::NodeType::Table {
                bound_entity, columns,
            }),
            scene::NodeType::DynamicForm { bound_entity, title, submit_label } => (format!("Dynamic Form {}", self.spawn_counter), scene::NodeType::DynamicForm {
                bound_entity, title, submit_label,
            }),
            scene::NodeType::Page | scene::NodeType::Group => (format!("Frame {}", self.spawn_counter), scene::NodeType::Frame),
        };
        let mut node = scene::Node::new(id.clone(), name, node_type);
        if matches!(node.node_type, scene::NodeType::Button { .. }) {
            node.style.bg_color = [0.31, 0.55, 0.93, 1.0];
            node.style.text_color = [1.0, 1.0, 1.0, 1.0];
            node.style.border_radius = 6.0;
            node.style.border_width = 0.0;
        }
        node.position = world_pos;
        node.layout.width = scene::Sizing::Fixed(size.0);
        node.layout.height = scene::Sizing::Fixed(size.1);
        if let Err(e) = self.project_doc.add_node(node, None) {
            self.toast(format!("Creation failed: {}", e));
        } else {
            self.designer_selected_node = Some(id.clone());
            self.editor_state.selected_node_ids = vec![id.clone()];
            self.toast("Element created ✓");
        }
        id
    }

    pub fn spawn_kpi_card(&mut self, world_pos: (f32, f32)) {
        self.push_undo();
        self.spawn_counter += 1;
        let card_id = format!("kpi-{}", self.spawn_counter);
        let mut card = scene::Node::new(card_id.clone(), "KPI Metric Card".into(), scene::NodeType::Frame);
        card.position = world_pos;
        card.layout.width = scene::Sizing::Fixed(220.0);
        card.layout.height = scene::Sizing::Fixed(100.0);
        card.style.bg_color = [0.08, 0.09, 0.12, 1.0];
        card.style.border_color = [0.18, 0.20, 0.26, 1.0];
        card.style.border_radius = 8.0;
        card.style.border_width = 1.0;

        let _ = self.project_doc.add_node(card, None);

        // Label
        self.spawn_counter += 1;
        let lbl_id = format!("lbl-{}", self.spawn_counter);
        let mut lbl = scene::Node::new(lbl_id, "KPI Label".into(), scene::NodeType::Text {
            content: "ACTIVE REPAIRS".into(),
            font: scene::FontSpec {
                family: "Inter".into(),
                size: 10.0,
                weight: 600,
                color: scene::Rgba { r: 156, g: 163, b: 175, a: 255 },
            },
        });
        lbl.position = (world_pos.0 + 16.0, world_pos.1 + 14.0);
        let _ = self.project_doc.add_node(lbl, Some(card_id.clone()));

        // Value
        self.spawn_counter += 1;
        let val_id = format!("val-{}", self.spawn_counter);
        let mut val = scene::Node::new(val_id, "KPI Value".into(), scene::NodeType::Text {
            content: "24".into(),
            font: scene::FontSpec {
                family: "Inter".into(),
                size: 28.0,
                weight: 700,
                color: scene::Rgba::WHITE,
            },
        });
        val.position = (world_pos.0 + 16.0, world_pos.1 + 32.0);
        let _ = self.project_doc.add_node(val, Some(card_id.clone()));

        // Trend
        self.spawn_counter += 1;
        let trd_id = format!("trd-{}", self.spawn_counter);
        let mut trd = scene::Node::new(trd_id, "KPI Trend".into(), scene::NodeType::Text {
            content: "↑ 12% this week".into(),
            font: scene::FontSpec {
                family: "Inter".into(),
                size: 11.0,
                weight: 500,
                color: scene::Rgba { r: 52, g: 211, b: 153, a: 255 },
            },
        });
        trd.position = (world_pos.0 + 16.0, world_pos.1 + 72.0);
        let _ = self.project_doc.add_node(trd, Some(card_id.clone()));

        self.designer_selected_node = Some(card_id.clone());
        self.editor_state.selected_node_ids = vec![card_id];
        self.toast("KPI Metric Card inserted ✓");
    }

    pub fn spawn_form_block(&mut self, world_pos: (f32, f32)) {
        self.push_undo();
        self.spawn_counter += 1;
        let form_id = format!("form-{}", self.spawn_counter);
        let mut form = scene::Node::new(form_id.clone(), "Intake Form".into(), scene::NodeType::Frame);
        form.position = world_pos;
        form.layout.width = scene::Sizing::Fixed(300.0);
        form.layout.height = scene::Sizing::Fixed(200.0);
        form.style.bg_color = [0.08, 0.09, 0.12, 1.0];
        form.style.border_color = [0.18, 0.20, 0.26, 1.0];
        form.style.border_radius = 8.0;
        form.style.border_width = 1.0;

        let _ = self.project_doc.add_node(form, None);

        // Title
        self.spawn_counter += 1;
        let title_id = format!("title-{}", self.spawn_counter);
        let mut title = scene::Node::new(title_id, "Form Title".into(), scene::NodeType::Text {
            content: "Quick Service Intake".into(),
            font: scene::FontSpec {
                family: "Inter".into(),
                size: 14.0,
                weight: 700,
                color: scene::Rgba::WHITE,
            },
        });
        title.position = (world_pos.0 + 16.0, world_pos.1 + 16.0);
        let _ = self.project_doc.add_node(title, Some(form_id.clone()));

        // Name input
        self.spawn_counter += 1;
        let name_id = format!("inp-name-{}", self.spawn_counter);
        let mut name_inp = scene::Node::new(name_id, "Customer Name Input".into(), scene::NodeType::TextInput {
            placeholder: "Customer Name *".into(),
            field_type: scene::FieldType::Text,
            bound_entity: Some("tickets".into()),
            bound_field: Some("customer_name".into()),
        });
        name_inp.position = (world_pos.0 + 16.0, world_pos.1 + 46.0);
        name_inp.layout.width = scene::Sizing::Fixed(268.0);
        name_inp.layout.height = scene::Sizing::Fixed(34.0);
        let _ = self.project_doc.add_node(name_inp, Some(form_id.clone()));

        // Phone input
        self.spawn_counter += 1;
        let phone_id = format!("inp-phone-{}", self.spawn_counter);
        let mut phone_inp = scene::Node::new(phone_id, "Phone Number Input".into(), scene::NodeType::TextInput {
            placeholder: "Phone Number *".into(),
            field_type: scene::FieldType::Text,
            bound_entity: Some("tickets".into()),
            bound_field: Some("customer_phone".into()),
        });
        phone_inp.position = (world_pos.0 + 16.0, world_pos.1 + 88.0);
        phone_inp.layout.width = scene::Sizing::Fixed(268.0);
        phone_inp.layout.height = scene::Sizing::Fixed(34.0);
        let _ = self.project_doc.add_node(phone_inp, Some(form_id.clone()));

        // Submit Button
        self.spawn_counter += 1;
        let btn_id = format!("btn-sub-{}", self.spawn_counter);
        let mut btn = scene::Node::new(btn_id, "Submit Button".into(), scene::NodeType::Button {
            label: "💾 Submit & Print".into(),
            style: scene::ButtonStyle::Primary,
        });
        btn.position = (world_pos.0 + 16.0, world_pos.1 + 134.0);
        btn.layout.width = scene::Sizing::Fixed(268.0);
        btn.layout.height = scene::Sizing::Fixed(38.0);
        btn.style.bg_color = [0.31, 0.55, 0.93, 1.0];
        btn.style.text_color = [1.0, 1.0, 1.0, 1.0];
        btn.style.border_radius = 6.0;
        let _ = self.project_doc.add_node(btn, Some(form_id.clone()));

        self.designer_selected_node = Some(form_id.clone());
        self.editor_state.selected_node_ids = vec![form_id];
        self.toast("Intake Form Card inserted ✓");
    }

    pub fn save_project(&mut self) {
        let flows_json = serde_json::to_string(&self.project_doc.flow_graph).unwrap_or_else(|_| "{}".into());
        let contacts_json = serde_json::to_string(&self.contacts).unwrap_or_else(|_| "[]".into());
        let deals_json = serde_json::to_string(&self.deals).unwrap_or_else(|_| "[]".into());
        let tasks_json = serde_json::to_string(&self.tasks).unwrap_or_else(|_| "[]".into());
        let combined = serde_json::json!({
            "contacts": serde_json::from_str::<serde_json::Value>(&contacts_json).unwrap_or_default(),
            "deals": serde_json::from_str::<serde_json::Value>(&deals_json).unwrap_or_default(),
            "tasks": serde_json::from_str::<serde_json::Value>(&tasks_json).unwrap_or_default()
        });
        let combined_str = serde_json::to_string(&combined).unwrap_or_else(|_| "[]".into());
        let pid = self.pid.clone();
        let pn = self.pname.clone();
        let saved = self.db.as_ref().and_then(|d| {
            d.lock().ok().and_then(|db| {
                db.upsert_project(&proteus_core::Project {
                    id: pid,
                    name: pn,
                    widgets: combined_str,
                    flows: flows_json,
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                }).ok()
            })
        }).is_some();
        if saved { self.toast("Project saved"); } else { self.toast("Cannot save (no DB)"); }

        if self.auto_deploy_on_save {
            if let Some(ref daemon) = self.lan_daemon {
                let mut pkg = proteus_core::package::PrPackage::new(
                    format!("PKG-{}", self.pid),
                    if self.pname.is_empty() { "Custom Template".to_string() } else { self.pname.clone() },
                    "Designer (PCD)",
                );
                pkg.views.push(proteus_core::package::PrViewLayout {
                    view_id: self.pid.clone(),
                    name: self.pname.clone(),
                    view_type: "designer_canvas".to_string(),
                    layout_json: serde_json::to_string(&self.project_doc).unwrap_or_default(),
                });
                let results = daemon.deploy_to_all_peers(&pkg);
                let ok_count = results.iter().filter(|(_, r)| r.is_ok()).count();
                if !results.is_empty() {
                    self.toast(format!("Auto-Deployed to {}/{} terminals ✓", ok_count, results.len()));
                }
            }
        }
    }

    pub fn build_candidate_package(&self) -> Result<proteus_core::package::PrPackage, String> {
        let pkg_id = format!("PKG-{}", if self.pid.is_empty() { "DEFAULT".to_string() } else { self.pid.clone() });
        let pkg_name = if self.pname.trim().is_empty() {
            "Custom Store Template".to_string()
        } else {
            self.pname.trim().to_string()
        };

        let mut pkg = proteus_core::package::PrPackage::new(
            pkg_id,
            pkg_name.clone(),
            "Proteus Designer (PCD)",
        );

        // 1. Pack Project Document Scene and View Layout
        let layout_json = serde_json::to_string(&self.project_doc)
            .map_err(|e| format!("Serialization error: {}", e))?;
        pkg.views.push(proteus_core::package::PrViewLayout {
            view_id: if self.pid.is_empty() { "main_view".to_string() } else { self.pid.clone() },
            name: pkg_name.clone(),
            view_type: "designer_canvas".to_string(),
            layout_json,
        });

        // 2. Pack Interactive Flow Triggers from Canvas FlowGraph
        for (node_id, node) in &self.project_doc.flow_graph.nodes {
            let (event_name, action_label) = match &node.kind {
                crate::flow::FlowNodeKind::TriggerClick { target_node_id } => {
                    ("OnClick".to_string(), format!("Click: {}", target_node_id))
                }
                crate::flow::FlowNodeKind::NavigateTo { page_id } => {
                    ("Navigate".to_string(), format!("Go to: {}", page_id))
                }
                crate::flow::FlowNodeKind::SaveToDatabase { entity } => {
                    ("OnSubmit".to_string(), format!("Save to: {}", entity))
                }
                crate::flow::FlowNodeKind::Condition { field, operator, target_value } => {
                    ("Condition".to_string(), format!("IF {} {} {}", field, operator, target_value))
                }
                crate::flow::FlowNodeKind::ShowToast { message } => {
                    ("Toast".to_string(), message.clone())
                }
                crate::flow::FlowNodeKind::FederationBridge { source_entity, target_entity, .. } => {
                    ("Federation".to_string(), format!("Bridge {} -> {}", source_entity, target_entity))
                }
                crate::flow::FlowNodeKind::HardwareRelay { device_id, channel, action, .. } => {
                    ("HardwareRelay".to_string(), format!("Relay {} ch{} [{}]", device_id, channel, action))
                }
                crate::flow::FlowNodeKind::HardwareSensor { device_id, metric, operator, threshold } => {
                    ("HardwareSensor".to_string(), format!("Sensor {} {} {} {}", device_id, metric, operator, threshold))
                }
                crate::flow::FlowNodeKind::HardwareScale { device_id, require_stable } => {
                    ("HardwareScale".to_string(), format!("Scale {} (require_stable: {})", device_id, require_stable))
                }
                crate::flow::FlowNodeKind::HardwareDisplay { device_id, .. } => {
                    ("HardwareDisplay".to_string(), format!("VFD Pole Display: {}", device_id))
                }
            };

            pkg.flows.push(proteus_core::package::PrFlowTrigger {
                id: node_id.clone(),
                trigger_event: event_name,
                action_type: action_label,
                config_json: serde_json::to_string(&node.kind).unwrap_or_else(|_| "{}".into()),
            });
        }

        // 3. Collect Additive DDL Statements and Schemas for cached tables
        for entity_name in self.table_cache.keys() {
            let ddl = format!(
                "CREATE TABLE IF NOT EXISTS {} (id TEXT PRIMARY KEY, data TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);",
                entity_name
            );
            if !pkg.schema.ddl_statements.contains(&ddl) {
                pkg.schema.ddl_statements.push(ddl);
            }

            let mut entity = proteus_core::schema::EntitySchema::new(entity_name, entity_name);
            entity.fields.push(proteus_core::schema::FieldDefinition {
                name: "id".to_string(),
                label: "ID".to_string(),
                field_type: proteus_core::schema::FieldType::Text,
                required: true,
                default_value: None,
            });
            entity.fields.push(proteus_core::schema::FieldDefinition {
                name: "data".to_string(),
                label: "Data".to_string(),
                field_type: proteus_core::schema::FieldType::Text,
                required: true,
                default_value: None,
            });
            pkg.schema.entity_schemas.push(entity);
        }

        Ok(pkg)
    }

    pub fn run_linter_preflight(&mut self) {
        match self.build_candidate_package() {
            Ok(pkg) => {
                let report = proteus_core::package::PackageLinter::lint_package(&pkg);
                self.candidate_package = Some(pkg);
                self.active_lint_report = Some(report);
                self.show_linter_modal = true;
            }
            Err(e) => {
                self.toast(format!("Σφάλμα δημιουργίας πακέτου: {}", e));
            }
        }
    }

    pub fn confirm_linter_export(&mut self) -> Result<std::path::PathBuf, String> {
        let pkg = match &self.candidate_package {
            Some(p) => p.clone(),
            None => self.build_candidate_package()?,
        };

        let report = proteus_core::package::PackageLinter::lint_package(&pkg);
        if !report.is_valid {
            return Err(format!("Export blocked: {} fatal lint errors detected", report.errors.len()));
        }

        let bytes = pkg.to_bytes().map_err(|e| format!("Packaging error: {}", e))?;
        let packages_dir = proteus_core::paths::get_packages_dir();
        let _ = std::fs::create_dir_all(&packages_dir);

        let clean_filename = pkg.manifest.name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();
        let target_path = packages_dir.join(format!("{}.pr", clean_filename));

        std::fs::write(&target_path, &bytes)
            .map_err(|e| format!("I/O Error writing package file: {}", e))?;

        self.show_linter_modal = false;
        self.toast(format!("✓ Εξήχθη πιστοποιημένο πακέτο .pr ({:.1} KB - {:.0}% Score)", bytes.len() as f64 / 1024.0, report.quality_score));
        Ok(target_path)
    }

    pub fn export_pr_package(&mut self) -> Result<std::path::PathBuf, String> {
        self.run_linter_preflight();
        self.confirm_linter_export()
    }

    pub fn load_sample_requirement(&mut self) {
        let fields = vec![
            proteus_core::package::FieldRequirement {
                name: "customer_name".to_string(),
                data_type: "Text".to_string(),
                is_required: true,
                description: "Full customer legal name".to_string(),
            },
            proteus_core::package::FieldRequirement {
                name: "tax_afm".to_string(),
                data_type: "Text".to_string(),
                is_required: true,
                description: "Greek AFM tax registration number".to_string(),
            },
            proteus_core::package::FieldRequirement {
                name: "vehicle_plate".to_string(),
                data_type: "Text".to_string(),
                is_required: true,
                description: "Vehicle registration plate".to_string(),
            },
            proteus_core::package::FieldRequirement {
                name: "fault_notes".to_string(),
                data_type: "Text".to_string(),
                is_required: false,
                description: "Reported mechanical or electrical fault".to_string(),
            },
        ];
        let workflows = vec![
            proteus_core::package::WorkflowIntent {
                title: "Intake Ticket Print".to_string(),
                trigger_event: "on_ticket_intake".to_string(),
                expected_action: "escpos_print_80mm".to_string(),
            },
        ];
        let req = proteus_core::package::RequirementsPackage::new(
            "Auto Workshop Intake System",
            "Automotive Services",
            "client_c8a49f_auto",
            fields,
            workflows,
            "Must feature quick plate search and Greek AFM validation.",
        );
        self.loaded_requirement = Some(req);
        self.toast("Loaded customer requirements (.prreq) ✓");
    }

    pub fn scaffold_canvas_from_requirements(&mut self) -> Result<usize, String> {
        let req = self.loaded_requirement.as_ref().ok_or("No requirements loaded")?.clone();
        self.push_undo();
        self.spawn_counter += 1;
        let frame_id = format!("frame-req-{}", self.spawn_counter);
        let mut frame = scene::Node::new(frame_id.clone(), format!("{} Screen", req.title), scene::NodeType::Frame);
        frame.position = (60.0, 60.0);
        frame.layout.width = scene::Sizing::Fixed(440.0);
        frame.layout.height = scene::Sizing::Fixed(580.0);
        frame.style.bg_color = [0.08, 0.09, 0.12, 1.0];
        frame.style.border_color = [0.18, 0.20, 0.26, 1.0];
        frame.style.border_radius = 8.0;
        frame.style.border_width = 1.0;
        let _ = self.project_doc.add_node(frame, None);

        let mut count = 1;
        let mut y = 80.0;
        for field in &req.fields {
            self.spawn_counter += 1;
            let inp_id = format!("inp-{}-{}", field.name, self.spawn_counter);
            let mut inp = scene::Node::new(inp_id, format!("Input {}", field.name), scene::NodeType::TextInput {
                placeholder: format!("{} *", field.description),
                field_type: scene::FieldType::Text,
                bound_entity: Some("tickets".into()),
                bound_field: Some(field.name.clone()),
            });
            inp.position = (80.0, y);
            inp.layout.width = scene::Sizing::Fixed(380.0);
            inp.layout.height = scene::Sizing::Fixed(36.0);
            let _ = self.project_doc.add_node(inp, Some(frame_id.clone()));
            y += 50.0;
            count += 1;
        }
        self.toast(format!("Scaffolded {} nodes from customer requirements ✓", count));
        Ok(count)
    }

    pub fn export_sandbox_preview(&mut self) -> Result<Vec<u8>, String> {
        let req_id = self.loaded_requirement.as_ref().map(|r| r.req_id.clone());
        let bundle_id = self.pname.replace(' ', "_").to_lowercase();
        let preview = proteus_core::package::PreviewBundle::new(
            req_id,
            &format!("pkg_{bundle_id}"),
            &format!("{} (Preview)", self.pname),
            "pcd_certified_author",
            r#"{"views":[{"view_id":"main","name":"Customer Intake"}]}"#,
            r#"{"schemas":[]}"#,
        );
        let bytes = preview.to_bytes().map_err(|e| e.to_string())?;
        self.generated_preview = Some(preview);
        self.toast("Generated watermarked sandbox preview (.prpreview) ✓");
        Ok(bytes)
    }

    pub fn simulate_client_review_approval(&mut self) {
        let Some(preview) = self.generated_preview.as_ref() else {
            self.toast("No preview available to review");
            return;
        };
        let items = vec![
            proteus_core::package::ReviewFeedbackItem::new("intake_artboard", "Layout looks great, typography is clear"),
            proteus_core::package::ReviewFeedbackItem::new("tax_afm", "AFM validation verified with Greek tax rules"),
        ];
        let session = proteus_core::package::ClientReviewSession::new(
            &preview.preview_id,
            "client_c8a49f_auto",
            proteus_core::package::ReviewVerdict::Approved,
            items,
        );
        let mut contract = proteus_core::package::EscrowEngine::create_contract(
            "lic_client_c8a49f_auto",
            "pcd_certified_author",
            15000,
        );
        let _ = proteus_core::package::EscrowEngine::fund_contract(&mut contract);
        let _ = proteus_core::package::EscrowEngine::start_review(&mut contract);
        if let Some(token) = &session.approval_token {
            let _ = proteus_core::package::EscrowEngine::submit_approval(&mut contract, token);
        }
        self.escrow_contract = Some(contract);
        self.active_review_session = Some(session);
        self.toast("Client review received: APPROVED (Escrow Release Ready) ✓");
    }

    pub fn submit_to_escrow_delivery(&mut self) -> Result<proteus_core::package::DeliveryRelease, String> {
        let pkg = self.build_candidate_package()?;
        let contract = self.escrow_contract.as_mut().ok_or("No active escrow contract")?;
        let (bound, release) = proteus_core::package::EscrowEngine::bind_and_deliver(contract, &pkg)
            .map_err(|e| e.to_string())?;
        self.candidate_package = Some(bound);
        self.toast(format!(
            "Released! Payout: {} € (90%), Platform: {} € (10%) ✓",
            release.designer_payout_cents / 100,
            release.platform_fee_cents / 100
        ));
        Ok(release)
    }


    pub fn load_project(&mut self) {
        if let Some(d) = &self.db {
            if let Ok(db) = d.lock() {
                if let Ok(proj) = db.load_project(&self.pid) {
                    self.pname = proj.name;
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&proj.widgets) {
                        if let Some(contacts_arr) = data["contacts"].as_array() {
                            self.contacts = contacts_arr.iter().map(|c| Contact {
                                id: c["id"].as_str().unwrap_or("?").into(),
                                name: c["name"].as_str().unwrap_or("").into(),
                                email: c["email"].as_str().unwrap_or("").into(),
                                phone: c["phone"].as_str().unwrap_or("").into(),
                                company: c["company"].as_str().unwrap_or("").into(),
                                tags: c["tags"].as_array().map(|a| a.iter().filter_map(|s| s.as_str().map(|x| x.to_string())).collect()).unwrap_or_default(),
                                notes: c["notes"].as_array().map(|a| a.iter().map(|n| ContactNote {
                                    id: n["id"].as_str().unwrap_or("").into(),
                                    text: n["text"].as_str().unwrap_or("").into(),
                                    created_at: n["created_at"].as_str().unwrap_or("").into(),
                                }).collect()).unwrap_or_default(),
                                created_at: c["created_at"].as_str().unwrap_or("").into(),
                                updated_at: c["updated_at"].as_str().unwrap_or("").into(),
                            }).collect();
                        }
                        if let Some(deals_arr) = data["deals"].as_array() {
                            self.deals = deals_arr.iter().map(|d| Deal {
                                id: d["id"].as_str().unwrap_or("?").into(),
                                title: d["title"].as_str().unwrap_or("").into(),
                                value: d["value"].as_f64().unwrap_or(0.0),
                                stage: d["stage"].as_str().unwrap_or("New").into(),
                                contact_id: d["contact_id"].as_str().map(|s| s.to_string()),
                                expected_close: d["expected_close"].as_str().unwrap_or("").into(),
                                notes: d["notes"].as_str().unwrap_or("").into(),
                                created_at: d["created_at"].as_str().unwrap_or("").into(),
                                updated_at: d["updated_at"].as_str().unwrap_or("").into(),
                            }).collect();
                        }
                        if let Some(tasks_arr) = data["tasks"].as_array() {
                            self.tasks = tasks_arr.iter().map(|t| Task {
                                id: t["id"].as_str().unwrap_or("?").into(),
                                title: t["title"].as_str().unwrap_or("").into(),
                                description: t["description"].as_str().unwrap_or("").into(),
                                contact_id: t["contact_id"].as_str().map(|s| s.to_string()),
                                deal_id: t["deal_id"].as_str().map(|s| s.to_string()),
                                assignee: t["assignee"].as_str().unwrap_or("").into(),
                                due_date: t["due_date"].as_str().unwrap_or("").into(),
                                status: match t["status"].as_str().unwrap_or("Open") { "InProgress" => TaskStatus::InProgress, "Done" => TaskStatus::Done, "Cancelled" => TaskStatus::Cancelled, _ => TaskStatus::Open },
                                priority: match t["priority"].as_str().unwrap_or("Medium") { "High" => TaskPriority::High, "Low" => TaskPriority::Low, _ => TaskPriority::Medium },
                                created_by: t["created_by"].as_str().unwrap_or("").into(),
                                created_at: t["created_at"].as_str().unwrap_or("").into(),
                                updated_at: t["updated_at"].as_str().unwrap_or("").into(),
                                completed_at: t["completed_at"].as_str().map(|s| s.to_string()),
                            }).collect();
                        }
                    }
                    if let Ok(fg) = serde_json::from_str::<crate::flow::FlowGraph>(&proj.flows) {
                        self.project_doc.flow_graph = fg;
                    }
                }
            }
        }
    }

    /// Load available bespoke project briefs from SQLite.
    pub fn load_available_briefs(&mut self) {
        if let Ok(conn) = proteus_core::paths::open_store_connection() {
            let _ = proteus_core::brief::seed_default_briefs_if_empty(&conn);
            if let Ok(briefs) = proteus_core::brief::list_all_briefs(&conn) {
                self.cached_briefs = briefs;
            }
        }
    }

    /// Scaffold directly from a ClientProjectBrief.
    pub fn scaffold_from_project_brief(&mut self, brief: &proteus_core::brief::ClientProjectBrief) {
        self.cached_briefs.push(brief.clone());
        let idx = self.cached_briefs.len() - 1;
        self.apply_brief_to_canvas(idx);
    }

    /// Automatically scaffold canvas artboards and UI elements from a bespoke brief.
    pub fn apply_brief_to_canvas(&mut self, brief_idx: usize) {
        if brief_idx >= self.cached_briefs.len() {
            return;
        }
        let brief = self.cached_briefs[brief_idx].clone();
        let scaffold_screens = proteus_core::brief::generate_scaffold_screens(&brief);

        let mut doc = crate::scene::ProjectDocument::new();
        for screen in &scaffold_screens {
            let page_id = screen.screen_id.clone();
            let page_node = crate::scene::Node {
                id: page_id.clone(),
                name: screen.title.clone(),
                node_type: crate::scene::NodeType::Frame,
                parent_id: None,
                children_ids: vec![],
                styling: crate::scene::Styling {
                    background: Some(crate::scene::Rgba { r: 24, g: 27, b: 34, a: 255 }),
                    corner_radius: [8., 8., 8., 8.],
                    border: Some(crate::scene::Border { width: 1.5, color: crate::scene::Rgba { r: 56, g: 189, b: 248, a: 200 } }),
                    padding: [16., 16., 16., 16.],
                    shadow: None,
                    opacity: 1.0,
                },
                style: crate::scene::NodeStyle::default(),
                layout: crate::scene::Layout {
                    width: crate::scene::Sizing::Fixed(screen.width),
                    height: crate::scene::Sizing::Fixed(screen.height),
                    ..Default::default()
                },
                position: (screen.pos_x, screen.pos_y),
                rotation: 0.0,
                visible: true,
                locked: false,
                z: 0,
            };
            let _ = doc.add_node(page_node, None);

            for (elem_idx, elem) in screen.elements.iter().enumerate() {
                let elem_id = format!("{}-elem-{}", page_id, elem_idx + 1);
                let (node_type, style, styling) = match elem.element_type.as_str() {
                    "Header" => (
                        crate::scene::NodeType::Text {
                            content: elem.label.clone(),
                            font: crate::scene::FontSpec {
                                family: "Inter".into(),
                                size: 16.0,
                                weight: 700,
                                color: crate::scene::Rgba { r: 240, g: 246, b: 252, a: 255 },
                            },
                        },
                        crate::scene::NodeStyle::default(),
                        crate::scene::Styling::default(),
                    ),
                    "Input" => (
                        crate::scene::NodeType::TextInput {
                            placeholder: elem.label.clone(),
                            field_type: crate::scene::FieldType::Text,
                            bound_entity: elem.binding.clone(),
                            bound_field: elem.binding.clone(),
                        },
                        crate::scene::NodeStyle {
                            bg_color: [0.12, 0.14, 0.18, 1.0],
                            text_color: [0.95, 0.95, 0.95, 1.0],
                            border_radius: 5.0,
                            border_width: 1.0,
                            border_color: [0.25, 0.3, 0.38, 1.0],
                        },
                        crate::scene::Styling::default(),
                    ),
                    "Button" => (
                        crate::scene::NodeType::Button {
                            label: elem.label.clone(),
                            style: crate::scene::ButtonStyle::Primary,
                        },
                        crate::scene::NodeStyle {
                            bg_color: [0.22, 0.74, 0.97, 1.0],
                            text_color: [0.05, 0.05, 0.08, 1.0],
                            border_radius: 6.0,
                            border_width: 0.0,
                            border_color: [0.0, 0.0, 0.0, 0.0],
                        },
                        crate::scene::Styling::default(),
                    ),
                    "Table" => (
                        crate::scene::NodeType::Table {
                            columns: vec!["Code".into(), "Description".into(), "Qty".into(), "Price".into(), "Total".into()],
                            bound_entity: elem.binding.clone(),
                        },
                        crate::scene::NodeStyle {
                            bg_color: [0.1, 0.12, 0.15, 1.0],
                            text_color: [0.9, 0.9, 0.9, 1.0],
                            border_radius: 6.0,
                            border_width: 1.0,
                            border_color: [0.2, 0.25, 0.3, 1.0],
                        },
                        crate::scene::Styling::default(),
                    ),
                    _ => (
                        crate::scene::NodeType::Text {
                            content: elem.label.clone(),
                            font: crate::scene::FontSpec::default(),
                        },
                        crate::scene::NodeStyle::default(),
                        crate::scene::Styling::default(),
                    ),
                };

                let child_node = crate::scene::Node {
                    id: elem_id.clone(),
                    name: elem.label.clone(),
                    node_type,
                    parent_id: Some(page_id.clone()),
                    children_ids: vec![],
                    styling,
                    style,
                    layout: crate::scene::Layout {
                        width: crate::scene::Sizing::Fixed(elem.width),
                        height: crate::scene::Sizing::Fixed(elem.height),
                        ..Default::default()
                    },
                    position: (elem.x, elem.y),
                    rotation: 0.0,
                    visible: true,
                    locked: false,
                    z: (elem_idx + 1) as i32,
                };
                let _ = doc.add_node(child_node, Some(page_id.clone()));
            }
        }

        self.project_doc = doc;
        self.pname = brief.business_name.clone();
        self.in_welcome_hub = false;
        self.mode = crate::models::Mode::Designer;
        self.toast(format!("Scaffolded {} bespoke screens for '{}' ✓", scaffold_screens.len(), brief.business_name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proteus_app_defaults() {
        let app = ProteusApp::default();
        assert_eq!(app.pid, "p1");
        assert_eq!(app.pname, "Demo Project");
        assert_eq!(app.mode, Mode::Designer);
        assert_eq!(app.contacts.len(), 3);
        assert_eq!(app.deals.len(), 3);
        assert_eq!(app.tasks.len(), 3);
        assert_eq!(app.studio_layers.len(), 3);
        assert_eq!(app.vrr_mode, crate::models::VrrMode::Fps60);
        assert_eq!(app.theme_mode, crate::theme::ThemeMode::Dark);
        assert_eq!(app.accent_preset, crate::theme::AccentPreset::NordicSlate);
    }

    #[test]
    fn test_undo_redo_history() {
        let mut app = ProteusApp::default();
        assert_eq!(app.editor_state.undo_stack.len(), 0);
        assert_eq!(app.editor_state.redo_stack.len(), 0);

        let id = app.spawn_designer_node_with_size(scene::NodeType::Frame, (10.0, 10.0), (100.0, 100.0));
        assert_eq!(app.editor_state.undo_stack.len(), 1);
        assert!(app.project_doc.nodes.contains_key(&id));

        app.undo();
        assert_eq!(app.editor_state.undo_stack.len(), 0);
        assert_eq!(app.editor_state.redo_stack.len(), 1);

        app.redo();
        assert_eq!(app.editor_state.undo_stack.len(), 1);
        assert_eq!(app.editor_state.redo_stack.len(), 0);
    }

    #[test]
    fn test_spawn_kpi_and_form_blocks() {
        let mut app = ProteusApp::default();
        let initial_node_count = app.project_doc.nodes.len();

        app.spawn_kpi_card((50.0, 50.0));
        assert!(app.project_doc.nodes.len() > initial_node_count);

        let kpi_node_count = app.project_doc.nodes.len();
        app.spawn_form_block((200.0, 200.0));
        assert!(app.project_doc.nodes.len() > kpi_node_count);
    }

    #[test]
    fn test_data_widgets_db_binding_and_table_cache() {
        let mut app = ProteusApp::default();

        // 1. Table cache should contain seeded contacts
        let contacts_cache = app.table_cache.get("contacts");
        assert!(contacts_cache.is_some(), "Contacts cache must be populated");
        let contacts = contacts_cache.unwrap();
        assert!(!contacts.is_empty(), "Contacts cache must contain seeded records");

        // 2. Sync a new contact and verify table cache reload
        let new_c = Contact {
            id: "c_test_99".into(),
            name: "Nikolaos Test".into(),
            email: "nikolaos@test.gr".into(),
            phone: "+30 210 9999999".into(),
            company: "Test Corp".into(),
            tags: vec!["VIP".into()],
            notes: vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        app.sync_contact_to_db(&new_c);

        let reloaded = app.table_cache.get("contacts").unwrap();
        assert!(reloaded.iter().any(|(id, val)| {
            id == "c_test_99" || val.get("name").and_then(|v| v.as_str()) == Some("Nikolaos Test")
        }), "New contact must appear in reloaded table cache");

        // 3. Delete contact and verify cache update
        app.delete_contact_from_db("c_test_99");
        let after_delete = app.table_cache.get("contacts").unwrap();
        assert!(!after_delete.iter().any(|(id, _)| id == "c_test_99"), "Deleted contact must not be in table cache");
    }

    #[test]
    fn test_brief_scaffolding_in_app() {
        let mut app = ProteusApp::default();
        let brief = proteus_core::brief::ClientProjectBrief {
            brief_id: "BRF-UNIT-01".into(),
            business_name: "Unit Test Workshop".into(),
            business_nature: "Hardware Lab".into(),
            daily_operations_desc: "Diagnostic tests".into(),
            track: proteus_core::brief::BriefTrack::PcdApp,
            required_screens: vec!["Service Intake Form".into(), "POS Counter".into()],
            hardware_peripherals: vec!["ESC/POS 80mm".into()],
            proposed_budget_eur: 400.0,
            domain_name_requested: None,
            hosting_preference: "SelfHosted".into(),
            contact_email: "unit@test.com".into(),
            submitted_at: "2026-09-28".into(),
        };

        app.cached_briefs = vec![brief];
        app.apply_brief_to_canvas(0);

        assert_eq!(app.pname, "Unit Test Workshop");
        assert_eq!(app.project_doc.root_node_ids.len(), 2);
        assert!(!app.in_welcome_hub);
        assert_eq!(app.mode, Mode::Designer);
    }

    #[test]
    fn test_export_pr_package_roundtrip() {
        let mut app = ProteusApp::default();
        app.pid = "test-export-proj".to_string();
        app.pname = "Test Export Store".to_string();
        app.table_cache.insert("inventory_items".to_string(), vec![]);

        let exported_path = app.export_pr_package().expect("Export must succeed");
        assert!(exported_path.exists());

        let bytes = std::fs::read(&exported_path).expect("Package file must be readable");
        let pkg = proteus_core::package::PrPackage::from_bytes(&bytes).expect("Valid sealed PR package");

        assert_eq!(pkg.manifest.bundle_id, "PKG-test-export-proj");
        assert_eq!(pkg.manifest.name, "Test Export Store");
        assert_eq!(pkg.views.len(), 1);
        assert_eq!(pkg.views[0].view_type, "designer_canvas");
        assert!(pkg.schema.ddl_statements.iter().any(|d| d.contains("inventory_items")));

        let _ = std::fs::remove_file(exported_path);
    }

    #[test]
    fn test_studio_linter_preflight_and_export_gate() {
        let mut app = ProteusApp::default();
        app.pid = "linter-gate-test".to_string();
        app.pname = "Linter Verified Shop".to_string();
        app.table_cache.insert("tickets".to_string(), vec![]);

        // 1. Run Pre-flight
        app.run_linter_preflight();
        assert!(app.show_linter_modal, "Preflight must open linter modal");
        assert!(app.active_lint_report.is_some(), "Must generate active lint report");

        let report = app.active_lint_report.as_ref().unwrap();
        assert!(report.is_valid, "Clean package must be valid");
        assert_eq!(report.errors.len(), 0, "No fatal lint errors expected");

        // 2. Confirm Export
        let exported_path = app.confirm_linter_export().expect("Export must succeed");
        assert!(exported_path.exists());
        assert!(!app.show_linter_modal, "Modal must close upon successful export");

        let _ = std::fs::remove_file(exported_path);
    }

    #[test]
    fn test_studio_sharepoint_and_escrow_flow() {
        let mut app = ProteusApp::default();
        app.pid = "sharepoint-test".to_string();
        app.pname = "Auto Workshop Pro".to_string();
        app.table_cache.insert("tickets".to_string(), vec![]);

        // 1. Ingest .prreq requirements
        app.load_sample_requirement();
        assert!(app.loaded_requirement.is_some());
        let req = app.loaded_requirement.as_ref().unwrap();
        assert_eq!(req.fields.len(), 4);
        assert!(req.verify_integrity());

        // 2. Scaffold Canvas
        let nodes_count = app.scaffold_canvas_from_requirements().expect("Scaffold canvas");
        assert_eq!(nodes_count, 5);

        // 3. Export watermarked .prpreview
        let preview_bytes = app.export_sandbox_preview().expect("Export preview");
        assert!(app.generated_preview.is_some());
        let preview = app.generated_preview.as_ref().unwrap();
        assert_eq!(preview.watermark_text, proteus_core::package::DEFAULT_PREVIEW_WATERMARK);
        assert!(!preview_bytes.is_empty());

        // 4. Simulate Client Review & Approval Handshake
        app.simulate_client_review_approval();
        assert!(app.active_review_session.is_some());
        let session = app.active_review_session.as_ref().unwrap();
        assert_eq!(session.verdict, proteus_core::package::ReviewVerdict::Approved);
        assert!(session.approval_token.is_some());

        // 5. Submit to Escrow & Execute Host Key Binding
        let release = app.submit_to_escrow_delivery().expect("Escrow delivery");
        assert_eq!(release.designer_payout_cents, 13500); // 135.00 EUR (90%)
        assert_eq!(release.platform_fee_cents, 1500); // 15.00 EUR (10%)
        assert!(release.settlement_signature.starts_with("sig_"));
        assert!(app.candidate_package.is_some());
        let candidate = app.candidate_package.as_ref().unwrap();
        assert_eq!(candidate.manifest.target_client_license, Some("lic_client_c8a49f_auto".to_string()));
    }
}

