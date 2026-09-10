use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

// ── Project Document (Arena — flat HashMap) ──
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDocument {
    pub version: u32,
    pub nodes: HashMap<String, Node>,
    pub root_node_ids: Vec<String>,
    pub flow_graph: crate::flow::FlowGraph,
}

impl Default for ProjectDocument {
    fn default() -> Self {
        Self::new()
    }
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
        let mut queue = vec![node_id.to_string()];
        while let Some(curr) = queue.pop() {
            to_remove.push(curr.clone());
            if let Some(node) = self.nodes.get(&curr) {
                queue.extend(node.children_ids.clone());
            }
        }

        for id in &to_remove {
            self.nodes.remove(id);
        }

        if let Some(pid) = parent_id {
            if let Some(parent) = self.nodes.get_mut(&pid) {
                parent.children_ids.retain(|id| id != node_id);
            }
        } else {
            self.root_node_ids.retain(|id| id != node_id);
        }

        Ok(())
    }

    /// Reparent a node to a new parent (or to root if None).
    /// Detects cycles: new_parent cannot be the node itself or any of its descendants.
    pub fn reparent_node(&mut self, node_id: &str, new_parent_id: Option<String>) -> Result<(), String> {
        if let Some(ref npid) = new_parent_id {
            if npid == node_id {
                return Err("Cannot reparent a node to itself".into());
            }
            if !self.nodes.contains_key(npid) {
                return Err(format!("Target parent '{}' not found", npid));
            }
            let mut curr = Some(npid.clone());
            while let Some(cid) = curr {
                if cid == node_id {
                    return Err("Cannot reparent to a descendant (cycle)".into());
                }
                curr = self.nodes.get(&cid).and_then(|n| n.parent_id.clone());
            }
        }

        let old_parent_id = self.nodes.get(node_id)
            .ok_or_else(|| format!("Node '{}' not found", node_id))?
            .parent_id.clone();

        if let Some(opid) = old_parent_id {
            if let Some(op) = self.nodes.get_mut(&opid) {
                op.children_ids.retain(|id| id != node_id);
            }
        } else {
            self.root_node_ids.retain(|id| id != node_id);
        }

        if let Some(ref npid) = new_parent_id {
            self.nodes.get_mut(npid)
                .ok_or_else(|| "Target parent vanished".to_string())?
                .children_ids.push(node_id.to_string());
        } else {
            self.root_node_ids.push(node_id.to_string());
        }

        if let Some(node) = self.nodes.get_mut(node_id) {
            node.parent_id = new_parent_id;
        }

        Ok(())
    }

    /// Move a node by (dx, dy) in world space
    pub fn move_node(&mut self, node_id: &str, delta: (f32, f32)) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{}' not found", node_id))?;
        if node.locked {
            return Ok(());
        }
        node.position.0 += delta.0;
        node.position.1 += delta.1;
        Ok(())
    }

    const MIN_SIZE: f32 = 10.0;

    /// Resize a node via a specific handle
    pub fn resize_node(&mut self, node_id: &str, handle: ResizeHandle, delta: (f32, f32)) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{}' not found", node_id))?;
        if node.locked {
            return Ok(());
        }

        let (mut new_x, mut new_y) = node.position;
        let w = match node.layout.width { Sizing::Fixed(v) | Sizing::Fill(v) => v, Sizing::Hug => 200. };
        let h = match node.layout.height { Sizing::Fixed(v) | Sizing::Fill(v) => v, Sizing::Hug => 100. };

        let (dx, dy) = delta;
        use ResizeHandle::*;

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
            NodeUpdate::Rename(s) => {
                node.name = s;
            }
            NodeUpdate::Placeholder(p) => {
                if let NodeType::TextInput { ref mut placeholder, .. } = &mut node.node_type {
                    *placeholder = p;
                }
            }
            NodeUpdate::DropdownOptions(opts) => {
                if let NodeType::Dropdown { ref mut options, .. } = &mut node.node_type {
                    *options = opts;
                }
            }
            NodeUpdate::CheckboxLabel(lbl) => {
                if let NodeType::Checkbox { ref mut label, .. } = &mut node.node_type {
                    *label = lbl;
                }
            }
            NodeUpdate::TextContent(s) => {
                match &mut node.node_type {
                    NodeType::Text { ref mut content, .. } => *content = s,
                    NodeType::Button { ref mut label, .. } => *label = s,
                    _ => node.name = s,
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
                    NodeType::Table { ref mut bound_entity, .. } => {
                        *bound_entity = entity;
                    }
                    _ => {}
                }
            }
            NodeUpdate::NodeStyle(ns) => {
                node.style = ns;
            }
            NodeUpdate::Move { x, y } => {
                node.position = (x, y);
            }
            NodeUpdate::Dimensions { width, height } => {
                node.layout.width = Sizing::Fixed(width);
                node.layout.height = Sizing::Fixed(height);
            }
            NodeUpdate::NodeTypeChange(nt) => {
                node.node_type = nt;
            }
            NodeUpdate::ToggleLock => {
                node.locked = !node.locked;
            }
            NodeUpdate::ToggleVisibility => {
                node.visible = !node.visible;
            }
            NodeUpdate::FontSize(sz) => {
                if let NodeType::Text { ref mut font, .. } = &mut node.node_type {
                    font.size = sz;
                }
            }
            NodeUpdate::FontWeight(w) => {
                if let NodeType::Text { ref mut font, .. } = &mut node.node_type {
                    font.weight = w;
                }
            }
        }
        Ok(())
    }

    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut Node> {
        self.nodes.get_mut(node_id)
    }

    /// Query action bindings (target_page, submit_entity) for a button from the flow graph
    pub fn get_button_action(&self, button_id: &str) -> (Option<String>, Option<String>) {
        let trigger_id = self.flow_graph.nodes.values().find_map(|n| {
            if let crate::flow::FlowNodeKind::TriggerClick { ref target_node_id } = n.kind {
                if target_node_id == button_id {
                    Some(n.id.clone())
                } else { None }
            } else { None }
        });

        let Some(tid) = trigger_id else { return (None, None); };

        let mut target_page = None;
        let mut submit_entity = None;

        let mut queue = vec![tid];
        let mut visited = std::collections::HashSet::new();

        while let Some(curr) = queue.pop() {
            if !visited.insert(curr.clone()) { continue; }
            for edge in &self.flow_graph.edges {
                if edge.from_node == curr {
                    if let Some(target_node) = self.flow_graph.nodes.get(&edge.to_node) {
                        match &target_node.kind {
                            crate::flow::FlowNodeKind::NavigateTo { page_id } => {
                                target_page = Some(page_id.clone());
                            }
                            crate::flow::FlowNodeKind::SaveToDatabase { entity } => {
                                submit_entity = Some(entity.clone());
                            }
                            _ => {}
                        }
                        queue.push(edge.to_node.clone());
                    }
                }
            }
        }

        (target_page, submit_entity)
    }

    /// Set or update the action bindings for a button in the flow graph
    pub fn set_button_action(
        &mut self,
        button_id: &str,
        target_page: Option<String>,
        submit_entity: Option<String>,
    ) {
        let trigger_ids: Vec<String> = self.flow_graph.nodes.values()
            .filter_map(|n| {
                if let crate::flow::FlowNodeKind::TriggerClick { ref target_node_id } = n.kind {
                    if target_node_id == button_id {
                        Some(n.id.clone())
                    } else { None }
                } else { None }
            })
            .collect();

        for tid in trigger_ids {
            let target_ids: Vec<String> = self.flow_graph.edges.iter()
                .filter(|e| e.from_node == tid)
                .map(|e| e.to_node.clone())
                .collect();
            self.flow_graph.remove_node(&tid);
            for tgt in target_ids {
                if tgt.starts_with("f-save-") || tgt.starts_with("f-nav-") || tgt.starts_with("f-login-") || tgt.starts_with("f-qa-") {
                    self.flow_graph.remove_node(&tgt);
                }
            }
        }

        let has_save = submit_entity.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
        let has_nav = target_page.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);

        if has_save || has_nav {
            let trigger_id = format!("f-trig-{}", button_id);
            self.flow_graph.nodes.insert(trigger_id.clone(), crate::flow::FlowNode {
                id: trigger_id.clone(),
                kind: crate::flow::FlowNodeKind::TriggerClick { target_node_id: button_id.to_string() },
                position: (400., 200.),
            });

            let mut prev_id = trigger_id;

            if let Some(entity) = submit_entity {
                let entity_trimmed = entity.trim().to_string();
                if !entity_trimmed.is_empty() {
                    let save_id = format!("f-save-{}", button_id);
                    self.flow_graph.nodes.insert(save_id.clone(), crate::flow::FlowNode {
                        id: save_id.clone(),
                        kind: crate::flow::FlowNodeKind::SaveToDatabase { entity: entity_trimmed },
                        position: (600., 200.),
                    });
                    self.flow_graph.edges.push(crate::flow::FlowEdge {
                        from_node: prev_id,
                        to_node: save_id.clone(),
                    });
                    prev_id = save_id;
                }
            }

            if let Some(page_id) = target_page {
                let page_trimmed = page_id.trim().to_string();
                if !page_trimmed.is_empty() {
                    let nav_id = format!("f-nav-{}", button_id);
                    self.flow_graph.nodes.insert(nav_id.clone(), crate::flow::FlowNode {
                        id: nav_id.clone(),
                        kind: crate::flow::FlowNodeKind::NavigateTo { page_id: page_trimmed },
                        position: (800., 200.),
                    });
                    self.flow_graph.edges.push(crate::flow::FlowEdge {
                        from_node: prev_id,
                        to_node: nav_id,
                    });
                }
            }
        }
    }

    /// Z-sorted render order for root-level traversals
    pub fn render_order(&self) -> Vec<&str> {
        let mut order: Vec<&str> = self.root_node_ids.iter().map(|s| s.as_str()).collect();
        order.sort_by_key(|id| self.nodes.get(*id).map(|n| n.z).unwrap_or(0));
        order
    }
}
