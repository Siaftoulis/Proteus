mod data_viewer;
mod db;
mod flow;
mod flow_renderer;
mod inspector;
mod palette;
mod renderer;
mod scene;
mod storage;
use scene::CanvasEvent;

use chrono::Utc;
use crm_core::Database;
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// ── Windows 11 / Linear.app Hybrid Dark Theme ──
mod theme {
    use eframe::egui::Color32;
    pub const BG: Color32       = Color32::from_rgb(32, 32, 32);
    pub const PANEL: Color32    = Color32::from_rgb(43, 43, 43);
    pub const WIDGET_BG: Color32= Color32::from_rgb(50, 50, 50);
    pub const HOVER: Color32    = Color32::from_rgb(60, 60, 60);
    pub const ACTIVE: Color32   = Color32::from_rgb(70, 70, 70);
    pub const ELEVATED: Color32 = Color32::from_rgb(38, 38, 38);
    pub const BORDER: Color32   = Color32::from_rgb(62, 62, 62);
    pub const FOCUS: Color32    = Color32::from_rgb(80, 80, 80);
    pub const TEXT: Color32     = Color32::from_rgb(215, 215, 215);
    pub const TEXT_DIM: Color32 = Color32::from_rgb(130, 130, 130);
    pub const ACCENT: Color32      = Color32::from_rgb(79, 140, 237);
    pub const ACCENT_DIM: Color32   = Color32::from_rgb(40, 90, 170);
    pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(255, 152, 0);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(76, 175, 80);
    pub const ACCENT_RED: Color32 = Color32::from_rgb(229, 115, 115);
    pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(156, 136, 255);
    pub const SELECTED: Color32 = Color32::from_rgb(79, 140, 237);
    pub const HEADER_TRIGGER: Color32 = Color32::from_rgb(229, 115, 115);
    pub const HEADER_ACTION: Color32 = Color32::from_rgb(79, 140, 237);
    pub const HEADER_CONDITION: Color32 = Color32::from_rgb(255, 152, 0);
    pub const HEADER_GATE: Color32 = Color32::from_rgb(76, 175, 80);
}

const GRID: f32 = 20.0;

#[derive(Clone, PartialEq)]
enum Mode { Designer, Play, DataViewer, FlowBuilder, Flow, Contacts, Pipeline, Studio, Tasks }
#[derive(Clone, PartialEq)]
enum LayoutMode { Free, Grid }

#[derive(Clone, PartialEq)]
enum DevicePreset {
    DesktopHD, Desktop, Laptop, TabletLandscape, TabletPortrait, Phone, PhoneSmall, Custom(f32, f32),
}
impl DevicePreset {
    fn label(&self) -> &str {
        match self { Self::DesktopHD => "Desktop HD", Self::Desktop => "Desktop", Self::Laptop => "Laptop", Self::TabletLandscape => "Tablet (L)", Self::TabletPortrait => "Tablet (P)", Self::Phone => "Phone", Self::PhoneSmall => "Phone (S)", Self::Custom(_, _) => "Custom" }
    }
    fn size(&self) -> (f32, f32) {
        match self { Self::DesktopHD => (1920., 1080.), Self::Desktop => (1440., 900.), Self::Laptop => (1366., 768.), Self::TabletLandscape => (1024., 768.), Self::TabletPortrait => (768., 1024.), Self::Phone => (375., 812.), Self::PhoneSmall => (320., 568.), Self::Custom(w, h) => (*w, *h) }
    }
    fn icon(&self) -> &str {
        match self { Self::DesktopHD | Self::Desktop | Self::Laptop => "🖥", Self::TabletLandscape | Self::TabletPortrait => "📱", Self::Phone | Self::PhoneSmall => "📲", Self::Custom(_, _) => "⚙" }
    }
}

// ── Studio (Design Tool) types ──
#[derive(Clone, PartialEq)]
enum StudioTool { Select, Brush, Rectangle, Ellipse, Line, Text, Fill, Eraser, Picker }

#[derive(Clone, Serialize, Deserialize)]
struct StudioLayer {
    id: String, name: String,
    x: f32, y: f32, w: f32, h: f32, z: i32,
    visible: bool, opacity: f32, locked: bool,
    content: LayerContent,
}
#[derive(Clone, Serialize, Deserialize)]
enum LayerContent {
    Shape { kind: String, x: f32, y: f32, w: f32, h: f32,
            fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
            stroke_r: u8, stroke_g: u8, stroke_b: u8, stroke_a: u8, stroke_width: f32 },
    Stroke { points: Vec<[f32;2]>, r: u8, g: u8, b: u8, a: u8, size: f32 },
    Text { content: String, font_size: f32, r: u8, g: u8, b: u8, a: u8 },
    Image { data_base64: String },
}

const STUDIO_TOOLS: &[(StudioTool, &str, &str)] = &[
    (StudioTool::Select,    "V", "Select"),
    (StudioTool::Brush,     "B", "Brush"),
    (StudioTool::Rectangle, "R", "Rect"),
    (StudioTool::Ellipse,   "E", "Ellipse"),
    (StudioTool::Line,      "L", "Line"),
    (StudioTool::Text,      "T", "Text"),
    (StudioTool::Fill,      "G", "Fill"),
    (StudioTool::Eraser,    "X", "Eraser"),
    (StudioTool::Picker,    "I", "Picker"),
];

#[derive(Clone, Serialize, Deserialize)]
struct Contact {
    id: String, name: String, email: String, phone: String, company: String,
    tags: Vec<String>, notes: Vec<ContactNote>, created_at: String, updated_at: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct ContactNote { id: String, text: String, created_at: String }

#[derive(Clone, Serialize, Deserialize)]
struct Deal {
    id: String, title: String, value: f64, stage: String,
    contact_id: Option<String>, expected_close: String, notes: String,
    created_at: String, updated_at: String,
}

const KANBAN_STAGES: &[&str] = &["New","Contacted","Qualified","Proposal","Negotiation","Closed Won","Closed Lost"];

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum TaskStatus { Open, InProgress, Done, Cancelled }
#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum TaskPriority { Low, Medium, High }

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    id: String, title: String, description: String,
    contact_id: Option<String>, deal_id: Option<String>,
    assignee: String, due_date: String,
    status: TaskStatus, priority: TaskPriority,
    created_by: String, created_at: String, updated_at: String, completed_at: Option<String>,
}

// ── Designer v2 types ──
#[derive(Clone)]
struct Viewport2D {
    pan: egui::Vec2,
    zoom: f32,
}
impl Viewport2D {
    fn new() -> Self { Self { pan: Vec2::ZERO, zoom: 1.0 } }
    fn world_to_screen(&self, world: Pos2, canvas_origin: Pos2) -> Pos2 {
        egui::pos2(
            world.x * self.zoom + self.pan.x + canvas_origin.x,
            world.y * self.zoom + self.pan.y + canvas_origin.y,
        )
    }
    fn screen_to_world(&self, screen: Pos2, canvas_origin: Pos2) -> Pos2 {
        egui::pos2(
            (screen.x - self.pan.x - canvas_origin.x) / self.zoom,
            (screen.y - self.pan.y - canvas_origin.y) / self.zoom,
        )
    }
}

#[derive(Clone)]
struct UiFlowNode { id: String, nt: String, label: String, x: f32, y: f32, extra: Option<serde_json::Value> }

struct ProteusApp {
    db: Option<Arc<Mutex<Database>>>,
    pid: String, pname: String,
    mode: Mode, layout: LayoutMode,
    project_doc: scene::ProjectDocument,
    editor_state: scene::EditorState,
    fns: Vec<UiFlowNode>, fes: Vec<(String,String,String)>,
    sel_fn: Option<String>,
    fdrag: Option<(String, Vec2, Vec2)>,
    palette_drag: Option<scene::NodeType>,
    viewport: Viewport2D,
    show_login: bool, auth: Option<String>,
    toast: Option<String>, tt: f32,
    flow_con: Option<String>,
    loaded: bool,
    _style_set: bool,
    db_conn: Option<rusqlite::Connection>,
    form_state: std::collections::HashMap<String, String>,
    viewer_selected_entity: String,
    viewer_entity_input: String,
    viewer_records: Vec<(String, serde_json::Value)>,
    editing_record: Option<(String, serde_json::Value)>,
    flow_viewport: Viewport2D,
    selected_flow_node: Option<String>,
    flow_drag_active: bool,
    flow_wiring_source: Option<String>,
    flow_wiring_pos: Option<(f32, f32)>,
    flow_canvas_center: (f32, f32),
    active_play_page: Option<String>,
    play_viewport: Viewport2D,
    designer_selected_node: Option<String>,
    designer_drag_active: bool,
    designer_drag_offset: (f32, f32),
    spawn_counter: u32,
    device_preset: DevicePreset,
    show_device_toolbar: bool,

    // Contacts & Pipeline
    contacts: Vec<Contact>,
    sel_contact: Option<String>,
    search_query: String,
    new_note_text: String,
    deals: Vec<Deal>,
    sel_deal: Option<String>,
    editing_contact: Option<String>,
    edit_name: String, edit_email: String, edit_phone: String, edit_company: String, edit_tags: String,
    // Studio state
    studio_layers: Vec<StudioLayer>,
    studio_sel_layer: Option<usize>,
    studio_tool: StudioTool,
    studio_color: Color32,
    studio_color2: Color32,
    studio_brush_size: f32,
    studio_dragging: Option<(usize, egui::Pos2, egui::Pos2)>,
    studio_shape_start: Option<egui::Pos2>,
    studio_shape_current: Option<egui::Pos2>,
    studio_show_text_dialog: bool,
    studio_text_input: String,
    studio_show_color_popup: bool,
    studio_color_picker_target: u8, // 0=primary, 1=secondary
    _studio_show_tool_options: bool,
    // Tasks state
    tasks: Vec<Task>,
    sel_task: Option<String>,
    task_filter_status: Option<TaskStatus>,
    task_filter_priority: Option<TaskPriority>,

}

impl Default for ProteusApp {
    fn default() -> Self {
        let db_path = app_dir(); std::fs::create_dir_all(&db_path).ok();
        let db = Database::new(&format!("{}/crm.db", db_path)).ok().map(|d| Arc::new(Mutex::new(d)));
        let app = Self {
            db, pid: "proj-1".into(), pname: "My Proteus".into(),
            mode: Mode::Designer, layout: LayoutMode::Free,
            project_doc: scene::create_dummy_document(),
            editor_state: scene::EditorState::new(),
            fns: vec![], fes: vec![], sel_fn: None,
            fdrag: None, palette_drag: None,
            viewport: Viewport2D::new(),
            show_login: false, auth: None,
            toast: None, tt: 0.,
            flow_con: None,
            contacts: vec![
                Contact{id:"c1".into(),name:"John Smith".into(),email:"john@acme.com".into(),phone:"555-0100".into(),company:"Acme Inc".into(),tags:vec!["VIP".into(),"tech".into()],notes:vec![ContactNote{id:"n1".into(),text:"Initial meeting — interested in enterprise plan".into(),created_at:Utc::now().to_rfc3339()}],created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339()},
                Contact{id:"c2".into(),name:"Jane Doe".into(),email:"jane@widgets.co".into(),phone:"555-0200".into(),company:"Widgets Co".into(),tags:vec!["lead".into()],notes:vec![],created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339()},
                Contact{id:"c3".into(),name:"Bob Wilson".into(),email:"bob@example.com".into(),phone:"555-0300".into(),company:"Example LLC".into(),tags:vec!["partner".into(),"enterprise".into()],notes:vec![ContactNote{id:"n2".into(),text:"Referred by John Smith".into(),created_at:Utc::now().to_rfc3339()}],created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339()},
            ], sel_contact: None, search_query: String::new(), new_note_text: String::new(),
            deals: vec![
                Deal{id:"d1".into(),title:"Enterprise License — Acme Inc".into(),value:12000.,stage:"Proposal".into(),contact_id:Some("c1".into()),expected_close:"2026-08-15".into(),notes:"Negotiating 3-year deal".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339()},
                Deal{id:"d2".into(),title:"Widgets Co Partnership".into(),value:5000.,stage:"Qualified".into(),contact_id:Some("c2".into()),expected_close:"2026-09-01".into(),notes:"".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339()},
                Deal{id:"d3".into(),title:"Consulting — Example LLC".into(),value:3000.,stage:"New".into(),contact_id:Some("c3".into()),expected_close:"2026-07-30".into(),notes:"Initial inquiry".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339()},
            ], sel_deal: None,
            editing_contact: None, edit_name: String::new(), edit_email: String::new(), edit_phone: String::new(), edit_company: String::new(), edit_tags: String::new(),
            loaded: false, _style_set: false,
            db_conn: db::init_db(std::path::Path::new("data.db")).ok(),
            form_state: std::collections::HashMap::new(),
            viewer_selected_entity: String::new(),
            viewer_entity_input: String::new(),
            viewer_records: vec![],
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
            show_device_toolbar: true,
            tasks: vec![
                Task{id:"t1".into(),title:"Follow up with John Smith — enterprise pricing".into(),description:"Send updated proposal with 3-year discount".into(),contact_id:Some("c1".into()),deal_id:Some("d1".into()),assignee:"me".into(),due_date:"2026-07-10".into(),status:TaskStatus::InProgress,priority:TaskPriority::High,created_by:"me".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339(),completed_at:None},
                Task{id:"t2".into(),title:"Demo for Jane Doe — Widgets Co".into(),description:"Schedule Zoom demo, prepare custom deck".into(),contact_id:Some("c2".into()),deal_id:Some("d2".into()),assignee:"me".into(),due_date:"2026-07-12".into(),status:TaskStatus::Open,priority:TaskPriority::Medium,created_by:"me".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339(),completed_at:None},
                Task{id:"t3".into(),title:"Send contract to Bob Wilson".into(),description:"Draft and send consulting agreement".into(),contact_id:Some("c3".into()),deal_id:Some("d3".into()),assignee:"me".into(),due_date:"2026-07-15".into(),status:TaskStatus::Open,priority:TaskPriority::Low,created_by:"me".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339(),completed_at:None},
            ], sel_task: None, task_filter_status: None, task_filter_priority: None,
            studio_layers: vec![
                StudioLayer{id:"sl1".into(),name:"Background".into(),x:0.,y:0.,w:800.,h:600.,z:0,visible:true,opacity:1.,locked:false,content:LayerContent::Shape{kind:"rect".into(),x:0.,y:0.,w:800.,h:600.,fill_r:30,fill_g:30,fill_b:30,fill_a:255,stroke_r:0,stroke_g:0,stroke_b:0,stroke_a:0,stroke_width:0.}},
                StudioLayer{id:"sl2".into(),name:"Circle".into(),x:340.,y:200.,w:120.,h:120.,z:1,visible:true,opacity:0.8,locked:false,content:LayerContent::Shape{kind:"ellipse".into(),x:340.,y:200.,w:120.,h:120.,fill_r:52,fill_g:152,fill_b:219,fill_a:200,stroke_r:255,stroke_g:255,stroke_b:255,stroke_a:255,stroke_width:2.}},
                StudioLayer{id:"sl3".into(),name:"Text".into(),x:320.,y:360.,w:160.,h:40.,z:2,visible:true,opacity:1.,locked:false,content:LayerContent::Text{content:"Hello World".into(),font_size:24.,r:255,g:255,b:255,a:255}},
            ], studio_sel_layer: None, studio_tool: StudioTool::Select,
            studio_color: Color32::from_rgb(52,152,219), studio_color2: Color32::from_rgb(255,255,255),
            studio_brush_size: 4., studio_dragging: None, studio_shape_start: None, studio_shape_current: None,
            studio_show_text_dialog: false, studio_text_input: String::new(),
            studio_show_color_popup: false, studio_color_picker_target: 0, _studio_show_tool_options: false,
        };
        app
    }
}
fn app_dir() -> String { std::env::var("APPDATA").or_else(|_|std::env::var("HOME")).unwrap_or_else(|_|".".into()) + "/proteus" }

impl ProteusApp {
    fn add_fn(&mut self, nt: &str, label: &str, extra: Option<serde_json::Value>) {
        let id = format!("fn{}", self.fns.len()+1);
        let x = 100. + (self.fns.len()%5) as f32 * 220.;
        let y = 80. + (self.fns.len()/5) as f32 * 120.;
        self.fns.push(UiFlowNode{id,nt:nt.into(),label:label.into(),x,y,extra});
    }

    fn spawn_designer_node(&mut self, nt: scene::NodeType, world_pos: egui::Pos2) {
        self.spawn_counter += 1;
        let id = format!("node-{}", self.spawn_counter);
        let (w, h, name, node_type) = match nt {
            scene::NodeType::Frame => (200., 200., format!("Frame {}", self.spawn_counter), scene::NodeType::Frame),
            scene::NodeType::Text { .. } => (100., 30., format!("Text {}", self.spawn_counter), scene::NodeType::Text {
                content: "New Text".into(),
                font: scene::FontSpec { family: "Inter".into(), size: 14., weight: 400, color: scene::Rgba { r: 215, g: 215, b: 215, a: 255 } },
            }),
            scene::NodeType::TextInput { .. } => (200., 36., format!("Input {}", self.spawn_counter), scene::NodeType::TextInput {
                placeholder: "Type...".into(), field_type: scene::FieldType::Text, bound_entity: None, bound_field: None,
            }),
            scene::NodeType::Button { .. } => (120., 40., format!("Button {}", self.spawn_counter), scene::NodeType::Button {
                label: "Submit".into(), style: scene::ButtonStyle::Primary,
            }),
            scene::NodeType::Checkbox { .. } => (160., 24., format!("Check {}", self.spawn_counter), scene::NodeType::Checkbox {
                label: "Check me".into(), bound_entity: None, bound_field: None,
            }),
            scene::NodeType::Dropdown { .. } => (200., 36., format!("Dropdown {}", self.spawn_counter), scene::NodeType::Dropdown {
                options: vec!["Option 1".into()], multiple: false, bound_entity: None, bound_field: None,
            }),
            scene::NodeType::NumberField { .. } => (120., 36., format!("Number {}", self.spawn_counter), scene::NodeType::NumberField {
                min: None, max: None, step: 1.0, bound_entity: None, bound_field: None,
            }),
            scene::NodeType::Image { .. } => (120., 120., format!("Image {}", self.spawn_counter), scene::NodeType::Image {
                url: String::new(), fit: scene::ImageFit::Cover,
            }),
            scene::NodeType::Shape { .. } => (100., 100., format!("Shape {}", self.spawn_counter), scene::NodeType::Shape {
                kind: scene::ShapeKind::Rectangle,
            }),
            scene::NodeType::Page | scene::NodeType::Group => (200., 200., format!("Frame {}", self.spawn_counter), scene::NodeType::Frame),
        };
        let mut node = scene::Node::new(id.clone(), name, node_type);
        node.position = (world_pos.x - w / 2., world_pos.y - h / 2.);
        node.layout.width = scene::Sizing::Fixed(w);
        node.layout.height = scene::Sizing::Fixed(h);
        if let Err(e) = self.project_doc.add_node(node, None) {
            self.toast(format!("Spawn failed: {}", e));
        } else {
            self.designer_selected_node = Some(id.clone());
            self.editor_state.selected_node_ids = vec![id];
            self.toast("Node added");
        }
    }

    fn toast(&mut self, msg: impl Into<String>) { self.toast=Some(msg.into()); self.tt=0.; }

    fn save_project(&mut self) {
        let nodes_list: Vec<serde_json::Value> = self.fns.iter().map(|n| serde_json::json!({"id":n.id,"type":n.nt,"label":n.label,"x":n.x,"y":n.y,"extra":n.extra})).collect();
        let edges_list: Vec<serde_json::Value> = self.fes.iter().map(|(id,s,t)| serde_json::json!({"id":id,"source":s,"target":t})).collect();
        let flows_json = serde_json::to_string(&serde_json::json!({"nodes":nodes_list,"edges":edges_list})).unwrap_or_else(|_|"{}".into());
        let contacts_json = serde_json::to_string(&self.contacts).unwrap_or_else(|_|"[]".into());
        let deals_json = serde_json::to_string(&self.deals).unwrap_or_else(|_|"[]".into());
        let tasks_json = serde_json::to_string(&self.tasks).unwrap_or_else(|_|"[]".into());
        let combined = serde_json::json!({"contacts":serde_json::from_str::<serde_json::Value>(&contacts_json).unwrap_or_default(),"deals":serde_json::from_str::<serde_json::Value>(&deals_json).unwrap_or_default(),"tasks":serde_json::from_str::<serde_json::Value>(&tasks_json).unwrap_or_default()});
        let combined_str = serde_json::to_string(&combined).unwrap_or_else(|_|"[]".into());
        let pid = self.pid.clone(); let pn = self.pname.clone();
        let saved = self.db.as_ref().and_then(|d| {
            d.lock().ok().and_then(|db| {
                db.upsert_project(&crm_core::Project{
                    id: pid, name: pn,
                    widgets: combined_str,
                    flows: flows_json,
                    created_at: Utc::now().to_rfc3339(), updated_at: Utc::now().to_rfc3339(),
                }).ok()
            })
        }).is_some();
        if saved { self.toast("Project saved"); } else { self.toast("Cannot save (no DB)"); }
    }

    fn load_project(&mut self) {
        if let Some(d) = &self.db {
            if let Ok(db) = d.lock() {
                if let Ok(proj) = db.load_project(&self.pid) {
                    self.pname = proj.name;
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&proj.widgets) {
                        if let Some(contacts_arr) = data["contacts"].as_array() {
                            self.contacts = contacts_arr.iter().map(|c| Contact{
                                id: c["id"].as_str().unwrap_or("?").into(),
                                name: c["name"].as_str().unwrap_or("").into(),
                                email: c["email"].as_str().unwrap_or("").into(),
                                phone: c["phone"].as_str().unwrap_or("").into(),
                                company: c["company"].as_str().unwrap_or("").into(),
                                tags: c["tags"].as_array().map(|a| a.iter().filter_map(|v|v.as_str().map(String::from)).collect()).unwrap_or_default(),
                                notes: c["notes"].as_array().map(|a| a.iter().map(|n| ContactNote{
                                    id: n["id"].as_str().unwrap_or("?").into(),
                                    text: n["text"].as_str().unwrap_or("").into(),
                                    created_at: n["created_at"].as_str().unwrap_or("").into(),
                                }).collect()).unwrap_or_default(),
                                created_at: c["created_at"].as_str().unwrap_or("").into(),
                                updated_at: c["updated_at"].as_str().unwrap_or("").into(),
                            }).collect();
                        }
                        if let Some(deals_arr) = data["deals"].as_array() {
                            self.deals = deals_arr.iter().map(|d| Deal{
                                id: d["id"].as_str().unwrap_or("?").into(),
                                title: d["title"].as_str().unwrap_or("").into(),
                                value: d["value"].as_f64().unwrap_or(0.),
                                stage: d["stage"].as_str().unwrap_or("New").into(),
                                contact_id: d["contact_id"].as_str().map(String::from),
                                expected_close: d["expected_close"].as_str().unwrap_or("").into(),
                                notes: d["notes"].as_str().unwrap_or("").into(),
                                created_at: d["created_at"].as_str().unwrap_or("").into(),
                                updated_at: d["updated_at"].as_str().unwrap_or("").into(),
                            }).collect();
                        }
                        if let Some(tasks_arr) = data["tasks"].as_array() {
                            self.tasks = tasks_arr.iter().map(|t| Task{
                                id: t["id"].as_str().unwrap_or("?").into(),
                                title: t["title"].as_str().unwrap_or("").into(),
                                description: t["description"].as_str().unwrap_or("").into(),
                                contact_id: t["contact_id"].as_str().map(String::from),
                                deal_id: t["deal_id"].as_str().map(String::from),
                                assignee: t["assignee"].as_str().unwrap_or("").into(),
                                due_date: t["due_date"].as_str().unwrap_or("").into(),
                                status: serde_json::from_value(t["status"].clone()).unwrap_or(TaskStatus::Open),
                                priority: serde_json::from_value(t["priority"].clone()).unwrap_or(TaskPriority::Medium),
                                created_by: t["created_by"].as_str().unwrap_or("").into(),
                                created_at: t["created_at"].as_str().unwrap_or("").into(),
                                updated_at: t["updated_at"].as_str().unwrap_or("").into(),
                                completed_at: t["completed_at"].as_str().map(String::from),
                            }).collect();
                        }
                    }
                    if let Ok(flow_val) = serde_json::from_str::<serde_json::Value>(&proj.flows) {
                        if let Some(nodes) = flow_val["nodes"].as_array() {
                            self.fns = nodes.iter().map(|n| UiFlowNode{
                                id: n["id"].as_str().unwrap_or("?").into(),
                                nt: n["type"].as_str().unwrap_or("trigger").into(),
                                label: n["label"].as_str().unwrap_or("").into(),
                                x: n["x"].as_f64().unwrap_or(0.) as f32,
                                y: n["y"].as_f64().unwrap_or(0.) as f32,
                                extra: n.get("extra").cloned(),
                            }).collect();
                        }
                        if let Some(edges) = flow_val["edges"].as_array() {
                            self.fes = edges.iter().map(|e| (
                                e["id"].as_str().unwrap_or("?").into(),
                                e["source"].as_str().unwrap_or("?").into(),
                                e["target"].as_str().unwrap_or("?").into(),
                            )).collect();
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for ProteusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-load saved project on first frame
        if !self.loaded { self.loaded = true; self.load_project(); }

        // Toast
        if self.toast.is_some() { self.tt += ctx.input(|i| i.unstable_dt); if self.tt > 3.0 { self.toast = None; } }

        // Win11 / Linear.app style — set once
        if !self._style_set {
            configure_egui_style(ctx);
            self._style_set = true;
        }

        // ── COMMAND BAR (top) ──
        egui::TopBottomPanel::top("cmd_bar")
            .min_height(44.)
            .resizable(false)
            .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::symmetric(12, 8), ..Default::default() })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("PROTEUS").size(16.).color(theme::ACCENT).strong());
                    ui.label(egui::RichText::new("Workspace").size(10.).color(theme::TEXT_DIM).strong());
                    ui.add_space(12.); ui.separator(); ui.add_space(12.);
                    ui.label(egui::RichText::new(format!("📂 {}", self.pname)).size(13.).color(theme::TEXT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(u) = &self.auth { ui.label(egui::RichText::new(format!("👤 {}",u)).size(12.).color(theme::TEXT_DIM)); }
                        else if ui.add(egui::Button::new("Sign In").min_size(egui::vec2(75.,28.))).clicked() { self.show_login = true; }
                        ui.add_space(8.); ui.separator(); ui.add_space(8.);
                        if ui.add(egui::Button::new("📥 Import").min_size(egui::vec2(75.,28.))).clicked() { self.toast("Import (todo)"); }
                        if ui.add(egui::Button::new("🔄 Load").min_size(egui::vec2(70.,28.))).clicked() { self.load_project(); self.toast("Loaded ✓"); }
                        if ui.add(egui::Button::new(egui::RichText::new("💾 Save Project").strong()).fill(theme::ACCENT).min_size(egui::vec2(110.,28.))).clicked() { self.save_project(); }
                        ui.add_space(4.); ui.separator(); ui.add_space(4.);
                        if ui.add(egui::Button::new("📂 Arena Load").min_size(egui::vec2(85.,28.))).clicked() {
                            match storage::load_project(std::path::Path::new("proteus_design.json")) {
                                Ok(doc) => { self.project_doc = doc; self.editor_state.selected_node_ids.clear(); self.toast("Arena loaded ✓"); }
                                Err(e) => { self.toast(&format!("Load failed: {}", e)); }
                            }
                        }
                        if ui.add(egui::Button::new(egui::RichText::new("💾 Arena Save").strong()).fill(theme::ACCENT_DIM).min_size(egui::vec2(85.,28.))).clicked() {
                            match storage::save_project(&self.project_doc, std::path::Path::new("proteus_design.json")) {
                                Ok(()) => self.toast("Arena saved ✓"),
                                Err(e) => self.toast(&format!("Save failed: {}", e)),
                            }
                        }
                        ui.add_space(4.); ui.separator(); ui.add_space(4.);
                        if ui.add(egui::Button::new(egui::RichText::new("🔒 Save Secure").strong()).fill(theme::ACCENT_ORANGE).min_size(egui::vec2(100.,28.))).clicked() {
                            match storage::save_project_secure(&self.project_doc, std::path::Path::new("workspace.proteus")) {
                                Ok(()) => self.toast("Secure saved ✓"),
                                Err(e) => self.toast(&format!("Secure save failed: {}", e)),
                            }
                        }
                        if ui.add(egui::Button::new("🔓 Load Secure").min_size(egui::vec2(95.,28.))).clicked() {
                            match storage::load_project_secure(std::path::Path::new("workspace.proteus")) {
                                Ok(doc) => { self.project_doc = doc; self.editor_state.selected_node_ids.clear(); self.toast("Secure loaded ✓"); }
                                Err(e) => self.toast(&format!("Secure load failed: {}", e)),
                            }
                        }
                        if ui.selectable_label(self.layout==LayoutMode::Grid, "🌐 Grid").clicked() { self.layout = if self.layout==LayoutMode::Grid { LayoutMode::Free } else { LayoutMode::Grid }; }
                    });
                });
            });
        // ── MODE SWITCHER ──
        egui::TopBottomPanel::top("mode_switcher")
            .min_height(34.)
            .resizable(false)
            .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::symmetric(12, 4), ..Default::default() })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let modes: &[(Mode, &str, &str)] = &[
                        (Mode::Designer,  "◇", "Design"),
                        (Mode::Play,      "▶", "Play"),
                        (Mode::DataViewer,"🗄", "Data"),
                        (Mode::FlowBuilder,"⚡", "Flow"),
                        (Mode::Contacts,  "◎", "Contacts"),
                        (Mode::Pipeline,  "▤", "Pipeline"),
                        (Mode::Tasks,     "☰", "Tasks"),
                        (Mode::Studio,    "✦", "Studio"),
                    ];
                    for (mode, icon, label) in modes {
                        let is_active = self.mode == *mode;
                        let clicked = ui.add(egui::Button::new(egui::RichText::new(format!("{}  {}", icon, label)).size(12.).color(if is_active{theme::TEXT}else{theme::TEXT_DIM}))
                            .fill(if is_active{theme::ELEVATED}else{theme::PANEL})
                            .stroke(if is_active{egui::Stroke::new(1.,theme::ACCENT)}else{egui::Stroke::NONE})
                            .min_size(egui::vec2(100.,26.))).clicked();
                        if clicked {
                            if *mode == Mode::Play && self.mode != Mode::Play {
                                self.form_state.clear();
                                self.editor_state.selected_node_ids.clear();
                                self.active_play_page = self.project_doc.root_node_ids.first().cloned();
                                self.play_viewport = Viewport2D::new();
                            }
                            if *mode == Mode::Designer && self.mode != Mode::Designer {
                                // Reset designer viewport
                            }
                            if *mode == Mode::DataViewer && self.mode != Mode::DataViewer {
                                self.viewer_selected_entity.clear();
                                self.viewer_entity_input.clear();
                                self.viewer_records.clear();
                            }
                            self.mode = mode.clone();
                        }
                    }
                });
            });

        // ── LEFT SIDEBAR ──
        egui::SidePanel::left("pal").resizable(false).default_width(180.).min_width(160.)
            .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::same(12), ..Default::default() })
            .show(ctx, |ui| {
            match self.mode {
                Mode::Designer => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("PAGES").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(2.);
                    for root_id in &self.project_doc.root_node_ids.clone() {
                        let name = self.project_doc.nodes.get(root_id)
                            .map(|n| format!("{}", n.name))
                            .unwrap_or_else(|| root_id.clone());
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("📄").size(10.));
                            ui.label(egui::RichText::new(name).size(10.).color(theme::TEXT));
                            if ui.add(egui::Button::new("✕").min_size(egui::vec2(16., 16.))).clicked() {
                                self.project_doc.delete_node(root_id);
                                if self.designer_selected_node.as_deref() == Some(root_id) {
                                    self.designer_selected_node = None;
                                    self.editor_state.selected_node_ids.clear();
                                }
                            }
                        });
                    }
                    if ui.add(egui::Button::new("+ Add Page").min_size(egui::vec2(ui.available_width(), 20.))).clicked() {
                        let page_num = self.project_doc.root_node_ids.len() + 1;
                        let page_id = format!("page-{}", page_num);
                        let page = scene::Node {
                            id: page_id.clone(),
                            name: format!("Page {}", page_num),
                            node_type: scene::NodeType::Frame,
                            parent_id: None,
                            children_ids: vec![],
                            position: (40., 60.),
                            visible: true,
                            locked: false,
                            z: 0,
                            styling: scene::Styling::default(),
                            style: scene::NodeStyle::default(),
                            layout: scene::Layout {
                                width: scene::Sizing::Fixed(500.),
                                height: scene::Sizing::Fixed(400.),
                                ..Default::default()
                            },
                        };
                        let _ = self.project_doc.add_node(page, None);
                        // Navigate to the new page in Play mode
                        self.toast(&format!("Page '{}' added", page_id));
                    }
                    ui.add_space(8.);
                    ui.separator();
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("COMPONENTS").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(4.);
                    let pal_nt = palette::draw_palette(ui);
                    if let Some(nt) = pal_nt {
                        self.palette_drag = Some(nt);
                    }
                }
                Mode::Flow => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("NODES").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(4.);
                    let nodes: &[(&str, &str, &str, Color32, Option<serde_json::Value>)] = &[
                        (">", "On Click", "trigger", theme::HEADER_TRIGGER, None),
                        (">", "Create Record", "action", theme::HEADER_ACTION, None),
                        (">", "Update Record", "action", theme::HEADER_ACTION, None),
                        (">", "Send Email", "action", theme::HEADER_ACTION, None),
                        (">", "API Call", "action", theme::HEADER_ACTION, None),
                        ("?", "Check Field", "condition", theme::HEADER_CONDITION, None),
                        ("?", "Compare Dates", "condition", theme::HEADER_CONDITION, None),
                        ("&", "AND Gate", "logicalGate", theme::HEADER_GATE, Some(serde_json::json!({"gateType":"and"}))),
                        ("|", "OR Gate", "logicalGate", theme::HEADER_GATE, Some(serde_json::json!({"gateType":"or"}))),
                    ];
                    for (icon, label, nt, color, extra) in nodes {
                        if ui.add(
                            egui::Button::new(egui::RichText::new(format!("{} {}", icon, label)).size(11.).color(*color))
                                .fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 24.))
                        ).clicked() {
                            self.add_fn(nt, label, extra.clone());
                        }
                    }
                }
                Mode::Contacts => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("CONTACTS").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search...").desired_width(ui.available_width()));
                    ui.add_space(4.);
                    if ui.add(egui::Button::new(egui::RichText::new("+ New Contact").size(10.).color(theme::ACCENT_GREEN)).fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 22.))).clicked() {
                        let id = format!("c{}", self.contacts.len()+1);
                        self.contacts.push(Contact{
                            id: id.clone(), name: "New Contact".into(), email: String::new(), phone: String::new(), company: String::new(),
                            tags: vec![], notes: vec![], created_at: Utc::now().to_rfc3339(), updated_at: Utc::now().to_rfc3339(),
                        });
                        self.sel_contact = Some(id.clone());
                        self.editing_contact = Some(id);
                        self.edit_name = "New Contact".into(); self.edit_email.clear(); self.edit_phone.clear(); self.edit_company.clear(); self.edit_tags.clear();
                    }
                    ui.add_space(4.);
                    let q = self.search_query.to_lowercase();
                    let filtered: Vec<usize> = self.contacts.iter().enumerate().filter(|(_,c)| q.is_empty() || c.name.to_lowercase().contains(&q) || c.email.to_lowercase().contains(&q) || c.company.to_lowercase().contains(&q)).map(|(i,_)|i).collect();
                    let scroll = egui::ScrollArea::vertical().max_height(ui.available_height());
                    scroll.show(ui, |ui| {
                        for &i in &filtered {
                            let c = &self.contacts[i];
                            let sel = self.sel_contact.as_ref() == Some(&c.id);
                            let resp = ui.add(
                                egui::Button::new(egui::RichText::new(format!("{}\n{}", c.name, c.email)).size(10.).color(if sel{theme::ACCENT}else{theme::TEXT}))
                                    .fill(if sel{Color32::from_rgb(35,35,50)}else{theme::WIDGET_BG})
                                    .min_size(egui::vec2(ui.available_width(), 36.))
                            );
                            if resp.clicked() { self.sel_contact = Some(c.id.clone()); self.editing_contact = None; }
                        }
                    });
                }
                Mode::Pipeline => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("DEALS").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new(format!("Total: {}", self.deals.len())).size(10.).color(theme::TEXT_DIM));
                    ui.add_space(4.);
                    if ui.add(egui::Button::new(egui::RichText::new("+ New Deal").size(10.).color(theme::ACCENT_GREEN)).fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 22.))).clicked() {
                        let id = format!("d{}", self.deals.len()+1);
                        self.deals.push(Deal{
                            id: id.clone(), title: "New Deal".into(), value: 0.0, stage: "New".into(),
                            contact_id: None, expected_close: String::new(), notes: String::new(),
                            created_at: Utc::now().to_rfc3339(), updated_at: Utc::now().to_rfc3339(),
                        });
                        self.sel_deal = Some(id);
                    }
                    ui.add_space(4.);
                    let scroll = egui::ScrollArea::vertical().max_height(ui.available_height());
                    scroll.show(ui, |ui| {
                        for d in &self.deals {
                            let sel = self.sel_deal.as_ref() == Some(&d.id);
                            let resp = ui.add(
                                egui::Button::new(egui::RichText::new(format!("{}\n${:.0} — {}", d.title, d.value, d.stage)).size(10.).color(if sel{theme::ACCENT}else{theme::TEXT}))
                                    .fill(if sel{Color32::from_rgb(35,35,50)}else{theme::WIDGET_BG})
                                    .min_size(egui::vec2(ui.available_width(), 36.))
                            );
                            if resp.clicked() { self.sel_deal = Some(d.id.clone()); }
                        }
                    });
                }
                Mode::Tasks => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("TASKS").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(2.);
                    if ui.add(egui::Button::new(egui::RichText::new("+ New Task").size(10.).color(theme::ACCENT_GREEN)).fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(),22.))).clicked() {
                        let id = format!("t{}",self.tasks.len()+1);
                        self.tasks.push(Task{
                            id:id.clone(),title:"New Task".into(),description:String::new(),
                            contact_id:None,deal_id:None,assignee:"me".into(),
                            due_date:(Utc::now()+chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
                            status:TaskStatus::Open,priority:TaskPriority::Medium,
                            created_by:"me".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339(),completed_at:None,
                        });
                        self.sel_task = Some(id);
                    }
                    ui.add_space(4.);
                    // Filter: status
                    ui.horizontal(|ui|{
                        ui.label(egui::RichText::new("Status:").size(9.).color(theme::TEXT_DIM));
                        if ui.selectable_label(self.task_filter_status.is_none(),"All").clicked() { self.task_filter_status=None; }
                        for s in [TaskStatus::Open,TaskStatus::InProgress,TaskStatus::Done] {
                            if ui.selectable_label(self.task_filter_status.as_ref()==Some(&s), task_status_label(&s)).clicked() { self.task_filter_status=Some(s); }
                        }
                    });
                    ui.add_space(2.);
                    // Filter: priority
                    ui.horizontal(|ui|{
                        ui.label(egui::RichText::new("Priority:").size(9.).color(theme::TEXT_DIM));
                        if ui.selectable_label(self.task_filter_priority.is_none(),"All").clicked() { self.task_filter_priority=None; }
                        for p in [TaskPriority::High,TaskPriority::Medium,TaskPriority::Low] {
                            if ui.selectable_label(self.task_filter_priority.as_ref()==Some(&p), task_priority_label(&p)).clicked() { self.task_filter_priority=Some(p); }
                        }
                    });
                    ui.add_space(4.);
                    let filtered: Vec<usize> = self.tasks.iter().enumerate().filter(|(_,t)| {
                        let ok_status = self.task_filter_status.as_ref().map_or(true, |s| t.status==*s);
                        let ok_priority = self.task_filter_priority.as_ref().map_or(true, |p| t.priority==*p);
                        ok_status && ok_priority
                    }).map(|(i,_)|i).collect();
                    let scroll = egui::ScrollArea::vertical().max_height(ui.available_height()-120.);
                    scroll.show(ui, |ui| {
                        for &i in &filtered {
                            let t = &self.tasks[i];
                            let sel = self.sel_task.as_ref() == Some(&t.id);
                            let color = task_priority_color(&t.priority);
                            let label = format!("{}  {}  {}", task_status_icon(&t.status), t.title, if t.due_date.len()>=10{&t.due_date[..10]}else{""});
                            if ui.add(egui::Button::new(egui::RichText::new(&label).size(10.).color(if sel{theme::ACCENT}else{color})).fill(if sel{Color32::from_rgb(35,35,50)}else{theme::WIDGET_BG}).min_size(egui::vec2(ui.available_width(),20.))).clicked() {
                                self.sel_task = Some(t.id.clone());
                            }
                        }
                    });
                }
                Mode::DataViewer => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("🗄 DATA VIEWER").size(9.).color(theme::ACCENT));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Browse submitted records from the local SQLite database.").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("Type an entity name (e.g. \"contacts\") and click Load.").size(9.).color(theme::TEXT_DIM));
                }
                Mode::FlowBuilder => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("⚡ FLOW BUILDER").size(9.).color(theme::ACCENT));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Add Nodes:").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(2.);
                    let kinds: &[(&str, crate::flow::FlowNodeKind)] = &[
                        ("+ Trigger Click", crate::flow::FlowNodeKind::TriggerClick { target_node_id: String::new() }),
                        ("+ Navigate To",  crate::flow::FlowNodeKind::NavigateTo { page_id: String::new() }),
                        ("+ Save To DB",   crate::flow::FlowNodeKind::SaveToDatabase { entity: String::new() }),
                    ];
                    for (label, kind_tpl) in kinds {
                        if ui.add(egui::Button::new(egui::RichText::new(*label).size(10.).color(theme::ACCENT_GREEN))
                            .fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 22.))).clicked()
                        {
                            let id = uuid::Uuid::new_v4().to_string();
                            let kind = kind_tpl.clone();
                            let pos = if self.flow_canvas_center == (100., 100.) {
                                (300. + self.project_doc.flow_graph.nodes.len() as f32 * 40., 150.)
                            } else {
                                self.flow_canvas_center
                            };
                            self.project_doc.flow_graph.nodes.insert(id.clone(), crate::flow::FlowNode {
                                id: id.clone(), kind, position: pos,
                            });
                            self.selected_flow_node = Some(id.clone());
                            self.toast("Flow node added");
                        }
                    }
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("Nodes in graph:").size(9.).color(theme::TEXT_DIM));
                    let count = self.project_doc.flow_graph.nodes.len();
                    ui.label(egui::RichText::new(count.to_string()).size(11.).color(theme::ACCENT));
                }
                Mode::Play => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("▶ PLAY MODE").size(9.).color(theme::ACCENT));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Fill in the form on the canvas, then click a Submit button.").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("Form fields with data-binding will be saved to the local SQLite DB.").size(9.).color(theme::TEXT_DIM));
                }
                Mode::Studio => {
                    ui.add_space(6.);
                    ui.label(egui::RichText::new("TOOLS").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(4.);
                    for (tool, icon, _) in STUDIO_TOOLS {
                        let sel = self.studio_tool == *tool;
                        if ui.add(
                            egui::Button::new(egui::RichText::new(format!(" {}  {}", icon, tool_name(tool))).size(11.).color(if sel{Color32::BLACK}else{theme::TEXT}))
                                .fill(if sel{theme::ACCENT}else{theme::WIDGET_BG})
                                .min_size(egui::vec2(ui.available_width(), 22.))
                        ).clicked() { self.studio_tool = tool.clone(); }
                    }
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("COLOR").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(2.);
                    // Simple color display
                    let cr = ui.allocate_exact_size(egui::vec2(20., 20.), egui::Sense::click());
                    ui.painter().rect_filled(cr.0, 0, self.studio_color);
                    ui.painter().rect_stroke(cr.0, 0, Stroke::new(1., theme::BORDER), egui::StrokeKind::Outside);
                    ui.horizontal(|ui| {
                        ui.add_space(24.);
                        if ui.button(egui::RichText::new("Pick").size(9.)).clicked() {
                            self.studio_show_color_popup = true; self.studio_color_picker_target = 0;
                        }
                    });
                    ui.add_space(2.);
                    ui.label(egui::RichText::new("SIZE").size(9.).color(theme::TEXT_DIM));
                    ui.add(egui::Slider::new(&mut self.studio_brush_size, 1. ..=100.).text("px").text_color(theme::TEXT_DIM));
                }
            }
        });

        // ── RIGHT PROPERTIES ──
        let right_title = match self.mode {
            Mode::Designer => "PROPERTIES",
            Mode::Play => "PLAY MODE",
            Mode::DataViewer => "DATA VIEWER",
            Mode::FlowBuilder => "FLOW INFO",
            Mode::Flow => "FLOW INFO",
            Mode::Contacts => "CONTACT",
            Mode::Pipeline => "DEAL DETAIL",
            Mode::Studio => "LAYERS",
            Mode::Tasks => "TASK DETAIL",
        };
        egui::SidePanel::right("prop").resizable(false).default_width(240.).min_width(200.)
            .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::same(12), ..Default::default() })
            .show(ctx, |ui| {
            ui.add_space(6.);
            ui.label(egui::RichText::new(right_title).size(9.).color(theme::TEXT_DIM));
            ui.add_space(4.);
            match self.mode {
                Mode::Designer => {
                    // ── Arena inspector (only) ──
                    if !self.editor_state.selected_node_ids.is_empty() {
                        let events = inspector::draw_inspector(ui, &self.project_doc, &self.editor_state);
                        for ev in events {
                            match ev {
                                scene::CanvasEvent::NodeModified { id, update } => {
                                    let _ = self.project_doc.update_node(&id, update);
                                }
                                scene::CanvasEvent::DeleteNode { id } => {
                                    self.project_doc.delete_node(&id);
                                    self.designer_selected_node = None;
                                    self.editor_state.selected_node_ids.clear();
                                    self.toast("Node deleted");
                                }
                                _ => {}
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new("No selection").size(10.).color(theme::TEXT_DIM));
                        ui.label(egui::RichText::new("Click an element on the canvas").size(8.).color(theme::TEXT_DIM));
                    }
                }
                Mode::Flow => {
                    if let Some(id) = &self.sel_fn {
                        if let Some(n) = self.fns.iter().find(|n| n.id == *id) {
                            ui.label(egui::RichText::new(&n.nt).size(10.).color(theme::ACCENT));
                            ui.label(egui::RichText::new(&n.label).size(11.).color(theme::TEXT));
                        }
                    } else {
                        ui.label(egui::RichText::new("No selection").size(10.).color(theme::TEXT_DIM));
                    }
                }
                Mode::Contacts => {
                    let sel_id = self.sel_contact.clone();
                    let del_clicked = if let Some(ref id) = sel_id {
                        if let Some(c) = self.contacts.iter_mut().find(|c| &c.id == id) {
                            let editing = self.editing_contact.as_deref() == Some(&c.id);
                            if ui.button(egui::RichText::new(if editing{"Done Editing"}else{"Edit"}).size(10.).color(theme::ACCENT)).clicked() {
                                if editing { self.editing_contact = None; c.updated_at = Utc::now().to_rfc3339(); }
                                else { self.editing_contact = Some(c.id.clone()); self.edit_name=c.name.clone(); self.edit_email=c.email.clone(); self.edit_phone=c.phone.clone(); self.edit_company=c.company.clone(); self.edit_tags=c.tags.join(", "); }
                            }
                            let ret = ui.button(egui::RichText::new("Delete").size(10.).color(theme::ACCENT_RED)).clicked();
                            ui.add_space(4.);
                            if editing {
                                ui.horizontal(|ui|{ui.label("Email:"); ui.text_edit_singleline(&mut self.edit_email);});
                                ui.horizontal(|ui|{ui.label("Phone:"); ui.text_edit_singleline(&mut self.edit_phone);});
                                ui.horizontal(|ui|{ui.label("Co:"); ui.text_edit_singleline(&mut self.edit_company);});
                                ui.horizontal(|ui|{ui.label("Tags:"); ui.text_edit_singleline(&mut self.edit_tags);});
                                if ui.button(egui::RichText::new("Save").size(10.).color(theme::ACCENT_GREEN)).clicked() {
                                    c.name = self.edit_name.clone(); c.email = self.edit_email.clone(); c.phone = self.edit_phone.clone(); c.company = self.edit_company.clone(); c.tags = self.edit_tags.split(',').map(|s| s.trim().to_string()).filter(|s|!s.is_empty()).collect();
                                    c.updated_at = Utc::now().to_rfc3339(); self.editing_contact = None;
                                }
                            } else {
                                ui.label(egui::RichText::new(&c.name).size(12.).color(theme::TEXT));
                                ui.label(egui::RichText::new(&c.email).size(10.).color(theme::ACCENT));
                                ui.label(egui::RichText::new(&c.phone).size(10.).color(theme::TEXT));
                                ui.label(egui::RichText::new(&c.company).size(10.).color(theme::TEXT));
                                ui.add_space(4.);
                                if !c.tags.is_empty() {
                                    ui.horizontal_wrapped(|ui| { for t in &c.tags { ui.label(egui::RichText::new(t).size(9.).color(theme::ACCENT_ORANGE).background_color(theme::WIDGET_BG)); } });
                                }
                                ui.add_space(4.);
                                ui.label(egui::RichText::new("Notes Timeline").size(9.).color(theme::TEXT_DIM));
                                ui.separator();
                                let scroll = egui::ScrollArea::vertical().max_height(ui.available_height()-60.);
                                scroll.show(ui, |ui| {
                                    for n in &c.notes {
                                        ui.add_space(2.);
                                        ui.label(egui::RichText::new(&n.text).size(9.).color(theme::TEXT));
                                        ui.label(egui::RichText::new(&n.created_at[..10]).size(7.).color(theme::TEXT_DIM));
                                    }
                                });
                                ui.add_space(4.);
                                ui.horizontal(|ui|{
                                    ui.add_sized(egui::vec2(ui.available_width()-40., 18.), egui::TextEdit::singleline(&mut self.new_note_text).hint_text("Add note...").desired_width(f32::INFINITY));
                                    if ui.button(egui::RichText::new("+").size(12.).color(theme::ACCENT_GREEN)).clicked() && !self.new_note_text.is_empty() {
                                        c.notes.push(ContactNote{id:format!("n{}",c.notes.len()+1),text:self.new_note_text.clone(),created_at:Utc::now().to_rfc3339()});
                                        c.updated_at = Utc::now().to_rfc3339(); self.new_note_text.clear();
                                    }
                                });
                            }
                            ret
                        } else { false }
                    } else { false };
                    if del_clicked { let did = self.sel_contact.clone().unwrap(); self.contacts.retain(|x| x.id != did); self.sel_contact = None; self.editing_contact = None; }
                }
                Mode::Pipeline => {
                    if let Some(id) = &self.sel_deal.clone() {
                        if let Some(d) = self.deals.iter_mut().find(|d| &d.id == id) {
                            ui.label(egui::RichText::new(&d.title).size(12.).color(theme::TEXT));
                            ui.label(egui::RichText::new(format!("${:.0}", d.value)).size(11.).color(theme::ACCENT_GREEN));
                            ui.add_space(4.);
                            ui.label(egui::RichText::new(format!("Stage: {}", d.stage)).size(10.).color(theme::ACCENT));
                            if let Some(cid) = &d.contact_id {
                                if let Some(c) = self.contacts.iter().find(|c| &c.id == cid) {
                                    ui.label(egui::RichText::new(&c.name).size(9.).color(theme::TEXT));
                                }
                            }
                            ui.add_space(4.);
                            ui.label(egui::RichText::new("Expected Close:").size(9.).color(theme::TEXT_DIM));
                            ui.label(egui::RichText::new(&d.expected_close).size(10.).color(theme::TEXT));
                            ui.separator();
                            ui.label(egui::RichText::new("Notes").size(9.).color(theme::TEXT_DIM));
                            let mut notes = d.notes.clone();
                            if ui.text_edit_multiline(&mut notes).changed() { d.notes = notes; }
                            ui.add_space(8.);
                            let delete = ui.button(egui::RichText::new("Delete Deal").size(10.).color(theme::ACCENT_RED)).clicked();
                            if delete {
                                let did = id.clone();
                                self.deals.retain(|x| x.id != did);
                                self.sel_deal = None;
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new("Select a deal").size(10.).color(theme::TEXT_DIM));
                    }
                }
                Mode::Studio => {
                    ui.add_space(4.);
                    let mut idxs: Vec<usize> = (0..self.studio_layers.len()).collect();
                    idxs.sort_by(|&a, &b| self.studio_layers[b].z.cmp(&self.studio_layers[a].z));
                    for &i in &idxs {
                        let l = &self.studio_layers[i];
                        let sel = self.studio_sel_layer == Some(i);
                        let resp = ui.add(
                            egui::Button::new(egui::RichText::new(if l.visible{format!("◉ {}",l.name)}else{format!("○ {}",l.name)}).size(10.).color(if sel{theme::ACCENT}else{theme::TEXT}))
                                .fill(if sel{Color32::from_rgb(35,35,50)}else{theme::WIDGET_BG})
                                .min_size(egui::vec2(ui.available_width(), 20.))
                        );
                        if resp.clicked() { self.studio_sel_layer = Some(i); }
                        resp.context_menu(|ui| {
                            if ui.button("Toggle Visibility").clicked() { if let Some(l)=self.studio_layers.get_mut(i){l.visible=!l.visible;} ui.close_menu(); }
                            if ui.button("Toggle Lock").clicked() { if let Some(l)=self.studio_layers.get_mut(i){l.locked=!l.locked;} ui.close_menu(); }
                            if ui.button("Duplicate").clicked() { let c=self.studio_layers[i].clone(); let new_id=format!("sl{}",self.studio_layers.len()+1); self.studio_layers.push(StudioLayer{id:new_id,..c}); ui.close_menu(); }
                            if ui.button("Delete").clicked() { self.studio_layers.remove(i); self.studio_sel_layer=None; ui.close_menu(); }
                        });
                    }
                    ui.add_space(4.);
                    if ui.button(egui::RichText::new("+ Layer").size(10.).color(theme::ACCENT_GREEN)).clicked() {
                        let z = self.studio_layers.iter().map(|x|x.z).max().unwrap_or(0)+1;
                        self.studio_layers.push(StudioLayer{
                            id: format!("sl{}",self.studio_layers.len()+1), name: format!("Layer {}",self.studio_layers.len()+1),
                            x:50.,y:50.,w:100.,h:100.,z,visible:true,opacity:1.,locked:false,
                            content: LayerContent::Shape{kind:"rect".into(),x:50.,y:50.,w:100.,h:100.,fill_r:52,fill_g:152,fill_b:219,fill_a:200,stroke_r:255,stroke_g:255,stroke_b:255,stroke_a:255,stroke_width:1.},
                        });
                        self.studio_sel_layer = Some(self.studio_layers.len()-1);
                    }
                    if let Some(li) = self.studio_sel_layer {
                        if let Some(l) = self.studio_layers.get(li) {
                            ui.add_space(8.);
                            ui.label(egui::RichText::new("PROPERTIES").size(9.).color(theme::TEXT_DIM));
                            ui.separator();
                            match &l.content {
                                LayerContent::Shape{kind,x,y,w,h,..} => { ui.label(egui::RichText::new(format!("{} at ({:.0},{:.0}) {}x{}",kind,x,y,w,h)).size(9.).color(theme::TEXT_DIM)); }
                                LayerContent::Stroke{points,..} => { ui.label(egui::RichText::new(format!("Brush stroke, {} pts",points.len())).size(9.).color(theme::TEXT_DIM)); }
                                LayerContent::Text{content,..} => { ui.label(egui::RichText::new(format!("Text: \"{}\"",content)).size(9.).color(theme::TEXT_DIM)); }
                                LayerContent::Image{..} => { ui.label(egui::RichText::new("Image layer").size(9.).color(theme::TEXT_DIM)); }
                            }
                        }
                    }
                }
                Mode::DataViewer => {
                    ui.label(egui::RichText::new("Viewing records from:").size(9.).color(theme::TEXT_DIM));
                    ui.add_space(2.);
                    let entity = if self.viewer_selected_entity.is_empty() { "(none loaded)" } else { &self.viewer_selected_entity };
                    ui.label(egui::RichText::new(entity).size(11.).color(theme::ACCENT));
                    ui.add_space(8.);
                    ui.label(egui::RichText::new(format!("Records loaded: {}", self.viewer_records.len())).size(9.).color(theme::TEXT));
                    if let Some(conn) = &self.db_conn {
                        ui.add_space(4.);
                        match db::get_records(conn, &self.viewer_selected_entity) {
                            Ok(recs) => { ui.label(egui::RichText::new(format!("Total in DB: {}", recs.len())).size(9.).color(theme::ACCENT_GREEN)); }
                            Err(_) => {}
                        }
                    }
                }
                Mode::FlowBuilder => {
                    if let Some(sel_id) = &self.selected_flow_node.clone() {
                        if let Some(node) = self.project_doc.flow_graph.nodes.get_mut(sel_id) {
                            ui.label(egui::RichText::new("NODE CONFIGURATION").size(9.).color(theme::TEXT_DIM));
                            ui.add_space(2.);
                            ui.label(egui::RichText::new(format!("ID: {}", node.id)).size(9.).color(theme::TEXT_DIM));
                            ui.add_space(4.);
                            match &mut node.kind {
                                crate::flow::FlowNodeKind::TriggerClick { ref mut target_node_id } => {
                                    ui.label(egui::RichText::new("Target UI Node ID:").size(10.).color(theme::TEXT));
                                    ui.add_space(2.);
                                    ui.add(egui::TextEdit::singleline(target_node_id).hint_text("e.g. btn-1").desired_width(f32::INFINITY));
                                }
                                crate::flow::FlowNodeKind::NavigateTo { ref mut page_id } => {
                                    ui.label(egui::RichText::new("Target Page ID:").size(10.).color(theme::TEXT));
                                    ui.add_space(2.);
                                    ui.add(egui::TextEdit::singleline(page_id).hint_text("e.g. page-2").desired_width(f32::INFINITY));
                                }
                                crate::flow::FlowNodeKind::SaveToDatabase { ref mut entity } => {
                                    ui.label(egui::RichText::new("Database Entity:").size(10.).color(theme::TEXT));
                                    ui.add_space(2.);
                                    ui.add(egui::TextEdit::singleline(entity).hint_text("e.g. contacts").desired_width(f32::INFINITY));
                                }
                            }
                        }
                    } else {
                        ui.label(egui::RichText::new("Select a node on the canvas to configure it.").size(10.).color(theme::TEXT_DIM));
                    }
                    ui.add_space(12.);
                    ui.separator();
                    ui.add_space(4.);
                    let count = self.project_doc.flow_graph.nodes.len();
                    let edge_count = self.project_doc.flow_graph.edges.len();
                    ui.label(egui::RichText::new(format!("Graph: {} nodes, {} edges", count, edge_count)).size(9.).color(theme::TEXT_DIM));
                }
                Mode::Play => {
                    ui.label(egui::RichText::new("Interact with form elements and submit to save data to the database.").size(10.).color(theme::TEXT_DIM));
                    if let Some(conn) = &self.db_conn {
                        ui.add_space(4.);
                        match db::get_records(conn, "") {
                            Ok(recs) => { ui.label(egui::RichText::new(format!("Total records in DB: {}", recs.len())).size(9.).color(theme::ACCENT_GREEN)); }
                            Err(_) => { ui.label(egui::RichText::new("DB connected").size(9.).color(theme::ACCENT_GREEN)); }
                        }
                    } else {
                        ui.label(egui::RichText::new("DB not connected — saving disabled").size(9.).color(theme::ACCENT_RED));
                    }
                }
                Mode::Tasks => {
                    if let Some(id) = &self.sel_task.clone() {
                        if let Some(t) = self.tasks.iter_mut().find(|t| &t.id == id) {
                            ui.label(egui::RichText::new(&t.title).size(12.).color(theme::TEXT));
                            ui.add_space(2.);
                            ui.horizontal(|ui|{
                                ui.label(egui::RichText::new("Status:").size(9.).color(theme::TEXT_DIM));
                                for s in [TaskStatus::Open,TaskStatus::InProgress,TaskStatus::Done,TaskStatus::Cancelled] {
                                    if ui.selectable_label(t.status==s, task_status_label(&s)).clicked() { t.status=s.clone(); t.updated_at=Utc::now().to_rfc3339(); if s==TaskStatus::Done{t.completed_at=Some(Utc::now().to_rfc3339());} }
                                }
                            });
                            ui.add_space(2.);
                            ui.horizontal(|ui|{
                                ui.label(egui::RichText::new("Priority:").size(9.).color(theme::TEXT_DIM));
                                for p in [TaskPriority::Low,TaskPriority::Medium,TaskPriority::High] {
                                    let _col = task_priority_color(&p);
                                    if ui.selectable_label(t.priority==p, task_priority_label(&p)).clicked() { t.priority=p.clone(); t.updated_at=Utc::now().to_rfc3339(); }
                                }
                            });
                            ui.add_space(4.);
                            ui.label(egui::RichText::new("Due:").size(9.).color(theme::TEXT_DIM));
                            let mut due = t.due_date.clone();
                            if ui.text_edit_singleline(&mut due).changed() { t.due_date = due; t.updated_at = Utc::now().to_rfc3339(); }
                            ui.add_space(2.);
                            ui.label(egui::RichText::new("Assignee:").size(9.).color(theme::TEXT_DIM));
                            let mut assignee = t.assignee.clone();
                            if ui.text_edit_singleline(&mut assignee).changed() { t.assignee = assignee; }
                            ui.add_space(4.);
                            ui.label(egui::RichText::new("Notes:").size(9.).color(theme::TEXT_DIM));
                            let mut desc = t.description.clone();
                            if ui.text_edit_multiline(&mut desc).changed() { t.description = desc; }
                            ui.add_space(4.);
                            if let Some(cid) = &t.contact_id {
                                if let Some(c) = self.contacts.iter().find(|c| &c.id == cid) {
                                    ui.label(egui::RichText::new(format!("Contact: {}",c.name)).size(9.).color(theme::ACCENT));
                                }
                            }
                            ui.add_space(4.);
                            let del = ui.button(egui::RichText::new("Delete Task").size(10.).color(theme::ACCENT_RED)).clicked();
                            if del { let did = id.clone(); self.tasks.retain(|x| x.id != did); self.sel_task = None; }
                        }
                    } else {
                        ui.label(egui::RichText::new("Select a task").size(10.).color(theme::TEXT_DIM));
                    }
                }
            }
        });

        // ── DEVICE TOOLBAR (Designer mode only) ──
        if self.mode == Mode::Designer && self.show_device_toolbar {
            egui::TopBottomPanel::top("device_bar")
                .min_height(32.)
                .resizable(false)
                .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::symmetric(12, 3), ..Default::default() })
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("📐 Canvas:").size(10.).color(theme::TEXT_DIM));
                        let cur_dev = self.device_preset.clone();
                        let (cw, ch) = cur_dev.size();
                        ui.label(egui::RichText::new(format!("{} {} {}×{}", cur_dev.icon(), cur_dev.label(), cw, ch)).size(11.).color(theme::TEXT));
                        ui.add_space(8.);
                        ui.separator();
                        ui.add_space(8.);
                        let presets = [
                            DevicePreset::DesktopHD, DevicePreset::Desktop, DevicePreset::Laptop,
                            DevicePreset::TabletLandscape, DevicePreset::TabletPortrait,
                            DevicePreset::Phone, DevicePreset::PhoneSmall,
                        ];
                        for p in &presets {
                            let (pw, ph) = p.size();
                            let is_active = cur_dev == *p;
                            let label = format!("{} {} {}×{}", p.icon(), p.label(), pw, ph);
                            if ui.add(egui::Button::new(egui::RichText::new(label).size(9.).color(if is_active{theme::TEXT}else{theme::TEXT_DIM}))
                                .fill(if is_active{theme::ELEVATED}else{theme::PANEL})
                                .min_size(egui::vec2(95., 22.)))
                                .clicked()
                            {
                                self.device_preset = p.clone();
                                self.toast(&format!("Canvas: {}", p.label()));
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("📋 Load CRM Template").size(9.).color(theme::ACCENT)).clicked() {
                                self.project_doc = scene::create_dummy_document();
                                self.project_doc = scene::create_crm_preset();
                                self.editor_state.selected_node_ids.clear();
                                self.designer_selected_node = None;
                                self.toast("CRM Template loaded ✓");
                            }
                            ui.add_space(4.);
                            if ui.button(egui::RichText::new(if self.show_device_toolbar { "✕ Hide" } else { "☰ Show" }).size(9.).color(theme::TEXT_DIM)).clicked() {
                                self.show_device_toolbar = !self.show_device_toolbar;
                            }
                        });
                    });
                });
        }

        // ── CENTRAL CANVAS ──
        egui::CentralPanel::default()
            .frame(egui::Frame { fill: theme::BG, ..Default::default() })
            .show(ctx, |ui| {
            let (resp, pnt) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            let r = resp.rect;
            pnt.rect_filled(r, 0, theme::BG);

            // Grid
            if self.layout == LayoutMode::Grid {
                let mut y = r.top(); while y <= r.bottom() { pnt.line_segment([egui::pos2(r.left(),y),egui::pos2(r.right(),y)], Stroke::new(1.,Color32::from_rgba_premultiplied(30,30,30,255))); y+=GRID; }
                let mut x = r.left(); while x <= r.right() { pnt.line_segment([egui::pos2(x,r.top()),egui::pos2(x,r.bottom())], Stroke::new(1.,Color32::from_rgba_premultiplied(30,30,30,255))); x+=GRID; }
            } else {
                let mut y = r.top()+12.; while y<r.bottom() { let mut x = r.left()+12.; while x<r.right() { pnt.circle_filled(egui::pos2(x,y),1.,Color32::from_rgb(25,25,25)); x+=24.; } y+=24.; }
            }

            ctx.request_repaint();

            let mpos = ui.input(|i| i.pointer.interact_pos());
            let mdown = ui.input(|i| i.pointer.any_down());
            let mup = ui.input(|i| i.pointer.any_released());
            let to_c = |pos: egui::Pos2| egui::pos2(pos.x - r.left(), pos.y - r.top());

            if self.mode == Mode::Contacts {
                // ── CONTACTS MODE ──
                let scroll = egui::ScrollArea::vertical().max_height(r.height());
                scroll.show_viewport(ui, |_ui, _| {
                    if let Some(id) = &self.sel_contact.clone() {
                        if let Some(c) = self.contacts.iter().find(|c| &c.id == id) {
                            pnt.text(egui::pos2(r.left()+20., r.top()+20.), egui::Align2::LEFT_TOP,
                                &format!("{} — {} — {}", c.name, c.email, c.phone),
                                egui::FontId::proportional(16.), theme::TEXT);
                            pnt.text(egui::pos2(r.left()+20., r.top()+44.), egui::Align2::LEFT_TOP,
                                &format!("Company: {}  |  Created: {}  |  Updated: {}", c.company, &c.created_at[..10], &c.updated_at[..10]),
                                egui::FontId::proportional(10.), theme::TEXT_DIM);
                            pnt.text(egui::pos2(r.left()+20., r.top()+64.), egui::Align2::LEFT_TOP,
                                &format!("Tags: {}", c.tags.join(", ")),
                                egui::FontId::proportional(10.), theme::ACCENT_ORANGE);
                            pnt.text(egui::pos2(r.left()+20., r.top()+90.), egui::Align2::LEFT_TOP,
                                "Notes Timeline:",
                                egui::FontId::proportional(11.), theme::TEXT_DIM);
                            for (i, n) in c.notes.iter().enumerate() {
                                let ny = r.top()+110.+i as f32*28.;
                                pnt.rect_filled(Rect::from_min_size(egui::pos2(r.left()+20.,ny),egui::vec2(r.width()-40.,24.)),2,theme::PANEL);
                                pnt.text(egui::pos2(r.left()+28.,ny+4.), egui::Align2::LEFT_TOP, &n.text, egui::FontId::proportional(10.), theme::TEXT);
                                pnt.text(egui::pos2(r.left()+28.,ny+16.), egui::Align2::LEFT_TOP, &n.created_at[..10], egui::FontId::proportional(7.), theme::TEXT_DIM);
                            }
                        } else {
                            pnt.text(r.center(), egui::Align2::CENTER_CENTER, "Contact not found", egui::FontId::proportional(14.), theme::TEXT_DIM);
                        }
                    } else {
                        pnt.text(r.center(), egui::Align2::CENTER_CENTER, "Select a contact from the left panel", egui::FontId::proportional(14.), theme::TEXT_DIM);
                    }
                });
            } else if self.mode == Mode::Pipeline {
                // ── PIPELINE MODE (Kanban) ──
                let stage_width = (r.width() - 48.) / KANBAN_STAGES.len() as f32;
                let header_h = 36.;
                let mpos_canvas = mpos;
                let mup_canvas = mup;

                for (si, stage) in KANBAN_STAGES.iter().enumerate() {
                    let cx = r.left() + 8. + si as f32 * (stage_width + 8.);
                    let col_r = Rect::from_min_size(egui::pos2(cx, r.top() + 8.), egui::vec2(stage_width, r.height() - 16.));
                    pnt.rect_filled(col_r, 4, theme::PANEL);
                    pnt.rect_stroke(col_r, 4, Stroke::new(1., theme::BORDER), egui::StrokeKind::Outside);

                    // Stage header
                    let stage_total: f64 = self.deals.iter().filter(|d| d.stage == *stage).map(|d| d.value).sum();
                    let stage_count = self.deals.iter().filter(|d| d.stage == *stage).count();
                    pnt.text(egui::pos2(cx+4., col_r.top()+8.), egui::Align2::LEFT_CENTER,
                        &format!("{}  (${:.0})", stage, stage_total),
                        egui::FontId::proportional(10.), theme::ACCENT);
                    pnt.text(egui::pos2(cx+stage_width-4., col_r.top()+8.), egui::Align2::RIGHT_CENTER,
                        &stage_count.to_string(),
                        egui::FontId::proportional(9.), theme::TEXT_DIM);

                    // Deal cards
                    let deals_in_stage: Vec<(usize, &Deal)> = self.deals.iter().enumerate().filter(|(_,d)| d.stage == *stage).collect();
                    for (di, (_i, deal)) in deals_in_stage.iter().enumerate() {
                        let card_y = col_r.top() + header_h + 4. + di as f32 * 52.;
                        let card_r = Rect::from_min_size(egui::pos2(cx+4., card_y), egui::vec2(stage_width-8., 46.));
                        let sel = self.sel_deal.as_ref() == Some(&deal.id);
                        pnt.rect_filled(card_r, 2, if sel{Color32::from_rgb(35,35,50)}else{theme::WIDGET_BG});
                        pnt.rect_stroke(card_r, 2, Stroke::new(if sel{2.}else{1.}, if sel{theme::SELECTED}else{theme::BORDER}), egui::StrokeKind::Outside);
                        // Colored left border
                        let border_colors = [theme::ACCENT, theme::ACCENT_ORANGE, theme::ACCENT_GREEN, theme::ACCENT_PURPLE, theme::ACCENT_RED, Color32::from_rgb(0,200,150), Color32::from_rgb(150,50,50)];
                        pnt.rect_filled(Rect::from_min_size(card_r.min, egui::vec2(4., card_r.height())), 2, border_colors[si % border_colors.len()]);
                        // Card text
                        pnt.text(egui::pos2(card_r.left()+8., card_r.top()+8.), egui::Align2::LEFT_TOP,
                            &deal.title,
                            egui::FontId::proportional(9.), theme::TEXT);
                        pnt.text(egui::pos2(card_r.left()+8., card_r.top()+26.), egui::Align2::LEFT_TOP,
                            &format!("${:.0}  |  Close: {}", deal.value, deal.expected_close),
                            egui::FontId::proportional(8.), theme::TEXT_DIM);
                        // Contact name if linked
                        if let Some(cid) = &deal.contact_id {
                            if let Some(c) = self.contacts.iter().find(|c| &c.id == cid) {
                                pnt.text(egui::pos2(card_r.right()-8., card_r.top()+26.), egui::Align2::RIGHT_TOP,
                                    &c.name,
                                    egui::FontId::proportional(8.), theme::ACCENT);
                            }
                        }
                        // Click on card → select
                        if let Some(pos) = mpos_canvas {
                            if mup_canvas && card_r.contains(pos) {
                                self.sel_deal = Some(deal.id.clone());
                            }
                        }
                    }
                }

                // Move deal between stages: click stage header with a deal selected
                if let Some(sel_id) = &self.sel_deal.clone() {
                    if mup_canvas {
                        if let Some(pos) = mpos_canvas {
                            for (si, stage) in KANBAN_STAGES.iter().enumerate() {
                                let cx = r.left() + 8. + si as f32 * (stage_width + 8.);
                                let header_r = Rect::from_min_size(egui::pos2(cx+4., r.top()+8.), egui::vec2(stage_width-8., 28.));
                                if header_r.contains(pos) {
                                    let (prev, deal_title, deal_contact) = if let Some(d) = self.deals.iter_mut().find(|d| &d.id == sel_id) {
                                        let p = d.stage.clone();
                                        d.stage = stage.to_string();
                                        d.updated_at = Utc::now().to_rfc3339();
                                        (p, d.title.clone(), d.contact_id.clone())
                                    } else { continue; };
                                    self.toast(format!("{} → {}", prev, stage));
                                    let tid = format!("t{}",self.tasks.len()+1);
                                    self.tasks.push(Task{
                                        id:tid,title:format!("Follow-up: {} moved to {}", deal_title, stage),description:String::new(),
                                        contact_id:deal_contact,deal_id:Some(sel_id.clone()),assignee:"me".into(),
                                        due_date:(Utc::now()+chrono::Duration::days(3)).format("%Y-%m-%d").to_string(),
                                        status:TaskStatus::Open,priority:TaskPriority::Medium,
                                        created_by:"me".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339(),completed_at:None,
                                    });
                                }
                            }
                        }
                    }
                }

                // Hint text
                pnt.text(egui::pos2(r.center().x, r.bottom()-20.), egui::Align2::CENTER_CENTER,
                    "Select a deal card, then click a stage header to move it",
                    egui::FontId::proportional(9.), theme::TEXT_DIM);
            } else if self.mode == Mode::Tasks {
                // ── TASKS MODE (Dashboard) ──
                let scroll = egui::ScrollArea::vertical().max_height(r.height());
                scroll.show_viewport(ui, |_ui, _| {
                    let today = Utc::now().format("%Y-%m-%d").to_string();
                    let mut overdue: Vec<&Task> = vec![];
                    let mut today_tasks: Vec<&Task> = vec![];
                    let mut upcoming: Vec<&Task> = vec![];
                    for t in &self.tasks {
                        if t.status == TaskStatus::Done || t.status == TaskStatus::Cancelled { continue; }
                        if t.due_date < today { overdue.push(t); }
                        else if t.due_date == today { today_tasks.push(t); }
                        else { upcoming.push(t); }
                    }
                    let mut y = r.top() + 20.;
                    // Sections
                    for (label, items, color) in [
                        ("OVERDUE", &overdue, theme::ACCENT_RED),
                        ("TODAY", &today_tasks, theme::ACCENT_ORANGE),
                        ("UPCOMING", &upcoming, theme::ACCENT_GREEN),
                    ] {
                        pnt.text(egui::pos2(r.left()+20., y), egui::Align2::LEFT_TOP,
                            &format!("{}  ({})", label, items.len()),
                            egui::FontId::proportional(12.), color);
                        y += 24.;
                        if items.is_empty() {
                            pnt.text(egui::pos2(r.left()+30., y), egui::Align2::LEFT_TOP,
                                "None", egui::FontId::proportional(10.), theme::TEXT_DIM);
                            y += 20.;
                        }
                        for t in items.iter() {
                            let card_r = Rect::from_min_size(egui::pos2(r.left()+20., y), egui::vec2(r.width()-60., 36.));
                            let sel = self.sel_task.as_ref() == Some(&t.id);
                            pnt.rect_filled(card_r, 2, if sel{Color32::from_rgb(35,35,50)}else{theme::PANEL});
                            pnt.rect_stroke(card_r, 2, Stroke::new(if sel{2.}else{1.}, if sel{theme::SELECTED}else{theme::BORDER}), egui::StrokeKind::Outside);
                            // Priority color bar
                            pnt.rect_filled(Rect::from_min_size(card_r.min, egui::vec2(3., card_r.height())), 2, task_priority_color(&t.priority));
                            pnt.text(egui::pos2(card_r.left()+10., card_r.top()+6.), egui::Align2::LEFT_TOP,
                                &format!("{}  {}", task_status_icon(&t.status), t.title),
                                egui::FontId::proportional(10.), theme::TEXT);
                            pnt.text(egui::pos2(card_r.left()+10., card_r.top()+20.), egui::Align2::LEFT_TOP,
                                &format!("Due: {}  |  {}", t.due_date, t.assignee),
                                egui::FontId::proportional(8.), theme::TEXT_DIM);
                            // Click to select
                            if let Some(pos) = mpos {
                                if mup && card_r.contains(pos) {
                                    self.sel_task = Some(t.id.clone());
                                }
                            }
                            y += 40.;
                        }
                        y += 8.;
                    }
                    // Quick-add task bar at bottom
                    let qr = Rect::from_min_size(egui::pos2(r.left()+20., r.bottom()-40.), egui::vec2(r.width()-60., 28.));
                    pnt.rect_filled(qr, 0, theme::WIDGET_BG);
                    pnt.rect_stroke(qr, 0, Stroke::new(1., theme::BORDER), egui::StrokeKind::Outside);
                });
                // Quick-add is drawn via painter — handle input separately
                let qr = Rect::from_min_size(egui::pos2(r.left()+20., r.bottom()-40.), egui::vec2(r.width()-60., 28.));
                if let Some(pos) = mpos {
                    if mup && qr.contains(pos) {
                        // Trigger quick-add via the new task fields in right panel
                        let id = format!("t{}",self.tasks.len()+1);
                        self.tasks.push(Task{
                            id:id.clone(),title:"New Task".into(),description:String::new(),
                            contact_id:None,deal_id:None,assignee:"me".into(),
                            due_date:(Utc::now()+chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
                            status:TaskStatus::Open,priority:TaskPriority::Medium,
                            created_by:"me".into(),created_at:Utc::now().to_rfc3339(),updated_at:Utc::now().to_rfc3339(),completed_at:None,
                        });
                        self.sel_task = Some(id);
                        self.toast("Task created — edit details in right panel");
                    }
                }
                // Draw the quick-add text on top
                pnt.text(qr.center(), egui::Align2::CENTER_CENTER,
                    "+ Quick Add Task",
                    egui::FontId::proportional(10.), theme::TEXT_DIM);
                if self.tasks.is_empty() {
                    pnt.text(r.center(), egui::Align2::CENTER_CENTER,
                        "No tasks yet. Click '+ Quick Add Task' to create your first one.",
                        egui::FontId::proportional(14.), theme::TEXT_DIM);
                }
            } else if self.mode == Mode::Studio {
                // ── STUDIO (Design Tool) ──
                // Render layers sorted by z
                let mut sorted: Vec<usize> = (0..self.studio_layers.len()).collect();
                sorted.sort_by_key(|&i| self.studio_layers[i].z);
                for &i in &sorted {
                    let l = &self.studio_layers[i];
                    if !l.visible { continue; }
                    let lr = Rect::from_min_size(egui::pos2(r.left()+l.x, r.top()+l.y), egui::vec2(l.w, l.h));
                    match &l.content {
                        LayerContent::Shape{kind,x,y,w,h,fill_r,fill_g,fill_b,fill_a,stroke_r,stroke_g,stroke_b,stroke_a,stroke_width} => {
                            let fill = Color32::from_rgba_premultiplied(*fill_r,*fill_g,*fill_b,*fill_a);
                            let stroke = Color32::from_rgba_premultiplied(*stroke_r,*stroke_g,*stroke_b,*stroke_a);
                            let sr = Rect::from_min_size(egui::pos2(r.left()+x, r.top()+y), egui::vec2(*w,*h));
                            match kind.as_str() {
                                "ellipse" => { pnt.circle_filled(sr.center(), sr.width().min(sr.height())/2., fill); if *stroke_width>0. { pnt.circle_stroke(sr.center(), sr.width().min(sr.height())/2., Stroke::new(*stroke_width, stroke)); } }
                                "line" => { pnt.line_segment([sr.left_top(), sr.right_bottom()], Stroke::new(*stroke_width, stroke)); }
                                _ => { pnt.rect_filled(sr, 0, fill); if *stroke_width>0. { pnt.rect_stroke(sr, 0, Stroke::new(*stroke_width, stroke), egui::StrokeKind::Outside); } }
                            }
                            if self.studio_sel_layer == Some(i) {
                                pnt.rect_stroke(sr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
                            }
                        }
                        LayerContent::Stroke{points,r:pr,g:pg,b:pb,a:pa,size} => {
                            if points.len() < 2 { continue; }
                            let col = Color32::from_rgba_premultiplied(*pr,*pg,*pb,*pa);
                            for j in 1..points.len() {
                                let p1 = egui::pos2(r.left()+points[j-1][0], r.top()+points[j-1][1]);
                                let p2 = egui::pos2(r.left()+points[j][0], r.top()+points[j][1]);
                                pnt.line_segment([p1,p2], Stroke::new(*size, col));
                            }
                        }
                        LayerContent::Text{content,font_size,r,g,b,a} => {
                            let col = Color32::from_rgba_premultiplied(*r,*g,*b,*a);
                            pnt.text(lr.left_top(), egui::Align2::LEFT_TOP, content, egui::FontId::proportional(*font_size), col);
                            if self.studio_sel_layer == Some(i) {
                                pnt.rect_stroke(lr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
                            }
                        }
                        LayerContent::Image{..} => {}
                    }
                }

                // Shape preview while dragging
                if let (Some(start), Some(curr)) = (self.studio_shape_start, self.studio_shape_current) {
                    let sr = Rect::from_min_max(start, curr);
                    let psr = Rect::from_min_max(egui::pos2(r.left()+sr.min.x, r.top()+sr.min.y), egui::pos2(r.left()+sr.max.x, r.top()+sr.max.y));
                    match self.studio_tool {
                        StudioTool::Rectangle => {
                            pnt.rect_filled(psr, 0, Color32::from_rgba_premultiplied(52,152,219,80));
                            pnt.rect_stroke(psr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
                        }
                        StudioTool::Ellipse => {
                            let r = psr.width().min(psr.height()) / 2.;
                            pnt.circle_filled(psr.center(), r, Color32::from_rgba_premultiplied(52,152,219,80));
                            pnt.circle_stroke(psr.center(), r, Stroke::new(1., theme::ACCENT));
                        }
                        _ => {}
                    }
                }

                // Canvas interactions
                let _canvas_rect = r;
                let cpos = mpos.map(|p| to_c(p));
                let c_down = mdown;
                let c_up = mup;

                match self.studio_tool {
                    StudioTool::Select => {
                        // Click to select layer (topmost visible)
                        if c_up {
                            if let Some(pos) = cpos {
                                let mut hit_idx: Option<usize> = None;
                                for &i in sorted.iter().rev() {
                                    let l = &self.studio_layers[i];
                                    if !l.visible || l.locked { continue; }
                                    let lr = Rect::from_min_size(egui::pos2(l.x, l.y), egui::vec2(l.w, l.h));
                                    if lr.contains(pos) { hit_idx = Some(i); break; }
                                }
                                self.studio_sel_layer = hit_idx;
                            }
                        }
                        // Drag to move selected layer
                        if c_down {
                            if let Some(pos) = cpos {
                                if self.studio_dragging.is_none() {
                                    if let Some(li) = self.studio_sel_layer {
                                        let l = &self.studio_layers[li];
                                        let lr = Rect::from_min_size(egui::pos2(l.x, l.y), egui::vec2(l.w, l.h));
                                        if lr.contains(pos) {
                                            self.studio_dragging = Some((li, egui::pos2(l.x,l.y), pos));
                                        }
                                    }
                                }
                            }
                        }
                        if let Some((li, start, off)) = self.studio_dragging {
                            if c_down {
                                let drag_id = if self.studio_dragging.is_some() { Some(self.studio_layers[li].id.clone()) } else { None };
                        if let Some(pos) = cpos {
                            if let Some(ref lid) = drag_id {
                                if let Some(l) = self.studio_layers.iter_mut().find(|x| x.id == *lid) {
                                    l.x = (start.x + pos.x - off.x).max(0.);
                                    l.y = (start.y + pos.y - off.y).max(0.);
                                }
                            }
                                }
                            } else { self.studio_dragging = None; }
                        }
                    }
                    StudioTool::Brush | StudioTool::Eraser => {
                        if let Some(pos) = cpos {
                            if c_down {
                                // Find or create stroke layer
                                let brush_idx = self.studio_layers.iter().position(|l| matches!(l.content, LayerContent::Stroke{..}));
                                let col = if self.studio_tool == StudioTool::Eraser { Color32::from_rgb(13,13,13) } else { self.studio_color };
                                let (cr,cg,cb,ca) = (col.r(),col.g(),col.b(),col.a());
                                let sz = if self.studio_tool == StudioTool::Eraser { self.studio_brush_size * 3. } else { self.studio_brush_size };
                                if let Some(si) = brush_idx {
                                    if let LayerContent::Stroke{points,..} = &mut self.studio_layers[si].content {
                                        points.push([pos.x, pos.y]);
                                    }
                                } else {
                                    let z = self.studio_layers.iter().map(|x|x.z).max().unwrap_or(0)+1;
                                    self.studio_layers.push(StudioLayer{
                                        id: format!("sl{}", self.studio_layers.len()+1), name: "Brush".into(),
                                        x: 0., y: 0., w: 0., h: 0., z, visible: true, opacity: 1., locked: false,
                                        content: LayerContent::Stroke{points: vec![[pos.x, pos.y]], r:cr,g:cg,b:cb,a:ca,size:sz},
                                    });
                                    self.studio_sel_layer = Some(self.studio_layers.len()-1);
                                }
                            } else if c_up {
                                // Finalize brush stroke — freeze it
                            }
                        }
                    }
                    StudioTool::Rectangle | StudioTool::Ellipse => {
                        if c_down && self.studio_shape_start.is_none() {
                            if let Some(pos) = cpos {
                                self.studio_shape_start = Some(pos);
                            }
                        }
                        if self.studio_shape_start.is_some() {
                            if let Some(pos) = cpos {
                                self.studio_shape_current = Some(pos);
                            }
                            if c_up {
                                if let (Some(s), Some(e)) = (self.studio_shape_start, self.studio_shape_current) {
                                    let (x,y,w,h) = (
                                        s.x.min(e.x), s.y.min(e.y),
                                        (s.x - e.x).abs(), (s.y - e.y).abs(),
                                    );
                                    let kind = if self.studio_tool == StudioTool::Rectangle { "rect" } else { "ellipse" };
                                    let col = self.studio_color;
                                    let z = self.studio_layers.iter().map(|x|x.z).max().unwrap_or(0)+1;
                                    self.studio_layers.push(StudioLayer{
                                        id: format!("sl{}", self.studio_layers.len()+1), name: kind.into(),
                                        x, y, w, h, z, visible: true, opacity: 1., locked: false,
                                        content: LayerContent::Shape{
                                            kind: kind.into(), x, y, w, h,
                                            fill_r: col.r(), fill_g: col.g(), fill_b: col.b(), fill_a: 180,
                                            stroke_r: 255, stroke_g: 255, stroke_b: 255, stroke_a: 255, stroke_width: 1.,
                                        },
                                    });
                                    self.studio_sel_layer = Some(self.studio_layers.len()-1);
                                }
                                self.studio_shape_start = None;
                                self.studio_shape_current = None;
                            }
                        }
                    }
                    StudioTool::Line => {
                        if c_down && self.studio_shape_start.is_none() {
                            if let Some(pos) = cpos { self.studio_shape_start = Some(pos); }
                        }
                        if self.studio_shape_start.is_some() {
                            if let Some(pos) = cpos { self.studio_shape_current = Some(pos); }
                            if c_up {
                                if let (Some(s), Some(e)) = (self.studio_shape_start, self.studio_shape_current) {
                                    let col = self.studio_color;
                                    let z = self.studio_layers.iter().map(|x|x.z).max().unwrap_or(0)+1;
                                    self.studio_layers.push(StudioLayer{
                                        id: format!("sl{}", self.studio_layers.len()+1), name: "Line".into(),
                                        x: s.x, y: s.y, w: (s.x-e.x).abs(), h: (s.y-e.y).abs(),
                                        z, visible: true, opacity: 1., locked: false,
                                        content: LayerContent::Shape{
                                            kind: "line".into(), x: s.x, y: s.y, w: (s.x-e.x).abs(), h: (s.y-e.y).abs(),
                                            fill_r: 0, fill_g: 0, fill_b: 0, fill_a: 0,
                                            stroke_r: col.r(), stroke_g: col.g(), stroke_b: col.b(), stroke_a: col.a(), stroke_width: self.studio_brush_size,
                                        },
                                    });
                                    self.studio_sel_layer = Some(self.studio_layers.len()-1);
                                }
                                self.studio_shape_start = None;
                                self.studio_shape_current = None;
                            }
                        }
                    }
                    StudioTool::Text => {
                        if c_up && self.studio_shape_start.is_none() {
                            if let Some(pos) = cpos {
                                self.studio_text_input.clear();
                                self.studio_show_text_dialog = true;
                                self.studio_shape_start = Some(pos);
                            }
                        }
                    }
                    StudioTool::Picker => {
                        if c_up {
                            // ponytail: not implementing pixel-level picker yet
                            self.toast("Color picker (todo — click to sample)");
                        }
                    }
                    StudioTool::Fill => {
                        if c_up {
                            if let Some(li) = self.studio_sel_layer {
                                if let Some(l) = self.studio_layers.get_mut(li) {
                                    if let LayerContent::Shape{fill_r,fill_g,fill_b,fill_a,..} = &mut l.content {
                                        let c = self.studio_color;
                                        *fill_r = c.r(); *fill_g = c.g(); *fill_b = c.b(); *fill_a = c.a();
                                    }
                                }
                            }
                        }
                    }
                }

                // Text dialog
                if self.studio_show_text_dialog {
                    egui::Area::new("text_dialog".into()).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.,0.)).show(ctx, |ui| {
                        ui.label("Enter text:");
                        ui.text_edit_multiline(&mut self.studio_text_input);
                        if ui.button("Add").clicked() && !self.studio_text_input.is_empty() {
                            if let Some(pos) = self.studio_shape_start {
                                let col = self.studio_color;
                                let z = self.studio_layers.iter().map(|x|x.z).max().unwrap_or(0)+1;
                                self.studio_layers.push(StudioLayer{
                                    id: format!("sl{}", self.studio_layers.len()+1), name: "Text".into(),
                                    x: pos.x, y: pos.y, w: 160., h: 30., z, visible: true, opacity: 1., locked: false,
                                    content: LayerContent::Text{
                                        content: self.studio_text_input.clone(), font_size: 18.,
                                        r: col.r(), g: col.g(), b: col.b(), a: col.a(),
                                    },
                                });
                                self.studio_sel_layer = Some(self.studio_layers.len()-1);
                            }
                            self.studio_shape_start = None;
                            self.studio_show_text_dialog = false;
                        }
                        if ui.button("Cancel").clicked() { self.studio_shape_start = None; self.studio_show_text_dialog = false; }
                    });
                }

                // Color picker popup
                if self.studio_show_color_popup {
                    egui::Area::new("color_popup".into()).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.,0.)).show(ctx, |ui| {
                        ui.label(egui::RichText::new(if self.studio_color_picker_target==0{"Primary Color"}else{"Secondary Color"}).size(11.).color(theme::TEXT));
                        ui.add_space(4.);
                        let col = if self.studio_color_picker_target==0 { &mut self.studio_color } else { &mut self.studio_color2 };
                        let mut color = [col.r() as f32 / 255., col.g() as f32 / 255., col.b() as f32 / 255., col.a() as f32 / 255.];
                        ui.color_edit_button_rgba_premultiplied(&mut color);
                        if ui.button("Close").clicked() { self.studio_show_color_popup = false; }
                    });
                }

                // Empty canvas hint
                if self.studio_layers.is_empty() {
                    pnt.text(r.center(), egui::Align2::CENTER_CENTER,
                        "Use the tool palette to create layers. Choose a tool and click/drag on canvas.",
                        egui::FontId::proportional(14.), theme::TEXT_DIM);
                }
            } else if self.mode == Mode::Designer {
                // ── DESIGNER v2 — Viewport Canvas ──
                let canvas_origin = r.left_top();

                // ── Palette click (fallback) → spawn at viewport center ──
                if let Some(nt) = self.palette_drag.take() {
                    let center = r.center();
                    let world_center = self.viewport.screen_to_world(center, canvas_origin);
                    self.spawn_designer_node(nt, world_center);
                }

                // ── Viewport input ──
                let hover_canvas = mpos.map(|p| r.contains(p)).unwrap_or(false);

                let scroll = ui.input(|i| i.raw_scroll_delta);
                if scroll.y != 0. && hover_canvas {
                    self.viewport.zoom = (self.viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
                    if let Some(cursor) = mpos {
                        let world = self.viewport.screen_to_world(cursor, canvas_origin);
                        self.viewport.pan.x = cursor.x - world.x * self.viewport.zoom - canvas_origin.x;
                        self.viewport.pan.y = cursor.y - world.y * self.viewport.zoom - canvas_origin.y;
                    }
                }
                
                let middle = ui.input(|i| i.pointer.middle_down());
                let space = ui.input(|i| i.key_down(egui::Key::Space));
                let mdown = ui.input(|i| i.pointer.primary_down());
                // Only start panning if hovering canvas or already panning
                let is_panning_input = middle || (space && mdown);
                let is_panning = is_panning_input && hover_canvas; // Ideally we track state to allow dragging outside, but this stops hijacking the right panel

                if is_panning {
                    let delta = ui.input(|i| i.pointer.delta());
                    self.viewport.pan += delta;
                }

                // ── Render dot-matrix grid ──
                if self.layout == LayoutMode::Grid {
                    let world_tl = self.viewport.screen_to_world(r.left_top(), canvas_origin);
                    let world_br = self.viewport.screen_to_world(r.right_bottom(), canvas_origin);
                    let start_x = (world_tl.x / GRID).floor() * GRID;
                    let start_y = (world_tl.y / GRID).floor() * GRID;
                    let mut wy = start_y;
                    while wy <= world_br.y {
                        let sy = self.viewport.world_to_screen(egui::pos2(0., wy), canvas_origin).y;
                        pnt.line_segment([egui::pos2(r.left(), sy), egui::pos2(r.right(), sy)],
                            Stroke::new(1., Color32::from_rgba_premultiplied(30,30,30,255)));
                        wy += GRID;
                    }
                    let mut wx = start_x;
                    while wx <= world_br.x {
                        let sx = self.viewport.world_to_screen(egui::pos2(wx, 0.), canvas_origin).x;
                        pnt.line_segment([egui::pos2(sx, r.top()), egui::pos2(sx, r.bottom())],
                            Stroke::new(1., Color32::from_rgba_premultiplied(30,30,30,255)));
                        wx += GRID;
                    }
                } else {
                    // Dot-matrix grid
                    let world_tl = self.viewport.screen_to_world(r.left_top(), canvas_origin);
                    let world_br = self.viewport.screen_to_world(r.right_bottom(), canvas_origin);
                    let start_x = (world_tl.x / 24.0).floor() * 24.0;
                    let start_y = (world_tl.y / 24.0).floor() * 24.0;
                    let mut wy = start_y;
                    while wy <= world_br.y {
                        let mut wx = start_x;
                        while wx <= world_br.x {
                            let sp = self.viewport.world_to_screen(egui::pos2(wx, wy), canvas_origin);
                            if r.contains(sp) {
                                pnt.circle_filled(sp, (1.0_f32).max(0.5 * self.viewport.zoom), Color32::from_rgb(25,25,25));
                            }
                            wx += 24.0;
                        }
                        wy += 24.0;
                    }
                }

                // ── Device frame overlay ──
                {
                    let (dw, dh) = self.device_preset.size();
                    let dev_origin = self.viewport.world_to_screen(egui::pos2(0., 0.), canvas_origin);
                    let dev_w = dw * self.viewport.zoom;
                    let dev_h = dh * self.viewport.zoom;
                    let dev_rect = Rect::from_min_size(dev_origin, egui::vec2(dev_w, dev_h));
                    // Dim outside
                    let outside_rects = [
                        Rect::from_min_max(egui::pos2(r.left(), r.top()), egui::pos2(r.right(), dev_rect.top())),
                        Rect::from_min_max(egui::pos2(r.left(), dev_rect.bottom()), egui::pos2(r.right(), r.bottom())),
                        Rect::from_min_max(egui::pos2(r.left(), dev_rect.top()), egui::pos2(dev_rect.left(), dev_rect.bottom())),
                        Rect::from_min_max(egui::pos2(dev_rect.right(), dev_rect.top()), egui::pos2(r.right(), dev_rect.bottom())),
                    ];
                    for or in &outside_rects {
                        pnt.rect_filled(*or, 0, Color32::from_black_alpha(80));
                    }
                    // Device border
                    pnt.rect_stroke(dev_rect, 4., Stroke::new(2., theme::ACCENT), egui::StrokeKind::Outside);
                    // Label
                    let label = format!("{} {} — {}×{}", self.device_preset.icon(), self.device_preset.label(), dw, dh);
                    pnt.text(egui::pos2(dev_rect.left() + 8., dev_rect.top() - 16.), egui::Align2::LEFT_BOTTOM,
                        &label, egui::FontId::proportional(10.), theme::ACCENT);
                }

                // ── Delete on keyboard shortcut ──
                let delete_pressed = ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
                if delete_pressed {
                    if let Some(id) = self.designer_selected_node.take() {
                        self.project_doc.delete_node(&id);
                        self.editor_state.selected_node_ids.clear();
                        self.toast("Node deleted");
                    }
                }

                // ── Render arena document ──
                let arena_events = renderer::draw_document(ui, &self.project_doc, &self.viewport, &self.editor_state, false, &mut self.form_state, None);
                
                let mut is_resizing = false;
                for ev in &arena_events {
                    if let CanvasEvent::NodeResizeStarted = ev {
                        is_resizing = true;
                    }
                }

                // ── Designer mouse interaction ──
                if !is_panning {
                    let pointer = ui.input(|i| i.pointer.clone());
                    if let Some(mp) = mpos {
                        let wm = self.viewport.screen_to_world(mp, canvas_origin);
                        let wm = (wm.x, wm.y);

                        // Only allow raw drag if a resize handle isn't being interacted with
                        if pointer.primary_pressed() && hover_canvas {
                            let hit = scene::hit_test_nodes(&self.project_doc, wm, self.viewport.zoom);
                            self.toast(format!("Press: hit={:?}, is_resizing={}", hit, is_resizing));
                            if !is_resizing {
                                if let Some(hit_id) = hit {
                                    self.designer_selected_node = Some(hit_id.clone());
                                    self.designer_drag_active = true;
                                    if let Some(node) = self.project_doc.nodes.get(&hit_id) {
                                        self.designer_drag_offset = (node.position.0 - wm.0, node.position.1 - wm.1);
                                    }
                                    self.editor_state.selected_node_ids = vec![hit_id];
                                } else {
                                    self.designer_selected_node = None;
                                    self.designer_drag_active = false;
                                    self.editor_state.selected_node_ids.clear();
                                }
                            }
                        }

                        if pointer.primary_down() && self.designer_drag_active {
                            if let Some(ref sel_id) = self.designer_selected_node.clone() {
                                let new_x = wm.0 + self.designer_drag_offset.0;
                                let new_y = wm.1 + self.designer_drag_offset.1;
                                self.toast(format!("Drag: moving to ({:.1}, {:.1})", new_x, new_y));
                                let _ = self.project_doc.update_node(sel_id, scene::NodeUpdate::Move { x: new_x, y: new_y });
                            }
                        }

                        if !pointer.primary_down() && self.designer_drag_active {
                            self.designer_drag_active = false;
                            self.toast("Drag released");
                        }
                    }
                }
                
                for ev in &arena_events {
                    match ev {
                        CanvasEvent::NodeClicked { id, shift_held } => {
                            if *shift_held {
                                // Toggle: remove if already selected, add if not
                                if let Some(pos) = self.editor_state.selected_node_ids.iter().position(|sid| sid == id) {
                                    self.editor_state.selected_node_ids.remove(pos);
                                } else {
                                    self.editor_state.selected_node_ids.push(id.clone());
                                }
                            } else {
                                self.editor_state.selected_node_ids = vec![id.clone()];
                            }
                        }
                        CanvasEvent::NodeDragged(id, delta) => {
                            let _ = self.project_doc.move_node(id, *delta);
                            // Multi-drag: if a selected node was dragged, move all selected nodes
                            if self.editor_state.selected_node_ids.contains(id) {
                                let others: Vec<String> = self.editor_state.selected_node_ids.iter()
                                    .filter(|sid| *sid != id)
                                    .cloned()
                                    .collect();
                                for other_id in &others {
                                    let _ = self.project_doc.move_node(other_id, *delta);
                                }
                            }
                        }
                        CanvasEvent::NodeResized { id, handle, delta } => {
                            let _ = self.project_doc.resize_node(id, *handle, *delta);
                        }
                        CanvasEvent::NodeModified { id, update } => {
                            let _ = self.project_doc.update_node(id, update.clone());
                        }
                        CanvasEvent::NodeResizeStarted => {} // Handled before loop
                        CanvasEvent::ClearSelection => {
                            self.editor_state.selected_node_ids.clear();
                        }
                        CanvasEvent::ActionTriggered { .. } => {} // not used in designer mode
                        CanvasEvent::DeleteNode { id } => {
                            self.project_doc.delete_node(id);
                            self.designer_selected_node = None;
                            self.editor_state.selected_node_ids.clear();
                            self.toast("Node deleted");
                        }
                    }
                }

                // ── Render snap guides ──
                // ── Viewport info ──
                pnt.text(egui::pos2(r.left()+8., r.bottom()-4.), egui::Align2::LEFT_BOTTOM,
                    &format!("Zoom: {:.0}%  |  Pan: ({:.0}, {:.0})  |  {} nodes",
                        self.viewport.zoom * 100., self.viewport.pan.x, self.viewport.pan.y, self.project_doc.nodes.len()),
                    egui::FontId::proportional(9.), theme::TEXT_DIM);

                if self.project_doc.nodes.is_empty() {
                    pnt.text(r.center(), egui::Align2::CENTER_CENTER,
                        "Click widgets from the left panel to start designing",
                        egui::FontId::proportional(14.), theme::TEXT_DIM);
                }
            } else if self.mode == Mode::Play {
                // ── PLAY MODE ──
                let _canvas_origin = r.left_top();
                let play_events = renderer::draw_document(ui, &self.project_doc, &self.play_viewport, &self.editor_state, true, &mut self.form_state, self.active_play_page.as_deref());
                for ev in &play_events {
                    match ev {
                        CanvasEvent::ActionTriggered { source_node_id } => {
                            let trigger_node = self.project_doc.flow_graph.nodes.values().find(|n| {
                                if let crate::flow::FlowNodeKind::TriggerClick { ref target_node_id } = n.kind {
                                    target_node_id == source_node_id
                                } else {
                                    false
                                }
                            }).cloned();
                            let Some(trigger) = trigger_node else { continue; };
                            let edge = self.project_doc.flow_graph.edges.iter().find(|e| e.from_node == trigger.id).cloned();
                            let Some(target_edge) = edge else { continue; };
                            let Some(target_node) = self.project_doc.flow_graph.nodes.get(&target_edge.to_node).cloned() else { continue; };
                            match target_node.kind {
                                crate::flow::FlowNodeKind::SaveToDatabase { ref entity } => {
                                    if entity.is_empty() {
                                        self.toast("Flow Error: Target entity not configured".to_string());
                                        continue;
                                    }
                                    let mut data = serde_json::Map::new();
                                    for (nid, node) in &self.project_doc.nodes {
                                        let (_bound_entity, bound_field) = match &node.node_type {
                                            scene::NodeType::TextInput { bound_entity, bound_field, .. }
                                            | scene::NodeType::Dropdown { bound_entity, bound_field, .. }
                                            | scene::NodeType::NumberField { bound_entity, bound_field, .. }
                                            | scene::NodeType::Checkbox { bound_entity, bound_field, .. } => (bound_entity, bound_field),
                                            _ => continue,
                                        };
                                        if let Some(field) = bound_field {
                                            if !field.is_empty() {
                                                let value = self.form_state.get(nid).cloned().unwrap_or_default();
                                                data.insert(field.clone(), serde_json::Value::String(value));
                                            }
                                        }
                                    }
                                    let json_data = serde_json::Value::Object(data);
                                    match &self.db_conn {
                                        Some(conn) => {
                                            match db::upsert_record(conn, entity, None, &json_data) {
                                                Ok(id) => {
                                                    self.form_state.clear();
                                                    self.toast(format!("Saved {} ✓ (id: {})", entity, &id[..8.min(id.len())]));
                                                }
                                                Err(e) => self.toast(format!("Save failed: {}", e)),
                                            }
                                        }
                                        None => self.toast("DB not connected".to_string()),
                                    }
                                }
                                crate::flow::FlowNodeKind::NavigateTo { ref page_id } => {
                                    if self.project_doc.nodes.contains_key(page_id) {
                                        self.active_play_page = Some(page_id.clone());
                                        self.form_state.clear();
                                        self.toast(format!("Navigated to page: {}", page_id));
                                    } else {
                                        self.toast(format!("Flow Error: Page '{}' not found", page_id));
                                    }
                                }
                                crate::flow::FlowNodeKind::TriggerClick { .. } => {
                                    self.toast("Flow Error: Trigger node cannot be a target".to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if self.project_doc.nodes.is_empty() {
                    pnt.text(r.center(), egui::Align2::CENTER_CENTER,
                        "Design your form in Designer mode, then switch to Play to interact",
                        egui::FontId::proportional(14.), theme::TEXT_DIM);
                }
            } else if self.mode == Mode::DataViewer {
                // ── DATA VIEWER ──
                let mut load_clicked = false;
                let mut edit_clicked: Option<String> = None;
                let mut delete_clicked: Option<String> = None;
                data_viewer::draw_data_viewer(ui, &mut self.viewer_entity_input, &self.viewer_records, &mut load_clicked, &mut edit_clicked, &mut delete_clicked);

                if let Some(edit_id) = edit_clicked {
                    if let Some((_, data)) = self.viewer_records.iter().find(|(id, _)| id == &edit_id) {
                        self.editing_record = Some((edit_id, data.clone()));
                    }
                }

                if let Some(delete_id) = delete_clicked {
                    let entity = self.viewer_selected_entity.clone();
                    let delete_result = self.db_conn.as_ref().map(|conn| db::delete_record(conn, &delete_id));
                    match delete_result {
                        Some(Ok(())) => {
                            self.toast("Deleted ✓");
                            if !entity.is_empty() {
                                if let Some(conn) = &self.db_conn {
                                    match db::get_records(conn, &entity) {
                                        Ok(recs) => { self.viewer_records = recs; }
                                        Err(e) => { self.toast(format!("Reload failed: {}", e)); }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => self.toast(format!("Delete failed: {}", e)),
                        None => self.toast("DB not connected"),
                    }
                }

                if load_clicked && !self.viewer_entity_input.is_empty() && self.db_conn.is_some() {
                    let entity = self.viewer_entity_input.clone();
                    match db::get_records(self.db_conn.as_ref().unwrap(), &entity) {
                        Ok(recs) => {
                            self.viewer_records = recs;
                            self.viewer_selected_entity = entity;
                            self.toast(format!("Loaded {} records ✓", self.viewer_records.len()));
                        }
                        Err(e) => {
                            self.viewer_records.clear();
                            self.viewer_selected_entity = entity;
                            self.toast(format!("Query failed: {}", e));
                        }
                    }
                }
            } else if self.mode == Mode::FlowBuilder {
                // ── FLOW BUILDER ──
                let canvas_origin = r.left_top();
                self.flow_canvas_center = self.flow_viewport.screen_to_world(r.center(), canvas_origin).into();

                // Viewport pan (middle mouse or Space+drag)
                let scroll = ui.input(|i| i.raw_scroll_delta);
                if scroll.y != 0. {
                    self.flow_viewport.zoom = (self.flow_viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
                    if let Some(cursor) = mpos {
                        let world = self.flow_viewport.screen_to_world(cursor, canvas_origin);
                        self.flow_viewport.pan.x = cursor.x - world.x * self.flow_viewport.zoom - canvas_origin.x;
                        self.flow_viewport.pan.y = cursor.y - world.y * self.flow_viewport.zoom - canvas_origin.y;
                    }
                }
                let middle = ui.input(|i| i.pointer.middle_down());
                let space = ui.input(|i| i.key_down(egui::Key::Space));
                let is_panning = middle || (space && mdown);
                if is_panning {
                    let delta = ui.input(|i| i.pointer.delta());
                    self.flow_viewport.pan += delta;
                }

                // Dot-matrix grid
                let world_tl = self.flow_viewport.screen_to_world(r.left_top(), canvas_origin);
                let world_br = self.flow_viewport.screen_to_world(r.right_bottom(), canvas_origin);
                let start_x = (world_tl.x / 24.0).floor() * 24.0;
                let start_y = (world_tl.y / 24.0).floor() * 24.0;
                let mut wy = start_y;
                while wy <= world_br.y {
                    let mut wx = start_x;
                    while wx <= world_br.x {
                        let sp = self.flow_viewport.world_to_screen(egui::pos2(wx, wy), canvas_origin);
                        if r.contains(sp) {
                            pnt.circle_filled(sp, (1.0_f32).max(0.5 * self.flow_viewport.zoom), Color32::from_rgb(25, 25, 25));
                        }
                        wx += 24.0;
                    }
                    wy += 24.0;
                }

                // Render flow graph
                flow_renderer::draw_flow_graph(
                    ui, &pnt, &self.project_doc.flow_graph,
                    &self.flow_viewport, canvas_origin, r,
                    self.selected_flow_node.as_deref(),
                    self.flow_wiring_source.as_deref(),
                    self.flow_wiring_pos,
                );

                // ── Node interaction (wiring, select, drag) ──
                let mclick = ui.input(|i| i.pointer.primary_clicked());
                let mdown = ui.input(|i| i.pointer.primary_down());
                let mreleased = ui.input(|i| i.pointer.primary_released());
                let mdelta = ui.input(|i| i.pointer.delta());

                // Update wiring end position while dragging
                if self.flow_wiring_source.is_some() {
                    if let Some(cursor) = mpos {
                        let world = self.flow_viewport.screen_to_world(cursor, canvas_origin);
                        self.flow_wiring_pos = Some((world.x, world.y));
                    }
                }

                if mclick {
                    if is_panning {
                        self.selected_flow_node = None;
                    } else if let Some(cursor) = mpos {
                        let world = self.flow_viewport.screen_to_world(cursor, canvas_origin);
                        let world_pos = (world.x, world.y);

                        // Check output port hit first (wiring)
                        let mut port_hit: Option<String> = None;
                        for node in self.project_doc.flow_graph.nodes.values() {
                            if flow_renderer::is_over_output_port(node.position, world_pos) {
                                port_hit = Some(node.id.clone());
                                break;
                            }
                        }
                        if let Some(src_id) = port_hit {
                            self.flow_wiring_source = Some(src_id);
                            self.flow_wiring_pos = Some(world_pos);
                        } else {
                            // Node body hit → select and drag
                            let mut hit: Option<String> = None;
                            for node in self.project_doc.flow_graph.nodes.values() {
                                let nx = node.position.0;
                                let ny = node.position.1;
                                if world.x >= nx && world.x <= nx + flow_renderer::NODE_W
                                    && world.y >= ny && world.y <= ny + flow_renderer::NODE_H
                                {
                                    hit = Some(node.id.clone());
                                    break;
                                }
                            }
                            self.selected_flow_node = hit.clone();
                            if hit.is_some() {
                                self.flow_drag_active = true;
                            } else {
                                self.selected_flow_node = None;
                                // Check edge hit → click-to-delete
                                let mut edge_to_remove: Option<usize> = None;
                                for (idx, edge) in self.project_doc.flow_graph.edges.iter().enumerate() {
                                    let from_pos = match self.project_doc.flow_graph.nodes.get(&edge.from_node) {
                                        Some(n) => flow_renderer::node_output_port(n.position),
                                        None => continue,
                                    };
                                    let to_pos = match self.project_doc.flow_graph.nodes.get(&edge.to_node) {
                                        Some(n) => flow_renderer::node_input_port(n.position),
                                        None => continue,
                                    };
                                    if flow_renderer::is_near_line_segment(world_pos, from_pos, to_pos, 8.0) {
                                        edge_to_remove = Some(idx);
                                        break;
                                    }
                                }
                                if let Some(idx) = edge_to_remove {
                                    self.project_doc.flow_graph.edges.remove(idx);
                                    self.toast("Edge removed");
                                }
                            }
                        }
                    } else {
                        self.selected_flow_node = None;
                    }
                }

                if self.flow_drag_active && mdown && !is_panning {
                    if let Some(ref sel_id) = self.selected_flow_node.clone() {
                        let world_delta = mdelta / self.flow_viewport.zoom;
                        if let Some(node) = self.project_doc.flow_graph.nodes.get_mut(sel_id) {
                            node.position.0 += world_delta.x;
                            node.position.1 += world_delta.y;
                        }
                    }
                }

                if mreleased {
                    if self.flow_wiring_source.is_some() {
                        // Check if released over a different node
                        let mut target: Option<String> = None;
                        if let Some(cursor) = mpos {
                            let world = self.flow_viewport.screen_to_world(cursor, canvas_origin);
                            for node in self.project_doc.flow_graph.nodes.values() {
                                let nx = node.position.0;
                                let ny = node.position.1;
                                if world.x >= nx && world.x <= nx + flow_renderer::NODE_W
                                    && world.y >= ny && world.y <= ny + flow_renderer::NODE_H
                                {
                                    target = Some(node.id.clone());
                                    break;
                                }
                            }
                        }
                        if let Some(src) = self.flow_wiring_source.clone() {
                            if let Some(tgt) = target {
                                if src != tgt {
                                    // Check edge doesn't already exist
                                    let already = self.project_doc.flow_graph.edges.iter()
                                        .any(|e| e.from_node == src && e.to_node == tgt);
                                    if !already {
                                        self.project_doc.flow_graph.edges.push(crate::flow::FlowEdge {
                                            from_node: src,
                                            to_node: tgt,
                                        });
                                    }
                                }
                            }
                        }
                        self.flow_wiring_source = None;
                        self.flow_wiring_pos = None;
                    }
                    self.flow_drag_active = false;
                }

                // ── Delete flow node on keyboard shortcut ──
                if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
                    if let Some(id) = self.selected_flow_node.take() {
                        self.project_doc.flow_graph.remove_node(&id);
                        self.toast("Flow node deleted");
                    }
                }

                // Status info
                pnt.text(egui::pos2(r.left()+8., r.bottom()-4.), egui::Align2::LEFT_BOTTOM,
                    &format!("Zoom: {:.0}%  |  {} flow nodes, {} edges",
                        self.flow_viewport.zoom * 100.,
                        self.project_doc.flow_graph.nodes.len(),
                        self.project_doc.flow_graph.edges.len(),
                    ),
                    egui::FontId::proportional(9.), Color32::from_rgb(130, 130, 130));

                if self.project_doc.flow_graph.nodes.is_empty() {
                    pnt.text(r.center(), egui::Align2::CENTER_CENTER,
                        "No flow nodes yet. Add triggers and actions to build your automation.",
                        egui::FontId::proportional(14.), Color32::from_rgb(130, 130, 130));
                }
            } else {
                // ── FLOW MODE ──
                // helper: get handle positions for a node
                let node_handles = |n: &UiFlowNode| -> (egui::Pos2, egui::Pos2) {
                    (egui::pos2(r.left()+n.x+100., r.top()+n.y),      // top (input)
                     egui::pos2(r.left()+n.x+100., r.top()+n.y+60.))  // bottom (output)
                };
                let handle_hit = |pos: egui::Pos2, center: egui::Pos2| -> bool {
                    (pos - center).length() < 20.
                };
                // Check if mpos is near any handle
                let hovered_handle = mpos.and_then(|pos| {
                    self.fns.iter().find_map(|n| {
                        let (top, bot) = node_handles(n);
                        if handle_hit(pos, top) || handle_hit(pos, bot) { Some(n.id.clone()) }
                        else { None }
                    })
                });

                // Draw edges
                for (_, src, tgt) in &self.fes {
                    let s = self.fns.iter().find(|n| n.id==*src);
                    let d = self.fns.iter().find(|n| n.id==*tgt);
                    if let (Some(sn),Some(dn))=(s,d) {
                        let (_, sp) = node_handles(sn);
                        let (dp, _) = node_handles(dn);
                        pnt.line_segment([sp,dp], Stroke::new(1.5, theme::ACCENT));
                        let dir=(dp-sp).normalized(); let per=egui::vec2(-dir.y,dir.x);
                        pnt.line_segment([dp,dp-dir*8.+per*4.], Stroke::new(1.5, theme::ACCENT));
                        pnt.line_segment([dp,dp-dir*8.-per*4.], Stroke::new(1.5, theme::ACCENT));
                    }
                }
                // Temporary connection line (from source bottom → cursor)
                if let (Some(src_id), Some(pos)) = (&self.flow_con, mpos) {
                    if let Some(sn) = self.fns.iter().find(|n| n.id==*src_id) {
                        let (_, sp) = node_handles(sn);
                        // Draw a bezier-like curved line
                        let mid = egui::pos2((sp.x+pos.x)/2., sp.y.max(pos.y).min(sp.y + (pos.y-sp.y).abs()/2.));
                        pnt.line_segment([sp, mid], Stroke::new(2., theme::ACCENT));
                        pnt.line_segment([mid, pos], Stroke::new(2., theme::ACCENT_ORANGE));
                        // Draw a ring at cursor to show drop target
                        pnt.circle_stroke(pos, 12., Stroke::new(1.5, theme::ACCENT_ORANGE));
                    }
                }

                // Node drag — skip if clicking on a handle
                if mpos.is_some() && hovered_handle.is_none() {
                    if mdown && self.fdrag.is_none() && self.flow_con.is_none() {
                        let c = to_c(mpos.unwrap());
                        for n in self.fns.iter().rev() {
                            if Rect::from_min_size(egui::pos2(n.x,n.y),egui::vec2(200.,60.)).contains(c) {
                                self.sel_fn = Some(n.id.clone());
                                self.fdrag = Some((n.id.clone(), egui::vec2(n.x,n.y), egui::vec2(c.x,c.y)));
                                break;
                            }
                        }
                    }
                }
                if let Some((id,sp,off)) = self.fdrag.clone() {
                    if mdown && self.flow_con.is_none() {
                        if let Some(pos)=mpos { let c=to_c(pos); if let Some(n)=self.fns.iter_mut().find(|n|n.id==id){n.x=(sp.x+c.x-off.x).max(0.);n.y=(sp.y+c.y-off.y).max(0.);} }
                    } else { self.fdrag = None; }
                }

                // Handle clicks on node handles for edge creation
                if mup {
                    if let Some(pos) = mpos {
                        for n in &self.fns {
                            let (top_h, bot_h) = node_handles(n);
                            let on_handle = handle_hit(pos, top_h) || handle_hit(pos, bot_h);
                            if on_handle {
                                if self.flow_con.is_none() {
                                    self.flow_con = Some(n.id.clone());
                                    self.toast("Click another node to connect");
                                } else if let Some(src) = &self.flow_con {
                                    if *src != n.id {
                                        let eid = format!("e{}", self.fes.len()+1);
                                        self.fes.push((eid, src.clone(), n.id.clone()));
                                        self.toast("Connected ✓");
                                    }
                                    self.flow_con = None;
                                }
                                return;
                            }
                        }
                        if self.flow_con.is_some() {
                            self.flow_con = None;
                            self.toast("Connection cancelled");
                        }
                    }
                }

                // Draw nodes (selected last = on top)
                let mut fns_sorted: Vec<_> = self.fns.iter().enumerate().collect();
                fns_sorted.sort_by_key(|(_,n)| if self.sel_fn.as_ref() == Some(&n.id) { 1 } else { 0 });
                for (_, n) in &fns_sorted {
                    let nr = Rect::from_min_size(egui::pos2(r.left()+n.x, r.top()+n.y), egui::vec2(200.,60.));
                    let sel = self.sel_fn.as_ref() == Some(&n.id);
                    let hcol = match n.nt.as_str() { "trigger"=>theme::HEADER_TRIGGER, "action"=>theme::HEADER_ACTION, "condition"=>theme::HEADER_CONDITION, "logicalGate"=>theme::HEADER_GATE, _=>theme::BORDER };
                    pnt.rect_filled(nr, 0, theme::WIDGET_BG);
                    pnt.rect_stroke(nr, 0, Stroke::new(if sel{2.}else{1.}, if sel{theme::SELECTED}else{theme::BORDER}), egui::StrokeKind::Outside);
                    // Header
                    let hr = Rect::from_min_size(nr.min, egui::vec2(200.,20.));
                    pnt.rect_filled(hr, 0, hcol);
                    pnt.text(egui::pos2(hr.left()+6., hr.center().y), egui::Align2::LEFT_CENTER, &n.nt, egui::FontId::proportional(9.), Color32::from_rgb(0,0,0));
                    pnt.text(egui::pos2(nr.left()+6., nr.top()+34.), egui::Align2::LEFT_CENTER, &n.label, egui::FontId::proportional(10.), theme::TEXT);
                    // Handles — larger, with hover highlight
                    let (th, bh) = node_handles(n);
                    let is_hovered = hovered_handle.as_deref() == Some(&n.id);
                    let hr_col = if is_hovered { theme::ACCENT } else { hcol };
                    pnt.circle_filled(th, 6., if is_hovered{theme::ACCENT_DIM}else{theme::BORDER});
                    pnt.circle_stroke(th, 6., Stroke::new(1.5, hr_col));
                    pnt.circle_filled(bh, 6., if is_hovered{theme::ACCENT_DIM}else{theme::BORDER});
                    pnt.circle_stroke(bh, 6., Stroke::new(1.5, hr_col));
                    // Tooltip on hover
                    if is_hovered {
                        pnt.text(egui::pos2(th.x, th.y-12.), egui::Align2::CENTER_CENTER,
                            "● click to connect", egui::FontId::proportional(7.), theme::ACCENT);
                    }
                }
                if self.fns.is_empty() { pnt.text(r.center(),egui::Align2::CENTER_CENTER,"Add flow nodes from the left panel",egui::FontId::proportional(14.),theme::TEXT_DIM); }
            }

            // Toast
            if let Some(msg) = &self.toast {
                let tr = Rect::from_min_size(egui::pos2(r.center().x-120., r.bottom()-40.), egui::vec2(240.,28.));
                pnt.rect_filled(tr, 0, Color32::from_rgba_premultiplied(20,20,20,220));
                pnt.rect_stroke(tr, 0, Stroke::new(1.,theme::ACCENT), egui::StrokeKind::Outside);
                pnt.text(tr.center(), egui::Align2::CENTER_CENTER, msg, egui::FontId::proportional(11.), theme::TEXT);
            }

            // Right-click context menu on designer canvas
            if self.mode == Mode::Designer {
                resp.context_menu(|ui| {
                    if ui.button("Go to Contracts").clicked() { self.mode = Mode::Contacts; ui.close_menu(); }
                    if ui.button("Go to Pipeline").clicked() { self.mode = Mode::Pipeline; ui.close_menu(); }
                    if ui.button("Go to Tasks").clicked() { self.mode = Mode::Tasks; ui.close_menu(); }
                    if ui.button("Go to Studio").clicked() { self.mode = Mode::Studio; ui.close_menu(); }
                });
            }
        });

        // ── EDIT RECORD MODAL ──
        if let Some((edit_id, mut edit_data)) = self.editing_record.clone() {
            let mut close_modal = false;
            let mut save_modal = false;

            egui::Window::new("Edit Record")
                .collapsible(false)
                .resizable(true)
                .default_width(360.)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(obj) = edit_data.as_object_mut() {
                            // Collect keys to avoid double borrow issues
                            let keys: Vec<String> = obj.keys().cloned().collect();
                            for key in &keys {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(key.as_str()).size(10.).color(crate::theme::ACCENT));
                                    ui.add_space(8.);
                                    if let Some(val) = obj.get_mut(key) {
                                        match val {
                                            serde_json::Value::String(s) => {
                                                let mut temp = s.clone();
                                                if ui.text_edit_singleline(&mut temp).changed() {
                                                    *s = temp;
                                                }
                                            }
                                            serde_json::Value::Number(n) => {
                                                let mut temp = n.to_string();
                                                if ui.text_edit_singleline(&mut temp).changed() {
                                                    if let Ok(num) = temp.parse::<f64>() {
                                                        // ponytail: store as f64 number, loses int distinction
                                                        if num.fract() == 0.0 && temp.find('.').is_none() {
                                                            *val = serde_json::Value::Number(serde_json::Number::from(num as i64));
                                                        } else {
                                                            *val = serde_json::json!(num);
                                                        }
                                                    }
                                                }
                                            }
                                            _ => {
                                                let mut temp = val.to_string();
                                                if ui.text_edit_singleline(&mut temp).changed() {
                                                    *val = serde_json::Value::String(temp);
                                                }
                                            }
                                        }
                                    }
                                });
                                ui.add_space(4.);
                            }
                        } else {
                            ui.label("Record data is not a JSON object.");
                        }
                    });
                    ui.add_space(8.);
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("💾 Save").min_size(egui::vec2(80., 24.))).clicked() {
                            save_modal = true;
                        }
                        if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(80., 24.))).clicked() {
                            close_modal = true;
                        }
                    });
                });

            if save_modal {
                let entity = self.viewer_selected_entity.clone();
                let save_result = self.db_conn.as_ref().map(|conn| db::upsert_record(conn, &entity, Some(&edit_id), &edit_data));
                match save_result {
                    Some(Ok(_)) => {
                        self.toast("Saved ✓");
                        if !entity.is_empty() {
                            if let Some(conn) = &self.db_conn {
                                match db::get_records(conn, &entity) {
                                    Ok(recs) => { self.viewer_records = recs; }
                                    Err(e) => { self.toast(format!("Reload failed: {}", e)); }
                                }
                            }
                        }
                        self.editing_record = None;
                    }
                    Some(Err(e)) => self.toast(format!("Save failed: {}", e)),
                    None => self.toast("DB not connected"),
                }
            }
            if close_modal {
                self.editing_record = None;
            }
        }

        // ── LOGIN ──
        if self.show_login {
            egui::Window::new("Sign In").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.,0.)).show(ctx, |ui| {
                ui.label(egui::RichText::new("Continue without signing in, or log in for sync.").size(11.).color(theme::TEXT));
                ui.add_space(8.);
                if ui.button(egui::RichText::new("Sign In (demo)").size(10.)).clicked() { self.auth=Some("demo".into()); self.show_login=false; self.toast("Signed in"); }
                if ui.button(egui::RichText::new("Continue Offline").size(10.)).clicked() { self.show_login=false; }
            });
        }
    }
}



fn configure_egui_style(ctx: &egui::Context) {
    use egui::style::*;
    ctx.set_pixels_per_point(1.25);
    let rounding = egui::CornerRadius::same(6);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10., 8.);
    style.spacing.button_padding = egui::vec2(14., 6.);
    style.spacing.indent = 16.;
    style.spacing.icon_width = 18.;
    style.spacing.icon_width_inner = 14.;
    style.spacing.interact_size = egui::vec2(40., 24.);
    style.spacing.slider_width = 140.;
    style.visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(theme::TEXT),
        window_fill: theme::ELEVATED,
        panel_fill: theme::PANEL,
        faint_bg_color: theme::PANEL,
        extreme_bg_color: theme::BG,
        code_bg_color: theme::WIDGET_BG,
        window_corner_radius: rounding,
        menu_corner_radius: rounding,
        warn_fg_color: theme::ACCENT_ORANGE,
        error_fg_color: theme::ACCENT_RED,
        hyperlink_color: theme::ACCENT,
        selection: Selection { bg_fill: theme::ACCENT_DIM, stroke: Stroke::new(1., theme::ACCENT) },
        widgets: Widgets {
            noninteractive: WidgetVisuals { bg_fill: theme::PANEL, weak_bg_fill: theme::WIDGET_BG, bg_stroke: Stroke::new(1., theme::BORDER), fg_stroke: Stroke::new(1., theme::TEXT_DIM), expansion: 0., corner_radius: rounding },
            inactive: WidgetVisuals { bg_fill: theme::WIDGET_BG, weak_bg_fill: theme::PANEL, bg_stroke: Stroke::new(1., theme::BORDER), fg_stroke: Stroke::new(1., theme::TEXT), expansion: 0., corner_radius: rounding },
            hovered: WidgetVisuals { bg_fill: theme::HOVER, weak_bg_fill: theme::PANEL, bg_stroke: Stroke::new(1., theme::FOCUS), fg_stroke: Stroke::new(1.5, theme::TEXT), expansion: 1., corner_radius: rounding },
            active: WidgetVisuals { bg_fill: theme::ACTIVE, weak_bg_fill: theme::PANEL, bg_stroke: Stroke::new(1., theme::ACCENT), fg_stroke: Stroke::new(2., theme::TEXT), expansion: 1., corner_radius: rounding },
            open: WidgetVisuals { bg_fill: theme::ELEVATED, weak_bg_fill: theme::PANEL, bg_stroke: Stroke::new(1., theme::ACCENT), fg_stroke: Stroke::new(1.5, theme::TEXT), expansion: 0., corner_radius: rounding },
        },
        popup_shadow: egui::epaint::Shadow { offset: [0, 2], blur: 12, spread: 0, color: Color32::from_black_alpha(80) },
        window_shadow: egui::epaint::Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(100) },
        ..Default::default()
    };
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::proportional(20.)),
        (egui::TextStyle::Body, egui::FontId::proportional(14.)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(13.)),
        (egui::TextStyle::Button, egui::FontId::proportional(13.)),
        (egui::TextStyle::Small, egui::FontId::proportional(11.)),
    ].into();
    ctx.set_style(style);
}

fn task_status_label(s: &TaskStatus) -> &'static str {
    match s { TaskStatus::Open => "Open", TaskStatus::InProgress => "In Progress", TaskStatus::Done => "Done", TaskStatus::Cancelled => "Cancelled" }
}
fn task_priority_label(p: &TaskPriority) -> &'static str {
    match p { TaskPriority::Low => "Low", TaskPriority::Medium => "Medium", TaskPriority::High => "High" }
}
fn task_priority_color(p: &TaskPriority) -> Color32 {
    match p { TaskPriority::Low => Color32::from_rgb(100,180,100), TaskPriority::Medium => theme::ACCENT_ORANGE, TaskPriority::High => theme::ACCENT_RED }
}
fn task_status_icon(s: &TaskStatus) -> &'static str {
    match s { TaskStatus::Open => "○", TaskStatus::InProgress => "◐", TaskStatus::Done => "●", TaskStatus::Cancelled => "✕" }
}

fn tool_name(t: &StudioTool) -> &'static str {
    STUDIO_TOOLS.iter().find(|(tool,_,_)| tool==t).map(|(_,_,n)| *n).unwrap_or("?")
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native("Proteus - The Visual OS for Business",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1400., 900.])
                .with_min_inner_size([800., 500.]),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(ProteusApp::default()))),
    )
}
