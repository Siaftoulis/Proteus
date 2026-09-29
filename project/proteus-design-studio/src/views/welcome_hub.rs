//! Welcome Hub & Project Launchpad for Proteus Design Studio (Affinity-style).
//! Allows selecting project, occupation/role, account credentials, and design templates.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Stroke, Vec2};
use crate::models::{DevicePreset, Mode, WelcomeTab};
use crate::ProteusApp;

pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    let p = app.palette(ctx);

    // ── TOP HEADER (Affinity Style) ──
    egui::TopBottomPanel::top("welcome_top_header")
        .exact_height(54.0)
        .resizable(false)
        .frame(Frame::new().fill(p.panel).inner_margin(Margin::symmetric(24, 12)).stroke(Stroke::new(1.0, p.border)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Brand Mark: Official King Proteus Emblem
                crate::components::pds::pds_logo_widget(ui, Vec2::new(28.0, 28.0), p.text_primary);
                ui.add_space(8.0);
                
                ui.vertical(|ui| {
                    ui.label(RichText::new("Welcome").size(15.0).strong().color(p.text_primary));
                    ui.label(RichText::new(&app.welcome_user_name).size(11.0).color(p.text_dim));
                });

                ui.add_space(16.0);
                ui.label(RichText::new("▾").size(11.0).color(p.text_dim));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // New Project Button (+)
                    let new_btn = ui.add(
                        egui::Button::new(RichText::new("＋").size(16.0).color(p.cta_primary_text).strong())
                            .fill(p.cta_primary_fill)
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(34.0, 30.0))
                    ).on_hover_text("Create New Blank Project");
                    if new_btn.clicked() {
                        app.project_doc = crate::scene::ProjectDocument::new();
                        app.pname = "Untitled Project".to_string();
                        app.in_welcome_hub = false;
                    }

                    ui.add_space(8.0);

                    // Open Folder Button
                    let open_btn = ui.add(
                        egui::Button::new(RichText::new("📁").size(14.0).color(p.text_primary))
                            .fill(p.surface_secondary)
                            .stroke(Stroke::new(1.0, p.border_subtle))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(34.0, 30.0))
                    ).on_hover_text("Open Existing Project File (.prproj / .pr)");
                    if open_btn.clicked() {
                        app.load_project();
                        app.in_welcome_hub = false;
                    }

                    ui.add_space(8.0);

                    // Home Icon
                    ui.add(
                        egui::Button::new(RichText::new("⌂").size(16.0).color(p.cta_primary_fill))
                            .fill(Color32::TRANSPARENT)
                            .min_size(Vec2::new(30.0, 30.0))
                    ).on_hover_text("Welcome Home");
                });
            });
        });

    // ── SUB-NAVIGATION TABS ──
    egui::TopBottomPanel::top("welcome_tabs_bar")
        .exact_height(42.0)
        .resizable(false)
        .frame(Frame::new().fill(p.bg).inner_margin(Margin::symmetric(24, 6)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tabs = [
                    (WelcomeTab::Home, "Home"),
                    (WelcomeTab::RecentDocuments, "Recent Documents"),
                    (WelcomeTab::Occupations, "Occupations & Roles"),
                    (WelcomeTab::Templates, "My Templates"),
                    (WelcomeTab::Learn, "Learn & Tutorials"),
                ];

                for (tab, label) in tabs {
                    let is_active = app.welcome_tab == tab;
                    let btn = ui.add(
                        egui::Button::new(
                            RichText::new(label)
                                .size(11.5)
                                .color(if is_active { p.cta_primary_text } else { p.text_dim })
                                .strong()
                        )
                        .fill(if is_active { p.cta_primary_fill } else { Color32::TRANSPARENT })
                        .corner_radius(CornerRadius::same(14))
                        .min_size(Vec2::new(80.0, 26.0))
                    );

                    if btn.clicked() {
                        app.welcome_tab = tab;
                    }
                }
            });
        });

    // ── CENTRAL MAIN CANVAS ──
    egui::CentralPanel::default()
        .frame(Frame::new().fill(p.bg).inner_margin(Margin::same(28)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match app.welcome_tab {
                    WelcomeTab::Home => render_home_tab(app, ui),
                    WelcomeTab::RecentDocuments => render_recent_tab(app, ui),
                    WelcomeTab::Occupations => render_occupations_tab(app, ui),
                    WelcomeTab::Templates => render_templates_tab(app, ui),
                    WelcomeTab::Learn => render_learn_tab(app, ui),
                }
            });
        });
}

fn render_home_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());

    // ── BRAND HERO BANNER ──
    ui.horizontal(|ui| {
        crate::components::pds::pds_logo_widget(ui, Vec2::new(48.0, 48.0), p.text_primary);
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("PROTEUS DESIGN STUDIO").size(16.0).strong().color(p.text_primary));
            ui.label(RichText::new("The Sovereign Visual OS & Component Architecture for Modern Business").size(11.0).color(p.text_dim));
        });
    });
    ui.add_space(16.0);

    // ── SECTION 1: START CREATING (PRESETS) ──
    ui.horizontal(|ui| {
        ui.label(RichText::new("Start Creating").size(13.0).strong().color(p.text_primary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new("See more...").size(10.5).color(p.text_dim));
        });
    });
    ui.add_space(10.0);

    let presets = [
        ("Desktop HD", "1920 × 1080", DevicePreset::DesktopHD, Vec2::new(72.0, 48.0)),
        ("POS Counter", "1024 × 768", DevicePreset::Desktop, Vec2::new(64.0, 48.0)),
        ("Laptop Studio", "1440 × 900", DevicePreset::Laptop, Vec2::new(66.0, 44.0)),
        ("Tablet Touch", "1024 × 1366", DevicePreset::TabletPortrait, Vec2::new(42.0, 56.0)),
        ("Mobile Handheld", "390 × 844", DevicePreset::Phone, Vec2::new(28.0, 56.0)),
        ("Thermal Receipt", "80 mm / 58 mm", DevicePreset::Thermal80mm, Vec2::new(32.0, 52.0)),
    ];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        for (idx, (name, dims, dev, icon_size)) in presets.iter().enumerate() {
            let is_sel = app.welcome_selected_preset == idx;
            let card = ui.add(
                egui::Button::new("")
                    .fill(if is_sel { p.surface } else { p.surface_secondary })
                    .stroke(Stroke::new(1.0, if is_sel { p.border_strong } else { p.border_subtle }))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(Vec2::new(110.0, 130.0))
            );

            // Draw miniature canvas inside button rect
            let r = card.rect;
            let pnt = ui.painter();
            let center_icon = Rect::from_center_size(Pos2::new(r.center().x, r.top() + 45.0), *icon_size);
            pnt.rect_filled(center_icon, CornerRadius::same(3), if is_sel { Color32::from_rgb(220, 230, 242) } else { Color32::from_rgb(180, 195, 210) });
            pnt.rect_stroke(center_icon, CornerRadius::same(3), Stroke::new(1.0, Color32::BLACK), egui::StrokeKind::Outside);

            pnt.text(Pos2::new(r.center().x, r.bottom() - 32.0), egui::Align2::CENTER_CENTER, *name, egui::FontId::proportional(11.0), p.text_primary);
            pnt.text(Pos2::new(r.center().x, r.bottom() - 16.0), egui::Align2::CENTER_CENTER, *dims, egui::FontId::proportional(9.5), p.text_dim);

            if card.clicked() {
                app.welcome_selected_preset = idx;
                app.device_preset = dev.clone();
                app.viewport_profile = match dev {
                    DevicePreset::AppleIPhone => crate::viewport::ViewportProfile::apple_iphone(),
                    DevicePreset::AppleMacBook => crate::viewport::ViewportProfile::apple_macos(),
                    DevicePreset::Phone => crate::viewport::ViewportProfile::mobile_touch(),
                    DevicePreset::TabletPortrait => crate::viewport::ViewportProfile::tablet_touch(),
                    DevicePreset::Thermal80mm => crate::viewport::ViewportProfile::thermal_receipt_80mm(),
                    _ => crate::viewport::ViewportProfile::desktop(),
                };
            }
            if card.double_clicked() {
                app.welcome_selected_preset = idx;
                app.device_preset = dev.clone();
                if *dev == DevicePreset::Thermal80mm {
                    app.viewport_profile = crate::viewport::ViewportProfile::thermal_receipt_80mm();
                    app.project_doc = crate::scene::create_thermal_receipt_preset(false, "Proteus Service Lab");
                }
                app.in_welcome_hub = false;
            }
        }
    });

    ui.add_space(28.0);

    // ── SECTION 2: OCCUPATIONS & ROLES ──
    ui.horizontal(|ui| {
        ui.label(RichText::new("Select Workspace & Role Specialization").size(13.0).strong().color(p.text_primary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new("Role-Based Access Control (RBAC)").size(10.5).color(p.text_dim));
        });
    });
    ui.add_space(10.0);

    let roles = [
        ("🎨", "UI/UX Designer (PCD)", "Desktop & POS layouts, design system tokens, 8-pt resize canvas", Mode::Designer),
        ("📊", "Data & Business Analyst (PCDA)", "SQLite schema inference, visual field mapping, rules engine", Mode::Analyst),
        ("🌐", "IT & Network Systems (PCSS)", "Enterprise LAN discovery, UDP beacons, outbox sync", Mode::Networking),
        ("🛠", "Field Deployer (PCDS)", "ESC/POS thermal printers, cash drawer kick, diagnostic logs", Mode::Troubleshoot),
        ("🗄", "Database & Merkle Audit (DA)", "Live SQLite state, audit chain verification, data viewer", Mode::ConnectedData),
    ];

    for (idx, (icon, title, desc, mode)) in roles.iter().enumerate() {
        let is_sel = app.welcome_selected_occupation == idx;
        let card = ui.add(
            egui::Button::new("")
                .fill(if is_sel { p.surface } else { p.surface_secondary })
                .stroke(Stroke::new(1.0, if is_sel { p.border_strong } else { p.border_subtle }))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(ui.available_width(), 48.0))
        );

        let r = card.rect;
        let pnt = ui.painter();
        pnt.text(Pos2::new(r.left() + 20.0, r.center().y), egui::Align2::LEFT_CENTER, *icon, egui::FontId::proportional(18.0), Color32::WHITE);
        pnt.text(Pos2::new(r.left() + 50.0, r.center().y - 8.0), egui::Align2::LEFT_CENTER, *title, egui::FontId::proportional(12.0), p.text_primary);
        pnt.text(Pos2::new(r.left() + 50.0, r.center().y + 10.0), egui::Align2::LEFT_CENTER, *desc, egui::FontId::proportional(10.0), p.text_dim);

        if is_sel {
            pnt.text(Pos2::new(r.right() - 20.0, r.center().y), egui::Align2::RIGHT_CENTER, "✓ ACTIVE", egui::FontId::proportional(10.5), p.cta_primary_fill);
        }

        if card.clicked() {
            app.welcome_selected_occupation = idx;
            app.welcome_user_role = title.to_string();
            app.mode = mode.clone();
        }
    }

    ui.add_space(24.0);

    // ── BOTTOM ACTION BAR ──
    Frame::new()
        .fill(p.surface_secondary)
        .stroke(Stroke::new(1.0, p.border_strong))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("Active Account: {} ({})", app.welcome_user_name, app.welcome_user_role)).size(11.5).strong().color(p.text_primary));
                    ui.label(RichText::new("All project data saved locally in SQLite with zero-privilege offline encryption.").size(10.0).color(p.text_dim));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let enter_btn = ui.add(
                        egui::Button::new(RichText::new("🚀 Open Project & Enter Studio").size(12.5).color(p.cta_primary_text).strong())
                            .fill(p.cta_primary_fill)
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(210.0, 34.0))
                    );
                    if enter_btn.clicked() {
                        app.in_welcome_hub = false;
                    }

                    if ui.add(
                        egui::Button::new(RichText::new("📋 Import Bespoke Brief").size(11.0).color(p.text_primary))
                            .fill(p.surface)
                            .stroke(Stroke::new(1.0, p.cta_primary_fill))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(160.0, 34.0))
                    ).on_hover_text("Load Authentic Client Brief from SQLite & Scaffold Artboards").clicked() {
                        app.load_available_briefs();
                        app.show_brief_modal = true;
                    }

                    ui.add_space(10.0);

                    if ui.add(
                        egui::Button::new(RichText::new("📋 Load Preset Template").size(11.0).color(p.text_primary))
                            .fill(p.surface)
                            .stroke(Stroke::new(1.0, p.border_subtle))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(150.0, 34.0))
                    ).clicked() {
                        if app.welcome_selected_preset == 5 {
                            app.device_preset = crate::models::DevicePreset::Thermal80mm;
                            app.viewport_profile = crate::viewport::ViewportProfile::thermal_receipt_80mm();
                            app.project_doc = crate::scene::create_thermal_receipt_preset(false, "Proteus Service Lab");
                        } else {
                            app.project_doc = crate::scene::create_crm_preset();
                        }
                        app.in_welcome_hub = false;
                    }
                });
            });
        });
}

fn render_recent_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());
    ui.label(RichText::new("Recent Documents & Projects").size(14.0).strong().color(p.text_primary));
    ui.add_space(12.0);

    let projects = [
        ("Acme Enterprise CRM", "Last modified: Just now", "12 Views • 4 Entities • SQLite", "data.db"),
        ("Automotive Service Counter BOS", "Last modified: Yesterday, 16:42", "6 Status Lanes • Thermal Spooler", "store.db"),
        ("Retail Storefront POS Blueprint", "Last modified: 3 days ago", "Cash Drawer • Barcode GS1-128", "retail.db"),
    ];

    for (name, time, details, _file) in projects {
        let card = ui.add(
            egui::Button::new("")
                .fill(p.surface_secondary)
                .stroke(Stroke::new(1.0, p.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(ui.available_width(), 52.0))
        );
        let r = card.rect;
        let pnt = ui.painter();
        pnt.text(Pos2::new(r.left() + 16.0, r.center().y - 8.0), egui::Align2::LEFT_CENTER, name, egui::FontId::proportional(12.5), p.text_primary);
        pnt.text(Pos2::new(r.left() + 16.0, r.center().y + 10.0), egui::Align2::LEFT_CENTER, format!("{}  •  {}", time, details), egui::FontId::proportional(10.0), p.text_dim);
        pnt.text(Pos2::new(r.right() - 20.0, r.center().y), egui::Align2::RIGHT_CENTER, "Open ➔", egui::FontId::proportional(11.0), p.cta_primary_fill);

        if card.clicked() {
            app.pname = name.to_string();
            app.in_welcome_hub = false;
        }
    }
}

fn render_occupations_tab(_app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Ecosystem 5-Role Certification & Occupation Curriculum").size(14.0).strong());
    ui.add_space(8.0);
    ui.label("Proteus is architected with strict separation of concerns across 5 professional roles:");
    ui.add_space(12.0);

    let items = [
        ("UI/UX Designer (PCD)", "Specializes in desktop ergonomics, 8-point resize transformation handles, and Penpot-style component token styling."),
        ("Data & Business Analyst (PCDA)", "Extracts raw CSV/JSON dumps, uses automatic schema inference, visual field mapping, and declarative business rules."),
        ("Systems & IT Network (PCSS)", "Configures autonomous LAN mesh peer discovery, Merkle hash chains, and replication outboxes."),
        ("Field Support Deployer (PCDS)", "Calibrates thermal POS receipt printers, hardware cash drawer kicks, and store barcode scanners."),
        ("Store Director / HQ", "Monitors enterprise multi-store hierarchies, real-time KPI metrics, and complete audit logging."),
    ];

    for (role, desc) in items {
        ui.group(|ui| {
            ui.label(RichText::new(role).size(12.5).strong());
            ui.add_space(4.0);
            ui.label(RichText::new(desc).size(11.0));
        });
        ui.add_space(8.0);
    }
}

fn render_templates_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Ready-to-Deploy Industry Templates").size(14.0).strong());
    ui.add_space(12.0);

    let templates = [
        ("Automotive Repair & Inspection BOS", "6 Status lanes, technician work orders, parts inventory, and Win32 receipt print."),
        ("Medical & Dental Patient Intake", "GDPR-compliant records, appointment scheduler, doctor consultation notes."),
        ("Retail Specialty Store & POS", "Barcode scanner intake, cash drawer kick, receipt printing, daily sales summary."),
        ("B2B SaaS Sales Pipeline & CRM", "Deals Kanban board, contact timeline notes, enterprise pricing calculator."),
    ];

    for (title, desc) in templates {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(12.5).strong());
                    ui.add_space(2.0);
                    ui.label(RichText::new(desc).size(10.5));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Load Template").clicked() {
                        app.project_doc = crate::scene::create_crm_preset();
                        app.pname = title.to_string();
                        app.in_welcome_hub = false;
                    }
                });
            });
        });
        ui.add_space(8.0);
    }
}

fn render_learn_tab(_app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.label(RichText::new("Learn & Video Tutorials").size(14.0).strong());
    ui.add_space(12.0);

    ui.group(|ui| {
        ui.label(RichText::new("Keyboard Shortcuts & Platform Switching (Windows / Mac)").size(12.0).strong());
        ui.add_space(6.0);
        ui.label("• Select Tool: V");
        ui.label("• Draw Rectangle: R");
        ui.label("• Add Text: T");
        ui.label("• CRM Table: G");
        ui.label("• Command Palette: Ctrl+K (Windows) / Cmd+K (Mac)");
        ui.label("• Settings: Ctrl+, (Windows) / Cmd+, (Mac)");
        ui.label("• Pan Canvas: Middle Mouse Button / Space + Drag");
    });
}
