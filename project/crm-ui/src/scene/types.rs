use serde::{Deserialize, Serialize};

// ── Interaction events (UI → data) ──

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum ResizeHandle {
    TopLeft, Top, TopRight,
    Right, BottomRight, Bottom,
    BottomLeft, Left,
}

#[derive(Debug, Clone)]
pub enum NodeUpdate {
    Rename(String),
    Placeholder(String),
    DropdownOptions(Vec<String>),
    CheckboxLabel(String),
    TextContent(String),
    BackgroundColor(Option<Rgba>),
    CornerRadius([f32; 4]),
    DataBinding { entity: Option<String>, field: Option<String> },
    NodeStyle(NodeStyle),
    Move { x: f32, y: f32 },
    Dimensions { width: f32, height: f32 },
    NodeTypeChange(NodeType),
    ToggleLock,
    ToggleVisibility,
    FontSize(f32),
    FontWeight(u16),
}

#[derive(Debug, Clone)]
pub enum CanvasEvent {
    NodeClicked { id: String, shift_held: bool },
    NodeDragged(String, (f32, f32)), // id + world-space delta
    NodeResized { id: String, handle: ResizeHandle, delta: (f32, f32) },
    NodeResizeStarted,
    NodeModified { id: String, update: NodeUpdate },
    ClearSelection,
    ActionTriggered { source_node_id: String },
    DeleteNode { id: String },
    SetButtonAction {
        button_id: String,
        target_page: Option<String>,
        submit_entity: Option<String>,
    },
}

// ── Color (decoupled from egui for serde) ──
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Rgba {
    pub const WHITE: Rgba = Rgba { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Rgba = Rgba { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: Rgba = Rgba { r: 0, g: 0, b: 0, a: 0 };
}

// ── Styling ──
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Styling {
    pub background: Option<Rgba>,
    /// [top-left, top-right, bottom-right, bottom-left]
    pub corner_radius: [f32; 4],
    pub border: Option<Border>,
    /// [left, top, right, bottom]
    pub padding: [f32; 4],
    pub shadow: Option<Shadow>,
    pub opacity: f32,
}
impl Default for Styling {
    fn default() -> Self {
        Self {
            background: None,
            corner_radius: [0.; 4],
            border: None,
            padding: [0.; 4],
            shadow: None,
            opacity: 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Border {
    pub width: f32,
    pub color: Rgba,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub color: Rgba,
}

// ── NodeStyle (visual styling engine) ──
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeStyle {
    pub bg_color: [f32; 4],
    pub text_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}
impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            bg_color: [0.2, 0.2, 0.2, 1.0],
            text_color: [1.0, 1.0, 1.0, 1.0],
            border_radius: 4.0,
            border_width: 1.0,
            border_color: [0.3, 0.3, 0.3, 1.0],
        }
    }
}

// ── Layout (Flexbox-like) ──
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Sizing {
    Fixed(f32),
    Fill(f32),     // stretch, with optional max
    Hug,            // shrink-to-content
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Direction { Row, Column }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Alignment { Start, Center, End, SpaceBetween }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub width: Sizing,
    pub height: Sizing,
    pub direction: Direction,
    pub main_axis: Alignment,
    pub cross_axis: Alignment,
    pub gap: f32,
}
impl Default for Layout {
    fn default() -> Self {
        Self {
            width: Sizing::Fixed(200.),
            height: Sizing::Hug,
            direction: Direction::Column,
            main_axis: Alignment::Start,
            cross_axis: Alignment::Start,
            gap: 0.,
        }
    }
}

// ── Shared enums ──
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontSpec {
    pub family: String,
    pub size: f32,
    pub weight: u16,
    pub color: Rgba,
}
impl Default for FontSpec {
    fn default() -> Self {
        Self {
            family: "Inter".into(),
            size: 14.,
            weight: 400,
            color: Rgba::BLACK,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShapeKind { Rectangle, Ellipse, Line, Triangle }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ImageFit { Cover, Contain, Fill, None }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FieldType { Text, Email, Phone, Number, Date, Password, Url }
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ButtonStyle { Primary, Secondary, Danger, Ghost }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {
    // Containers
    Frame,
    Page,
    Group,
    // Inputs
    TextInput { placeholder: String, field_type: FieldType, bound_entity: Option<String>, bound_field: Option<String> },
    Dropdown { options: Vec<String>, multiple: bool, bound_entity: Option<String>, bound_field: Option<String> },
    NumberField { min: Option<f64>, max: Option<f64>, step: f64, bound_entity: Option<String>, bound_field: Option<String> },
    Checkbox { label: String, bound_entity: Option<String>, bound_field: Option<String> },
    // Static
    Text { content: String, font: FontSpec },
    Image { url: String, fit: ImageFit },
    Shape { kind: ShapeKind },
    // Actions
    Button { label: String, style: ButtonStyle },
    // Data components
    Table { bound_entity: Option<String>, columns: Vec<String> },
}

// ── Node (single scene-graph element) ──
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub parent_id: Option<String>,
    pub children_ids: Vec<String>,
    pub styling: Styling,
    pub style: NodeStyle,
    pub layout: Layout,
    /// Position relative to parent's content area (x, y)
    pub position: (f32, f32),
    pub visible: bool,
    pub locked: bool,
    pub z: i32,
}
impl Node {
    pub fn new(id: String, name: String, node_type: NodeType) -> Self {
        Self {
            id,
            name,
            node_type,
            parent_id: None,
            children_ids: Vec::new(),
            styling: Styling::default(),
            style: NodeStyle::default(),
            layout: Layout::default(),
            position: (0., 0.),
            visible: true,
            locked: false,
            z: 0,
        }
    }
}
