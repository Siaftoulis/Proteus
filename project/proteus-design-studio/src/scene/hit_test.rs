use super::types::*;
use super::document::ProjectDocument;

// ── Editor State (transient, do NOT serialize) ──
pub struct EditorState {
    pub selected_node_ids: Vec<String>,
    pub clipboard: Option<Vec<Node>>,
    pub undo_stack: Vec<ProjectDocument>,
    pub redo_stack: Vec<ProjectDocument>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
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
