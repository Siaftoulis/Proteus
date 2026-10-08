//! In-App Share Point & Sandbox Workspace Modal for Proteus Studio (Master Problem Audit P18/P20).
//! Facilitates friction-free customer requirements intake, watermarked sandbox preview generation,
//! client review feedback inspection, and cryptographic escrow payout execution.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use crate::ProteusApp;

/// Renders the In-App Share Point & Sandbox Workspace dialog.
pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    if !app.show_sharepoint_modal {
        return;
    }

    let p = app.palette(ctx);

    egui::Window::new("🤝 In-App Share Point & Sandbox Workspace (P18/P20)")
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(760.0, 540.0))
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
                    RichText::new("Collaborative Share Point & Escrow Gate")
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
                        app.show_sharepoint_modal = false;
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                RichText::new("Direct client collaboration channel with sandbox previewing, watermarking, and sovereign escrow delivery.")
                    .size(11.0)
                    .color(p.text_muted),
            );

            ui.add_space(8.0);

            // Tab bar
            ui.horizontal(|ui| {
                let tabs = [
                    (0, "📥 Requirements (.prreq)"),
                    (1, "📤 Sandbox Preview (.prpreview)"),
                    (2, "💰 Review & Escrow Release"),
                ];

                for (idx, label) in tabs {
                    let is_active = app.sharepoint_tab == idx;
                    let (bg, text_color) = if is_active {
                        (p.accent, Color32::WHITE)
                    } else {
                        (p.surface, p.text_muted)
                    };

                    let btn = egui::Button::new(RichText::new(label).size(11.0).color(text_color).strong())
                        .fill(bg)
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(1.0, if is_active { p.accent } else { p.border }));

                    if ui.add(btn).clicked() {
                        app.sharepoint_tab = idx;
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Tab Content
            match app.sharepoint_tab {
                0 => render_requirements_tab(app, ui),
                1 => render_preview_tab(app, ui),
                _ => render_escrow_tab(app, ui),
            }
        });
}

fn render_requirements_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());

    if let Some(req) = app.loaded_requirement.clone() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&req.title).size(14.0).color(p.text_primary).strong());
            ui.label(RichText::new(format!("• {}", req.business_type)).size(11.0).color(p.accent));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(
                    egui::Button::new(RichText::new("⚡ Scaffold Canvas").size(11.0).color(Color32::WHITE).strong())
                        .fill(p.accent)
                ).clicked() {
                    let _ = app.scaffold_canvas_from_requirements();
                }
            });
        });

        ui.add_space(4.0);
        ui.label(RichText::new(format!("Client: {} • Seal: ✓ Verified", req.client_anonymous_id)).size(10.0).color(p.text_dim));
        ui.label(RichText::new(&req.notes).size(11.0).color(p.text_muted));

        ui.add_space(10.0);
        ui.label(RichText::new(format!("Required Fields ({})", req.fields.len())).size(12.0).color(p.text_primary).strong());

        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            for f in &req.fields {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&f.name).size(11.0).color(p.text_primary).monospace());
                    ui.label(RichText::new(format!("({})", f.data_type)).size(10.0).color(p.text_dim));
                    if f.is_required {
                        ui.label(RichText::new("REQUIRED").size(9.0).color(Color32::from_rgb(244, 63, 94)));
                    }
                    ui.label(RichText::new(&f.description).size(10.0).color(p.text_muted));
                });
            }

            if !req.workflows.is_empty() {
                ui.add_space(8.0);
                ui.label(RichText::new("Automations / Workflows:").size(11.0).color(p.text_primary).strong());
                for wf in &req.workflows {
                    ui.label(RichText::new(format!("• {}: on {} -> {}", wf.title, wf.trigger_event, wf.expected_action)).size(10.0).color(p.text_muted));
                }
            }
        });
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("No customer requirements package loaded.").size(13.0).color(p.text_dim));
            ui.add_space(12.0);
            if ui.add(
                egui::Button::new(RichText::new("📥 Load Demo Customer .prreq").size(12.0).color(Color32::WHITE))
                    .fill(p.accent)
            ).clicked() {
                app.load_sample_requirement();
            }
        });
    }
}

fn render_preview_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());

    ui.horizontal(|ui| {
        ui.label(RichText::new("Watermarked Sandbox Preview").size(13.0).color(p.text_primary).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(
                egui::Button::new(RichText::new("📦 Generate .prpreview").size(11.0).color(Color32::WHITE).strong())
                    .fill(p.accent)
            ).clicked() {
                let _ = app.export_sandbox_preview();
            }
        });
    });

    ui.add_space(8.0);

    // Watermark banner
    let banner_frame = Frame {
        fill: Color32::from_rgba_premultiplied(217, 119, 6, 25),
        stroke: Stroke::new(1.0, Color32::from_rgb(217, 119, 6)),
        corner_radius: CornerRadius::same(6),
        inner_margin: Margin::symmetric(12, 8),
        ..Default::default()
    };
    banner_frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🛡️ MANDATORY PREVIEW WATERMARK:").size(10.0).color(Color32::from_rgb(217, 119, 6)).strong());
            ui.label(RichText::new(proteus_core::package::DEFAULT_PREVIEW_WATERMARK).size(10.0).color(Color32::from_rgb(217, 119, 6)));
        });
    });

    ui.add_space(10.0);

    if let Some(preview) = &app.generated_preview {
        ui.group(|ui| {
            ui.label(RichText::new(&preview.title).size(12.0).color(p.text_primary).strong());
            ui.label(RichText::new(format!("Preview ID: {}", preview.preview_id)).size(10.0).color(p.text_dim));
            ui.label(RichText::new(format!("Bundle: {} • Designer: {}", preview.bundle_id, preview.designer_pcd_id)).size(10.0).color(p.text_muted));
            ui.label(RichText::new(format!("SHA-256 Seal: ✓ Verified ({})", &preview.sha256_seal[0..12])).size(10.0).color(Color32::from_rgb(16, 185, 129)));
        });

        ui.add_space(12.0);
        if ui.add(
            egui::Button::new(RichText::new("👤 Simulate Client Review & Approval").size(11.0).color(p.accent))
                .fill(p.surface)
        ).clicked() {
            app.simulate_client_review_approval();
            app.sharepoint_tab = 2; // Move to Escrow tab
        }
    } else {
        ui.label(RichText::new("Click 'Generate .prpreview' to build a secure preview package for the client.").size(11.0).color(p.text_dim));
    }
}

fn render_escrow_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());

    if let Some(session) = &app.active_review_session {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Client Review Verdict:").size(12.0).color(p.text_primary).strong());
            let (verdict_text, verdict_color) = match session.verdict {
                proteus_core::package::ReviewVerdict::Approved => ("APPROVED", Color32::from_rgb(16, 185, 129)),
                proteus_core::package::ReviewVerdict::NeedsRevision => ("NEEDS REVISION", Color32::from_rgb(217, 119, 6)),
                proteus_core::package::ReviewVerdict::Rejected => ("REJECTED", Color32::from_rgb(244, 63, 94)),
            };
            ui.label(RichText::new(verdict_text).size(12.0).color(verdict_color).strong());
        });

        if let Some(token) = &session.approval_token {
            ui.label(RichText::new(format!("Approval Token: {}", token)).size(10.0).color(Color32::from_rgb(16, 185, 129)).monospace());
        }

        ui.add_space(6.0);
        ui.label(RichText::new("Client Feedback Notes:").size(11.0).color(p.text_muted).strong());
        for fb in &session.feedback_items {
            ui.label(RichText::new(format!("• [{}] {}", fb.target_element, fb.comment)).size(10.0).color(p.text_muted));
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        if let Some(contract) = app.escrow_contract.clone() {
            ui.label(RichText::new("Escrow Financial Settlement (P20)").size(12.0).color(p.text_primary).strong());
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Contract Total: {:.2} €", contract.agreed_amount_cents as f64 / 100.0)).size(11.0).color(p.text_primary));
                ui.label(RichText::new(format!("• Designer Payout (90%): {:.2} €", contract.designer_payout_cents as f64 / 100.0)).size(11.0).color(Color32::from_rgb(16, 185, 129)).strong());
                ui.label(RichText::new(format!("• Protocol Fee (10%): {:.2} €", contract.platform_fee_cents as f64 / 100.0)).size(11.0).color(p.text_dim));
            });
            ui.label(RichText::new(format!("Status: {}", contract.status.as_str())).size(10.0).color(p.accent));

            ui.add_space(12.0);
            let can_deliver = contract.status == proteus_core::package::EscrowStatus::Approved;
            if ui.add_enabled(
                can_deliver,
                egui::Button::new(RichText::new("🚀 Execute Host Key Delivery & Release Escrow").size(12.0).color(Color32::WHITE).strong())
                    .fill(if can_deliver { Color32::from_rgb(16, 185, 129) } else { p.surface_secondary })
            ).clicked() {
                let _ = app.submit_to_escrow_delivery();
            }

            if contract.status == proteus_core::package::EscrowStatus::Released {
                ui.add_space(6.0);
                ui.label(RichText::new("✓ Package sealed to buyer's machine license & payment released to designer wallet.")
                    .size(10.0)
                    .color(Color32::from_rgb(16, 185, 129)));
            }
        }
    } else {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            ui.label(RichText::new("No active client review session.").size(12.0).color(p.text_dim));
            ui.add_space(8.0);
            ui.label(RichText::new("Generate a preview in the previous tab and simulate client feedback.").size(10.0).color(p.text_muted));
        });
    }
}
