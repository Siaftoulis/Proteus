//! Sovereign Setup / Preflight Verification Engine for Proteus BOS.
//! Handles database integrity verification (store.db), core schema table validation,
//! workspace initialization, and transitions cleanly into the main studio window.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightStage {
    ProbeStoreDb,
    CheckSchema,
    InitWorkspace,
    Ready,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightAction {
    Continue,
    TransitionToStudio,
    Cancel,
}

pub struct PreflightState {
    pub enabled: bool,
    pub stage: PreflightStage,
    pub progress: f32,
    pub target_progress: f32,
    pub status_message: String,
    pub db_probe_ok: bool,
    pub schema_ok: bool,
    pub workspace_ok: bool,
    pub error_message: Option<String>,
    pub step_timer: f32,
    pub completed: bool,
    pub custom_db_path: Option<PathBuf>,
}

impl Default for PreflightState {
    fn default() -> Self {
        Self::new(false)
    }
}

impl PreflightState {
    pub fn new(enabled: bool) -> Self {
        if enabled {
            Self {
                enabled: true,
                stage: PreflightStage::ProbeStoreDb,
                progress: 0.05,
                target_progress: 0.33,
                status_message: "Probing store.db integrity...".to_string(),
                db_probe_ok: false,
                schema_ok: false,
                workspace_ok: false,
                error_message: None,
                step_timer: 0.0,
                completed: false,
                custom_db_path: None,
            }
        } else {
            Self {
                enabled: false,
                stage: PreflightStage::Completed,
                progress: 1.0,
                target_progress: 1.0,
                status_message: "Ready".to_string(),
                db_probe_ok: true,
                schema_ok: true,
                workspace_ok: true,
                error_message: None,
                step_timer: 0.0,
                completed: true,
                custom_db_path: None,
            }
        }
    }

    pub fn is_active(&self) -> bool {
        self.enabled && !self.completed
    }

    pub fn retry(&mut self) {
        self.stage = PreflightStage::ProbeStoreDb;
        self.progress = 0.05;
        self.target_progress = 0.33;
        self.status_message = "Probing store.db integrity...".to_string();
        self.error_message = None;
        self.step_timer = 0.0;
        self.db_probe_ok = false;
        self.schema_ok = false;
        self.workspace_ok = false;
    }

    pub fn resolve_db_path(&self) -> PathBuf {
        self.custom_db_path.clone().unwrap_or_else(proteus_core::paths::get_database_path)
    }

    pub fn tick_logic(&mut self, dt: f32, db_conn: &mut Option<rusqlite::Connection>) {
        if self.stage == PreflightStage::Completed || self.stage == PreflightStage::Failed {
            return;
        }
        self.step_timer += dt;

        match self.stage {
            PreflightStage::ProbeStoreDb => {
                let db_path = self.resolve_db_path();
                if let Err(e) = proteus_core::paths::ensure_database_dir_exists(&db_path) {
                    self.fail(format!("Database directory error: {}", e));
                    return;
                }
                match rusqlite::Connection::open(&db_path) {
                    Ok(conn) => {
                        let check: Result<String, _> = conn.query_row("PRAGMA quick_check;", [], |r| r.get(0));
                        if check.as_deref() == Ok("ok") {
                            self.db_probe_ok = true;
                            if self.step_timer >= 0.12 {
                                self.stage = PreflightStage::CheckSchema;
                                self.target_progress = 0.66;
                                self.status_message = "Verifying database schema & tables...".to_string();
                                self.step_timer = 0.0;
                            }
                        } else {
                            self.fail(format!("store.db corruption detected: {:?}", check));
                        }
                    }
                    Err(e) => self.fail(format!("Failed to open store.db: {}", e)),
                }
            }
            PreflightStage::CheckSchema => {
                let db_path = self.resolve_db_path();
                match crate::db::init_db(&db_path) {
                    Ok(conn) => {
                        let count: Result<i64, _> = conn.query_row(
                            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('records', 'audit_logs', 'serial_genealogies', '_proteus_outbox', 'tamper_proof_audit_backlog')",
                            [],
                            |r| r.get(0),
                        );
                        if let Ok(c) = count {
                            if c >= 5 {
                                self.schema_ok = true;
                                *db_conn = Some(conn);
                                if self.step_timer >= 0.12 {
                                    self.stage = PreflightStage::InitWorkspace;
                                    self.target_progress = 0.95;
                                    self.status_message = "Initializing workspace environment...".to_string();
                                    self.step_timer = 0.0;
                                }
                            } else {
                                self.fail(format!("Incomplete core tables: found {} / 5", c));
                            }
                        } else {
                            self.fail("Failed to verify schema tables".to_string());
                        }
                    }
                    Err(e) => self.fail(format!("Failed to init schema: {}", e)),
                }
            }
            PreflightStage::InitWorkspace => {
                let db_path = self.resolve_db_path();
                let parent = db_path.parent().unwrap_or_else(|| std::path::Path::new("."));
                if let Err(e) = std::fs::create_dir_all(parent.join("workspace")) {
                    self.fail(format!("Failed to create workspace directory: {}", e));
                    return;
                }
                self.workspace_ok = true;
                if self.step_timer >= 0.12 {
                    self.stage = PreflightStage::Ready;
                    self.target_progress = 1.0;
                    self.status_message = "Proteus is ready".to_string();
                    self.step_timer = 0.0;
                }
            }
            PreflightStage::Ready if self.progress >= 0.98 && self.step_timer >= 0.1 => {
                self.stage = PreflightStage::Completed;
            }
            _ => {}
        }
    }

    fn fail(&mut self, err: String) {
        self.stage = PreflightStage::Failed;
        self.status_message = err.clone();
        self.error_message = Some(err);
    }

    pub fn update_and_render(
        &mut self,
        ctx: &egui::Context,
        p: &crate::theme::Palette,
        db_conn: &mut Option<rusqlite::Connection>,
    ) -> PreflightAction {
        let dt = ctx.input(|i| i.unstable_dt).min(0.05);
        self.progress += (self.target_progress - self.progress) * (dt * 10.0).min(1.0);
        self.tick_logic(dt, db_conn);

        if !self.completed {
            ctx.request_repaint();
        }

        if self.stage == PreflightStage::Completed && !self.completed {
            self.completed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Title("Proteus - The Visual OS for Business".to_string()));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(960.0, 600.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(1280.0, 800.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            return PreflightAction::TransitionToStudio;
        }

        self.render(ctx, p)
    }

    pub fn render(&mut self, ctx: &egui::Context, _p: &crate::theme::Palette) -> PreflightAction {
        let mut action = PreflightAction::Continue;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(18, 19, 24)).inner_margin(egui::Margin::symmetric(24, 20)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(30.0, 30.0), egui::Sense::hover());
                    ui.painter().rect_filled(badge_rect, 6.0, Color32::from_rgb(30, 41, 59));
                    ui.painter().rect_stroke(badge_rect, 6.0, Stroke::new(1.0, Color32::from_rgb(56, 189, 248)), egui::StrokeKind::Inside);
                    ui.painter().text(badge_rect.center(), egui::Align2::CENTER_CENTER, "P", egui::FontId::proportional(16.0), Color32::from_rgb(56, 189, 248));

                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Setting up Proteus").size(16.0).strong().color(Color32::from_rgb(245, 245, 250)));
                        ui.label(egui::RichText::new("Sovereign Business Operating System").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                    });
                });

                ui.add_space(14.0);
                let msg_color = if self.error_message.is_some() { Color32::from_rgb(248, 113, 113) } else { Color32::from_rgb(203, 213, 225) };
                ui.label(egui::RichText::new(&self.status_message).size(11.5).color(msg_color));
                ui.add_space(6.0);

                let available_width = ui.available_width();
                let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(available_width, 6.0), egui::Sense::hover());
                ui.painter().rect_filled(bar_rect, 3.0, Color32::from_rgb(30, 32, 40));

                let fill_w = (bar_rect.width() * self.progress.clamp(0.0, 1.0)).max(2.0);
                let fill_rect = Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height()));
                let fill_color = if self.error_message.is_some() { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(56, 189, 248) };
                ui.painter().rect_filled(fill_rect, 3.0, fill_color);

                if self.stage != PreflightStage::Completed && self.error_message.is_none() {
                    let sweep = (((ctx.input(|i| i.time) as f32) * 1.5) % 1.4) - 0.2;
                    if (0.0..=1.0).contains(&sweep) {
                        let shim_x = fill_rect.min.x + fill_rect.width() * sweep;
                        let shim_w = 24.0_f32.min(fill_rect.width() * 0.3);
                        let shim_rect = Rect::from_min_size(Pos2::new(shim_x - shim_w / 2.0, fill_rect.min.y), Vec2::new(shim_w, fill_rect.height())).intersect(fill_rect);
                        ui.painter().rect_filled(shim_rect, 3.0, Color32::from_white_alpha(90));
                    }
                }

                ui.add_space(14.0);
                let check_items = [
                    ("store.db Integrity Probe", self.db_probe_ok, self.stage == PreflightStage::ProbeStoreDb),
                    ("Database Schema & Tables", self.schema_ok, self.stage == PreflightStage::CheckSchema),
                    ("Workspace Environment", self.workspace_ok, self.stage == PreflightStage::InitWorkspace),
                ];

                for (label, is_ok, is_active) in check_items {
                    let (icon, color) = if is_ok {
                        ("✓", Color32::from_rgb(74, 222, 128))
                    } else if is_active {
                        ("●", Color32::from_rgb(56, 189, 248))
                    } else {
                        ("○", Color32::from_rgb(100, 116, 139))
                    };
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(icon).size(11.0).color(color));
                        let text_color = if is_ok || is_active { Color32::from_rgb(226, 232, 240) } else { Color32::from_rgb(100, 116, 139) };
                        ui.label(egui::RichText::new(label).size(10.5).color(text_color));
                    });
                    ui.add_space(2.0);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        if self.error_message.is_some() {
                            let retry_btn = egui::Button::new(egui::RichText::new("Retry").size(11.0).color(Color32::from_rgb(240, 240, 245)))
                                .fill(Color32::from_rgb(30, 41, 59))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(56, 189, 248)))
                                .corner_radius(egui::CornerRadius::same(4));
                            if ui.add(retry_btn).clicked() {
                                self.retry();
                            }
                        }

                        let cancel_btn = egui::Button::new(egui::RichText::new("Cancel").size(11.0).color(Color32::from_rgb(203, 213, 225)))
                            .fill(Color32::from_rgb(30, 32, 40))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(71, 85, 105)))
                            .corner_radius(egui::CornerRadius::same(4));
                        if ui.add(cancel_btn).clicked() {
                            action = PreflightAction::Cancel;
                        }
                    });
                });
            });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_initial_state_disabled() {
        let preflight = PreflightState::new(false);
        assert!(!preflight.enabled && preflight.completed && !preflight.is_active());
        assert_eq!(preflight.stage, PreflightStage::Completed);
        assert_eq!(preflight.progress, 1.0);
    }

    #[test]
    fn test_preflight_initial_state_enabled() {
        let preflight = PreflightState::new(true);
        assert!(preflight.enabled && !preflight.completed && preflight.is_active());
        assert_eq!(preflight.stage, PreflightStage::ProbeStoreDb);
        assert_eq!(preflight.progress, 0.05);
        assert!(preflight.status_message.contains("store.db"));
    }

    #[test]
    fn test_preflight_state_machine_transitions() {
        let temp_dir = std::env::temp_dir().join(format!("proteus_preflight_test_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut preflight = PreflightState::new(true);
        preflight.custom_db_path = Some(temp_dir.join("store.db"));
        let mut db_conn: Option<rusqlite::Connection> = None;

        preflight.tick_logic(0.2, &mut db_conn);
        assert!(preflight.db_probe_ok);
        assert_eq!(preflight.stage, PreflightStage::CheckSchema);

        preflight.tick_logic(0.2, &mut db_conn);
        assert!(preflight.schema_ok);
        assert!(db_conn.is_some());
        assert_eq!(preflight.stage, PreflightStage::InitWorkspace);

        preflight.tick_logic(0.2, &mut db_conn);
        assert!(preflight.workspace_ok);
        assert_eq!(preflight.stage, PreflightStage::Ready);

        preflight.progress = 0.99;
        preflight.tick_logic(0.15, &mut db_conn);
        assert_eq!(preflight.stage, PreflightStage::Completed);
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_preflight_retry_resets_error() {
        let mut preflight = PreflightState::new(true);
        preflight.fail("Corrupted db".to_string());
        assert_eq!(preflight.stage, PreflightStage::Failed);
        preflight.retry();
        assert_eq!(preflight.stage, PreflightStage::ProbeStoreDb);
        assert!(preflight.error_message.is_none() && !preflight.db_probe_ok);
    }
}
