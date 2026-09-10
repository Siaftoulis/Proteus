use crate::scene::{FieldType, FontSpec, ImageFit, NodeType, Rgba};
use eframe::egui::{self, Color32, Vec2};

struct PaletteItem {
    icon: &'static str,
    label: &'static str,
    make: fn() -> NodeType,
}

const ITEMS: &[PaletteItem] = &[
    PaletteItem { icon: "◻", label: "Frame", make: || NodeType::Frame },
    PaletteItem { icon: "Aa", label: "Text", make: || NodeType::Text {
        content: "Text".into(),
        font: FontSpec { family: "Inter".into(), size: 14., weight: 400, color: Rgba { r: 215, g: 215, b: 215, a: 255 } },
    }},
    PaletteItem { icon: "▢", label: "Text Input", make: || NodeType::TextInput {
        placeholder: "Type...".into(), field_type: FieldType::Text, bound_entity: None, bound_field: None,
    }},
    PaletteItem { icon: "▾", label: "Dropdown", make: || NodeType::Dropdown {
        options: vec!["Option 1".into()], multiple: false, bound_entity: None, bound_field: None,
    }},
    PaletteItem { icon: "#", label: "Number", make: || NodeType::NumberField {
        min: None, max: None, step: 1.0, bound_entity: None, bound_field: None,
    }},
    PaletteItem { icon: "☑", label: "Checkbox", make: || NodeType::Checkbox {
        label: "Check me".into(), bound_entity: None, bound_field: None,
    }},
    PaletteItem { icon: "▷", label: "Button", make: || NodeType::Button {
        label: "Button".into(), style: crate::scene::ButtonStyle::Primary,
    }},
    PaletteItem { icon: "◫", label: "Image", make: || NodeType::Image {
        url: String::new(), fit: ImageFit::Cover,
    }},
    PaletteItem { icon: "⊞", label: "Data Table", make: || NodeType::Table {
        bound_entity: Some("contacts".into()),
        columns: vec!["ID".into(), "Name".into(), "Email".into(), "Status".into()],
    }},
];

/// Draw the widget palette. Returns the `NodeType` if a drag was initiated this frame.
pub fn draw_palette(ui: &mut egui::Ui) -> Option<NodeType> {
    let mut result = None;
    for item in ITEMS {
        let resp = ui.add(
            egui::Button::new(
                egui::RichText::new(format!("{}  {}", item.icon, item.label)).size(11.).color(Color32::from_rgb(215, 215, 215)),
            )
            .fill(Color32::from_rgb(50, 50, 50))
            .min_size(Vec2::new(ui.available_width(), 24.)),
        );
        if resp.clicked() {
            result = Some((item.make)());
        }
        resp.on_hover_text("Click to add");
    }
    result
}
