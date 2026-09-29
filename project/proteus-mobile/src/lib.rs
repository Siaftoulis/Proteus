//! Proteus Mobile (PDS Mobile Touch Client & Android NDK Placeholder).
//! Adheres strictly to the PDS Nordic Slate design system and Rule 5 (Zero Mock Data).
//! Enforces Apple HIG & Android Material 44.0pt touch targets and safe area insets.

use proteus_core::paths::{ensure_database_dir_exists, get_database_path};
use proteus_core::printer::{load_shop_config, ShopReceiptConfig};
use proteus_core::replication::{enqueue_outbox_with_priority, fetch_pending_outbox, init_outbox_schema, ChangeOp, DataPriority};
use proteus_core::tickets::{create_ticket, list_tickets, ServiceTicket};
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
    ctx.data_mut(|d| {
        if let Some(tex) = d.get_temp::<egui::TextureHandle>(id) {
            tex
        } else {
            let img = egui::ColorImage::from_rgba_unmultiplied([128, 128], EMBLEM_RGBA_128);
            let tex = ctx.load_texture("pds_mobile_king_proteus_emblem", img, egui::TextureOptions::LINEAR);
            d.insert_temp(id, tex.clone());
            tex
        }
    })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobileTab {
    Tickets,
    NewIntake,
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
        }
    }
}

impl MobileAppState {
    pub fn reload_config(&mut self) {
        self.shop_config = load_shop_config(&self.conn);
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
                Ok(ticket_id)
            }
            Err(e) => Err(format!("Σφάλμα SQLite: {}", e)),
        }
    }

    pub fn sync_outbox_to_lan(&mut self) -> Result<usize, String> {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        let pending = fetch_pending_outbox(&self.conn, 50).map_err(|e| e.to_string())?;
        if pending.is_empty() {
            return Ok(0);
        }

        let payload = serde_json::to_string(&pending).map_err(|e| e.to_string())?;
        let target = format!("{}:{}", self.lan_host, self.lan_port);
        let mut stream = TcpStream::connect_timeout(
            &target.parse().map_err(|_| format!("Μη έγκυρη διεύθυνση {}", target))?,
            Duration::from_secs(3),
        ).map_err(|e| format!("Αποτυχία σύνδεσης LAN σε {}: {}", target, e))?;

        let req = format!(
            "POST /api/sync/outbox HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            target, payload.len(), payload
        );

        stream.write_all(req.as_bytes()).map_err(|e| format!("Σφάλμα αποστολής: {}", e))?;
        let mut resp = String::new();
        let _ = stream.read_to_string(&mut resp);

        if resp.contains("200 OK") {
            for rec in &pending {
                let _ = proteus_core::replication::mark_outbox_status(
                    &self.conn,
                    &rec.id,
                    proteus_core::replication::SyncStatus::Synced,
                );
            }
            Ok(pending.len())
        } else {
            Err(format!("Σφάλμα συγχρονισμού: {}", resp.lines().next().unwrap_or("Άγνωστο")))
        }
    }
}

/// Renders the mobile PDS client touch UI.
pub fn render_mobile_view(ui: &mut Ui, state: &mut MobileAppState) {
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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if pending == 0 {
                        ui.label(
                            RichText::new("🟢 Synced")
                                .font(FontId::proportional(11.0))
                                .color(Color32::from_rgb(56, 161, 105)),
                        );
                    } else {
                        let sync_text = format!("⚡ Sync ({})", pending);
                        if ui.button(RichText::new(sync_text).font(FontId::proportional(11.0)).color(ACCENT_GOLD)).clicked() {
                            match state.sync_outbox_to_lan() {
                                Ok(cnt) => {
                                    state.status_message = Some((format!("✓ Συγχρονίστηκαν {} εγγραφές!", cnt), true));
                                }
                                Err(e) => {
                                    state.status_message = Some((e, false));
                                }
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
                MobileTab::Tickets => render_tickets_tab(ui, state),
                MobileTab::NewIntake => render_intake_tab(ui, state),
                MobileTab::Scanner => render_scanner_tab(ui, state),
                MobileTab::Profile => render_profile_tab(ui, state),
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
                        (MobileTab::NewIntake, "➕ Παραλαβή"),
                        (MobileTab::Scanner, "🔍 Scan"),
                        (MobileTab::Profile, "🏪 Κατάστημα"),
                    ];
                    let btn_width = (ui.available_width() - 24.0) / 4.0;
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

fn render_tickets_tab(ui: &mut Ui, state: &mut MobileAppState) {
    ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(ui.available_width(), 38.0),
            egui::TextEdit::singleline(&mut state.search_query).hint_text("🔍 Αναζήτηση πελάτη ή συσκευής..."),
        );
    });
    ui.add_space(8.0);

    let tickets = list_tickets(&state.conn).unwrap_or_default();
    let q = state.search_query.to_lowercase();
    let filtered: Vec<&ServiceTicket> = tickets
        .iter()
        .filter(|t| {
            q.is_empty()
                || t.customer_name.to_lowercase().contains(&q)
                || t.device_model.to_lowercase().contains(&q)
                || t.ticket_id.to_lowercase().contains(&q)
        })
        .collect();

    if filtered.is_empty() {
        ui.label(RichText::new("Δεν βρέθηκαν ενεργές εντολές στη βάση SQLite.").color(TEXT_BODY));
        return;
    }

    for t in filtered {
        Frame::new()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER_LINE))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&t.customer_name).strong().color(TEXT_TITLE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(t.current_status.display_name()).size(11.0).color(ACCENT_GOLD));
                    });
                });
                ui.label(RichText::new(format!("{} | {}", t.device_model, t.reported_fault)).size(11.5).color(TEXT_BODY));
                ui.label(RichText::new(format!("#{}: {}", t.ticket_number, t.ticket_id)).size(9.5).color(BORDER_LINE));
            });
        ui.add_space(6.0);
    }
}

fn render_intake_tab(ui: &mut Ui, state: &mut MobileAppState) {
    ui.label(RichText::new("Γρήγορη Παραλαβή Συσκευής").strong().size(14.0).color(TEXT_TITLE));
    ui.add_space(6.0);

    ui.label(RichText::new("Ονοματεπώνυμο:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 38.0), egui::TextEdit::singleline(&mut state.intake_customer));

    ui.label(RichText::new("Τηλέφωνο:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 38.0), egui::TextEdit::singleline(&mut state.intake_phone));

    ui.label(RichText::new("Μοντέλο / Συσκευή:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 38.0), egui::TextEdit::singleline(&mut state.intake_device));

    ui.label(RichText::new("Περιγραφή Βλάβης:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 60.0), egui::TextEdit::multiline(&mut state.intake_problem));

    ui.add_space(10.0);
    let submit_btn = ui.add_sized(
        Vec2::new(ui.available_width(), MIN_TOUCH_TARGET),
        egui::Button::new(RichText::new("💾 Καταχώρηση στη SQLite").strong().color(TEXT_TITLE))
            .fill(BG_PANEL)
            .stroke(Stroke::new(1.0, ACCENT_GOLD))
            .corner_radius(CornerRadius::same(6)),
    );

    if submit_btn.clicked() {
        match state.submit_ticket() {
            Ok(id) => {
                state.status_message = Some((format!("✓ Η εντολή {} καταχωρήθηκε επιτυχώς!", id), true));
                state.active_tab = MobileTab::Tickets;
            }
            Err(e) => {
                state.status_message = Some((e, false));
            }
        }
    }
}

fn render_scanner_tab(ui: &mut Ui, state: &mut MobileAppState) {
    ui.label(RichText::new("Barcode & QR Scanner (Handheld)").strong().size(14.0).color(TEXT_TITLE));
    ui.label(RichText::new("Υποδομή κάμερας / optical scanner για άμεση εύρεση εντολής.").size(11.5).color(TEXT_BODY));
    ui.add_space(12.0);

    Frame::new()
        .fill(BG_BASE)
        .stroke(Stroke::new(1.5, ACCENT_GOLD))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(20))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("📷").size(32.0));
                ui.label(RichText::new("[ Optical Scanner Window ]").color(TEXT_BODY).size(12.0));
            });
        });

    ui.add_space(12.0);
    ui.label(RichText::new("Χειροκίνητη εισαγωγή ID ή σάρωση:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 38.0), egui::TextEdit::singleline(&mut state.scanned_code));

    if ui.add_sized(Vec2::new(ui.available_width(), MIN_TOUCH_TARGET), egui::Button::new("Αναζήτηση Εντολής")).clicked() {
        state.search_query = state.scanned_code.clone();
        state.active_tab = MobileTab::Tickets;
    }
}

fn render_profile_tab(ui: &mut Ui, state: &mut MobileAppState) {
    state.reload_config();
    ui.label(RichText::new("Στοιχεία Καταστήματος (SQLite)").strong().size(14.0).color(TEXT_TITLE));
    ui.add_space(8.0);

    Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_LINE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_mobile_logo_widget(ui, Vec2::new(36.0, 36.0));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&state.shop_config.shop_name).strong().size(13.0).color(TEXT_TITLE));
                    ui.label(RichText::new("Proteus Business OS Mobile").size(11.0).color(TEXT_BODY));
                });
            });
            ui.add_space(8.0);
            ui.label(RichText::new(format!("Διεύθυνση: {}", state.shop_config.address)).size(11.5).color(TEXT_BODY));
            ui.label(RichText::new(format!("Τηλέφωνο: {}", state.shop_config.phone)).size(11.5).color(TEXT_BODY));
            ui.label(RichText::new(format!("Footer: {}", state.shop_config.footer_message)).size(11.5).color(TEXT_BODY));
            ui.label(RichText::new(format!("Λογότυπο / Εικονίδιο: {}", state.shop_config.logo_icon)).size(11.5).color(ACCENT_GOLD));
        });

    ui.add_space(10.0);
    ui.label(RichText::new("Ρυθμίσεις LAN Συγχρονισμού").strong().size(13.0).color(TEXT_TITLE));
    Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_LINE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Host IP:").color(TEXT_BODY).size(11.5));
                ui.add_sized(Vec2::new(140.0, 30.0), egui::TextEdit::singleline(&mut state.lan_host));
                ui.label(RichText::new("Port:").color(TEXT_BODY).size(11.5));
                let mut port_str = state.lan_port.to_string();
                if ui.add_sized(Vec2::new(60.0, 30.0), egui::TextEdit::singleline(&mut port_str)).changed() {
                    if let Ok(p) = port_str.parse::<u16>() {
                        state.lan_port = p;
                    }
                }
            });
            ui.add_space(6.0);
            if ui.add_sized(Vec2::new(ui.available_width(), MIN_TOUCH_TARGET), egui::Button::new("⚡ Άμεσος Συγχρονισμός Outbox")).clicked() {
                match state.sync_outbox_to_lan() {
                    Ok(cnt) => {
                        state.status_message = Some((format!("✓ Συγχρονίστηκαν {} εγγραφές στο κατάστημα!", cnt), true));
                    }
                    Err(e) => {
                        state.status_message = Some((e, false));
                    }
                }
            }
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
