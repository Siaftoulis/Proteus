use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use crate::ProteusApp;

/// Display the Universal UI Layout Importer Modal for Figma, Penpot, and JSON schemas.
pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    if !app.show_import_schema_modal {
        return;
    }

    let p = app.palette(ctx);

    egui::Window::new("📥 Import Universal UI Layout (Figma / Penpot / JSON)")
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(760.0, 520.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(Frame {
            fill: p.panel,
            stroke: Stroke::new(1.0, p.border_strong),
            corner_radius: CornerRadius::same(10),
            inner_margin: Margin::same(16),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Universal Design Importer (Figma / Penpot / Web Layouts)")
                        .size(15.0)
                        .color(p.text_primary)
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("✕").size(14.0).color(p.text_dim))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE),
                    ).clicked() {
                        app.show_import_schema_modal = false;
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("Paste or drop a Figma, Penpot, or generic visual component tree JSON to convert into native Proteus canvas nodes.")
                    .size(11.0)
                    .color(p.text_dim),
            );
            ui.add_space(8.0);

            // Sample Quick Fill Buttons
            ui.horizontal(|ui| {
                ui.label(RichText::new("Sample Templates:").size(10.5).color(p.text_dim));
                if ui.button("📱 Figma Checkout Card").clicked() {
                    app.import_schema_input = r#"{
  "id": "frame_checkout",
  "name": "Checkout Artboard",
  "type": "FRAME",
  "x": 80.0,
  "y": 80.0,
  "width": 380.0,
  "height": 560.0,
  "children": [
    {
      "id": "lbl_title",
      "name": "Title",
      "type": "TEXT",
      "characters": "Payment Details",
      "fontSize": 20.0,
      "x": 24.0,
      "y": 28.0,
      "width": 240.0,
      "height": 30.0
    },
    {
      "id": "inp_name",
      "name": "Cardholder Name",
      "type": "INPUT",
      "placeholder": "Full Name as on Card",
      "x": 24.0,
      "y": 80.0,
      "width": 332.0,
      "height": 44.0
    },
    {
      "id": "inp_card",
      "name": "Card Number",
      "type": "INPUT",
      "placeholder": "4000 1234 5678 9010",
      "x": 24.0,
      "y": 140.0,
      "width": 332.0,
      "height": 44.0
    },
    {
      "id": "btn_submit",
      "name": "Submit Action",
      "type": "BUTTON",
      "label": "Pay Now ($89.00)",
      "x": 24.0,
      "y": 210.0,
      "width": 332.0,
      "height": 48.0
    }
  ]
}"#.into();
                }

                if ui.button("▤ Penpot Data Ledger").clicked() {
                    app.import_schema_input = r#"{
  "id": "frame_ledger",
  "name": "Contractor Invoices",
  "type": "FRAME",
  "x": 60.0,
  "y": 60.0,
  "width": 640.0,
  "height": 420.0,
  "children": [
    {
      "id": "tbl_invoices",
      "name": "Invoices Table",
      "type": "TABLE",
      "columns": ["Invoice #", "Contractor", "AFM", "Amount (€)", "Status"],
      "x": 20.0,
      "y": 30.0,
      "width": 600.0,
      "height": 340.0
    }
  ]
}"#.into();
                }

                if ui.button("Clear").clicked() {
                    app.import_schema_input.clear();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Multiline Editor
            ui.label(RichText::new("Design JSON Schema:").size(11.0).strong().color(p.text_primary));
            ui.add_space(4.0);

            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut app.import_schema_input)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(12)
                        .hint_text("Paste Figma / Penpot / generic JSON here..."),
                );
            });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(8.0);

            // Footer Action Buttons
            ui.horizontal(|ui| {
                let has_content = !app.import_schema_input.trim().is_empty();
                if ui.add_enabled(
                    has_content,
                    egui::Button::new(RichText::new("📥 Ingest & Render to Canvas").size(12.0).strong().color(Color32::WHITE))
                        .fill(p.accent)
                        .corner_radius(CornerRadius::same(6))
                        .min_size(Vec2::new(200.0, 32.0)),
                ).clicked() {
                    match crate::scene::import_external_layout_json(&app.import_schema_input) {
                        Ok(doc) => {
                            let count = doc.nodes.len();
                            app.project_doc = doc;
                            app.editor_state.undo_stack.clear();
                            app.editor_state.redo_stack.clear();
                            app.in_welcome_hub = false;
                            app.show_import_schema_modal = false;
                            app.toast(format!("✓ Imported {} nodes from external schema", count));
                        }
                        Err(e) => {
                            app.toast(format!("❌ Import error: {}", e));
                        }
                    }
                }

                if ui.add(
                    egui::Button::new(RichText::new("Cancel").size(12.0).color(p.text_dim))
                        .min_size(Vec2::new(80.0, 32.0)),
                ).clicked() {
                    app.show_import_schema_modal = false;
                }
            });
        });
}
