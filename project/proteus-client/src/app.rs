//! Main Application State & Coordinator for Proteus Client (Proteus.exe).
//! Standalone Shop Counter Runtime.

use crate::views::intake::{draw_intake_view, IntakeFormState};
use crate::views::pipeline::draw_pipeline_view;
use crate::views::settings::{draw_settings_view, SettingsViewState};
use crate::views::ticket_detail::{draw_ticket_detail_modal, TicketDetailState};
use crm_core::paths::{ensure_database_dir_exists, get_database_path};
use crm_core::printer::ShopReceiptConfig;
use crm_core::tickets::{init_tickets_schema, list_tickets};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke};
use rusqlite::Connection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    Intake,
    Pipeline,
    Settings,
}

pub struct ProteusClientApp {
    conn: Connection,
    active_tab: NavTab,
    intake_state: IntakeFormState,
    pipeline_search: String,
    selected_ticket_id: Option<String>,
    ticket_detail_state: TicketDetailState,
    receipt_config: ShopReceiptConfig,
    settings_state: SettingsViewState,
}

impl ProteusClientApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initialize SQLite at standard path
        let db_path = get_database_path();
        let _ = ensure_database_dir_exists(&db_path);

        let conn = Connection::open(&db_path).unwrap_or_else(|_| {
            // In-memory fallback if disk error
            Connection::open_in_memory().expect("Critical: Failed to open SQLite")
        });

        let _ = init_tickets_schema(&conn);

        Self {
            conn,
            active_tab: NavTab::Intake,
            intake_state: IntakeFormState::default(),
            pipeline_search: String::new(),
            selected_ticket_id: None,
            ticket_detail_state: TicketDetailState::default(),
            receipt_config: ShopReceiptConfig::default(),
            settings_state: SettingsViewState::default(),
        }
    }
}

impl eframe::App for ProteusClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::theme::apply_theme(ctx);

        // Top Navigation Bar
        egui::TopBottomPanel::top("top_nav_bar")
            .frame(
                Frame::new()
                    .fill(crate::theme::BG_PANEL)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .inner_margin(Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo & Brand
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("PROTEUS").strong().size(18.0).color(crate::theme::ACCENT_CYAN));
                        ui.label(RichText::new("BOS").size(14.0).color(crate::theme::TEXT_MUTED));
                    });

                    ui.add_space(24.0);

                    // Nav Tabs
                    let tabs = [
                        (NavTab::Intake, "⚡ Νέα Παραλαβή"),
                        (NavTab::Pipeline, "📋 Ροή Επισκευών"),
                        (NavTab::Settings, "⚙ Ρυθμίσεις"),
                    ];

                    for (tab, label) in tabs {
                        let is_active = self.active_tab == tab;
                        let btn = if is_active {
                            egui::Button::new(RichText::new(label).strong().size(14.0).color(Color32::WHITE))
                                .fill(crate::theme::ACCENT_PRIMARY)
                        } else {
                            egui::Button::new(RichText::new(label).size(14.0).color(crate::theme::TEXT_SECONDARY))
                                .fill(crate::theme::BG_CARD)
                        };

                        if ui.add_sized([150.0, 32.0], btn).clicked() {
                            self.active_tab = tab;
                        }
                        ui.add_space(4.0);
                    }

                    // Right Side: Active count badge & offline pill
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Offline-first pill
                        Frame::new()
                            .fill(Color32::from_rgb(16, 50, 35))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(RichText::new("● 100% Τοπική Λειτουργία").size(11.0).color(Color32::from_rgb(52, 211, 153)));
                            });

                        let total_tickets = list_tickets(&self.conn).map(|t| t.len()).unwrap_or(0);
                        ui.label(RichText::new(format!("Σύνολο: {} Επισκευές", total_tickets)).size(12.0).color(crate::theme::TEXT_MUTED));
                    });
                });
            });

        // Central View Area
        egui::CentralPanel::default()
            .frame(
                Frame::new()
                    .fill(crate::theme::BG_BASE)
                    .inner_margin(Margin::same(16)),
            )
            .show(ctx, |ui| {
                match self.active_tab {
                    NavTab::Intake => {
                        draw_intake_view(
                            ui,
                            &self.conn,
                            &mut self.intake_state,
                            &self.receipt_config,
                            &self.settings_state.printer_name,
                        );
                    }
                    NavTab::Pipeline => {
                        draw_pipeline_view(
                            ui,
                            &self.conn,
                            &mut self.pipeline_search,
                            &mut self.selected_ticket_id,
                        );
                    }
                    NavTab::Settings => {
                        draw_settings_view(
                            ui,
                            &mut self.receipt_config,
                            &mut self.settings_state,
                        );
                    }
                }
            });

        // Ticket Detail Modal (if a card is clicked)
        draw_ticket_detail_modal(
            ctx,
            &self.conn,
            &mut self.selected_ticket_id,
            &mut self.ticket_detail_state,
            &self.receipt_config,
            &self.settings_state.printer_name,
        );
    }
}
