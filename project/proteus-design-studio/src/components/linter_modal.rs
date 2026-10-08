//! Pre-Flight Quality Linter Modal & 1-Click Package Export Gate for Proteus Studio.
//! Enforces Master Problem Audit P19/P21 certification gate before package export or publication.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use crate::ProteusApp;

/// Renders the Pre-Flight Quality Linter Modal dialog.
pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    if !app.show_linter_modal {
        return;
    }

    let p = app.palette(ctx);

    let report_clone = app.active_lint_report.clone();
    let Some(report) = report_clone else {
        return;
    };

    let (status_text, status_color, badge_bg) = if report.is_valid {
        ("✓ CERTIFIED SAFE FOR DEPLOYMENT", Color32::from_rgb(16, 185, 129), Color32::from_rgba_premultiplied(16, 185, 129, 30))
    } else {
        ("⛔ EXPORT BLOCKED (ERRORS DETECTED)", Color32::from_rgb(244, 63, 94), Color32::from_rgba_premultiplied(244, 63, 94, 30))
    };

    let mut close_modal = false;
    let mut trigger_export = false;

    egui::Window::new("🛡️ Template Quality Linter Pre-Flight Gate (P19/P21)")
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(720.0, 520.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .frame(Frame {
            fill: p.panel,
            stroke: Stroke::new(1.0, p.border_strong),
            corner_radius: CornerRadius::same(12),
            inner_margin: Margin::same(18),
            ..Default::default()
        })
        .show(ctx, |ui| {
            // Header Bar
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Automated Pre-Flight Quality Gate")
                        .size(16.0)
                        .color(p.text_primary)
                        .strong(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new(RichText::new("✕").size(14.0).color(p.text_dim))
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::NONE),
                    ).clicked() {
                        close_modal = true;
                    }

                    // Score pill
                    let score_color = if report.quality_score >= 90.0 {
                        Color32::from_rgb(16, 185, 129)
                    } else if report.quality_score >= 70.0 {
                        Color32::from_rgb(245, 158, 11)
                    } else {
                        Color32::from_rgb(244, 63, 94)
                    };
                    ui.label(
                        RichText::new(format!("{:.0}% Quality Score", report.quality_score))
                            .size(13.0)
                            .color(score_color)
                            .strong(),
                    );
                });
            });

            ui.add_space(6.0);

            // Status Banner
            Frame::NONE
                .fill(badge_bg)
                .stroke(Stroke::new(1.0, status_color))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(status_text).size(12.0).color(status_color).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Passed: {}/{} Checks | Errors: {} | Warnings: {}",
                                    report.passed_checks, report.total_checks, report.errors.len(), report.warnings.len()
                                ))
                                .size(11.0)
                                .color(p.text_muted),
                            );
                        });
                    });
                });

            ui.add_space(10.0);

            // 5-Point Quality Vectors Status Grid
            ui.label(RichText::new("Verification Vectors (P19 Standards):").size(12.0).color(p.text_muted).strong());
            ui.add_space(4.0);

            ui.horizontal_wrapped(|ui| {
                render_vector_pill(ui, "1. Schema Primary Keys", !report.errors.iter().any(|e| e.category == proteus_core::package::LintCategory::SchemaConsistency));
                render_vector_pill(ui, "2. Additive Safety (No DROP)", !report.errors.iter().any(|e| e.category == proteus_core::package::LintCategory::AdditiveSafety));
                render_vector_pill(ui, "3. View Reachability", !report.warnings.iter().any(|w| w.category == proteus_core::package::LintCategory::NavigationReachability));
                render_vector_pill(ui, "4. Workflow DAG Cycles", !report.errors.iter().any(|e| e.category == proteus_core::package::LintCategory::DagCycle));
                render_vector_pill(ui, "5. ESC/POS Bounds", !report.warnings.iter().any(|w| w.category == proteus_core::package::LintCategory::HardwareEscPos));
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // Diagnostics and Issues Detail Scroll Area
            ui.label(RichText::new("Diagnostics & Findings:").size(12.0).color(p.text_muted).strong());
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(210.0)
                .show(ui, |ui| {
                    if report.errors.is_empty() && report.warnings.is_empty() {
                        Frame::NONE
                            .fill(p.surface)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(12))
                            .show(ui, |ui| {
                                ui.label(RichText::new("✓ All quality checks passed with zero defects.").size(12.0).color(Color32::from_rgb(16, 185, 129)));
                                ui.label(RichText::new("The package conforms to Proteus Certified Designer (PCD) standards and is verified safe for client deployment.").size(11.0).color(p.text_dim));
                            });
                    } else {
                        // Display Errors
                        for err in &report.errors {
                            render_issue_card(ui, err, p.surface, Color32::from_rgb(244, 63, 94));
                            ui.add_space(4.0);
                        }
                        // Display Warnings
                        for warn in &report.warnings {
                            render_issue_card(ui, warn, p.surface, Color32::from_rgb(245, 158, 11));
                            ui.add_space(4.0);
                        }
                    }
                });

            ui.add_space(12.0);

            // Footer / Action Bar
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Cancel").size(12.0)).clicked() {
                    close_modal = true;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if report.is_valid {
                        let export_btn = egui::Button::new(
                            RichText::new("✓ 1-Click Export Verified .pr Package")
                                .size(12.0)
                                .color(Color32::BLACK)
                                .strong(),
                        )
                        .fill(Color32::from_rgb(16, 185, 129))
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(export_btn).clicked() {
                            trigger_export = true;
                        }
                    } else {
                        let blocked_btn = egui::Button::new(
                            RichText::new("⛔ Export Blocked (Resolve Errors First)")
                                .size(12.0)
                                .color(Color32::from_rgb(180, 180, 180)),
                        )
                        .fill(Color32::from_rgb(50, 50, 50))
                        .corner_radius(CornerRadius::same(6));

                        ui.add_enabled(false, blocked_btn);
                    }
                });
            });
        });

    if close_modal {
        app.show_linter_modal = false;
    }

    if trigger_export {
        let _ = app.confirm_linter_export();
    }
}

fn render_vector_pill(ui: &mut egui::Ui, name: &str, passed: bool) {
    let (icon, color) = if passed {
        ("✓", Color32::from_rgb(16, 185, 129))
    } else {
        ("✕", Color32::from_rgb(244, 63, 94))
    };

    Frame::NONE
        .fill(Color32::from_rgba_premultiplied(40, 40, 40, 150))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::symmetric(8, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(10.5).color(color).strong());
                ui.label(RichText::new(name).size(10.5).color(Color32::from_rgb(220, 220, 220)));
            });
        });
}

fn render_issue_card(ui: &mut egui::Ui, issue: &proteus_core::package::LintIssue, bg: Color32, accent: Color32) {
    Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(1.0, accent))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("[{}]", issue.code)).size(10.5).color(accent).strong());
                ui.label(RichText::new(&issue.target).size(10.5).color(Color32::WHITE).strong());
            });
            ui.label(RichText::new(&issue.message).size(11.0).color(Color32::from_rgb(230, 230, 230)));
            ui.label(RichText::new(format!("Suggested fix: {}", issue.suggested_fix)).size(10.0).color(Color32::from_rgb(160, 160, 160)).italics());
        });
}
