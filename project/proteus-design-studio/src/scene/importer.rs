//! Universal UI layout / schema importer for external design tools (Figma, Penpot, generic JSON).
//!
//! Parses external nested visual hierarchies, geometries, typography, and styling,
//! converting them cleanly into native Proteus `ProjectDocument` scene graphs.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use super::document::ProjectDocument;
use super::types::*;

/// Intermediate representation of an external node from Figma, Penpot, or web design JSON schemas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDesignNode {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "type")]
    pub node_type: Option<String>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    #[serde(default)]
    pub bounds: Option<ExternalBounds>,
    #[serde(default, rename = "absoluteBoundingBox")]
    pub bounding_box: Option<ExternalBounds>,
    pub text: Option<String>,
    pub characters: Option<String>,
    pub placeholder: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<String>>,
    pub label: Option<String>,
    #[serde(rename = "boundEntity")]
    pub bound_entity: Option<String>,
    #[serde(rename = "boundField")]
    pub bound_field: Option<String>,
    pub columns: Option<Vec<String>>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<f32>,
    #[serde(rename = "fontFamily")]
    pub font_family: Option<String>,
    #[serde(rename = "fontWeight")]
    pub font_weight: Option<u16>,
    #[serde(rename = "backgroundColor")]
    pub bg_color: Option<String>,
    #[serde(rename = "borderRadius")]
    pub border_radius: Option<f32>,
    #[serde(default)]
    pub children: Vec<ExternalDesignNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Parses an external design layout JSON string into a native `ProjectDocument`.
pub fn import_external_layout_json(json_str: &str) -> Result<ProjectDocument, String> {
    let mut doc = ProjectDocument::new();

    // Check if the JSON is a wrapper with `document` / `canvas` / `root` or an array / single node
    let val: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON syntax: {}", e))?;

    let root_node: ExternalDesignNode = if let Some(doc_obj) = val.get("document").or_else(|| val.get("root")) {
        serde_json::from_value(doc_obj.clone())
            .map_err(|e| format!("Failed to parse document root: {}", e))?
    } else if let Some(arr) = val.as_array() {
        ExternalDesignNode {
            id: Some("imported_root".into()),
            name: Some("Imported Layout".into()),
            node_type: Some("FRAME".into()),
            x: Some(0.0),
            y: Some(0.0),
            width: Some(1200.0),
            height: Some(900.0),
            bounds: None,
            bounding_box: None,
            text: None,
            characters: None,
            placeholder: None,
            options: None,
            label: None,
            bound_entity: None,
            bound_field: None,
            columns: None,
            font_size: None,
            font_family: None,
            font_weight: None,
            bg_color: None,
            border_radius: None,
            children: serde_json::from_value(Value::Array(arr.clone()))
                .map_err(|e| format!("Failed to parse nodes array: {}", e))?,
        }
    } else {
        serde_json::from_value(val)
            .map_err(|e| format!("Failed to parse design schema: {}", e))?
    };

    convert_external_node_recursive(&mut doc, &root_node, None)?;
    Ok(doc)
}

/// Recursively converts an `ExternalDesignNode` and adds it to the target `ProjectDocument`.
fn convert_external_node_recursive(
    doc: &mut ProjectDocument,
    ext: &ExternalDesignNode,
    parent_id: Option<&str>,
) -> Result<String, String> {
    let node_id = ext.id.clone().unwrap_or_else(|| format!("node_{}", Uuid::new_v4().simple()));
    let name = ext.name.clone().unwrap_or_else(|| format!("Imported Node {}", &node_id[..6.min(node_id.len())]));

    // Determine dimensions and coordinate
    let (mut x, mut y, mut w, mut h) = (0.0, 0.0, 200.0, 100.0);
    if let Some(ref bb) = ext.bounding_box {
        x = bb.x;
        y = bb.y;
        w = bb.width;
        h = bb.height;
    } else if let Some(ref b) = ext.bounds {
        x = b.x;
        y = b.y;
        w = b.width;
        h = b.height;
    } else {
        if let Some(px) = ext.x { x = px; }
        if let Some(py) = ext.y { y = py; }
        if let Some(pw) = ext.width { w = pw; }
        if let Some(ph) = ext.height { h = ph; }
    }

    let type_str = ext.node_type.as_deref().unwrap_or("FRAME").to_uppercase();

    // Map external type to Proteus native NodeType
    let node_type = match type_str.as_str() {
        "TEXT" | "LABEL" | "HEADING" | "PARAGRAPH" => {
            let content = ext.characters.clone()
                .or_else(|| ext.text.clone())
                .unwrap_or_else(|| name.clone());
            let font = FontSpec {
                family: ext.font_family.clone().unwrap_or_else(|| "Inter".into()),
                size: ext.font_size.unwrap_or(14.0),
                weight: ext.font_weight.unwrap_or(400),
                color: Rgba::WHITE,
            };
            NodeType::Text { content, font }
        }
        "BUTTON" | "ACTION" => {
            let label = ext.label.clone()
                .or_else(|| ext.text.clone())
                .or_else(|| ext.characters.clone())
                .unwrap_or_else(|| "Action".into());
            NodeType::Button { label, style: ButtonStyle::Primary }
        }
        "INPUT" | "TEXT_INPUT" | "TEXTFIELD" => {
            NodeType::TextInput {
                placeholder: ext.placeholder.clone().unwrap_or_else(|| "Enter value...".into()),
                field_type: FieldType::Text,
                bound_entity: ext.bound_entity.clone(),
                bound_field: ext.bound_field.clone(),
            }
        }
        "DROPDOWN" | "SELECT" => {
            NodeType::Dropdown {
                options: ext.options.clone().unwrap_or_else(|| vec!["Option 1".into(), "Option 2".into()]),
                multiple: false,
                bound_entity: ext.bound_entity.clone(),
                bound_field: ext.bound_field.clone(),
            }
        }
        "CHECKBOX" | "SWITCH" => {
            NodeType::Checkbox {
                label: ext.label.clone().unwrap_or_else(|| "Confirm".into()),
                bound_entity: ext.bound_entity.clone(),
                bound_field: ext.bound_field.clone(),
            }
        }
        "TABLE" | "DATATABLE" | "GRID" => {
            NodeType::Table {
                bound_entity: ext.bound_entity.clone(),
                columns: ext.columns.clone().unwrap_or_else(|| vec!["ID".into(), "Name".into(), "Status".into()]),
            }
        }
        "DYNAMIC_FORM" | "FORM" => {
            NodeType::DynamicForm {
                bound_entity: ext.bound_entity.clone().unwrap_or_else(|| "contacts".into()),
                title: ext.label.clone().unwrap_or_else(|| "Edit Form".into()),
                submit_label: "Save Record".into(),
            }
        }
        "RECTANGLE" | "SHAPE" | "VECTOR" => {
            NodeType::Shape { kind: ShapeKind::Rectangle }
        }
        "ELLIPSE" | "CIRCLE" => {
            NodeType::Shape { kind: ShapeKind::Ellipse }
        }
        "IMAGE" | "PHOTO" => {
            NodeType::Image { url: String::new(), fit: ImageFit::Cover }
        }
        _ => NodeType::Frame,
    };

    let mut node = Node::new(node_id.clone(), name, node_type);
    node.position = (x, y);
    node.layout.width = Sizing::Fixed(w);
    node.layout.height = Sizing::Fixed(h);

    if let Some(r) = ext.border_radius {
        node.style.border_radius = r;
    }

    if let Some(ref hex) = ext.bg_color {
        if let Some(c) = parse_hex_color(hex) {
            node.style.bg_color = [c[0], c[1], c[2], c[3]];
        }
    }

    doc.add_node(node, parent_id.map(String::from))?;

    // Recurse children
    for child in &ext.children {
        convert_external_node_recursive(doc, child, Some(&node_id))?;
    }

    Ok(node_id)
}

/// Helper to parse color hex strings like "#3498db" or "#3498dbff".
fn parse_hex_color(hex: &str) -> Option<[f32; 4]> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() >= 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f32 / 255.0;
        let a = if clean.len() >= 8 {
            u8::from_str_radix(&clean[6..8], 16).ok()? as f32 / 255.0
        } else {
            1.0
        };
        Some([r, g, b, a])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_figma_penpot_json_tree() {
        let json = r#"{
            "id": "frame_main",
            "name": "Checkout Screen",
            "type": "FRAME",
            "x": 40.0,
            "y": 60.0,
            "width": 380.0,
            "height": 640.0,
            "children": [
                {
                    "id": "heading_1",
                    "name": "Title",
                    "type": "TEXT",
                    "characters": "Payment Details",
                    "fontSize": 20.0,
                    "x": 20.0,
                    "y": 30.0,
                    "width": 200.0,
                    "height": 30.0
                },
                {
                    "id": "input_name",
                    "name": "Customer Name",
                    "type": "INPUT",
                    "placeholder": "Full Legal Name",
                    "x": 20.0,
                    "y": 80.0,
                    "width": 340.0,
                    "height": 44.0
                },
                {
                    "id": "btn_pay",
                    "name": "Pay Button",
                    "type": "BUTTON",
                    "label": "Authorize Payment ($49.00)",
                    "x": 20.0,
                    "y": 140.0,
                    "width": 340.0,
                    "height": 48.0
                }
            ]
        }"#;

        let doc = import_external_layout_json(json).expect("Failed to import layout");
        assert_eq!(doc.nodes.len(), 4);

        let root = doc.nodes.get("frame_main").expect("Root missing");
        assert_eq!(root.name, "Checkout Screen");
        assert_eq!(root.position, (40.0, 60.0));
        assert_eq!(root.children_ids.len(), 3);

        let title = doc.nodes.get("heading_1").expect("Title missing");
        if let NodeType::Text { content, font } = &title.node_type {
            assert_eq!(content, "Payment Details");
            assert_eq!(font.size, 20.0);
        } else {
            panic!("Expected text node");
        }

        let input = doc.nodes.get("input_name").expect("Input missing");
        if let NodeType::TextInput { placeholder, .. } = &input.node_type {
            assert_eq!(placeholder, "Full Legal Name");
        } else {
            panic!("Expected input node");
        }

        let btn = doc.nodes.get("btn_pay").expect("Button missing");
        if let NodeType::Button { label, .. } = &btn.node_type {
            assert_eq!(label, "Authorize Payment ($49.00)");
        } else {
            panic!("Expected button node");
        }
    }

    #[test]
    fn test_import_array_of_components() {
        let json = r#"[
            {
                "id": "table_ledger",
                "name": "Contractor Invoices",
                "type": "TABLE",
                "columns": ["Invoice #", "Contractor", "Amount", "Status"],
                "width": 600.0,
                "height": 300.0
            }
        ]"#;

        let doc = import_external_layout_json(json).expect("Failed to import array");
        assert_eq!(doc.nodes.len(), 2); // Root wrapper + table
        let table = doc.nodes.get("table_ledger").expect("Table missing");
        if let NodeType::Table { columns, .. } = &table.node_type {
            assert_eq!(columns.len(), 4);
            assert_eq!(columns[1], "Contractor");
        } else {
            panic!("Expected table node");
        }
    }
}
