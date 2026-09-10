pub mod components;
pub mod data_viewer;
pub mod db;
pub mod flow;
pub mod flow_renderer;
pub mod inspector;
pub mod models;
pub mod palette;
pub mod renderer;
pub mod scene;
pub mod storage;
pub mod theme;
pub mod views;

pub use models::*;

use chrono::Utc;
use crm_core::Database;
use eframe::egui::{self, Color32, Rect, Sense, Stroke, Vec2};
use std::sync::{Arc, Mutex};

pub struct ProteusApp {
    pub db: Option<Arc<Mutex<Database>>>,
    pub pid: String,
    pub pname: String,
    pub mode: Mode,
    pub layout: LayoutMode,
    pub project_doc: scene::ProjectDocument,
    pub editor_state: scene::EditorState,
    pub fns: Vec<UiFlowNode>,
    pub fes: Vec<(String, String, String)>,
    pub sel_fn: Option<String>,
    pub fdrag: Option<(String, Vec2, Vec2)>,
    pub palette_drag: Option<scene::NodeType>,
    pub viewport: Viewport2D,
    pub show_login: bool,
    pub auth: Option<String>,
    pub toast: Option<String>,
    pub tt: f32,
    pub flow_con: Option<String>,
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
}

fn app_dir() -> String {
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
            pid: "proj-1".into(),
            pname: "My Proteus".into(),
            mode: Mode::Designer,
            layout: LayoutMode::Free,
            project_doc: scene::create_dummy_document(),
            editor_state: scene::EditorState::new(),
            fns: vec![],
            fes: vec![],
            sel_fn: None,
            fdrag: None,
            palette_drag: None,
            viewport: Viewport2D::new(),
            show_login: false,
            auth: None,
            toast: None,
            tt: 0.,
            flow_con: None,
            contacts: vec![
                Contact {
                    id: "c1".into(),
                    name: "John Smith".into(),
                    email: "john@acme.com".into(),
                    phone: "555-0100".into(),
                    company: "Acme Inc".into(),
                    tags: vec!["VIP".into(), "tech".into()],
                    notes: vec![ContactNote {
                        id: "n1".into(),
                        text: "Initial meeting — interested in enterprise plan".into(),
                        created_at: Utc::now().to_rfc3339(),
                    }],
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
                Contact {
                    id: "c2".into(),
                    name: "Jane Doe".into(),
                    email: "jane@widgets.co".into(),
                    phone: "555-0200".into(),
                    company: "Widgets Co".into(),
                    tags: vec!["lead".into()],
                    notes: vec![],
                    created_at: Utc::now().to_rfc3339(),
                    updated_at: Utc::now().to_rfc3339(),
                },
                Contact {
                    id: "c3".into(),
                    name: "Bob Wilson".into(),
                    email: "bob@example.com".into(),
                    phone: "555-0300".into(),
                    company: "Example LLC".into(),
                    tags: vec!["partner".into(), "enterprise".into()],
                    notes: vec![ContactNote {
                        id: "n2".into(),
                        text: "Referred by John Smith".into(),
                        created_at: Utc::now().to_rfc3339(),
                    }],
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
                    title: "Enterprise License — Acme Inc".into(),
                    value: 12000.,
                    stage: "Proposal".into(),
                    contact_id: Some("c1".into()),
                    expected_close: "2026-08-15".into(),
                    notes: "Negotiating 3-year deal".into(),
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

    pub fn add_fn(&mut self, nt: &str, label: &str, extra: Option<serde_json::Value>) {
        let id = format!("fn{}", self.fns.len() + 1);
        let x = 100. + (self.fns.len() % 5) as f32 * 220.;
        let y = 80. + (self.fns.len() / 5) as f32 * 120.;
        self.fns.push(UiFlowNode { id, nt: nt.into(), label: label.into(), x, y, extra });
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

    pub fn spawn_designer_node_with_size(
        &mut self,
        nt: scene::NodeType,
        world_pos: (f32, f32),
        size: (f32, f32),
    ) -> String {
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

    pub fn save_project(&mut self) {
        let nodes_list: Vec<serde_json::Value> = self.fns.iter().map(|n| serde_json::json!({"id": n.id, "type": n.nt, "label": n.label, "x": n.x, "y": n.y, "extra": n.extra})).collect();
        let edges_list: Vec<serde_json::Value> = self.fes.iter().map(|(id, s, t)| serde_json::json!({"id": id, "source": s, "target": t})).collect();
        let flows_json = serde_json::to_string(&serde_json::json!({"nodes": nodes_list, "edges": edges_list})).unwrap_or_else(|_| "{}".into());
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
                }
            }
        }
    }
}

impl eframe::App for ProteusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.loaded {
            self.loaded = true;
            self.load_project();
        }

        if self.toast.is_some() {
            self.tt += ctx.input(|i| i.unstable_dt);
            if self.tt > 3.0 {
                self.toast = None;
            }
        }

        if !self._style_set {
            theme::configure_egui_style(ctx);
            self._style_set = true;
        }

        // ── TOP BARS ──
        components::top_bar::show(self, ctx);
        components::mode_bar::show(self, ctx);

        // ── LEFT SIDEBAR (Ultra-slim 38px toolbar in Designer like Affinity) ──
        let (left_w, left_margin) = if self.mode == Mode::Designer {
            (38., egui::Margin::symmetric(3, 8))
        } else {
            (180., egui::Margin::same(12))
        };
        egui::SidePanel::left("pal")
            .resizable(false)
            .exact_width(left_w)
            .frame(egui::Frame { fill: theme::PANEL, inner_margin: left_margin, ..Default::default() })
            .show(ctx, |ui| {
                match self.mode {
                    Mode::Designer => views::designer::show_left(self, ui),
                    Mode::Flow => views::flow_legacy::show_left(self, ui),
                    Mode::FlowBuilder => views::flow_builder::show_left(self, ui),
                    Mode::Contacts => views::contacts::show_left(self, ui),
                    Mode::Pipeline => views::pipeline::show_left(self, ui),
                    Mode::Tasks => views::tasks::show_left(self, ui),
                    Mode::Studio => views::studio::show_left(self, ui),
                    Mode::Play => views::play::show_left(self, ui),
                    Mode::DataViewer => views::data_viewer::show_left(self, ui),
                }
            });

        // ── RIGHT PROPERTIES PANEL ──
        let right_title = match self.mode {
            Mode::Designer => "PROPERTIES",
            Mode::Play => "PLAY MODE",
            Mode::DataViewer => "DATA VIEWER",
            Mode::FlowBuilder | Mode::Flow => "FLOW INFO",
            Mode::Contacts => "CONTACT",
            Mode::Pipeline => "DEAL DETAIL",
            Mode::Studio => "LAYERS",
            Mode::Tasks => "TASK DETAIL",
        };
        egui::SidePanel::right("prop")
            .resizable(false)
            .default_width(240.)
            .min_width(200.)
            .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::same(12), ..Default::default() })
            .show(ctx, |ui| {
                ui.add_space(6.);
                ui.label(egui::RichText::new(right_title).size(9.).color(theme::TEXT_DIM));
                ui.add_space(4.);
                match self.mode {
                    Mode::Designer => views::designer::show_right(self, ui),
                    Mode::Flow => views::flow_legacy::show_right(self, ui),
                    Mode::FlowBuilder => views::flow_builder::show_right(self, ui),
                    Mode::Contacts => views::contacts::show_right(self, ui),
                    Mode::Pipeline => views::pipeline::show_right(self, ui),
                    Mode::Tasks => views::tasks::show_right(self, ui),
                    Mode::Studio => views::studio::show_right(self, ui),
                    Mode::Play => views::play::show_right(self, ui),
                    Mode::DataViewer => views::data_viewer::show_right(self, ui),
                }
            });

        // ── DEVICE TOOLBAR (Designer only, toggleable) ──
        if self.show_device_toolbar {
            components::device_bar::show(self, ctx);
        }

        // ── CENTRAL CANVAS ──
        egui::CentralPanel::default()
            .frame(egui::Frame { fill: theme::BG, ..Default::default() })
            .show(ctx, |ui| {
                let (resp, pnt) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
                let r = resp.rect;
                pnt.rect_filled(r, 0, theme::BG);

                // Grid background for non-designer modes
                if self.mode != Mode::Designer && self.mode != Mode::FlowBuilder {
                    if self.layout == LayoutMode::Grid {
                        let mut y = r.top();
                        while y <= r.bottom() {
                            pnt.line_segment([egui::pos2(r.left(), y), egui::pos2(r.right(), y)], Stroke::new(1., Color32::from_rgba_premultiplied(30, 30, 30, 255)));
                            y += GRID;
                        }
                        let mut x = r.left();
                        while x <= r.right() {
                            pnt.line_segment([egui::pos2(x, r.top()), egui::pos2(x, r.bottom())], Stroke::new(1., Color32::from_rgba_premultiplied(30, 30, 30, 255)));
                            x += GRID;
                        }
                    } else {
                        let mut y = r.top() + 12.;
                        while y < r.bottom() {
                            let mut x = r.left() + 12.;
                            while x < r.right() {
                                pnt.circle_filled(egui::pos2(x, y), 1., Color32::from_rgb(25, 25, 25));
                                x += 24.;
                            }
                            y += 24.;
                        }
                    }
                }

                ctx.request_repaint();

                let mpos = ui.input(|i| i.pointer.interact_pos());
                let mdown = ui.input(|i| i.pointer.any_down());
                let mup = ui.input(|i| i.pointer.any_released());

                match self.mode {
                    Mode::Contacts => views::contacts::show_central(self, &pnt, r, ui),
                    Mode::Pipeline => views::pipeline::show_central(self, &pnt, r, mpos, mup),
                    Mode::Tasks => views::tasks::show_central(self, &pnt, r, mpos, mup, ui),
                    Mode::Studio => views::studio::show_central(self, ctx, &pnt, r, mpos, mdown, mup),
                    Mode::Designer => views::designer::show_central(self, ctx, ui, &pnt, r, mpos, &resp),
                    Mode::Play => views::play::show_central(self, ui, &pnt, r),
                    Mode::DataViewer => views::data_viewer::show_central(self, ui),
                    Mode::FlowBuilder => views::flow_builder::show_central(self, ctx, ui, &pnt, r, mpos, mdown),
                    Mode::Flow => views::flow_legacy::show_central(self, &pnt, r, mpos, mdown, mup),
                }

                // Toast overlay
                if let Some(msg) = &self.toast {
                    let tr = Rect::from_min_size(egui::pos2(r.center().x - 120., r.bottom() - 40.), egui::vec2(240., 28.));
                    pnt.rect_filled(tr, 0, Color32::from_rgba_premultiplied(20, 20, 20, 220));
                    pnt.rect_stroke(tr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
                    pnt.text(tr.center(), egui::Align2::CENTER_CENTER, msg, egui::FontId::proportional(11.), theme::TEXT);
                }
            });

        // ── MODALS ──
        views::data_viewer::show_edit_modal(self, ctx);

        if self.show_login {
            egui::Window::new("Sign In")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Continue without signing in, or log in for sync.").size(11.).color(theme::TEXT));
                    ui.add_space(8.);
                    if ui.button(egui::RichText::new("Sign In (demo)").size(10.)).clicked() {
                        self.auth = Some("demo".into());
                        self.show_login = false;
                        self.toast("Signed in");
                    }
                    if ui.button(egui::RichText::new("Continue Offline").size(10.)).clicked() {
                        self.show_login = false;
                    }
                });
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Proteus - The Visual OS for Business",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1400., 900.])
                .with_min_inner_size([800., 500.]),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(ProteusApp::default()))),
    )
}
