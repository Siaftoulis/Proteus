//! Command and mutation generators for inspector node updates.

use crate::scene::{CanvasEvent, NodeType, NodeUpdate, NodeStyle};

pub fn rename_node(node_id: String, new_name: String) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::Rename(new_name),
    }
}

pub fn move_node(node_id: String, x: f32, y: f32) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::Move { x, y },
    }
}

pub fn resize_node(node_id: String, width: f32, height: f32) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::Dimensions { width, height },
    }
}

pub fn change_node_type(node_id: String, new_type: NodeType) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::NodeTypeChange(new_type),
    }
}

pub fn update_node_style(node_id: String, style: NodeStyle) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::NodeStyle(style),
    }
}

pub fn update_corner_radius(node_id: String, radius: f32) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::CornerRadius([radius, radius, radius, radius]),
    }
}

pub fn bind_data(node_id: String, entity: Option<String>, field: Option<String>) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::DataBinding { entity, field },
    }
}

pub fn update_font_size(node_id: String, size: f32) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::FontSize(size),
    }
}

pub fn update_font_weight(node_id: String, weight: u16) -> CanvasEvent {
    CanvasEvent::NodeModified {
        id: node_id,
        update: NodeUpdate::FontWeight(weight),
    }
}

pub fn set_button_action(
    button_id: String,
    target_page: Option<String>,
    submit_entity: Option<String>,
) -> CanvasEvent {
    CanvasEvent::SetButtonAction {
        button_id,
        target_page,
        submit_entity,
    }
}
