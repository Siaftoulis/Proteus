//! Welcome Hub & Project Launchpad for Proteus Design Studio (Affinity-style).
//! Allows selecting project, occupation/role, account credentials, and design templates.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Stroke, Vec2};
use crate::models::{Mode, WelcomeTab};
use crate::ProteusApp;

pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    let p = app.palette(ctx);

    // ── TOP HEADER (Affinity Style: Unified Single Bar) ──
    egui::TopBottomPanel::top("welcome_top_header")
        .exact_height(54.0)
        .resizable(false)
        .frame(Frame::new().fill(p.panel).inner_margin(Margin::symmetric(24, 10)).stroke(Stroke::new(1.0, p.border)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // 1. Left: Official Emblem + "Welcome" + User Profile with Role Dropdown
                crate::components::pds::pds_logo_widget(ui, Vec2::new(26.0, 26.0), p.text_primary);
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new("Welcome").size(13.5).strong().color(p.text_primary));

                    let role_abbr = match app.welcome_selected_occupation {
                        0 => "UI/UX Designer",
                        1 => "Business Analyst",
                        2 => "IT Systems",
                        3 => "Field Deployer",
                        4 => "Database Audit",
                        _ => "Designer",
                    };
                    let user_text = format!("{} ({}) ▾", if app.welcome_user_name.is_empty() { "Panagiotis Siaftoulis" } else { &app.welcome_user_name }, role_abbr);
                    
                    let role_btn = ui.add(
                        egui::Label::new(RichText::new(user_text).size(11.0).color(p.text_dim))
                            .sense(egui::Sense::click())
                    ).on_hover_text("Click to switch Workspace & Role Specialization (RBAC)");
                    if role_btn.clicked() {
                        app.welcome_selected_occupation = (app.welcome_selected_occupation + 1) % 5;
                        app.welcome_user_role = match app.welcome_selected_occupation {
                            0 => "UI/UX Designer (PCD)".to_string(),
                            1 => "Data & Business Analyst (PCDA)".to_string(),
                            2 => "IT & Network Systems (PCSS)".to_string(),
                            3 => "Field Deployer (PCDS)".to_string(),
                            4 => "Database & Merkle Audit (DA)".to_string(),
                            _ => "UI/UX Designer (PCD)".to_string(),
                        };
                        app.mode = match app.welcome_selected_occupation {
                            0 => Mode::Designer,
                            1 => Mode::Analyst,
                            2 => Mode::Networking,
                            3 => Mode::Troubleshoot,
                            4 => Mode::ConnectedData,
                            _ => Mode::Designer,
                        };
                    }
                });

                // 2. Right Actions: Home, Open Folder, New Project (+)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // New Project Button: Affinity signature vibrant green (+)
                    let new_btn = ui.add(
                        egui::Button::new(RichText::new("＋").size(15.0).color(Color32::BLACK).strong())
                            .fill(Color32::from_rgb(74, 222, 128))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(32.0, 28.0))
                    ).on_hover_text("Create New Blank Project");
                    if new_btn.clicked() {
                        let new_id = uuid::Uuid::new_v4().to_string();
                        app.pid = new_id;
                        app.pname = "Untitled Project".to_string();
                        app.project_doc = crate::scene::ProjectDocument::new();
                        app.contacts.clear();
                        app.deals.clear();
                        app.tasks.clear();
                        app.save_project();
                        app.in_welcome_hub = false;
                    }

                    ui.add_space(6.0);

                    // Open Folder Button
                    let open_btn = ui.add(
                        egui::Button::new(RichText::new("📁").size(13.0).color(p.text_primary))
                            .fill(p.surface_secondary)
                            .stroke(Stroke::new(1.0, p.border_subtle))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(32.0, 28.0))
                    ).on_hover_text("Open Existing Project File (.prproj / .pr)");
                    if open_btn.clicked() {
                        app.load_project();
                        app.in_welcome_hub = false;
                    }

                    ui.add_space(6.0);

                    // Home Icon
                    let is_home = app.welcome_tab == WelcomeTab::Home;
                    let home_btn = ui.add(
                        egui::Button::new(RichText::new("⌂").size(15.0).color(if is_home { p.cta_primary_fill } else { p.text_dim }))
                            .fill(Color32::TRANSPARENT)
                            .min_size(Vec2::new(28.0, 28.0))
                    ).on_hover_text("Welcome Home");
                    if home_btn.clicked() {
                        app.welcome_tab = WelcomeTab::Home;
                    }

                    ui.add_space(20.0);

                    // 3. Center: Segmented Navigation Pills (Affinity Style)
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let tabs = [
                            (WelcomeTab::Home, "Home"),
                            (WelcomeTab::RecentDocuments, "Recent Documents"),
                            (WelcomeTab::Learn, "Learn"),
                            (WelcomeTab::Templates, "My Templates"),
                            (WelcomeTab::Favorites, "Favorites"),
                        ];

                        for (tab, label) in tabs {
                            let is_active = app.welcome_tab == tab;
                            let pill_bg = if is_active {
                                Color32::from_rgb(0, 122, 255) // Affinity royal blue
                            } else {
                                Color32::TRANSPARENT
                            };
                            let text_color = if is_active {
                                Color32::WHITE
                            } else {
                                p.text_dim
                            };

                            let btn = ui.add(
                                egui::Button::new(
                                    RichText::new(label)
                                        .size(11.5)
                                        .color(text_color)
                                        .strong()
                                )
                                .fill(pill_bg)
                                .corner_radius(CornerRadius::same(12))
                                .min_size(Vec2::new(60.0, 24.0))
                            );

                            if btn.clicked() {
                                app.welcome_tab = tab;
                            }
                        }
                    });
                });
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
                    WelcomeTab::Favorites => render_favorites_tab(app, ui),
                }
            });
        });
}

fn render_home_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());

    // ── SECTION 1: RECENT DOCUMENTS (Affinity Card Grid) ──
    ui.horizontal(|ui| {
        ui.label(RichText::new("Recent Documents").size(13.5).strong().color(p.text_primary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let see_more = ui.add(
                egui::Label::new(RichText::new("See more...").size(11.0).color(p.text_dim))
                    .sense(egui::Sense::click())
            );
            if see_more.clicked() {
                app.welcome_tab = WelcomeTab::RecentDocuments;
            }
        });
    });
    ui.add_space(10.0);

    let recent_projects = app.list_recent_projects();

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;
        for (idx, proj) in recent_projects.iter().take(5).enumerate() {
            let label = if proj.name.len() > 18 {
                format!("{}...", &proj.name[..15])
            } else {
                proj.name.clone()
            };
            let kind = idx % 5;
            let card_size = Vec2::new(118.0, 114.0);
            let (rect, resp) = ui.allocate_exact_size(card_size, egui::Sense::click());
            let is_hovered = resp.hovered();

            let pnt = ui.painter();
            // Outer card frame
            let card_bg = if is_hovered { Color32::from_rgb(38, 38, 44) } else { Color32::from_rgb(26, 26, 30) };
            let card_stroke = if is_hovered { Stroke::new(1.0, Color32::from_rgb(70, 70, 82)) } else { Stroke::new(1.0, Color32::from_rgb(42, 42, 48)) };
            pnt.rect_filled(rect, CornerRadius::same(6), card_bg);
            pnt.rect_stroke(rect, CornerRadius::same(6), card_stroke, egui::StrokeKind::Outside);

            // Thumbnail artboard canvas (top ~74px)
            let thumb_rect = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::new(rect.width() - 16.0, 70.0));
            pnt.rect_filled(thumb_rect, CornerRadius::same(4), Color32::from_rgb(16, 17, 20));
            pnt.rect_stroke(thumb_rect, CornerRadius::same(4), Stroke::new(1.0, Color32::from_rgb(32, 34, 40)), egui::StrokeKind::Outside);

            // Draw miniature document preview graphics
            match kind {
                0 => {
                    // Design System preview: mock layout cards & palette dots
                    let c = thumb_rect.center();
                    pnt.circle_filled(Pos2::new(c.x - 20.0, c.y - 12.0), 7.0, Color32::from_rgb(56, 189, 248));
                    pnt.circle_filled(Pos2::new(c.x, c.y - 12.0), 7.0, Color32::from_rgb(192, 132, 252));
                    pnt.circle_filled(Pos2::new(c.x + 20.0, c.y - 12.0), 7.0, Color32::from_rgb(74, 222, 128));
                    pnt.rect_filled(Rect::from_min_size(Pos2::new(c.x - 28.0, c.y + 4.0), Vec2::new(56.0, 16.0)), CornerRadius::same(2), Color32::from_rgb(30, 41, 59));
                }
                1 => {
                    // POS Counter preview: cart list & keypad
                    let c = thumb_rect.center();
                    pnt.rect_filled(Rect::from_min_size(Pos2::new(c.x - 30.0, c.y - 20.0), Vec2::new(36.0, 42.0)), CornerRadius::same(2), Color32::from_rgb(24, 28, 36));
                    pnt.rect_filled(Rect::from_min_size(Pos2::new(c.x + 10.0, c.y - 20.0), Vec2::new(22.0, 42.0)), CornerRadius::same(2), Color32::from_rgb(30, 41, 59));
                    pnt.rect_filled(Rect::from_min_size(Pos2::new(c.x + 12.0, c.y + 10.0), Vec2::new(18.0, 10.0)), CornerRadius::same(2), Color32::from_rgb(34, 197, 94));
                }
                2 => {
                    // Thermal Receipt preview: narrow paper slip with barcode lines
                    let c = thumb_rect.center();
                    let paper = Rect::from_center_size(c, Vec2::new(34.0, 48.0));
                    pnt.rect_filled(paper, CornerRadius::same(2), Color32::from_rgb(230, 235, 242));
                    pnt.rect_stroke(paper, CornerRadius::same(2), Stroke::new(0.5, Color32::from_rgb(180, 190, 205)), egui::StrokeKind::Outside);
                    for i in 0..4 {
                        let y = paper.top() + 10.0 + (i as f32) * 6.0;
                        pnt.line_segment([Pos2::new(paper.left() + 6.0, y), Pos2::new(paper.right() - 6.0, y)], Stroke::new(1.0, Color32::from_rgb(70, 80, 95)));
                    }
                }
                3 => {
                    // Automotive Kanban preview: 3 lanes
                    let c = thumb_rect.center();
                    for col in 0..3 {
                        let x = c.x - 30.0 + (col as f32) * 22.0;
                        let lane = Rect::from_min_size(Pos2::new(x, c.y - 20.0), Vec2::new(18.0, 42.0));
                        pnt.rect_filled(lane, CornerRadius::same(2), Color32::from_rgb(24, 28, 36));
                        pnt.rect_filled(Rect::from_min_size(Pos2::new(x + 2.0, c.y - 14.0), Vec2::new(14.0, 10.0)), CornerRadius::same(1), Color32::from_rgb(56, 189, 248));
                    }
                }
                _ => {
                    // Generic project diagram preview
                    let c = thumb_rect.center();
                    pnt.circle_stroke(c, 16.0, Stroke::new(1.5, Color32::from_rgb(90, 105, 125)));
                    pnt.circle_stroke(c, 10.0, Stroke::new(1.0, Color32::from_rgb(70, 85, 105)));
                    pnt.circle_filled(c, 4.0, Color32::from_rgb(56, 189, 248));
                }
            }

            // Truncated document title label below thumbnail
            pnt.text(
                Pos2::new(rect.left() + 8.0, rect.bottom() - 18.0),
                egui::Align2::LEFT_CENTER,
                &label,
                egui::FontId::proportional(10.5),
                if is_hovered { Color32::WHITE } else { p.text_primary },
            );

            if resp.clicked() {
                app.pid = proj.id.clone();
                app.pname = proj.name.clone();
            }
            if resp.double_clicked() {
                app.pid = proj.id.clone();
                app.pname = proj.name.clone();
                app.load_project();
                app.in_welcome_hub = false;
            }
        }
    });

    ui.add_space(26.0);

    // ── SECTION 2: WELCOME TO PROTEUS (Affinity Starter & Media Grid) ──
    ui.horizontal(|ui| {
        ui.label(RichText::new("Welcome to Proteus").size(13.5).strong().color(p.text_primary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let see_more = ui.add(
                egui::Label::new(RichText::new("See more...").size(11.0).color(p.text_dim))
                    .sense(egui::Sense::click())
            );
            if see_more.clicked() {
                app.welcome_tab = WelcomeTab::Templates;
            }
        });
    });
    ui.add_space(10.0);

    let welcome_cards = [
        ("Intro to Proteus", "A quick overview of Proteus Studio & PCD", "52.4 MB", 0),
        ("Retail & POS Counter", "Shop checkout, cash drawer & receipts", "8.2 MB", 1),
        ("Technician Service BOS", "6-Lane Kanban & work order intake", "14.5 MB", 2),
        ("Thermal Receipt Engine", "ESC/POS spooler & Greek myDATA QR", "839 KB", 3),
        ("LAN Mesh & Replication", "Autonomous P2P UDP sync & outbox", "12.3 MB", 4),
        ("Medical Patient Intake", "GDPR right to erasure & scheduler", "29.9 MB", 5),
    ];

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 16.0;
        for (title, desc, size_label, art_kind) in welcome_cards {
            let card_size = Vec2::new(150.0, 168.0);
            let (rect, resp) = ui.allocate_exact_size(card_size, egui::Sense::click());
            let is_hovered = resp.hovered();

            let pnt = ui.painter();
            // Card container
            let card_bg = if is_hovered { Color32::from_rgb(34, 34, 40) } else { Color32::from_rgb(24, 24, 28) };
            let card_stroke = if is_hovered { Stroke::new(1.0, Color32::from_rgb(80, 80, 96)) } else { Stroke::new(1.0, Color32::from_rgb(38, 38, 44)) };
            pnt.rect_filled(rect, CornerRadius::same(6), card_bg);
            pnt.rect_stroke(rect, CornerRadius::same(6), card_stroke, egui::StrokeKind::Outside);

            // Artwork canvas header (top ~100px)
            let art_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 102.0));
            let (art_color_a, art_color_b) = match art_kind {
                0 => (Color32::from_rgb(30, 58, 138), Color32::from_rgb(56, 189, 248)),
                1 => (Color32::from_rgb(20, 83, 45), Color32::from_rgb(74, 222, 128)),
                2 => (Color32::from_rgb(120, 53, 15), Color32::from_rgb(251, 191, 36)),
                3 => (Color32::from_rgb(76, 29, 149), Color32::from_rgb(192, 132, 252)),
                4 => (Color32::from_rgb(136, 19, 55), Color32::from_rgb(251, 113, 133)),
                _ => (Color32::from_rgb(15, 23, 42), Color32::from_rgb(148, 163, 184)),
            };
            pnt.rect_filled(art_rect, CornerRadius::same(6), art_color_a);

            // Decorative modern vector pattern in art header
            let center = art_rect.center();
            pnt.circle_stroke(center, 28.0, Stroke::new(1.5, art_color_b));
            pnt.circle_stroke(center, 18.0, Stroke::new(1.0, art_color_b));
            pnt.circle_filled(center, 8.0, Color32::WHITE);

            // Dark semi-transparent scrim footer (bottom 66px)
            let scrim_rect = Rect::from_min_size(Pos2::new(rect.left(), rect.top() + 102.0), Vec2::new(rect.width(), 66.0));
            pnt.rect_filled(scrim_rect, CornerRadius::same(6), Color32::from_rgba_unmultiplied(18, 18, 22, 235));

            // Title
            pnt.text(
                Pos2::new(scrim_rect.left() + 8.0, scrim_rect.top() + 14.0),
                egui::Align2::LEFT_CENTER,
                title,
                egui::FontId::proportional(11.5),
                Color32::WHITE,
            );

            // Subtitle
            let short_desc = if desc.len() > 28 { format!("{}...", &desc[..26]) } else { desc.to_string() };
            pnt.text(
                Pos2::new(scrim_rect.left() + 8.0, scrim_rect.top() + 30.0),
                egui::Align2::LEFT_CENTER,
                short_desc,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(160, 165, 175),
            );

            // Cloud download icon & size badge
            let badge_text = format!("☁ {}", size_label);
            pnt.text(
                Pos2::new(scrim_rect.right() - 8.0, scrim_rect.bottom() - 10.0),
                egui::Align2::RIGHT_CENTER,
                badge_text,
                egui::FontId::proportional(9.0),
                Color32::from_rgb(140, 145, 155),
            );

            if resp.clicked() {
                match art_kind {
                    3 => {
                        app.device_preset = crate::models::DevicePreset::Thermal80mm;
                        app.viewport_profile = crate::viewport::ViewportProfile::thermal_receipt_80mm();
                        app.project_doc = crate::scene::create_thermal_receipt_preset(false, "Proteus Service Lab");
                    }
                    _ => {
                        app.project_doc = crate::scene::create_crm_preset();
                    }
                }
                app.pname = title.to_string();
                app.in_welcome_hub = false;
            }
        }
    });

    ui.add_space(26.0);

    // ── BOTTOM ACTION BAR ──
    Frame::new()
        .fill(p.surface_secondary)
        .stroke(Stroke::new(1.0, p.border_strong))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(20, 14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let role_abbr = match app.welcome_selected_occupation {
                        0 => "UI/UX Designer",
                        1 => "Business Analyst",
                        2 => "IT Systems",
                        3 => "Field Deployer",
                        4 => "Database Audit",
                        _ => "Designer",
                    };
                    ui.label(RichText::new(format!("Active Account: {} ({})", app.welcome_user_name, role_abbr)).size(11.5).strong().color(p.text_primary));
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

                    if ui.add(
                        egui::Button::new(RichText::new("⚡ Import .SQL Schema").size(11.0).color(p.text_primary))
                            .fill(p.surface)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(56, 189, 248)))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(Vec2::new(160.0, 34.0))
                    ).on_hover_text("Ingest .SQL DDL script, map relational tables & build visual flow nodes").clicked() {
                        app.in_welcome_hub = false;
                        app.mode = Mode::FlowBuilder;
                        app.show_sql_import_modal = true;
                    }
                });
            });
        });
}

fn render_recent_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());
    ui.label(RichText::new("Recent Documents & Projects").size(14.0).strong().color(p.text_primary));
    ui.add_space(12.0);

    let projects = app.list_recent_projects();
    if projects.is_empty() {
        ui.label(RichText::new("No projects found in store.db. Click ＋ above to create one.").size(11.0).color(p.text_dim));
        return;
    }

    for proj in projects {
        let card = ui.add(
            egui::Button::new("")
                .fill(p.surface_secondary)
                .stroke(Stroke::new(1.0, p.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(ui.available_width(), 52.0))
        );
        let r = card.rect;
        let pnt = ui.painter();
        pnt.text(Pos2::new(r.left() + 16.0, r.center().y - 8.0), egui::Align2::LEFT_CENTER, &proj.name, egui::FontId::proportional(12.5), p.text_primary);
        let updated_display: String = proj.updated_at.chars().take(19).collect();
        let details = format!("ID: {}  •  Updated: {}  •  store.db", &proj.id, updated_display);
        pnt.text(Pos2::new(r.left() + 16.0, r.center().y + 10.0), egui::Align2::LEFT_CENTER, details, egui::FontId::proportional(10.0), p.text_dim);
        pnt.text(Pos2::new(r.right() - 20.0, r.center().y), egui::Align2::RIGHT_CENTER, "Open ➔", egui::FontId::proportional(11.0), p.cta_primary_fill);

        if card.clicked() {
            app.pid = proj.id;
            app.pname = proj.name;
            app.load_project();
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
                        let new_id = uuid::Uuid::new_v4().to_string();
                        app.pid = new_id;
                        app.pname = title.to_string();
                        app.project_doc = crate::scene::create_crm_preset();
                        app.save_project();
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

fn render_favorites_tab(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let p = app.palette(ui.ctx());
    ui.label(RichText::new("Favorite Documents & Templates").size(14.0).strong().color(p.text_primary));
    ui.add_space(8.0);
    ui.label(RichText::new("Quickly access your pinned projects and production blueprints.").size(11.0).color(p.text_dim));
    ui.add_space(16.0);

    let favorites = [
        ("⭐ Proteus Design System Master", "Design tokens, 8-pt typography & component library", "pds_master.pr"),
        ("⭐ Automotive Service Counter BOS", "6 Status lanes, technician work orders & POS thermal spooler", "store.db"),
    ];

    for (title, desc, _file) in favorites {
        let card = ui.add(
            egui::Button::new("")
                .fill(p.surface_secondary)
                .stroke(Stroke::new(1.0, p.border_subtle))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(ui.available_width(), 52.0))
        );
        let r = card.rect;
        let pnt = ui.painter();
        pnt.text(Pos2::new(r.left() + 16.0, r.center().y - 8.0), egui::Align2::LEFT_CENTER, title, egui::FontId::proportional(12.5), p.text_primary);
        pnt.text(Pos2::new(r.left() + 16.0, r.center().y + 10.0), egui::Align2::LEFT_CENTER, desc, egui::FontId::proportional(10.0), p.text_dim);
        pnt.text(Pos2::new(r.right() - 20.0, r.center().y), egui::Align2::RIGHT_CENTER, "Open ➔", egui::FontId::proportional(11.0), p.cta_primary_fill);

        if card.clicked() {
            app.pname = title.trim_start_matches("⭐ ").to_string();
            app.in_welcome_hub = false;
        }
    }
}
