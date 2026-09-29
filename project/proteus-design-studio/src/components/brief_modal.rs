use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use crate::ProteusApp;

/// Display the Bespoke Brief Ingestion & Auto-Scaffold Modal.
pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    if !app.show_brief_modal {
        return;
    }

    let p = app.palette(ctx);

    egui::Window::new("📋 Import Bespoke Project Brief")
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
                    RichText::new("Bespoke Client Briefs (Zero Presets)")
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
                        app.show_brief_modal = false;
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("Select an authentic client brief from SQLite to automatically scaffold production-ready canvas artboards.")
                    .size(11.0)
                    .color(p.text_dim),
            );
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            if app.cached_briefs.is_empty() {
                app.load_available_briefs();
            }

            let sel_idx = app.selected_brief_idx.min(app.cached_briefs.len().saturating_sub(1));

            ui.columns(2, |cols| {
                // ── LEFT COLUMN: Briefs List ──
                cols[0].vertical(|ui| {
                    ui.label(RichText::new("Available Client Requests:").size(11.5).strong().color(p.text_primary));
                    ui.add_space(6.0);

                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            for (i, brief) in app.cached_briefs.iter().enumerate() {
                                let is_selected = i == sel_idx;
                                let card_bg = if is_selected {
                                    p.surface_secondary
                                } else {
                                    p.surface
                                };
                                let border_color = if is_selected {
                                    p.cta_primary_fill
                                } else {
                                    p.border_subtle
                                };

                                let frame = Frame::new()
                                    .fill(card_bg)
                                    .stroke(Stroke::new(1.0, border_color))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(Margin::same(10));

                                frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let track_color = match brief.track {
                                            proteus_core::brief::BriefTrack::PcdApp => Color32::from_rgb(56, 189, 248),
                                            proteus_core::brief::BriefTrack::PcdWeb => Color32::from_rgb(192, 132, 252),
                                            proteus_core::brief::BriefTrack::DualStack => Color32::from_rgb(251, 191, 36),
                                        };

                                        ui.label(
                                            RichText::new(brief.track.as_str())
                                                .size(9.0)
                                                .strong()
                                                .color(track_color),
                                        );
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(
                                                RichText::new(format!("€{:.0}", brief.proposed_budget_eur))
                                                    .size(10.5)
                                                    .strong()
                                                    .color(p.text_primary),
                                            );
                                        });
                                    });

                                    ui.add_space(2.0);
                                    let btn = ui.add(
                                        egui::Button::new(
                                            RichText::new(&brief.business_name)
                                                .size(12.5)
                                                .strong()
                                                .color(p.text_primary),
                                        )
                                        .fill(Color32::TRANSPARENT)
                                        .stroke(Stroke::NONE),
                                    );
                                    if btn.clicked() {
                                        app.selected_brief_idx = i;
                                    }

                                    ui.label(
                                        RichText::new(&brief.business_nature)
                                            .size(10.5)
                                            .color(p.text_dim),
                                    );
                                });
                                ui.add_space(6.0);
                            }
                        });
                });

                // ── RIGHT COLUMN: Brief Details & Scaffolding Preview ──
                cols[1].vertical(|ui| {
                    if let Some(brief) = app.cached_briefs.get(sel_idx) {
                        ui.label(RichText::new("Project Specifications:").size(11.5).strong().color(p.text_primary));
                        ui.add_space(6.0);

                        Frame::new()
                            .fill(p.surface_secondary)
                            .stroke(Stroke::new(1.0, p.border_subtle))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(12))
                            .show(ui, |ui| {
                                ui.label(RichText::new(&brief.business_name).size(14.0).strong().color(p.text_primary));
                                ui.label(RichText::new(&brief.business_nature).size(11.0).color(p.cta_primary_fill));
                                ui.add_space(6.0);

                                ui.label(RichText::new("Daily Operational Flows:").size(10.5).strong().color(p.text_dim));
                                ui.label(RichText::new(&brief.daily_operations_desc).size(10.5).color(p.text_primary));
                                ui.add_space(8.0);

                                ui.label(RichText::new("Required Artboards / Screens:").size(10.5).strong().color(p.text_dim));
                                ui.horizontal_wrapped(|ui| {
                                    for s in &brief.required_screens {
                                        ui.label(
                                            RichText::new(format!("• {}", s))
                                                .size(10.0)
                                                .color(Color32::from_rgb(148, 163, 184)),
                                        );
                                    }
                                });
                                ui.add_space(6.0);

                                ui.label(RichText::new("Hardware & Peripherals:").size(10.5).strong().color(p.text_dim));
                                ui.horizontal_wrapped(|ui| {
                                    for h in &brief.hardware_peripherals {
                                        ui.label(
                                            RichText::new(format!("🖨 {}", h))
                                                .size(10.0)
                                                .color(Color32::from_rgb(74, 222, 128)),
                                        );
                                    }
                                });
                                ui.add_space(6.0);

                                if let Some(dom) = &brief.domain_name_requested {
                                    ui.label(
                                        RichText::new(format!("🌐 Domain: {} ({})", dom, brief.hosting_preference))
                                            .size(10.0)
                                            .color(p.text_dim),
                                    );
                                }
                            });
                    }
                });
            });

            ui.add_space(14.0);
            ui.separator();
            ui.add_space(10.0);

            // ── ACTION BUTTONS ──
            ui.horizontal(|ui| {
                if ui.add(
                    egui::Button::new(RichText::new("Cancel").size(11.5).color(p.text_dim))
                        .fill(p.surface_secondary)
                        .stroke(Stroke::new(1.0, p.border_subtle))
                        .corner_radius(CornerRadius::same(5))
                        .min_size(Vec2::new(80.0, 32.0)),
                ).clicked() {
                    app.show_brief_modal = false;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let scaffold_btn = ui.add(
                        egui::Button::new(
                            RichText::new("✨ Auto-Scaffold Canvas Artboards")
                                .size(12.0)
                                .strong()
                                .color(p.cta_primary_text),
                        )
                        .fill(p.cta_primary_fill)
                        .corner_radius(CornerRadius::same(6))
                        .min_size(Vec2::new(260.0, 32.0)),
                    );

                    if scaffold_btn.clicked() {
                        app.apply_brief_to_canvas(sel_idx);
                        app.show_brief_modal = false;
                    }
                });
            });
        });
}
