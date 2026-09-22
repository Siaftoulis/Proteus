//! ProteusApp application state, default initialization, entity mutations, and persistence engine.

use chrono::Utc;
use crm_core::Database;
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
    pub active_tool: crate::models::DesignerTool,
    pub copied_dimensions: Option<(f32, f32)>,
    pub marquee_start: Option<egui::Pos2>,
    pub draw_start: Option<egui::Pos2>,

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
    pub lan_daemon: Option<crm_core::lan::LanDiscoveryDaemon>,
    pub auto_deploy_on_save: bool,

    // Role Workspaces
    pub analyst_state: views::analyst::AnalystState,
    pub networking_state: views::networking::NetworkingState,
    pub troubleshoot_state: views::troubleshoot::TroubleshootState,
    pub connected_data_state: views::connected_data::ConnectedDataState,
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

        Self {
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
            active_tool: crate::models::DesignerTool::Select,
            copied_dimensions: None,
            marquee_start: None,
            draw_start: None,
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
            lan_daemon: crm_core::lan::LanDiscoveryDaemon::start(
                "designer-master".to_string(),
                "ProteusDesigner".to_string(),
                std::sync::Arc::new(std::sync::Mutex::new("Proteus Studio Master".to_string())),
                0,
                crm_core::lan::DEFAULT_BEACON_PORT,
                false,
            ).ok(),
            auto_deploy_on_save: false,
            analyst_state: views::analyst::AnalystState::default(),
            networking_state: views::networking::NetworkingState::default(),
            troubleshoot_state: views::troubleshoot::TroubleshootState::default(),
            connected_data_state: views::connected_data::ConnectedDataState::default(),
        }
    }
}

impl ProteusApp {
    pub fn toast(&mut self, msg: impl Into<String>) {
        self.toast = Some(msg.into());
        self.tt = 0.;
    }

    pub fn reload_table_cache(&mut self, entity: &str) {
        if let Some(conn) = &self.db_conn {
            if let Ok(recs) = db::get_records(conn, entity) {
                self.table_cache.insert(entity.to_string(), recs);
            }
        }
    }

    pub fn sync_contact_to_db(&mut self, contact: &Contact) {
        if let Some(conn) = &self.db_conn {
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
            let _ = db::upsert_record(conn, "contacts", Some(&contact.id), &data);
        }
        self.reload_table_cache("contacts");
    }

    pub fn delete_contact_from_db(&mut self, id: &str) {
        if let Some(conn) = &self.db_conn {
            let _ = db::delete_record(conn, id);
        }
        self.reload_table_cache("contacts");
    }

    pub fn sync_deal_to_db(&mut self, deal: &Deal) {
        if let Some(conn) = &self.db_conn {
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
            let _ = db::upsert_record(conn, "deals", Some(&deal.id), &data);
        }
        self.reload_table_cache("deals");
    }

    pub fn delete_deal_from_db(&mut self, id: &str) {
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
            scene::NodeType::Table { .. } => (360., 160.),
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
            scene::NodeType::Table { bound_entity, columns } => (format!("Table {}", self.spawn_counter), scene::NodeType::Table {
                bound_entity, columns,
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
                db.upsert_project(&crm_core::Project {
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
                let mut pkg = crm_core::package::PrPackage::new(
                    format!("PKG-{}", self.pid),
                    if self.pname.is_empty() { "Custom Template".to_string() } else { self.pname.clone() },
                    "Designer (PCD)",
                );
                pkg.views.push(crm_core::package::PrViewLayout {
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
}
