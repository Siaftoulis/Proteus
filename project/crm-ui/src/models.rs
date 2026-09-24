use eframe::egui::{self, Color32, Pos2, Vec2};
use serde::{Deserialize, Serialize};
use crate::theme;

pub const GRID: f32 = 20.0;

#[derive(Clone, PartialEq, Debug)]
pub enum Mode {
    Designer,
    Analyst,
    Networking,
    Troubleshoot,
    ConnectedData,
    Play,
    DataViewer,
    FlowBuilder,
    Contacts,
    Pipeline,
    Studio,
    Tasks,
    Settings,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LayoutMode {
    Free,
    Grid,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DesignerTool {
    Select,
    Rectangle,
    Text,
    Button,
    Table,
    Crop,
    Eraser,
    Hand,
    Zoom,
    Ruler,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum DesignerLeftTab {
    #[default]
    Layers,
    Assets,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum InspectorTab {
    #[default]
    Design,
    Prototype,
}

#[derive(Clone, PartialEq, Debug)]
pub enum DevicePreset {
    DesktopHD,
    Desktop,
    Laptop,
    TabletLandscape,
    TabletPortrait,
    Phone,
    PhoneSmall,
    Custom(f32, f32),
}

impl DevicePreset {
    pub fn label(&self) -> &str {
        match self {
            Self::DesktopHD => "Desktop HD",
            Self::Desktop => "Desktop",
            Self::Laptop => "Laptop",
            Self::TabletLandscape => "Tablet (L)",
            Self::TabletPortrait => "Tablet (P)",
            Self::Phone => "Phone",
            Self::PhoneSmall => "Phone (S)",
            Self::Custom(_, _) => "Custom",
        }
    }
    pub fn size(&self) -> (f32, f32) {
        match self {
            Self::DesktopHD => (1920., 1080.),
            Self::Desktop => (1440., 900.),
            Self::Laptop => (1366., 768.),
            Self::TabletLandscape => (1024., 768.),
            Self::TabletPortrait => (768., 1024.),
            Self::Phone => (375., 812.),
            Self::PhoneSmall => (320., 568.),
            Self::Custom(w, h) => (*w, *h),
        }
    }
    pub fn icon(&self) -> &str {
        match self {
            Self::DesktopHD | Self::Desktop | Self::Laptop => "🖥",
            Self::TabletLandscape | Self::TabletPortrait => "📱",
            Self::Phone | Self::PhoneSmall => "📲",
            Self::Custom(_, _) => "⚙",
        }
    }
}

// ── Studio (Design Tool) types ──
#[derive(Clone, PartialEq, Debug)]
pub enum StudioTool {
    Select,
    Brush,
    Rectangle,
    Ellipse,
    Line,
    Text,
    Fill,
    Eraser,
    Picker,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StudioLayer {
    pub id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub z: i32,
    pub visible: bool,
    pub opacity: f32,
    pub locked: bool,
    pub content: LayerContent,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LayerContent {
    Shape {
        kind: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill_r: u8,
        fill_g: u8,
        fill_b: u8,
        fill_a: u8,
        stroke_r: u8,
        stroke_g: u8,
        stroke_b: u8,
        stroke_a: u8,
        stroke_width: f32,
    },
    Stroke {
        points: Vec<[f32; 2]>,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
        size: f32,
    },
    Text {
        content: String,
        font_size: f32,
        r: u8,
        g: u8,
        b: u8,
        a: u8,
    },
    Image {
        data_base64: String,
    },
}

pub const STUDIO_TOOLS: &[(StudioTool, &str, &str)] = &[
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

pub fn tool_name(t: &StudioTool) -> &'static str {
    STUDIO_TOOLS.iter().find(|(tool, _, _)| tool == t).map(|(_, _, n)| *n).unwrap_or("?")
}

// ── Contact & Deal types ──
#[derive(Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub company: String,
    pub tags: Vec<String>,
    pub notes: Vec<ContactNote>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ContactNote {
    pub id: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: String,
    pub title: String,
    pub value: f64,
    pub stage: String,
    pub contact_id: Option<String>,
    pub expected_close: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

pub const KANBAN_STAGES: &[&str] = &["New", "Contacted", "Qualified", "Proposal", "Negotiation", "Closed Won", "Closed Lost"];

// ── Tasks types ──
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum TaskStatus {
    Open,
    InProgress,
    Done,
    Cancelled,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub contact_id: Option<String>,
    pub deal_id: Option<String>,
    pub assignee: String,
    pub due_date: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

pub fn task_status_label(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Open => "Open",
        TaskStatus::InProgress => "In Progress",
        TaskStatus::Done => "Done",
        TaskStatus::Cancelled => "Cancelled",
    }
}

pub fn task_priority_label(p: &TaskPriority) -> &'static str {
    match p {
        TaskPriority::Low => "Low",
        TaskPriority::Medium => "Medium",
        TaskPriority::High => "High",
    }
}

pub fn task_priority_color(p: &TaskPriority) -> Color32 {
    match p {
        TaskPriority::Low => Color32::from_rgb(100, 180, 100),
        TaskPriority::Medium => theme::ACCENT_ORANGE,
        TaskPriority::High => theme::ACCENT_RED,
    }
}

pub fn task_status_icon(s: &TaskStatus) -> &'static str {
    match s {
        TaskStatus::Open => "○",
        TaskStatus::InProgress => "◐",
        TaskStatus::Done => "●",
        TaskStatus::Cancelled => "✕",
    }
}

// ── 2D Viewport ──
#[derive(Clone)]
pub struct Viewport2D {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for Viewport2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Viewport2D {
    pub fn new() -> Self {
        Self { pan: Vec2::ZERO, zoom: 1.0 }
    }
    pub fn world_to_screen(&self, world: Pos2, canvas_origin: Pos2) -> Pos2 {
        egui::pos2(
            world.x * self.zoom + self.pan.x + canvas_origin.x,
            world.y * self.zoom + self.pan.y + canvas_origin.y,
        )
    }
    pub fn screen_to_world(&self, screen: Pos2, canvas_origin: Pos2) -> Pos2 {
        egui::pos2(
            (screen.x - self.pan.x - canvas_origin.x) / self.zoom,
            (screen.y - self.pan.y - canvas_origin.y) / self.zoom,
        )
    }
}

// ── Variable Refresh Rate (VRR) Engine ──
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum VrrMode {
    Reactive,
    Fps30,
    #[default]
    Fps60,
    Fps120,
    Continuous,
}

impl VrrMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Reactive => "Reactive (0% Idle)",
            Self::Fps30 => "30 FPS (Power Saver)",
            Self::Fps60 => "60 FPS (Balanced)",
            Self::Fps120 => "120 FPS (High Refresh)",
            Self::Continuous => "Continuous (Uncapped)",
        }
    }

    pub fn fps_badge(&self) -> &'static str {
        match self {
            Self::Reactive => "VRR: Reactive",
            Self::Fps30 => "VRR: 30Hz",
            Self::Fps60 => "VRR: 60Hz",
            Self::Fps120 => "VRR: 120Hz",
            Self::Continuous => "VRR: Max",
        }
    }

    pub fn target_duration(&self) -> Option<std::time::Duration> {
        match self {
            Self::Reactive => None,
            Self::Fps30 => Some(std::time::Duration::from_millis(33)),
            Self::Fps60 => Some(std::time::Duration::from_millis(16)),
            Self::Fps120 => Some(std::time::Duration::from_millis(8)),
            Self::Continuous => Some(std::time::Duration::ZERO),
        }
    }

    pub fn apply_to_ctx(&self, ctx: &egui::Context, is_active_interaction: bool) {
        match self {
            Self::Reactive if is_active_interaction => ctx.request_repaint_after(std::time::Duration::from_millis(16)),
            Self::Reactive => {}
            Self::Fps30 => ctx.request_repaint_after(std::time::Duration::from_millis(33)),
            Self::Fps60 => ctx.request_repaint_after(std::time::Duration::from_millis(16)),
            Self::Fps120 => ctx.request_repaint_after(std::time::Duration::from_millis(8)),
            Self::Continuous => ctx.request_repaint(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vrr_mode_durations_and_labels() {
        assert_eq!(VrrMode::default(), VrrMode::Fps60);
        assert_eq!(VrrMode::Reactive.target_duration(), None);
        assert_eq!(VrrMode::Fps30.target_duration(), Some(std::time::Duration::from_millis(33)));
        assert_eq!(VrrMode::Fps60.target_duration(), Some(std::time::Duration::from_millis(16)));
        assert_eq!(VrrMode::Fps120.target_duration(), Some(std::time::Duration::from_millis(8)));
        assert_eq!(VrrMode::Continuous.target_duration(), Some(std::time::Duration::ZERO));
        assert_eq!(VrrMode::Fps60.fps_badge(), "VRR: 60Hz");
    }
}

