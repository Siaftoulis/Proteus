//! Lunacy-Grade Top Tool Ribbon for Vector, Primitives & Precision Canvas Controls.
//! Provides unified quick-switch tooling (Select, Frame, Rectangle, Ellipse, Line, Text,
//! Vector Pen, Presets, Orientation, Canvas Zoom & Snapping) under a sleek native header.

use eframe::egui::{self, Color32, Key, Margin, Response, RichText, Stroke, Vec2};
use crate::models::{DesignerTool, DevicePreset, LayoutMode};
use crate::theme;
use crate::ProteusApp;

/// Tool ribbon action descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RibbonToolItem {
    Select,
    Hand,
    Frame,
    Rectangle,
    Ellipse,
    Line,
    Text,
    Pen,
    Button,
    Table,
}

impl RibbonToolItem {
    pub fn to_designer_tool(self) -> DesignerTool {
        match self {
            Self::Select => DesignerTool::Select,
            Self::Hand => DesignerTool::Hand,
            Self::Frame => DesignerTool::Frame,
            Self::Rectangle => DesignerTool::Rectangle,
            Self::Ellipse => DesignerTool::Ellipse,
            Self::Line => DesignerTool::Line,
            Self::Text => DesignerTool::Text,
            Self::Pen => DesignerTool::Pen,
            Self::Button => DesignerTool::Button,
            Self::Table => DesignerTool::Table,
        }
    }

    pub fn from_designer_tool(tool: DesignerTool) -> Option<Self> {
        match tool {
            DesignerTool::Select => Some(Self::Select),
            DesignerTool::Hand => Some(Self::Hand),
            DesignerTool::Frame => Some(Self::Frame),
            DesignerTool::Rectangle => Some(Self::Rectangle),
            DesignerTool::Ellipse => Some(Self::Ellipse),
            DesignerTool::Line => Some(Self::Line),
            DesignerTool::Text => Some(Self::Text),
            DesignerTool::Pen => Some(Self::Pen),
            DesignerTool::Button => Some(Self::Button),
            DesignerTool::Table => Some(Self::Table),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Hand => "Hand",
            Self::Frame => "Frame",
            Self::Rectangle => "Rect",
            Self::Ellipse => "Ellipse",
            Self::Line => "Line",
            Self::Text => "Text",
            Self::Pen => "Pen",
            Self::Button => "Button",
            Self::Table => "Table",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Select => "↖",
            Self::Hand => "✋",
            Self::Frame => "◻",
            Self::Rectangle => "▢",
            Self::Ellipse => "◯",
            Self::Line => "╱",
            Self::Text => "T",
            Self::Pen => "✒",
            Self::Button => "🔘",
            Self::Table => "⊞",
        }
    }

    pub fn shortcut(self) -> &'static str {
        match self {
            Self::Select => "V",
            Self::Hand => "H",
            Self::Frame => "F",
            Self::Rectangle => "R",
            Self::Ellipse => "O",
            Self::Line => "L",
            Self::Text => "T",
            Self::Pen => "P",
            Self::Button => "B",
            Self::Table => "G",
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Self::Select => "Select, transform & move layers (V)",
            Self::Hand => "Pan infinite canvas surface (H)",
            Self::Frame => "Artboard / Screen container frame (F)",
            Self::Rectangle => "Vector rectangle primitive (R)",
            Self::Ellipse => "Vector circle or ellipse shape (O)",
            Self::Line => "Vector straight line connector (L)",
            Self::Text => "Typography label and content block (T)",
            Self::Pen => "Vector bezier pen and freeform path (P)",
            Self::Button => "Interactive trigger action button (B)",
            Self::Table => "Real-time SQLite data grid container (G)",
        }
    }
}

/// Primary tool sets displayed across ribbon sections.
pub const TOOL_SELECT_GROUP: &[RibbonToolItem] = &[
    RibbonToolItem::Select,
    RibbonToolItem::Hand,
];

pub const TOOL_SHAPES_GROUP: &[RibbonToolItem] = &[
    RibbonToolItem::Frame,
    RibbonToolItem::Rectangle,
    RibbonToolItem::Ellipse,
    RibbonToolItem::Line,
];

pub const TOOL_CONTENT_GROUP: &[RibbonToolItem] = &[
    RibbonToolItem::Text,
    RibbonToolItem::Pen,
    RibbonToolItem::Button,
    RibbonToolItem::Table,
];

/// Render single segmented tool button.
fn ribbon_tool_button(
    ui: &mut egui::Ui,
    item: RibbonToolItem,
    is_active: bool,
) -> Response {
    let text = format!("{} {}", item.icon(), item.label());
    let color = if is_active {
        theme::SKY_CYAN
    } else {
        theme::FROST_GRAY
    };
    let bg = if is_active {
        Color32::from_rgba_unmultiplied(56, 189, 248, 35)
    } else {
        Color32::TRANSPARENT
    };
    let border = if is_active {
        Stroke::new(1.0, theme::SKY_CYAN)
    } else {
        Stroke::NONE
    };

    let btn = egui::Button::new(RichText::new(text).size(11.0).color(color))
        .fill(bg)
        .stroke(border)
        .corner_radius(egui::CornerRadius::same(4))
        .min_size(Vec2::new(58.0, 24.0));

    ui.add(btn).on_hover_ui(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(item.label()).size(11.0).strong().color(Color32::WHITE));
            ui.label(RichText::new(format!("({})", item.shortcut())).size(10.0).color(theme::SKY_CYAN));
        });
        ui.label(RichText::new(item.tooltip()).size(10.0).color(theme::FROST_GRAY));
    })
}

/// Global shortcut handler for canvas tools.
pub fn handle_shortcuts(app: &mut ProteusApp, ctx: &egui::Context) {
    if ctx.wants_keyboard_input() {
        return;
    }

    ctx.input(|i| {
        if i.key_pressed(Key::V) {
            app.active_tool = DesignerTool::Select;
        } else if i.key_pressed(Key::H) {
            app.active_tool = DesignerTool::Hand;
        } else if i.key_pressed(Key::F) {
            app.active_tool = DesignerTool::Frame;
        } else if i.key_pressed(Key::R) {
            app.active_tool = DesignerTool::Rectangle;
        } else if i.key_pressed(Key::O) {
            app.active_tool = DesignerTool::Ellipse;
        } else if i.key_pressed(Key::L) {
            app.active_tool = DesignerTool::Line;
        } else if i.key_pressed(Key::T) {
            app.active_tool = DesignerTool::Text;
        } else if i.key_pressed(Key::P) {
            app.active_tool = DesignerTool::Pen;
        } else if i.key_pressed(Key::B) {
            app.active_tool = DesignerTool::Button;
        } else if i.key_pressed(Key::G) {
            app.active_tool = DesignerTool::Table;
        }

        if (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::Num0) {
            app.viewport.zoom = 1.0;
            app.viewport.pan = Vec2::ZERO;
        }
    });
}

/// Render the complete top tool ribbon panel.
pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    handle_shortcuts(app, ctx);

    egui::TopBottomPanel::top("lunacy_top_tool_ribbon")
        .min_height(34.0)
        .resizable(false)
        .frame(egui::Frame {
            fill: theme::OBSIDIAN_SURFACE,
            inner_margin: Margin::symmetric(10, 4),
            stroke: Stroke::new(1.0, theme::OBSIDIAN_BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;

                // ── Group 1: Pointer & Pan ──
                for &item in TOOL_SELECT_GROUP {
                    let is_active = app.active_tool == item.to_designer_tool();
                    if ribbon_tool_button(ui, item, is_active).clicked() {
                        app.active_tool = item.to_designer_tool();
                    }
                }

                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);

                // ── Group 2: Frame & Shape Primitives ──
                for &item in TOOL_SHAPES_GROUP {
                    let is_active = app.active_tool == item.to_designer_tool();
                    if ribbon_tool_button(ui, item, is_active).clicked() {
                        app.active_tool = item.to_designer_tool();
                    }
                }

                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);

                // ── Group 3: Text, Pen, Button & Data ──
                for &item in TOOL_CONTENT_GROUP {
                    let is_active = app.active_tool == item.to_designer_tool();
                    if ribbon_tool_button(ui, item, is_active).clicked() {
                        app.active_tool = item.to_designer_tool();
                    }
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // ── Group 4: Device Presets & Form Factors ──
                let cur_preset = app.device_preset.clone();
                let (cw, ch) = cur_preset.size();
                let preset_label = format!("{} {} ({:.0}×{:.0})", cur_preset.icon(), cur_preset.label(), cw, ch);

                egui::ComboBox::from_id_salt("ribbon_device_preset")
                    .selected_text(RichText::new(preset_label).size(11.0).color(theme::ICY_MIST))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        let presets = [
                            DevicePreset::AppleIPhone,
                            DevicePreset::DesktopHD,
                            DevicePreset::Laptop,
                            DevicePreset::TabletPortrait,
                            DevicePreset::Thermal80mm,
                            DevicePreset::Thermal58mm,
                        ];

                        for p in presets {
                            let (w, h) = p.size();
                            let label = format!("{} {} ({:.0}×{:.0})", p.icon(), p.label(), w, h);
                            if ui.selectable_label(app.device_preset == p, label).clicked() {
                                app.device_preset = p.clone();
                                match &p {
                                    DevicePreset::AppleIPhone => app.viewport_profile = crate::viewport::ViewportProfile::apple_iphone(),
                                    DevicePreset::DesktopHD | DevicePreset::Laptop => app.viewport_profile = crate::viewport::ViewportProfile::desktop(),
                                    DevicePreset::TabletPortrait | DevicePreset::TabletLandscape => app.viewport_profile = crate::viewport::ViewportProfile::tablet_touch(),
                                    DevicePreset::Thermal80mm => app.viewport_profile = crate::viewport::ViewportProfile::thermal_receipt_80mm(),
                                    DevicePreset::Thermal58mm => app.viewport_profile = crate::viewport::ViewportProfile::thermal_receipt_58mm(),
                                    _ => {}
                                }
                                app.toast(format!("Preset switched: {}", p.label()));
                            }
                        }
                    });

                // Orientation toggle button (Swap W × H)
                let rotate_btn = ui.add(
                    egui::Button::new(RichText::new("🔄").size(12.0).color(theme::FROST_GRAY))
                        .min_size(Vec2::new(24.0, 22.0))
                ).on_hover_text("Toggle Portrait / Landscape Orientation");
                if rotate_btn.clicked() {
                    let (w, h) = app.device_preset.size();
                    app.device_preset = DevicePreset::Custom(h, w);
                    app.toast(format!("Canvas flipped: {:.0} × {:.0}", h, w));
                }

                // ── Right-aligned Viewport & Canvas Precision Controls ──
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    // Redo (Ctrl+Y)
                    let can_redo = !app.editor_state.redo_stack.is_empty();
                    let redo_btn = ui.add_enabled(
                        can_redo,
                        egui::Button::new(RichText::new("↻").size(13.0).color(if can_redo { theme::ICY_MIST } else { theme::STEEL_SLATE }))
                            .min_size(Vec2::new(24.0, 22.0))
                    ).on_hover_text("Redo last modification (Ctrl+Y)");
                    if redo_btn.clicked() {
                        app.redo();
                    }

                    // Undo (Ctrl+Z)
                    let can_undo = !app.editor_state.undo_stack.is_empty();
                    let undo_btn = ui.add_enabled(
                        can_undo,
                        egui::Button::new(RichText::new("↺").size(13.0).color(if can_undo { theme::ICY_MIST } else { theme::STEEL_SLATE }))
                            .min_size(Vec2::new(24.0, 22.0))
                    ).on_hover_text("Undo previous modification (Ctrl+Z)");
                    if undo_btn.clicked() {
                        app.undo();
                    }

                    ui.separator();

                    // Snap to grid toggle
                    let is_grid = app.layout == LayoutMode::Grid;
                    let grid_btn = ui.add(
                        egui::Button::new(RichText::new(if is_grid { "🧲 Snap: ON" } else { "🧲 Snap: OFF" }).size(10.0).color(if is_grid { theme::SKY_CYAN } else { theme::FROST_GRAY }))
                            .fill(if is_grid { Color32::from_rgba_unmultiplied(56, 189, 248, 25) } else { Color32::TRANSPARENT })
                            .min_size(Vec2::new(64.0, 22.0))
                    ).on_hover_text("Toggle magnetic grid snapping");
                    if grid_btn.clicked() {
                        app.layout = if is_grid { LayoutMode::Free } else { LayoutMode::Grid };
                    }

                    ui.separator();

                    // Zoom Fit
                    let fit_btn = ui.add(
                        egui::Button::new(RichText::new("⊡ Fit").size(10.0).color(theme::FROST_GRAY))
                            .min_size(Vec2::new(36.0, 22.0))
                    ).on_hover_text("Center canvas & fit visible layers");
                    if fit_btn.clicked() {
                        app.viewport.pan = Vec2::ZERO;
                        app.viewport.zoom = 1.0;
                    }

                    // Zoom In
                    let zoom_in_btn = ui.add(
                        egui::Button::new(RichText::new("+").size(13.0).color(theme::FROST_GRAY))
                            .min_size(Vec2::new(22.0, 22.0))
                    ).on_hover_text("Zoom in canvas");
                    if zoom_in_btn.clicked() {
                        app.viewport.zoom = (app.viewport.zoom * 1.15).min(5.0);
                    }

                    // Zoom level percentage reset button
                    let zoom_pct = format!("{:.0}%", app.viewport.zoom * 100.0);
                    let zoom_reset_btn = ui.add(
                        egui::Button::new(RichText::new(zoom_pct).size(10.5).color(theme::ICY_MIST))
                            .fill(Color32::from_rgba_unmultiplied(39, 55, 77, 80))
                            .min_size(Vec2::new(42.0, 22.0))
                    ).on_hover_text("Reset zoom to 100% (Ctrl+0)");
                    if zoom_reset_btn.clicked() {
                        app.viewport.zoom = 1.0;
                        app.viewport.pan = Vec2::ZERO;
                    }

                    // Zoom Out
                    let zoom_out_btn = ui.add(
                        egui::Button::new(RichText::new("–").size(13.0).color(theme::FROST_GRAY))
                            .min_size(Vec2::new(22.0, 22.0))
                    ).on_hover_text("Zoom out canvas");
                    if zoom_out_btn.clicked() {
                        app.viewport.zoom = (app.viewport.zoom / 1.15).max(0.15);
                    }
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ribbon_tool_roundtrip() {
        let tools = [
            RibbonToolItem::Select,
            RibbonToolItem::Hand,
            RibbonToolItem::Frame,
            RibbonToolItem::Rectangle,
            RibbonToolItem::Ellipse,
            RibbonToolItem::Line,
            RibbonToolItem::Text,
            RibbonToolItem::Pen,
            RibbonToolItem::Button,
            RibbonToolItem::Table,
        ];

        for &t in &tools {
            let dt = t.to_designer_tool();
            let back = RibbonToolItem::from_designer_tool(dt);
            assert_eq!(back, Some(t));
            assert!(!t.label().is_empty());
            assert!(!t.icon().is_empty());
            assert!(!t.shortcut().is_empty());
        }
    }

    #[test]
    fn test_device_preset_sizes_and_rotation() {
        let preset = DevicePreset::AppleIPhone;
        let (w, h) = preset.size();
        assert_eq!((w, h), (393.0, 852.0));

        let rotated = DevicePreset::Custom(h, w);
        assert_eq!(rotated.size(), (852.0, 393.0));
    }
}
