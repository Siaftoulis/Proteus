//! Proteus Mobile (PDS Mobile Touch Client & Android NDK Placeholder).
//! Adheres strictly to the PDS Nordic Slate design system and Rule 5 (Zero Mock Data).
//! Enforces Apple HIG & Android Material 44.0pt touch targets and safe area insets.

use proteus_core::paths::{ensure_database_dir_exists, get_database_path};
use proteus_core::printer::{load_shop_config, ShopReceiptConfig};
use proteus_core::replication::{enqueue_outbox_with_priority, fetch_pending_outbox, init_outbox_schema, ChangeOp, DataPriority};
use proteus_core::tickets::{create_ticket, ServiceTicket};
use egui::{Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, Ui, Vec2};
use rusqlite::Connection;

pub const MIN_TOUCH_TARGET: f32 = 44.0;

// ── PDS Nordic Slate Mobile Tokens ──
pub const BG_BASE: Color32 = Color32::from_rgb(27, 36, 48);     // #1B2430
pub const BG_PANEL: Color32 = Color32::from_rgb(30, 42, 58);    // #1E2A3A
pub const BG_CARD: Color32 = Color32::from_rgb(39, 55, 77);     // #27374D
pub const BORDER_LINE: Color32 = Color32::from_rgb(82, 109, 130); // #526D82
pub const TEXT_TITLE: Color32 = Color32::from_rgb(221, 230, 237); // #DDE6ED
pub const TEXT_BODY: Color32 = Color32::from_rgb(157, 178, 191);  // #9DB2BF
pub const ACCENT_GOLD: Color32 = Color32::from_rgb(214, 158, 46);

static EMBLEM_RGBA_128: &[u8] = include_bytes!("../../assets/proteus_emblem_128.rgba");

/// Obtains or caches the King Proteus emblem texture handle for mobile.
pub fn get_or_load_mobile_emblem_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let id = egui::Id::new("pds_mobile_king_proteus_emblem_tex");
    if let Some(tex) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return tex;
    }
    let img = egui::ColorImage::from_rgba_unmultiplied([128, 128], EMBLEM_RGBA_128);
    let tex = ctx.load_texture("pds_mobile_king_proteus_emblem", img, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    tex
}

/// Helper widget to render the official King Proteus logo emblem on mobile touch surfaces.
pub fn render_mobile_logo_widget(ui: &mut Ui, size: Vec2) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let tex = get_or_load_mobile_emblem_texture(ui.ctx());
        ui.painter().image(
            tex.id(),
            rect,
            egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    resp
}

pub mod device;
pub mod hotspot_bridge;
pub mod intake_tab;
pub mod profile;
pub mod scanner;
pub mod sign_on_glass;
pub mod sync_daemon;
pub mod tickets_tab;
pub mod van_sales;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileTab {
    Tickets,
    NewIntake,
    VanSales,
    Scanner,
    Profile,
}

pub struct MobileAppState {
    pub conn: Connection,
    pub active_tab: MobileTab,
    pub shop_config: ShopReceiptConfig,
    pub search_query: String,
    pub intake_customer: String,
    pub intake_phone: String,
    pub intake_device: String,
    pub intake_problem: String,
    pub status_message: Option<(String, bool)>,
    pub scanned_code: String,
    pub lan_host: String,
    pub lan_port: u16,
    pub paired_terminal: Option<String>,
    pub store_id: Option<String>,
    pub terminal_id: Option<String>,
    pub relay_url: Option<String>,
    pub auth_token: Option<String>,
    pub sync_controller: crate::sync_daemon::MobileSyncController,
    pub van_sales_state: crate::van_sales::VanSalesState,
    pub device_ctx: crate::device::DeviceContext,
}

impl Default for MobileAppState {
    fn default() -> Self {
        let db_path = get_database_path();
        let _ = ensure_database_dir_exists(&db_path);
        let conn = Connection::open(&db_path).unwrap_or_else(|_| {
            Connection::open_in_memory().expect("SQLite DB failed")
        });
        let _ = proteus_core::tickets::init_tickets_schema(&conn);
        let _ = proteus_core::printer::init_shop_settings_schema(&conn);
        let _ = init_outbox_schema(&conn);
        let _ = proteus_core::shipping_note::init_shipping_schema(&conn);
        let shop_config = load_shop_config(&conn);

        Self {
            conn,
            active_tab: MobileTab::Tickets,
            shop_config,
            search_query: String::new(),
            intake_customer: String::new(),
            intake_phone: String::new(),
            intake_device: String::new(),
            intake_problem: String::new(),
            status_message: None,
            scanned_code: String::new(),
            lan_host: "127.0.0.1".into(),
            lan_port: 7443,
            paired_terminal: None,
            store_id: None,
            terminal_id: None,
            relay_url: Some("https://relay.proteus-bos.gr:8443".to_string()),
            auth_token: None,
            sync_controller: crate::sync_daemon::MobileSyncController::default(),
            van_sales_state: crate::van_sales::VanSalesState::default(),
            device_ctx: crate::device::DeviceContext::default(),
        }
    }
}

impl MobileAppState {
    pub fn reload_config(&mut self) {
        self.shop_config = load_shop_config(&self.conn);
    }

    pub fn relay_params(&self) -> Option<crate::sync_daemon::RelaySyncParams> {
        let relay_url = self.relay_url.as_ref()?;
        let store_id = self.store_id.as_ref()?;
        let terminal_id = self.terminal_id.as_ref()?;
        let secret = self.auth_token.as_ref()?;

        Some(crate::sync_daemon::RelaySyncParams {
            relay_url: relay_url.clone(),
            session_id: format!("sess-{}-{}", store_id, crate::sync_daemon::current_epoch_secs()),
            client_id: "mobile-companion".to_string(),
            store_id: store_id.clone(),
            recipient_terminal_id: terminal_id.clone(),
            pairing_secret: secret.clone(),
        })
    }

    pub fn submit_ticket(&mut self) -> Result<String, String> {
        if self.intake_customer.trim().is_empty() || self.intake_device.trim().is_empty() {
            return Err("Συμπληρώστε όνομα πελάτη και συσκευή".into());
        }

        let mut new_t = ServiceTicket::new(
            self.intake_customer.trim(),
            self.intake_phone.trim(),
            self.intake_device.trim(),
            self.intake_problem.trim(),
        );
        let ticket_id = new_t.ticket_id.clone();
        let payload = serde_json::to_string(&new_t).unwrap_or_default();

        match create_ticket(&self.conn, &mut new_t) {
            Ok(()) => {
                let _ = enqueue_outbox_with_priority(
                    &self.conn,
                    "service_tickets",
                    &ticket_id,
                    ChangeOp::Insert,
                    &payload,
                    DataPriority::High,
                );
                self.intake_customer.clear();
                self.intake_phone.clear();
                self.intake_device.clear();
                self.intake_problem.clear();
                self.sync_controller.trigger_immediate();
                Ok(ticket_id)
            }
            Err(e) => Err(format!("Σφάλμα SQLite: {}", e)),
        }
    }

    pub fn sync_outbox_to_lan(&mut self) -> Result<usize, String> {
        let r_params = self.relay_params();
        let res = crate::sync_daemon::perform_resilient_sync(
            &self.conn,
            &self.lan_host,
            self.lan_port,
            self.auth_token.as_deref(),
            r_params.as_ref(),
        );
        match &res {
            Ok((cnt, transport)) => self.sync_controller.record_success(*cnt, *transport),
            Err(e) => self.sync_controller.record_failure(e.clone()),
        }
        res.map(|(cnt, _)| cnt)
    }

    pub fn auto_sync_tick(&mut self) {
        let r_params = self.relay_params();
        if let Some(res) = crate::sync_daemon::step_resilient_auto_sync(
            &self.conn,
            &mut self.sync_controller,
            &self.lan_host,
            self.lan_port,
            self.auth_token.as_deref(),
            r_params.as_ref(),
        ) {
            match res {
                Ok((cnt, transport)) if cnt > 0 => {
                    let transport_label = match transport {
                        crate::sync_daemon::SyncTransport::LanDirect => "LAN",
                        crate::sync_daemon::SyncTransport::CloudRelay => "Cloud Relay",
                    };
                    self.status_message = Some((
                        format!("✓ Συγχρονίστηκαν {} εγγραφές μέσω {}!", cnt, transport_label),
                        true,
                    ));
                }
                Err(e) => {
                    self.status_message = Some((format!("📶 Σφάλμα συγχρονισμού: {}", e), false));
                }
                _ => {}
            }
        }
    }
}

/// Renders the mobile PDS client touch UI.
pub fn render_mobile_view(ui: &mut Ui, state: &mut MobileAppState) {
    // 0. Auto-sync periodic tick
    state.auto_sync_tick();

    // 1. Mobile Top Status Bar & Branding (Safe Area Top)
    Frame::new()
        .fill(BG_PANEL)
        .stroke(Stroke::new(1.0, BORDER_LINE))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_mobile_logo_widget(ui, Vec2::new(24.0, 24.0));
                ui.add_space(6.0);
                ui.label(
                    RichText::new(&state.shop_config.shop_name)
                        .font(FontId::proportional(15.0))
                        .color(TEXT_TITLE),
                );
                let pending = fetch_pending_outbox(&state.conn, 100).map(|v| v.len()).unwrap_or(0);
                let now = crate::sync_daemon::current_epoch_secs();
                let status_info = state.sync_controller.status_display(pending, now);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    match status_info {
                        crate::sync_daemon::SyncStatusInfo::Synced { transport } => {
                            let (icon, label, color) = match transport {
                                crate::sync_daemon::SyncTransport::LanDirect => {
                                    ("🟢", "LAN Synced", Color32::from_rgb(56, 161, 105))
                                }
                                crate::sync_daemon::SyncTransport::CloudRelay => {
                                    ("🌐", "Relay Synced", Color32::from_rgb(129, 140, 248))
                                }
                            };
                            ui.label(
                                RichText::new(format!("{} {}", icon, label))
                                    .font(FontId::proportional(11.0))
                                    .color(color),
                            );
                        }
                        crate::sync_daemon::SyncStatusInfo::Syncing => {
                            ui.label(
                                RichText::new("🔄 Syncing...")
                                    .font(FontId::proportional(11.0))
                                    .color(Color32::from_rgb(56, 189, 248)),
                            );
                        }
                        crate::sync_daemon::SyncStatusInfo::Pending(cnt) => {
                            let sync_text = format!("⚡ Sync ({})", cnt);
                            if ui.button(RichText::new(sync_text).font(FontId::proportional(11.0)).color(ACCENT_GOLD)).clicked() {
                                let _ = state.sync_outbox_to_lan();
                            }
                        }
                        crate::sync_daemon::SyncStatusInfo::Offline { retrying_in_secs } => {
                            let retry_text = format!("📶 Offline ({}s)", retrying_in_secs);
                            if ui.button(RichText::new(retry_text).font(FontId::proportional(10.5)).color(TEXT_BODY)).clicked() {
                                state.sync_controller.trigger_immediate();
                                let _ = state.sync_outbox_to_lan();
                            }
                        }
                    }
                });
            });
        });

    ui.add_space(8.0);

    // 2. Feedback Banner
    if let Some((msg, is_ok)) = &state.status_message {
        let col = if *is_ok { Color32::from_rgb(56, 161, 105) } else { Color32::from_rgb(229, 62, 62) };
        Frame::new().fill(BG_PANEL).inner_margin(Margin::same(8)).corner_radius(CornerRadius::same(4)).show(ui, |ui| {
            ui.label(RichText::new(msg).color(col).font(FontId::proportional(12.0)));
        });
        ui.add_space(6.0);
    }

    // 3. Central Content Area
    let avail_height = ui.available_height() - 56.0; // Reserve space for bottom touch bar
    egui::ScrollArea::vertical()
        .max_height(avail_height)
        .show(ui, |ui| {
            match state.active_tab {
                MobileTab::Tickets => crate::tickets_tab::render_tickets_tab(ui, state),
                MobileTab::NewIntake => crate::intake_tab::render_intake_tab(ui, state),
                MobileTab::VanSales => crate::van_sales::render_van_sales_tab(ui, state),
                MobileTab::Scanner => crate::scanner::render_scanner_tab(ui, state),
                MobileTab::Profile => crate::profile::render_profile_tab(ui, state),
            }
        });

    // 4. Bottom Touch Bar (Apple HIG / Android Material 44pt Touch Targets)
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        Frame::new()
            .fill(BG_PANEL)
            .stroke(Stroke::new(1.0, BORDER_LINE))
            .inner_margin(Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let tabs = [
                        (MobileTab::Tickets, "📋 Εντολές"),
                        (MobileTab::NewIntake, "➕ Νέα"),
                        (MobileTab::VanSales, "🚚 Van"),
                        (MobileTab::Scanner, "🔍 Scan"),
                        (MobileTab::Profile, "🏪 Store"),
                    ];
                    let btn_width = (ui.available_width() - 32.0) / 5.0;
                    for (tab, label) in tabs {
                        let is_active = state.active_tab == tab;
                        let btn = ui.add_sized(
                            Vec2::new(btn_width, MIN_TOUCH_TARGET),
                            egui::Button::new(
                                RichText::new(label)
                                    .font(FontId::proportional(12.0))
                                    .color(if is_active { TEXT_TITLE } else { TEXT_BODY }),
                            )
                            .fill(if is_active { BG_CARD } else { Color32::TRANSPARENT })
                            .corner_radius(CornerRadius::same(6)),
                        );
                        if btn.clicked() {
                            state.active_tab = tab;
                            state.status_message = None;
                        }
                    }
                });
            });
    });
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_touch_minimum_target() {
        assert_eq!(MIN_TOUCH_TARGET, 44.0);
    }

    #[test]
    fn test_mobile_state_initialization() {
        let state = MobileAppState::default();
        assert_eq!(state.active_tab, MobileTab::Tickets);
        assert!(!state.shop_config.shop_name.is_empty());
    }

    #[test]
    fn test_mobile_ticket_creation() {
        let mut state = MobileAppState::default();
        state.intake_customer = "Test Customer".into();
        state.intake_device = "iPhone 15".into();
        let res = state.submit_ticket();
        assert!(res.is_ok());

        // Verify outbox queue persistence
        let pending = fetch_pending_outbox(&state.conn, 10).unwrap();
        assert!(!pending.is_empty());
        assert_eq!(pending.last().unwrap().entity, "service_tickets");
    }
}
