//! ESC/POS Thermal Hardware Inspector Tab.
//! Provides paper roll selection (58mm/80mm), real-time column overflow validation,
//! and native Win32 raw print spooler testing.

use eframe::egui::{self, Color32, ProgressBar, RichText, Vec2};
use crate::scene::{Node, NodeType, ProjectDocument};
use crate::theme;

#[derive(Debug, Clone)]
pub enum HardwareAction {
    SpoolTestPrint(String),
    KickCashDrawer(String),
    SwitchPaperWidth(bool),
    LoadThermalPreset(bool),
}

pub fn draw_hardware_tab(
    ui: &mut egui::Ui,
    selected_node: Option<&Node>,
    _doc: &ProjectDocument,
    printer_name: &mut String,
    is_58mm: &mut bool,
) -> Option<HardwareAction> {
    let mut action = None;

    ui.add_space(4.);
    ui.label(RichText::new("🖨 ESC/POS THERMAL HARDWARE").size(11.).strong().color(theme::TEXT));
    ui.label(RichText::new("Real-time monospace canvas & Win32 raw spooler").size(9.).color(theme::TEXT_DIM));
    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    // 1. Paper Width Selector
    ui.label(RichText::new("PAPER SPECIFICATION").size(9.5).strong().color(theme::TEXT_DIM));
    ui.horizontal(|ui| {
        let is_80 = !*is_58mm;
        if ui.add(
            egui::Button::new(RichText::new("80mm (48 Cols)").size(9.5).color(if is_80 { theme::TEXT } else { theme::TEXT_DIM }))
                .fill(if is_80 { theme::ELEVATED } else { theme::PANEL })
                .min_size(Vec2::new(95., 24.))
        ).on_hover_text("Standard 80mm roll: 48 columns in Font A, 576 dots at 203 DPI").clicked() {
            *is_58mm = false;
            action = Some(HardwareAction::SwitchPaperWidth(false));
        }

        if ui.add(
            egui::Button::new(RichText::new("58mm (32 Cols)").size(9.5).color(if *is_58mm { theme::TEXT } else { theme::TEXT_DIM }))
                .fill(if *is_58mm { theme::ELEVATED } else { theme::PANEL })
                .min_size(Vec2::new(95., 24.))
        ).on_hover_text("Compact 58mm roll: 32 columns in Font A, 384 dots at 203 DPI").clicked() {
            *is_58mm = true;
            action = Some(HardwareAction::SwitchPaperWidth(true));
        }
    });

    ui.add_space(10.);
    ui.separator();
    ui.add_space(8.);

    // 2. Monospace Column Validator (if text node selected)
    let max_cols = if *is_58mm { 32 } else { 48 };
    ui.label(RichText::new("CHARACTER COLUMN VALIDATOR").size(9.5).strong().color(theme::TEXT_DIM));

    if let Some(node) = selected_node {
        if let NodeType::Text { content, font: _ } = &node.node_type {
            let char_count = content.chars().count();
            let ratio = (char_count as f32 / max_cols as f32).min(1.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Selected Text: {} chars", char_count)).size(10.).color(theme::TEXT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("Max: {} cols", max_cols)).size(9.).color(theme::TEXT_DIM));
                });
            });

            // Column progress bar
            let bar_color = if char_count > max_cols {
                Color32::from_rgb(239, 68, 68) // Red overflow
            } else if char_count > max_cols - 4 {
                Color32::from_rgb(245, 158, 11) // Amber near limit
            } else {
                Color32::from_rgb(34, 197, 94) // Green OK
            };

            ui.add(ProgressBar::new(ratio).show_percentage().fill(bar_color).desired_height(10.));

            if char_count > max_cols {
                let overflow = char_count - max_cols;
                ui.add_space(2.);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚠️ OVERFLOW:").size(9.5).strong().color(Color32::from_rgb(239, 68, 68)));
                    ui.label(RichText::new(format!("+{} characters will wrap or truncate!", overflow)).size(9.).color(Color32::from_rgb(239, 68, 68)));
                });
            } else {
                ui.add_space(2.);
                ui.label(RichText::new(format!("✓ Fits safely on {}mm receipt line", if *is_58mm { "58" } else { "80" })).size(9.).color(Color32::from_rgb(34, 197, 94)));
            }
        } else {
            ui.label(RichText::new("Selected node is not a Text element.").size(9.5).color(theme::TEXT_DIM));
        }
    } else {
        ui.label(RichText::new("No node selected. Click any text line to validate column count.").size(9.5).color(theme::TEXT_DIM));
    }

    ui.add_space(10.);
    ui.separator();
    ui.add_space(8.);

    // 3. Win32 Raw Spooler Test Controls
    ui.label(RichText::new("WIN32 RAW PRINT SPOOLER").size(9.5).strong().color(theme::TEXT_DIM));
    ui.horizontal(|ui| {
        ui.label(RichText::new("Printer Name:").size(9.5).color(theme::TEXT));
        ui.add(egui::TextEdit::singleline(printer_name).desired_width(120.));
    });
    ui.label(RichText::new("Bypasses USB locks via winspool.drv RAW").size(8.5).color(Color32::GRAY));

    ui.add_space(6.);
    ui.horizontal(|ui| {
        if ui.add(
            egui::Button::new(RichText::new("🖨 Spool Test Print").size(10.).color(Color32::WHITE))
                .fill(theme::ACCENT)
                .min_size(Vec2::new(105., 28.))
        ).on_hover_text("Send native ESC/POS test packet to Windows Print Spooler").clicked() {
            action = Some(HardwareAction::SpoolTestPrint(printer_name.clone()));
        }

        if ui.add(
            egui::Button::new(RichText::new("💵 Kick Drawer").size(10.).color(theme::TEXT))
                .fill(theme::ELEVATED)
                .min_size(Vec2::new(95., 28.))
        ).on_hover_text("Send ESC p 0 25 250 pulse to RJ-11 printer drawer port").clicked() {
            action = Some(HardwareAction::KickCashDrawer(printer_name.clone()));
        }
    });

    ui.add_space(12.);
    ui.separator();
    ui.add_space(8.);

    // 4. Quick Artboard Scaffold
    ui.label(RichText::new("RECEIPT ARTBOARD TEMPLATES").size(9.5).strong().color(theme::TEXT_DIM));
    if ui.add(
        egui::Button::new(RichText::new("✨ Load 80mm Receipt Artboard").size(9.5).color(theme::TEXT))
            .fill(theme::ELEVATED)
            .min_size(Vec2::new(ui.available_width(), 26.))
    ).clicked() {
        action = Some(HardwareAction::LoadThermalPreset(false));
    }

    ui.add_space(4.);
    if ui.add(
        egui::Button::new(RichText::new("✨ Load 58mm Receipt Artboard").size(9.5).color(theme::TEXT))
            .fill(theme::ELEVATED)
            .min_size(Vec2::new(ui.available_width(), 26.))
    ).clicked() {
        action = Some(HardwareAction::LoadThermalPreset(true));
    }

    action
}
