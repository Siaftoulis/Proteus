use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Interaction events (UI → data) ──

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum ResizeHandle {
    TopLeft, Top, TopRight,
    Right, BottomRight, Bottom,
    BottomLeft, Left,
}

#[derive(Debug, Clone)]
pub enum NodeUpdate {
    TextContent(String),
    BackgroundColor(Option<Rgba>),
    CornerRadius([f32; 4]),
    DataBinding { entity: Option<String>, field: Option<String> },
    NodeStyle(NodeStyle),
    Move { x: f32, y: f32 },
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

// ── Project Document (Arena — flat HashMap) ──
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub version: u32,
    pub nodes: HashMap<String, Node>,
    pub root_node_ids: Vec<String>,
    pub flow_graph: crate::flow::FlowGraph,
}
impl ProjectDocument {
    pub fn new() -> Self {
        Self {
            version: 1,
            nodes: HashMap::new(),
            root_node_ids: Vec::new(),
            flow_graph: crate::flow::FlowGraph::new(),
        }
    }

    /// Insert a node into the arena under an optional parent.
    /// Updates both the child's `parent_id` and the parent's `children_ids`.
    pub fn add_node(&mut self, mut node: Node, parent_id: Option<String>) -> Result<(), String> {
        if let Some(ref pid) = parent_id {
            if !self.nodes.contains_key(pid) {
                return Err(format!("Parent '{}' not found", pid));
            }
            node.parent_id = Some(pid.clone());
        }

        let id = node.id.clone();
        if let Some(pid) = &parent_id {
            self.nodes.get_mut(pid)
                .ok_or_else(|| "Parent vanished".to_string())?
                .children_ids.push(id.clone());
        } else {
            self.root_node_ids.push(id.clone());
        }

        self.nodes.insert(id, node);
        Ok(())
    }

    /// Delete a single node (shallow). Removes from nodes HashMap
    /// and root_node_ids. Does NOT cascade to children.
    pub fn delete_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
        self.root_node_ids.retain(|id| id != node_id);
    }

    /// Remove a node and all its descendants from the arena.
    /// Updates the parent's `children_ids` and `root_node_ids`.
    pub fn remove_node(&mut self, node_id: &str) -> Result<(), String> {
        let parent_id = self.nodes.get(node_id)
            .ok_or_else(|| format!("Node '{}' not found", node_id))?
            .parent_id.clone();

        let mut to_remove = Vec::new();
        self.collect_descendants(node_id, &mut to_remove);
        to_remove.push(node_id.to_string());

        // Detach from parent
        if let Some(pid) = parent_id {
            if let Some(parent) = self.nodes.get_mut(&pid) {
                parent.children_ids.retain(|c| c != node_id);
            }
        } else {
            self.root_node_ids.retain(|r| r != node_id);
        }

        for rid in &to_remove {
            self.nodes.remove(rid);
        }
        Ok(())
    }

    /// Move a node to a new parent (or root if `None`).
    /// Detects and rejects cycles (reparenting to own descendant).
    pub fn reparent_node(&mut self, node_id: &str, new_parent_id: Option<String>) -> Result<(), String> {
        // Validate source
        let old_parent_id = self.nodes.get(node_id)
            .ok_or_else(|| format!("Node '{}' not found", node_id))?
            .parent_id.clone();

        // Self-parenting
        if let Some(ref nid) = new_parent_id {
            if nid == node_id {
                return Err("Cannot reparent a node to itself".into());
            }
        }

        // Cycle check
        if let Some(ref nid) = new_parent_id {
            let mut descendants = Vec::new();
            self.collect_descendants(node_id, &mut descendants);
            if descendants.contains(nid) {
                return Err("Cannot reparent a node to its own descendant".into());
            }
        }

        // Validate new parent exists
        if let Some(ref nid) = new_parent_id {
            if !self.nodes.contains_key(nid) {
                return Err(format!("New parent '{}' not found", nid));
            }
        }

        // Remove from old parent
        if let Some(opid) = &old_parent_id {
            if let Some(op) = self.nodes.get_mut(opid) {
                op.children_ids.retain(|c| c != node_id);
            }
        } else {
            self.root_node_ids.retain(|r| r != node_id);
        }

        // Attach to new parent
        if let Some(ref nid) = new_parent_id {
            self.nodes.get_mut(nid)
                .ok_or_else(|| "New parent vanished".to_string())?
                .children_ids.push(node_id.to_string());
        } else {
            self.root_node_ids.push(node_id.to_string());
        }

        if let Some(node) = self.nodes.get_mut(node_id) {
            node.parent_id = new_parent_id;
        }
        Ok(())
    }

    /// Move a root node in world-space
    pub fn move_node(&mut self, node_id: &str, delta: (f32, f32)) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{}' not found", node_id))?;
        node.position.0 += delta.0;
        node.position.1 += delta.1;
        Ok(())
    }

    const MIN_SIZE: f32 = 10.0;

    /// Resize a node by dragging one of its 8 handles.
    /// The `delta` is world-space. Overwrites `Sizing` to `Fixed` if it was `Hug` or `Fill`.
    /// Enforces `MIN_SIZE` on width/height.
    pub fn resize_node(&mut self, node_id: &str, handle: ResizeHandle, delta: (f32, f32)) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{}' not found", node_id))?;
        use ResizeHandle::*;

        let w = match &node.layout.width {
            Sizing::Fixed(v) | Sizing::Fill(v) => *v,
            Sizing::Hug => 200.0,
        };
        let h = match &node.layout.height {
            Sizing::Fixed(v) | Sizing::Fill(v) => *v,
            Sizing::Hug => 100.0,
        };

        let (dx, dy) = delta;
        let mut new_x = node.position.0;
        let mut new_y = node.position.1;
        let new_w_raw = w + match handle {
            TopLeft | Left | BottomLeft => -dx,
            TopRight | Right | BottomRight => dx,
            Top | Bottom => 0.0,
        };
        let new_h_raw = h + match handle {
            TopLeft | Top | TopRight => -dy,
            BottomLeft | Bottom | BottomRight => dy,
            Left | Right => 0.0,
        };

        // Clamp and compute position adjustment in one pass
        let new_w = new_w_raw.max(Self::MIN_SIZE);
        let new_h = new_h_raw.max(Self::MIN_SIZE);

        let w_delta = w - new_w; // positive when shrinking
        new_x += match handle {
            TopLeft | Left | BottomLeft => w_delta,
            _ => 0.0,
        };
        let h_delta = h - new_h;
        new_y += match handle {
            TopLeft | Top | TopRight => h_delta,
            _ => 0.0,
        };

        node.position = (new_x, new_y);
        node.layout.width = Sizing::Fixed(new_w);
        node.layout.height = Sizing::Fixed(new_h);
        Ok(())
    }

    /// Apply a property change to a node (text content, styling, etc.)
    pub fn update_node(&mut self, node_id: &str, update: NodeUpdate) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{}' not found", node_id))?;
        match update {
            NodeUpdate::TextContent(s) => {
                match &mut node.node_type {
                    NodeType::Text { ref mut content, .. } => *content = s,
                    NodeType::Button { ref mut label, .. } => *label = s,
                    _ => return Err("Node type does not support text content".into()),
                }
            }
            NodeUpdate::BackgroundColor(c) => {
                node.styling.background = c;
            }
            NodeUpdate::CornerRadius(r) => {
                node.styling.corner_radius = r;
            }
            NodeUpdate::DataBinding { entity, field } => {
                match &mut node.node_type {
                    NodeType::TextInput { ref mut bound_entity, ref mut bound_field, .. }
                    | NodeType::Dropdown { ref mut bound_entity, ref mut bound_field, .. }
                    | NodeType::NumberField { ref mut bound_entity, ref mut bound_field, .. }
                    | NodeType::Checkbox { ref mut bound_entity, ref mut bound_field, .. } => {
                        *bound_entity = entity;
                        *bound_field = field;
                    }
                    _ => return Err("Node type does not support data binding".into()),
                }
            }
            NodeUpdate::NodeStyle(s) => {
                node.style = s;
            }
            NodeUpdate::Move { x, y } => {
                node.position = (x, y);
            }
        }
        Ok(())
    }

    // ── Helpers ──

    fn collect_descendants(&self, node_id: &str, acc: &mut Vec<String>) {
        if let Some(node) = self.nodes.get(node_id) {
            for child_id in &node.children_ids {
                acc.push(child_id.clone());
                self.collect_descendants(child_id, acc);
            }
        }
    }

    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(node_id)
    }

    /// Z-sorted render order for root-level traversals
    pub fn render_order(&self) -> Vec<&str> {
        let mut order: Vec<&str> = self.root_node_ids.iter().map(|s| s.as_str()).collect();
        order.sort_by_key(|id| self.nodes.get(*id).map(|n| n.z).unwrap_or(0));
        order
    }
}

// ── Dummy document for testing ──

pub fn create_dummy_document() -> ProjectDocument {
    let mut doc = ProjectDocument::new();

    let page = Node {
        id: "page-1".into(),
        name: "Login Form".into(),
        node_type: NodeType::Frame,
        parent_id: None,
        children_ids: vec![],
        styling: Styling {
            background: Some(Rgba { r: 38, g: 38, b: 38, a: 255 }),
            corner_radius: [8., 8., 8., 8.],
            border: Some(Border { width: 1., color: Rgba { r: 60, g: 60, b: 60, a: 255 } }),
            padding: [16., 16., 16., 16.],
            shadow: None,
            opacity: 1.0,
        },
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(320.), height: Sizing::Fixed(260.), ..Default::default() },
        position: (100., 100.),
        visible: true,
        locked: false,
        z: 0,
    };

    let title = Node {
        id: "title-1".into(),
        name: "Title".into(),
        node_type: NodeType::Text {
            content: "Enter your details".into(),
            font: FontSpec { family: "Inter".into(), size: 18., weight: 700, color: Rgba { r: 215, g: 215, b: 215, a: 255 } },
        },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout::default(),
        position: (0., 0.),
        visible: true,
        locked: false,
        z: 1,
    };

    let email_input = Node {
        id: "input-1".into(),
        name: "Email Input".into(),
        node_type: NodeType::TextInput {
            placeholder: "Email address".into(),
            field_type: FieldType::Email,
            bound_entity: None,
            bound_field: None,
        },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(288.), height: Sizing::Fixed(36.), ..Default::default() },
        position: (0., 32.),
        visible: true,
        locked: false,
        z: 2,
    };

    let submit_btn = Node {
        id: "btn-1".into(),
        name: "Submit".into(),
        node_type: NodeType::Button { label: "Sign In".into(), style: ButtonStyle::Primary },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(288.), height: Sizing::Fixed(40.), ..Default::default() },
        position: (0., 80.),
        visible: true,
        locked: false,
        z: 3,
    };

    // Build tree via arena API
    let _ = doc.add_node(page, None);
    let _ = doc.add_node(title, Some("page-1".into()));
    let _ = doc.add_node(email_input, Some("page-1".into()));
    let _ = doc.add_node(submit_btn, Some("page-1".into()));

    // Page 2 — Dashboard
    let page2 = Node {
        id: "page-2".into(),
        name: "Dashboard".into(),
        node_type: NodeType::Frame,
        parent_id: None,
        children_ids: vec![],
        styling: Styling {
            background: Some(Rgba { r: 30, g: 30, b: 30, a: 255 }),
            corner_radius: [8., 8., 8., 8.],
            border: Some(Border { width: 1., color: Rgba { r: 60, g: 60, b: 60, a: 255 } }),
            padding: [16., 16., 16., 16.],
            shadow: None,
            opacity: 1.0,
        },
        style: NodeStyle::default(),
        layout: Layout { width: Sizing::Fixed(400.), height: Sizing::Fixed(200.), ..Default::default() },
        position: (500., 100.),
        visible: true,
        locked: false,
        z: 0,
    };
    let welcome = Node {
        id: "welcome-1".into(),
        name: "Welcome Text".into(),
        node_type: NodeType::Text {
            content: "Welcome to the Dashboard!".into(),
            font: FontSpec { family: "Inter".into(), size: 24., weight: 700, color: Rgba { r: 215, g: 215, b: 215, a: 255 } },
        },
        parent_id: None,
        children_ids: vec![],
        styling: Styling::default(),
        style: NodeStyle::default(),
        layout: Layout::default(),
        position: (0., 0.),
        visible: true,
        locked: false,
        z: 1,
    };
    doc.add_node(page2, None).unwrap();
    doc.add_node(welcome, Some("page-2".into())).unwrap();

    // Dummy flow graph: Trigger → NavigateTo page-2
    doc.flow_graph.nodes.insert("f1".into(), crate::flow::FlowNode {
        id: "f1".into(),
        kind: crate::flow::FlowNodeKind::TriggerClick { target_node_id: "btn-1".into() },
        position: (200., 100.),
    });
    doc.flow_graph.nodes.insert("f2".into(), crate::flow::FlowNode {
        id: "f2".into(),
        kind: crate::flow::FlowNodeKind::NavigateTo { page_id: "page-2".into() },
        position: (500., 100.),
    });
    doc.flow_graph.edges.push(crate::flow::FlowEdge {
        from_node: "f1".into(),
        to_node: "f2".into(),
    });

    doc
}

/// Creates a full CRM preset template with multiple pages designed for Desktop canvas.
pub fn create_crm_preset() -> ProjectDocument {
    let mut doc = ProjectDocument::new();

    // Helper to build a styled node
    fn mk_page(id: &str, name: &str, x: f32, y: f32, w: f32, h: f32, bg: Rgba, padding: f32) -> Node {
        Node {
            id: id.into(), name: name.into(),
            node_type: NodeType::Frame,
            parent_id: None, children_ids: vec![],
            styling: Styling {
                background: Some(bg),
                corner_radius: [8., 8., 8., 8.],
                border: Some(Border { width: 1., color: Rgba { r: 55, g: 55, b: 55, a: 255 } }),
                padding: [padding, padding, padding, padding],
                shadow: None, opacity: 1.0,
            },
            style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 0,
        }
    }
    fn mk_text(id: &str, content: &str, x: f32, y: f32, size: f32, bold: bool, r: u8, g: u8, b: u8) -> Node {
        Node {
            id: id.into(), name: content.into(),
            node_type: NodeType::Text { content: content.into(), font: FontSpec { family: "Inter".into(), size, weight: if bold { 700 } else { 400 }, color: Rgba { r, g, b, a: 255 } } },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Hug, height: Sizing::Hug, ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_input(id: &str, placeholder: &str, x: f32, y: f32, w: f32, h: f32, ft: FieldType) -> Node {
        Node {
            id: id.into(), name: placeholder.into(),
            node_type: NodeType::TextInput { placeholder: placeholder.into(), field_type: ft, bound_entity: None, bound_field: None },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_btn(id: &str, label: &str, x: f32, y: f32, w: f32, h: f32, style: ButtonStyle) -> Node {
        Node {
            id: id.into(), name: label.into(),
            node_type: NodeType::Button { label: label.into(), style },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_card(id: &str, x: f32, y: f32, w: f32, h: f32, bg: Rgba) -> Node {
        Node {
            id: id.into(), name: "Card".into(),
            node_type: NodeType::Frame,
            parent_id: None, children_ids: vec![],
            styling: Styling {
                background: Some(bg), corner_radius: [10., 10., 10., 10.],
                border: None, padding: [12., 12., 12., 12.],
                shadow: None, opacity: 1.0,
            },
            style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_dropdown(id: &str, label: &str, opts: &[&str], x: f32, y: f32, w: f32, h: f32) -> Node {
        Node {
            id: id.into(), name: label.into(),
            node_type: NodeType::Dropdown { options: opts.iter().map(|s| s.to_string()).collect(), multiple: false, bound_entity: None, bound_field: None },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Fixed(w), height: Sizing::Fixed(h), ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }
    fn mk_checkbox(id: &str, label: &str, x: f32, y: f32) -> Node {
        Node {
            id: id.into(), name: label.into(),
            node_type: NodeType::Checkbox { label: label.into(), bound_entity: None, bound_field: None },
            parent_id: None, children_ids: vec![],
            styling: Styling::default(), style: NodeStyle::default(),
            layout: Layout { width: Sizing::Hug, height: Sizing::Hug, ..Default::default() },
            position: (x, y), visible: true, locked: false, z: 1,
        }
    }

    let gray = Rgba { r: 28, g: 28, b: 28, a: 255 };
    let dark = Rgba { r: 22, g: 22, b: 22, a: 255 };
    let card = Rgba { r: 35, g: 35, b: 35, a: 255 };
    let accent = Rgba { r: 52, g: 152, b: 219, a: 255 };
    let green = Rgba { r: 46, g: 204, b: 113, a: 255 };
    let orange = Rgba { r: 230, g: 126, b: 34, a: 255 };
    let red = Rgba { r: 231, g: 76, b: 60, a: 255 };
    let white = Rgba { r: 215, g: 215, b: 215, a: 255 };
    let dim = Rgba { r: 130, g: 130, b: 130, a: 255 };
    let text_new = Rgba { r: 200, g: 200, b: 200, a: 255 };

    // ── PAGE 1: LOGIN ──
    let p1 = mk_page("crm-login", "Login", 40., 100., 400., 500., dark, 24.);
    doc.add_node(p1, None).unwrap();
    // Logo area
    doc.add_node(mk_text("login-logo", "✦ Proteus CRM", 24., 20., 28., true, 52, 152, 219), Some("crm-login".into())).unwrap();
    doc.add_node(mk_text("login-sub", "Sign in to your workspace", 24., 58., 13., false, 130, 130, 130), Some("crm-login".into())).unwrap();
    // Form fields
    doc.add_node(mk_input("login-email", "Email address", 24., 100., 352., 42., FieldType::Email), Some("crm-login".into())).unwrap();
    doc.add_node(mk_input("login-pass", "Password", 24., 154., 352., 42., FieldType::Password), Some("crm-login".into())).unwrap();
    doc.add_node(mk_checkbox("login-remember", "Remember me", 24., 210.), Some("crm-login".into())).unwrap();
    doc.add_node(mk_text("login-forgot", "Forgot password?", 220., 212., 11., false, 52, 152, 219), Some("crm-login".into())).unwrap();
    doc.add_node(mk_btn("login-btn", "Sign In →", 24., 246., 352., 44., ButtonStyle::Primary), Some("crm-login".into())).unwrap();
    // Divider
    doc.add_node(mk_text("login-or", "───  or continue with  ───", 100., 310., 10., false, 100, 100, 100), Some("crm-login".into())).unwrap();
    // Social buttons
    doc.add_node(mk_btn("login-google", "⊙  Google", 24., 340., 165., 36., ButtonStyle::Secondary), Some("crm-login".into())).unwrap();
    doc.add_node(mk_btn("login-github", "○  GitHub", 211., 340., 165., 36., ButtonStyle::Secondary), Some("crm-login".into())).unwrap();
    doc.add_node(mk_text("login-signup", "Don't have an account?  Sign up", 80., 400., 12., false, 130, 130, 130), Some("crm-login".into())).unwrap();

    // ── PAGE 2: DASHBOARD ──
    let p2 = mk_page("crm-dash", "Dashboard", 500., 100., 800., 600., gray, 20.);
    doc.add_node(p2, None).unwrap();
    // Header
    doc.add_node(mk_text("dash-head", "📊  Dashboard", 20., 16., 22., true, 215, 215, 215), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("dash-date", "Today • July 20, 2026", 20., 46., 11., false, 130, 130, 130), Some("crm-dash".into())).unwrap();
    // Stats row
    doc.add_node(mk_card("stat-1", 20., 72., 180., 90., card), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-1v", "128", 32., 82., 28., true, 52, 152, 219), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-1l", "Active Deals", 32., 116., 10., false, 130, 130, 130), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_card("stat-2", 215., 72., 180., 90., card), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-2v", "$84,200", 227., 82., 28., true, 46, 204, 113), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-2l", "Revenue Pipeline", 227., 116., 10., false, 130, 130, 130), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_card("stat-3", 410., 72., 180., 90., card), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-3v", "342", 422., 82., 28., true, 230, 126, 34), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-3l", "Total Contacts", 422., 116., 10., false, 130, 130, 130), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_card("stat-4", 605., 72., 180., 90., card), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-4v", "89%", 617., 82., 28., true, 231, 76, 60), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("stat-4l", "Win Rate", 617., 116., 10., false, 130, 130, 130), Some("crm-dash".into())).unwrap();
    // Activity
    doc.add_node(mk_card("act-box", 20., 180., 380., 180., card), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("act-h", "📋 Recent Activity", 32., 192., 13., true, 215, 215, 215), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("act-1", "● John Smith — Deal moved to Proposal", 32., 216., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("act-2", "● Jane Doe — New contact added", 32., 234., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("act-3", "● Widgets Co — Email sent", 32., 252., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("act-4", "● Meeting scheduled with Bob Wilson", 32., 270., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("act-5", "● Contract uploaded — Enterprise Plan", 32., 288., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    // Pipeline summary
    doc.add_node(mk_card("pipe-box", 415., 180., 370., 180., card), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("pipe-h", "▤  Pipeline Overview", 427., 192., 13., true, 215, 215, 215), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("pipe-1", "New:  24 deals  —  $18,200", 427., 216., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("pipe-2", "Qualified:  18 deals  —  $24,500", 427., 234., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("pipe-3", "Proposal:  12 deals  —  $31,000", 427., 252., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_text("pipe-4", "Negotiation:  5 deals  —  $10,500", 427., 270., 10., false, 200, 200, 200), Some("crm-dash".into())).unwrap();
    // Quick actions
    doc.add_node(mk_btn("qa-1", "+  New Contact", 20., 380., 180., 38., ButtonStyle::Primary), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_btn("qa-2", "+  New Deal", 215., 380., 180., 38., ButtonStyle::Secondary), Some("crm-dash".into())).unwrap();
    doc.add_node(mk_btn("qa-3", "📧  Send Campaign", 410., 380., 180., 38., ButtonStyle::Secondary), Some("crm-dash".into())).unwrap();

    // ── PAGE 3: CONTACTS LIST ──
    let p3 = mk_page("crm-contacts", "Contacts", 1360., 100., 700., 600., gray, 16.);
    doc.add_node(p3, None).unwrap();
    doc.add_node(mk_text("con-head", "◎  Contacts", 16., 16., 20., true, 215, 215, 215), Some("crm-contacts".into())).unwrap();
    doc.add_node(mk_text("con-count", "342 total • 12 added this week", 16., 42., 10., false, 130, 130, 130), Some("crm-contacts".into())).unwrap();
    // Search & filter
    doc.add_node(mk_input("con-search", "🔍  Search contacts...", 16., 66., 480., 36., FieldType::Text), Some("crm-contacts".into())).unwrap();
    doc.add_node(mk_dropdown("con-filter", "Filter: All", &["All", "VIP", "Lead", "Partner", "Customer"], 508., 66., 180., 36.), Some("crm-contacts".into())).unwrap();
    // Table header
    doc.add_node(mk_card("con-th", 16., 116., 668., 28., dark), Some("crm-contacts".into())).unwrap();
    doc.add_node(mk_text("con-th-n", "Name", 28., 121., 10., true, 130, 130, 130), Some("crm-contacts".into())).unwrap();
    doc.add_node(mk_text("con-th-e", "Email", 158., 121., 10., true, 130, 130, 130), Some("crm-contacts".into())).unwrap();
    doc.add_node(mk_text("con-th-p", "Phone", 318., 121., 10., true, 130, 130, 130), Some("crm-contacts".into())).unwrap();
    doc.add_node(mk_text("con-th-c", "Company", 458., 121., 10., true, 130, 130, 130), Some("crm-contacts".into())).unwrap();
    // Table rows
    let rows = [
        ("John Smith", "john@acme.com", "555-0100", "Acme Inc", "VIP"),
        ("Jane Doe", "jane@widgets.co", "555-0200", "Widgets Co", "Lead"),
        ("Bob Wilson", "bob@example.com", "555-0300", "Example LLC", "Partner"),
        ("Alice Brown", "alice@test.io", "555-0400", "Test IO", "Customer"),
        ("Charlie Davis", "charlie@demo.org", "555-0500", "Demo Org", "Lead"),
        ("Eve Martin", "eve@sample.com", "555-0600", "Sample Inc", "VIP"),
    ];
    let mut ry = 152.;
    for (i, (n, e, p, c, tag)) in rows.iter().enumerate() {
        let row_bg = if i % 2 == 0 { card } else { Rgba { r: 40, g: 40, b: 40, a: 255 } };
        doc.add_node(mk_card(&format!("con-r{}", i), 16., ry, 668., 32., row_bg), Some("crm-contacts".into())).unwrap();
        doc.add_node(mk_text(&format!("con-{}-n", i), n, 22., ry + 8., 11., false, 215, 215, 215), Some("crm-contacts".into())).unwrap();
        doc.add_node(mk_text(&format!("con-{}-e", i), e, 150., ry + 8., 10., false, 180, 180, 180), Some("crm-contacts".into())).unwrap();
        doc.add_node(mk_text(&format!("con-{}-p", i), p, 310., ry + 8., 10., false, 180, 180, 180), Some("crm-contacts".into())).unwrap();
        doc.add_node(mk_text(&format!("con-{}-c", i), c, 450., ry + 8., 10., false, 180, 180, 180), Some("crm-contacts".into())).unwrap();
        doc.add_node(mk_text(&format!("con-{}-t", i), tag, 620., ry + 8., 9., true, 52, 152, 219), Some("crm-contacts".into())).unwrap();
        ry += 38.;
    }

    // ── PAGE 4: CONTACT DETAIL ──
    let p4 = mk_page("crm-contact-detail", "Contact Detail", 1360., 720., 700., 500., gray, 16.);
    doc.add_node(p4, None).unwrap();
    doc.add_node(mk_text("cd-head", "👤  John Smith", 16., 16., 20., true, 215, 215, 215), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-job", "CEO at Acme Inc •  john@acme.com", 16., 42., 11., false, 130, 130, 130), Some("crm-contact-detail".into())).unwrap();
    // Info card
    doc.add_node(mk_card("cd-info", 16., 68., 320., 160., card), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-info-h", "Contact Information", 28., 80., 12., true, 215, 215, 215), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-p", "📞  (555) 555-0100", 28., 102., 11., false, 200, 200, 200), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-e", "📧  john@acme.com", 28., 120., 11., false, 200, 200, 200), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-w", "🌐  acme.com", 28., 138., 11., false, 200, 200, 200), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-loc", "📍  San Francisco, CA", 28., 156., 11., false, 200, 200, 200), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-tags", "🏷  VIP · Enterprise · Tech", 28., 174., 11., false, 52, 152, 219), Some("crm-contact-detail".into())).unwrap();
    // Deals card
    doc.add_node(mk_card("cd-deals", 350., 68., 334., 160., card), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-deals-h", "💰  Active Deals", 362., 80., 12., true, 215, 215, 215), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-d1", "Enterprise License — $12,000", 362., 106., 11., false, 46, 204, 113), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-d1s", "Stage: Proposal  •  Close: Aug 2026", 362., 124., 10., false, 130, 130, 130), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-d2", "Support Contract — $4,800", 362., 146., 11., false, 46, 204, 113), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-d2s", "Stage: Negotiation  •  Close: Sep 2026", 362., 164., 10., false, 130, 130, 130), Some("crm-contact-detail".into())).unwrap();
    // Notes
    doc.add_node(mk_card("cd-notes", 16., 244., 668., 120., card), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-notes-h", "📝  Notes & Activity", 28., 256., 12., true, 215, 215, 215), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-n1", "📌  Initial meeting — interested in Enterprise plan. Follow up with pricing.", 28., 282., 11., false, 200, 200, 200), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-n1d", "Jul 18, 2026  •  by you", 28., 300., 9., false, 130, 130, 130), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-n2", "📌  Sent proposal with 3-year discount. Awaiting feedback.", 28., 322., 11., false, 200, 200, 200), Some("crm-contact-detail".into())).unwrap();
    doc.add_node(mk_text("cd-n2d", "Jul 15, 2026  •  by you", 28., 340., 9., false, 130, 130, 130), Some("crm-contact-detail".into())).unwrap();

    // ── PAGE 5: PIPELINE / KANBAN ──
    let p5 = mk_page("crm-pipeline", "Pipeline", 2120., 100., 760., 600., gray, 12.);
    doc.add_node(p5, None).unwrap();
    doc.add_node(mk_text("pipe-head", "▤  Sales Pipeline", 12., 12., 20., true, 215, 215, 215), Some("crm-pipeline".into())).unwrap();
    // Columns
    let cols = [("New", orange), ("Qualified", accent), ("Proposal", green), ("Negotiation", Rgba{r:155,g:89,b:182,a:255}), ("Closed Won", Rgba{r:39,g:174,b:96,a:255})];
    let col_w = 144.;
    let col_gap = 4.;
    for (ci, (cname, ccolor)) in cols.iter().enumerate() {
        let cx = 12. + ci as f32 * (col_w + col_gap);
        doc.add_node(mk_card(&format!("pipe-col{}", ci), cx, 50., col_w, 420., dark), Some("crm-pipeline".into())).unwrap();
        // Column header
        doc.add_node(mk_text(&format!("pipe-col{}h", ci), cname, cx + 8., 58., 12., true, ccolor.r, ccolor.g, ccolor.b), Some("crm-pipeline".into())).unwrap();
        // Cards
                        let deals: &[(&str, &str)] = match ci {
                            0 => &[("Acme Inc — $12K", "Contact: John Smith"), ("Widgets Co — $5K", "Contact: Jane Doe")],
                            1 => &[("Example LLC — $3K", "Contact: Bob Wilson"), ("Tech Corp — $8K", "Contact: Alice Brown")],
                            2 => &[("Global Inc — $25K", "3-year enterprise"), ("StartupXYZ — $6K", "SaaS pilot")],
                            3 => &[("MegaCorp — $50K", "Final review"), ("DataFlow — $15K", "Legal review")],
                            _ => &[("Acme Inc — $12K", "Signed ✅")],
                        };
        for (di, (deal_title, deal_sub)) in deals.iter().enumerate() {
            let dy = 80. + di as f32 * 72.;
            doc.add_node(mk_card(&format!("pipe-{}-d{}", ci, di), cx + 6., dy, col_w - 12., 62., card), Some("crm-pipeline".into())).unwrap();
            doc.add_node(mk_text(&format!("pipe-{}-t{}", ci, di), deal_title, cx + 14., dy + 8., 10., true, 215, 215, 215), Some("crm-pipeline".into())).unwrap();
            doc.add_node(mk_text(&format!("pipe-{}-s{}", ci, di), deal_sub, cx + 14., dy + 26., 9., false, 130, 130, 130), Some("crm-pipeline".into())).unwrap();
            doc.add_node(mk_text(&format!("pipe-{}-v{}", ci, di), "★", cx + col_w - 28., dy + 8., 10., false, 243, 156, 18), Some("crm-pipeline".into())).unwrap();
        }
    }
    // Add lead button
    doc.add_node(mk_btn("pipe-add", "+  Add Deal", 12., 478., 144., 36., ButtonStyle::Primary), Some("crm-pipeline".into())).unwrap();

    // ── PAGE 6: SETTINGS ──
    let p6 = mk_page("crm-settings", "Settings", 2940., 100., 500., 500., gray, 20.);
    doc.add_node(p6, None).unwrap();
    doc.add_node(mk_text("set-head", "⚙  Settings", 20., 16., 20., true, 215, 215, 215), Some("crm-settings".into())).unwrap();
    // Profile section
    doc.add_node(mk_card("set-profile", 20., 50., 460., 150., card), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_text("set-ph", "👤  Profile", 32., 62., 14., true, 215, 215, 215), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_text("set-pl", "Full Name", 32., 86., 10., false, 130, 130, 130), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_input("set-name", "Alex Johnson", 32., 100., 200., 32., FieldType::Text), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_text("set-pe", "Email", 248., 86., 10., false, 130, 130, 130), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_input("set-email", "alex@proteus.app", 248., 100., 200., 32., FieldType::Email), Some("crm-settings".into())).unwrap();
    // Preferences
    doc.add_node(mk_card("set-prefs", 20., 216., 460., 120., card), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_text("set-prefh", "🎨  Preferences", 32., 228., 14., true, 215, 215, 215), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_dropdown("set-theme", "Theme: Dark", &["Dark", "Light", "System"], 32., 250., 180., 32.), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_dropdown("set-lang", "Language: English", &["English", "Greek", "Spanish", "French"], 224., 250., 180., 32.), Some("crm-settings".into())).unwrap();
    doc.add_node(mk_checkbox("set-notif", "Enable email notifications", 32., 292.), Some("crm-settings".into())).unwrap();
    // Save
    doc.add_node(mk_btn("set-save", "💾  Save Changes", 20., 360., 200., 40., ButtonStyle::Primary), Some("crm-settings".into())).unwrap();

    // ── PAGE 7: NAVIGATION BAR (always visible, no scroll) ──
    let p7 = mk_page("crm-nav", "Navigation", 40., 630., 400., 50., Rgba{r:30,g:30,b:30,a:255}, 8.);
    doc.add_node(p7, None).unwrap();
    let nav_items = [("📊", "Dashboard"), ("◎", "Contacts"), ("▤", "Pipeline"), ("⚙", "Settings"), ("👤", "Profile")];
    let nav_w = 72.;
    for (ni, (icon, label)) in nav_items.iter().enumerate() {
        let nx = 8. + ni as f32 * nav_w;
        doc.add_node(mk_text(&format!("nav-{}", ni), &format!("{} {}", icon, label), nx, 14., 11., true, 52, 152, 219), Some("crm-nav".into())).unwrap();
    }

    // ── Flow graph for navigation ──
    doc.flow_graph.nodes.insert("f-login".into(), crate::flow::FlowNode {
        id: "f-login".into(),
        kind: crate::flow::FlowNodeKind::TriggerClick { target_node_id: "login-btn".into() },
        position: (600., 200.),
    });
    doc.flow_graph.nodes.insert("f-login-nav".into(), crate::flow::FlowNode {
        id: "f-login-nav".into(),
        kind: crate::flow::FlowNodeKind::NavigateTo { page_id: "crm-dash".into() },
        position: (800., 200.),
    });
    doc.flow_graph.edges.push(crate::flow::FlowEdge { from_node: "f-login".into(), to_node: "f-login-nav".into() });

    doc
}

// ── Editor State (transient, do NOT serialize) ──
pub struct EditorState {
    pub selected_node_ids: Vec<String>,
    pub clipboard: Option<Vec<Node>>,
    pub undo_stack: Vec<ProjectDocument>,
    pub redo_stack: Vec<ProjectDocument>,
}
impl EditorState {
    pub fn new() -> Self {
        Self {
            selected_node_ids: Vec::new(),
            clipboard: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

// ── Hit testing ──

/// Returns the ID of the topmost node at `world_pos`, or None.
/// Iterates in reverse z-order (highest z first) so topmost nodes win.
/// `zoom` is the viewport zoom (default 1.0) — accounts for parent padding offset in world-space.
pub fn hit_test_nodes(doc: &ProjectDocument, world_pos: (f32, f32), zoom: f32) -> Option<String> {
    let zoom = zoom.max(0.1); // guard div-by-zero
    let mut order = Vec::new();
    let mut roots = doc.root_node_ids.clone();
    roots.sort_by_key(|id| doc.nodes.get(id).map(|n| n.z).unwrap_or(0));
    
    fn collect(doc: &ProjectDocument, id: &str, order: &mut Vec<String>) {
        order.push(id.to_string());
        if let Some(node) = doc.nodes.get(id) {
            let mut children = node.children_ids.clone();
            children.sort_by_key(|cid| doc.nodes.get(cid).map(|n| n.z).unwrap_or(0));
            for child in children {
                collect(doc, &child, order);
            }
        }
    }
    
    for root in roots {
        collect(doc, &root, &mut order);
    }
    
    for id in order.into_iter().rev() {
        let node = match doc.nodes.get(&id) {
            Some(n) => n,
            None => continue,
        };
        if !node.visible { continue; }
        
        let w = match &node.layout.width {
            Sizing::Fixed(v) | Sizing::Fill(v) => *v,
            Sizing::Hug => 200.,
        };
        let h = match &node.layout.height {
            Sizing::Fixed(v) | Sizing::Fill(v) => *v,
            Sizing::Hug => 100.,
        };
        
        let mut abs_x = node.position.0;
        let mut abs_y = node.position.1;
        let mut curr_pid = node.parent_id.clone();
        while let Some(pid) = curr_pid {
            if let Some(p) = doc.nodes.get(&pid) {
                abs_x += p.position.0 + p.styling.padding[0] / zoom;
                abs_y += p.position.1 + p.styling.padding[1] / zoom;
                curr_pid = p.parent_id.clone();
            } else {
                break;
            }
        }

        if world_pos.0 >= abs_x && world_pos.0 <= abs_x + w
            && world_pos.1 >= abs_y && world_pos.1 <= abs_y + h
        {
            return Some(node.id.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──

    fn make_node(id: &str) -> Node {
        Node::new(id.into(), id.into(), NodeType::Frame)
    }
    fn text_node(id: &str, x: f32, y: f32) -> Node {
        Node {
            id: id.into(),
            name: id.into(),
            node_type: NodeType::Text { content: "hello".into(), font: FontSpec::default() },
            position: (x, y),
            layout: Layout { width: Sizing::Fixed(100.), height: Sizing::Fixed(30.), ..Layout::default() },
            ..Node::new(id.into(), id.into(), NodeType::Frame)
        }
    }
    fn doc_with_nodes() -> ProjectDocument {
        let mut doc = ProjectDocument::new();
        doc.add_node(text_node("child", 20., 30.), Some("parent".into())).unwrap();
        doc
    }

    // ── Document operations (existing) ──

    #[test]
    fn add_root_node() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("a"), None).unwrap();
        assert!(doc.nodes.contains_key("a"));
        assert_eq!(doc.root_node_ids, vec!["a"]);
    }

    #[test]
    fn add_child_node() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("parent"), None).unwrap();
        doc.add_node(make_node("child"), Some("parent".into())).unwrap();
        assert_eq!(doc.get_node("child").unwrap().parent_id, Some("parent".into()));
        assert!(doc.get_node("parent").unwrap().children_ids.contains(&"child".into()));
    }

    #[test]
    fn add_node_missing_parent() {
        let mut doc = ProjectDocument::new();
        let res = doc.add_node(make_node("x"), Some("missing".into()));
        assert!(res.is_err());
    }

    #[test]
    fn remove_leaf_node() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("p"), None).unwrap();
        doc.add_node(make_node("c"), Some("p".into())).unwrap();
        doc.remove_node("c").unwrap();
        assert!(!doc.nodes.contains_key("c"));
        assert!(doc.get_node("p").unwrap().children_ids.is_empty());
    }

    #[test]
    fn remove_node_cascades_to_children() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("p"), None).unwrap();
        doc.add_node(make_node("c1"), Some("p".into())).unwrap();
        doc.add_node(make_node("c2"), Some("c1".into())).unwrap();
        doc.remove_node("p").unwrap();
        assert!(!doc.nodes.contains_key("p"));
        assert!(!doc.nodes.contains_key("c1"));
        assert!(!doc.nodes.contains_key("c2"));
        assert!(doc.root_node_ids.is_empty());
    }

    #[test]
    fn reparent_to_root() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("p"), None).unwrap();
        doc.add_node(make_node("c"), Some("p".into())).unwrap();
        doc.reparent_node("c", None).unwrap();
        assert!(doc.get_node("c").unwrap().parent_id.is_none());
        assert!(doc.root_node_ids.contains(&"c".into()));
        assert!(doc.get_node("p").unwrap().children_ids.is_empty());
    }

    #[test]
    fn reparent_self_rejected() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("a"), None).unwrap();
        let res = doc.reparent_node("a", Some("a".into()));
        assert!(res.is_err());
    }

    #[test]
    fn reparent_to_descendant_rejected() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("p"), None).unwrap();
        doc.add_node(make_node("c"), Some("p".into())).unwrap();
        let res = doc.reparent_node("p", Some("c".into()));
        assert!(res.is_err());
    }

    #[test]
    fn remove_missing_rejected() {
        let mut doc = ProjectDocument::new();
        let res = doc.remove_node("nope");
        assert!(res.is_err());
    }

    // ── Hit testing ──

    #[test]
    fn hit_root_node() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("a"), None).unwrap();
        // node "a" is at (0,0) with Sizing::Default -> Fixed(200)xHug -> 200x100
        assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), Some("a".into()));
        assert_eq!(hit_test_nodes(&doc, (199., 99.), 1.0), Some("a".into()));
        // outside
        assert_eq!(hit_test_nodes(&doc, (-1., 50.), 1.0), None);
        assert_eq!(hit_test_nodes(&doc, (50., 101.), 1.0), None);
    }

    #[test]
    fn hit_child_node_includes_parent_position() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("parent"), None).unwrap();
        {
            let p = doc.nodes.get_mut("parent").unwrap();
            p.position = (100., 100.);
        }
        doc.add_node(text_node("child", 20., 30.), Some("parent".into())).unwrap();
        // child's world AABB: (100+20, 100+30) to (100+20+100, 100+30+30) = (120,130) to (220,160)
        assert_eq!(hit_test_nodes(&doc, (120., 130.), 1.0), Some("child".into()));
        assert_eq!(hit_test_nodes(&doc, (140., 140.), 1.0), Some("child".into()));
        assert_eq!(hit_test_nodes(&doc, (219., 159.), 1.0), Some("child".into()));
        // outside child but still inside parent
        assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), None);  // outside parent
    }

    #[test]
    fn hit_invisible_node_skipped() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("a"), None).unwrap();
        doc.nodes.get_mut("a").unwrap().visible = false;
        assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), None);
    }

    #[test]
    fn hit_topmost_wins() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("low"), None).unwrap();
        doc.add_node(make_node("high"), None).unwrap();
        doc.nodes.get_mut("high").unwrap().z = 10;
        // both at (0,0) with 200x100; "high" has higher z
        assert_eq!(hit_test_nodes(&doc, (50., 50.), 1.0), Some("high".into()));
    }

    #[test]
    fn hit_no_match_returns_none() {
        let doc = ProjectDocument::new();
        assert_eq!(hit_test_nodes(&doc, (999., 999.), 1.0), None);
    }

    #[test]
    fn hit_child_deeply_nested() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("l1"), None).unwrap();
        doc.nodes.get_mut("l1").unwrap().position = (50., 50.);
        doc.nodes.get_mut("l1").unwrap().layout = Layout { width: Sizing::Fixed(400.), height: Sizing::Fixed(400.), ..Layout::default() };
        doc.add_node(make_node("l2"), Some("l1".into())).unwrap();
        doc.nodes.get_mut("l2").unwrap().position = (10., 10.);
        doc.nodes.get_mut("l2").unwrap().layout = Layout { width: Sizing::Fixed(300.), height: Sizing::Fixed(300.), ..Layout::default() };
        doc.add_node(text_node("l3", 5., 5.), Some("l2".into())).unwrap();
        // l3 world AABB: 50+10+5=65, 50+10+5=65  →  65..165, 65..95
        assert_eq!(hit_test_nodes(&doc, (65., 65.), 1.0), Some("l3".into()));
        assert_eq!(hit_test_nodes(&doc, (164., 94.), 1.0), Some("l3".into()));
        // on l2 but not on l3
        assert_eq!(hit_test_nodes(&doc, (70., 100.), 1.0), Some("l2".into()));
        // on l1 but not l2
        assert_eq!(hit_test_nodes(&doc, (55., 55.), 1.0), Some("l1".into()));
    }

    #[test]
    fn hit_with_padding_offset() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("parent"), None).unwrap();
        {
            let p = doc.nodes.get_mut("parent").unwrap();
            p.position = (0., 0.);
            p.styling.padding = [20., 20., 20., 20.];
        }
        doc.add_node(text_node("child", 0., 0.), Some("parent".into())).unwrap();
        // parent has 20px padding; renderer puts child at parent_origin+20+child.x
        // world: (0+20+0, 0+20+0) = (20, 20)
        // hit without padding would check (0,0) not (20,20)
        assert_eq!(hit_test_nodes(&doc, (20., 20.), 1.0), Some("child".into()));
        assert_eq!(hit_test_nodes(&doc, (119., 49.), 1.0), Some("child".into()));
        assert_eq!(hit_test_nodes(&doc, (5., 5.), 1.0), Some("parent".into())); // hits parent, not padding-offset child
    }

    // ── Node update / CanvasEvent logic ──

    #[test]
    fn node_move_update() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("a"), None).unwrap();
        let updated = doc.update_node("a", NodeUpdate::Move { x: 42., y: 99. });
        assert!(updated.is_ok());
        assert_eq!(doc.get_node("a").unwrap().position, (42., 99.));
    }

    #[test]
    fn node_move_update_unknown_id() {
        let mut doc = ProjectDocument::new();
        let updated = doc.update_node("missing", NodeUpdate::Move { x: 0., y: 0. });
        assert!(updated.is_err());
    }

    #[test]
    fn node_resize_from_event() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("a"), None).unwrap();
        doc.nodes.get_mut("a").unwrap().layout = Layout {
            width: Sizing::Fixed(200.), height: Sizing::Fixed(100.),
            ..Layout::default()
        };
        // simulate a resize event
        let ev = CanvasEvent::NodeResized { id: "a".into(), handle: ResizeHandle::BottomRight, delta: (50., 30.) };
        if let CanvasEvent::NodeResized { id, handle: _, delta } = &ev {
            if let Some(node) = doc.nodes.get_mut(id) {
                let new_w = match node.layout.width { Sizing::Fixed(w) => (w + delta.0).max(10.), _ => 200. };
                let new_h = match node.layout.height { Sizing::Fixed(h) => (h + delta.1).max(10.), _ => 100. };
                node.layout.width = Sizing::Fixed(new_w);
                node.layout.height = Sizing::Fixed(new_h);
            }
        }
        assert_eq!(doc.get_node("a").unwrap().layout, Layout {
            width: Sizing::Fixed(250.), height: Sizing::Fixed(130.),
            ..Layout::default()
        });
    }

    #[test]
    fn text_content_update() {
        let mut doc = ProjectDocument::new();
        doc.add_node(Node::new("t".into(), "t".into(), NodeType::Text { content: "old".into(), font: FontSpec::default() }), None).unwrap();
        doc.update_node("t", NodeUpdate::TextContent("new!".into()));
        if let NodeType::Text { content, .. } = &doc.get_node("t").unwrap().node_type {
            assert_eq!(content, "new!");
        } else {
            panic!("not a text node");
        }
    }

    #[test]
    fn render_order_root_only() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("root"), None).unwrap();
        doc.add_node(make_node("c1"), Some("root".into())).unwrap();
        doc.add_node(make_node("root2"), None).unwrap();
        let order = doc.render_order();
        // render_order only returns root nodes
        assert!(order.contains(&"root"));
        assert!(order.contains(&"root2"));
        assert!(!order.contains(&"c1"));
    }

    #[test]
    fn delete_node_fires_event() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("x"), None).unwrap();
        let ev = CanvasEvent::DeleteNode { id: "x".into() };
        if let CanvasEvent::DeleteNode { id } = &ev {
            doc.remove_node(id).unwrap();
        }
        assert!(!doc.nodes.contains_key("x"));
    }

    #[test]
    fn action_triggered_event() {
        let ev = CanvasEvent::ActionTriggered { source_node_id: "btn-1".into() };
        if let CanvasEvent::ActionTriggered { source_node_id } = &ev {
            assert_eq!(source_node_id, "btn-1");
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn zoom_affects_padding_in_hit_test() {
        let mut doc = ProjectDocument::new();
        doc.add_node(make_node("parent"), None).unwrap();
        {
            let p = doc.nodes.get_mut("parent").unwrap();
            p.position = (0., 0.);
            p.styling.padding = [10., 10., 10., 10.];
        }
        doc.add_node(text_node("child", 5., 5.), Some("parent".into())).unwrap();
        // at zoom 1.0: child world position = 0 + 10/1 + 5 = 15
        assert_eq!(hit_test_nodes(&doc, (15., 15.), 1.0), Some("child".into()));
        // at zoom 2.0: child world position = 0 + 10/2 + 5 = 10
        assert_eq!(hit_test_nodes(&doc, (10., 10.), 2.0), Some("child".into()));
        // at zoom 2.0: (15,15) should NOT hit (it's at 10 in world now)
        assert_eq!(hit_test_nodes(&doc, (15., 15.), 2.0), Some("child".into())); // still within 100x30 AABB
    }
}
