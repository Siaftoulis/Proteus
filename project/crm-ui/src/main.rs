#![cfg_attr(not(test), windows_subsystem = "windows")]

//! Proteus UI & Designer Studio main entry point.
//! Initializes eframe native window and coordinates the UI layout.

pub mod app_state;
pub mod components;
pub mod csv_utils;
pub mod data_viewer;
pub mod db;
pub mod flow;
pub mod flow_renderer;
pub mod inspector;
pub mod menu_bar;
pub mod models;
pub mod palette;
pub mod renderer;
pub mod scene;
pub mod storage;
pub mod theme;
pub mod viewport;
pub mod views;

pub use app_state::ProteusApp;
pub use models::*;
pub use viewport::ViewportProfile;

use eframe::egui::{self, Color32, Rect, Sense, Stroke};

impl eframe::App for ProteusApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.loaded {
            self.loaded = true;
            self.load_project();
        }

        if self.toast.is_some() {
            self.tt += ctx.input(|i| i.unstable_dt);
            if self.tt > 3.0 {
                self.toast = None;
            }
        }

        if !self._style_set {
            theme::configure_egui_style(ctx, self.theme_mode, self.accent_preset);
            self._style_set = true;
        }

        let p = self.palette(ctx);

        // ── TOP BARS ──
        menu_bar::show(self, ctx);
        components::top_bar::show(self, ctx);
        components::mode_bar::show(self, ctx);

        // ── LEFT SIDEBAR (Penpot Studio Sidebar: Tools + Layers/Assets) ──
        let (left_w, left_margin) = if self.mode == Mode::Designer {
            (220., egui::Margin::symmetric(6, 8))
        } else {
            (180., egui::Margin::same(12))
        };
        egui::SidePanel::left("pal")
            .resizable(false)
            .exact_width(left_w)
            .frame(egui::Frame { fill: p.panel, inner_margin: left_margin, stroke: egui::Stroke::new(1., p.border), ..Default::default() })
            .show(ctx, |ui| {
                match self.mode {
                    Mode::Designer => views::designer::show_left(self, ui),
                    Mode::Analyst => views::analyst::show_left(&mut self.analyst_state, ui),
                    Mode::Networking => views::networking::show_left(&mut self.networking_state, ui),
                    Mode::Troubleshoot => views::troubleshoot::show_left(&mut self.troubleshoot_state, ui),
                    Mode::ConnectedData => views::connected_data::show_left(&mut self.connected_data_state, self.db_conn.as_ref(), ui),
                    Mode::FlowBuilder => views::flow_builder::show_left(self, ui),
                    Mode::Contacts => views::contacts::show_left(self, ui),
                    Mode::Pipeline => views::pipeline::show_left(self, ui),
                    Mode::Tasks => views::tasks::show_left(self, ui),
                    Mode::Studio => views::studio::show_left(self, ui),
                    Mode::Play => views::play::show_left(self, ui),
                    Mode::DataViewer => views::data_viewer::show_left(self, ui),
                }
            });

        // ── RIGHT PROPERTIES PANEL (Penpot Inspector) ──
        let right_title = match self.mode {
            Mode::Designer => "INSPECTOR",
            Mode::Analyst => "ANALYST GUIDE",
            Mode::Networking => "NETWORK GUIDE",
            Mode::Troubleshoot => "HARDWARE GUIDE",
            Mode::ConnectedData => "DATABASE GUIDE",
            Mode::Play => "PLAY MODE",
            Mode::DataViewer => "DATA VIEWER",
            Mode::FlowBuilder => "FLOW INFO",
            Mode::Contacts => "CONTACT",
            Mode::Pipeline => "DEAL DETAIL",
            Mode::Studio => "LAYERS",
            Mode::Tasks => "TASK DETAIL",
        };
        let (right_w, right_min) = if self.mode == Mode::Designer {
            (260., 240.)
        } else {
            (240., 200.)
        };
        egui::SidePanel::right("prop")
            .resizable(false)
            .default_width(right_w)
            .min_width(right_min)
            .frame(egui::Frame { fill: p.panel, inner_margin: egui::Margin::same(10), stroke: egui::Stroke::new(1., p.border), ..Default::default() })
            .show(ctx, |ui| {
                ui.add_space(4.);
                ui.label(egui::RichText::new(right_title).size(9.).color(p.text_dim));
                ui.add_space(4.);
                match self.mode {
                    Mode::Designer => views::designer::show_right(self, ui),
                    Mode::Analyst => {
                        ui.label(egui::RichText::new("Ingest raw data and infer schema to export as a .pr package.").size(10.5).color(p.text_dim));
                    }
                    Mode::Networking => {
                        ui.label(egui::RichText::new("UDP Beacon broadcasts on port 7444. Manage external server gateways.").size(10.5).color(p.text_dim));
                    }
                    Mode::Troubleshoot => {
                        ui.label(egui::RichText::new("Direct spooler and hardware probes for thermal printers and cash drawers.").size(10.5).color(p.text_dim));
                    }
                    Mode::ConnectedData => {
                        ui.label(egui::RichText::new("Live SQLite state, prioritized outbox sync, and Merkle audit chain verification.").size(10.5).color(p.text_dim));
                    }
                    Mode::FlowBuilder => views::flow_builder::show_right(self, ui),
                    Mode::Contacts => views::contacts::show_right(self, ui),
                    Mode::Pipeline => views::pipeline::show_right(self, ui),
                    Mode::Tasks => views::tasks::show_right(self, ui),
                    Mode::Studio => views::studio::show_right(self, ui),
                    Mode::Play => views::play::show_right(self, ui),
                    Mode::DataViewer => views::data_viewer::show_right(self, ui),
                }
            });

        // ── DEVICE TOOLBAR (Designer only, toggleable) ──
        if self.show_device_toolbar {
            components::device_bar::show(self, ctx);
        }

        // ── CENTRAL CANVAS ──
        egui::CentralPanel::default()
            .frame(egui::Frame { fill: p.bg, ..Default::default() })
            .show(ctx, |ui| {
                let (resp, pnt) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
                let r = resp.rect;
                pnt.rect_filled(r, 0, p.bg);

                // Grid background for non-designer modes
                if self.mode != Mode::Designer && self.mode != Mode::FlowBuilder {
                    if self.layout == LayoutMode::Grid {
                        let mut y = r.top();
                        while y <= r.bottom() {
                            pnt.line_segment([egui::pos2(r.left(), y), egui::pos2(r.right(), y)], Stroke::new(1., p.border));
                            y += GRID;
                        }
                        let mut x = r.left();
                        while x <= r.right() {
                            pnt.line_segment([egui::pos2(x, r.top()), egui::pos2(x, r.bottom())], Stroke::new(1., p.border));
                            x += GRID;
                        }
                    } else {
                        let mut y = r.top() + 12.;
                        while y < r.bottom() {
                            let mut x = r.left() + 12.;
                            while x < r.right() {
                                pnt.circle_filled(egui::pos2(x, y), 1., p.border_light);
                                x += 24.;
                            }
                            y += 24.;
                        }
                    }
                }

                let is_animating = self.toast.is_some();
                let is_interacting = ui.input(|i| i.pointer.any_down() || i.pointer.delta() != egui::Vec2::ZERO || !i.events.is_empty());
                let needs_wake = is_animating || is_interacting || self.designer_drag_active || self.flow_drag_active;
                self.vrr_mode.apply_to_ctx(ctx, needs_wake);

                let mpos = ui.input(|i| i.pointer.interact_pos());
                let mdown = ui.input(|i| i.pointer.any_down());
                let mup = ui.input(|i| i.pointer.any_released());

                match self.mode {
                    Mode::Contacts => views::contacts::show_central(self, &pnt, r, ui),
                    Mode::Pipeline => views::pipeline::show_central(self, &pnt, r, mpos, mup),
                    Mode::Tasks => views::tasks::show_central(self, &pnt, r, mpos, mup, ui),
                    Mode::Studio => views::studio::show_central(self, ctx, &pnt, r, mpos, mdown, mup),
                    Mode::Designer => views::designer::show_central(self, ctx, ui, &pnt, r, mpos, &resp),
                    Mode::Analyst => views::analyst::show_central(&mut self.analyst_state, ui),
                    Mode::Networking => views::networking::show_central(&mut self.networking_state, ui),
                    Mode::Troubleshoot => views::troubleshoot::show_central(&mut self.troubleshoot_state, ui),
                    Mode::ConnectedData => {
                        if let Some(msg) = views::connected_data::show_central(
                            &mut self.connected_data_state,
                            &self.contacts,
                            &self.deals,
                            &self.tasks,
                            ui,
                        ) {
                            self.toast(msg);
                        }
                    }
                    Mode::Play => views::play::show_central(self, ui, &pnt, r),
                    Mode::DataViewer => views::data_viewer::show_central(self, ui),
                    Mode::FlowBuilder => views::flow_builder::show_central(self, ctx, ui, &pnt, r, mpos, mdown),
                }

                // Toast overlay
                if let Some(msg) = &self.toast {
                    let tr = Rect::from_min_size(egui::pos2(r.center().x - 120., r.bottom() - 40.), egui::vec2(240., 28.));
                    pnt.rect_filled(tr, 0, Color32::from_rgba_premultiplied(20, 20, 20, 220));
                    pnt.rect_stroke(tr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
                    pnt.text(tr.center(), egui::Align2::CENTER_CENTER, msg, egui::FontId::proportional(11.), theme::TEXT);
                }
            });

        // ── MODALS ──
        views::data_viewer::show_edit_modal(self, ctx);

        if self.show_login {
            egui::Window::new("Sign In")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.))
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Continue without signing in, or log in for sync.").size(11.).color(theme::TEXT));
                    ui.add_space(8.);
                    if ui.button(egui::RichText::new("Sign In (demo)").size(10.)).clicked() {
                        self.auth = Some("demo".into());
                        self.show_login = false;
                        self.toast("Signed in");
                    }
                    if ui.button(egui::RichText::new("Continue Offline").size(10.)).clicked() {
                        self.show_login = false;
                    }
                });
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        use std::io::Write;
        let msg = format!("[PROTEUS PANIC] {}\n", info);
        let _ = std::io::stderr().write_all(msg.as_bytes());
        let _ = std::io::stderr().flush();
        if let Ok(mut f) = std::fs::File::create("proteus_panic.txt") {
            let _ = f.write_all(msg.as_bytes());
        }
    }));

    eprintln!("[PROTEUS] Starting Proteus UI...");
    let res = eframe::run_native(
        "Proteus - The Visual OS for Business",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Proteus - The Visual OS for Business")
                .with_maximized(true)
                .with_min_inner_size([960., 600.])
                .with_active(true),
            ..Default::default()
        },
        Box::new(|_cc| {
            eprintln!("[PROTEUS] Initializing ProteusApp...");
            Ok(Box::new(ProteusApp::default()))
        }),
    );
    if let Err(e) = res {
        eprintln!("[PROTEUS ERROR] eframe failed: {:?}", e);
    }
    eprintln!("[PROTEUS] Exiting main.");
}
