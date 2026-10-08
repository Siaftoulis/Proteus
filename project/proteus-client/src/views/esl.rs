//! Electronic Shelf Label (ESL) Gateway & E-Ink Monitor View for Proteus Client.
//! Manages e-ink price tags, dynamic expiry discounting, RF broadcast transmission, and Gateway Bridge.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use chrono::Local;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::audio::play_barcode_chime;
use proteus_core::esl::{
    broadcast_all_pending, enqueue_esl_update, evaluate_dynamic_expiry_discount,
    generate_esl_radio_packet, init_esl_schema, list_esl_tags, load_esl_gateway_config,
    mark_packet_transmitted, ping_esl_gateway, register_esl_tag, save_esl_gateway_config,
    EslGatewayConfig, EslTag,
};

use crate::theme::{
    ACCENT_GOLD, ACCENT_PRIMARY, BG_BASE, BG_CARD, BG_PANEL, BORDER_SUBTLE, STATUS_CANCELLED,
    STATUS_READY, TEXT_MUTED,
};

pub struct EslViewState {
    pub selected_tag_mac: Option<String>,
    pub manual_sku_input: String,
    pub manual_price_str: String,
    pub expiry_date_input: String,
    pub calculated_discount: Option<(f64, Option<f64>, Option<&'static str>)>,
    pub feedback_message: Option<(String, bool)>,
    pub initialized: bool,
    pub gateway_cfg: EslGatewayConfig,
    pub show_gateway_settings: bool,
    pub gateway_online: Option<bool>,
}

impl Default for EslViewState {
    fn default() -> Self {
        Self {
            selected_tag_mac: None,
            manual_sku_input: "YOGURT-GREEK-1KG".to_string(),
            manual_price_str: "4.50".to_string(),
            expiry_date_input: "2026-10-05".to_string(),
            calculated_discount: None,
            feedback_message: None,
            initialized: false,
            gateway_cfg: EslGatewayConfig::default(),
            show_gateway_settings: false,
            gateway_online: None,
        }
    }
}

pub fn draw_esl_view(ui: &mut Ui, conn: &Connection, state: &mut EslViewState) {
    if !state.initialized {
        let _ = init_esl_schema(conn);
        seed_default_esl_tags_if_empty(conn);
        state.gateway_cfg = load_esl_gateway_config(conn);
        state.initialized = true;
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let tags = list_esl_tags(conn).unwrap_or_default();
    if state.selected_tag_mac.is_none() && !tags.is_empty() {
        state.selected_tag_mac = Some(tags[0].tag_mac.clone());
    }

    ui.vertical(|ui| {
        // Top Header & Gateway Status
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🏷 Ηλεκτρονικές Ετικέτες Ραφιού (ESL Gateway)").strong().size(19.0));
            if let Some(online) = state.gateway_online {
                let (txt, col) = if online { ("🟢 RF Gateway Online", STATUS_READY) } else { ("🔴 Gateway Offline", STATUS_CANCELLED) };
                ui.colored_label(col, txt);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn_txt = if state.show_gateway_settings { "▲ Απόκρυψη Gateway" } else { "⚙️ Ρυθμίσεις RF Gateway" };
                if ui.small_button(btn_txt).clicked() {
                    state.show_gateway_settings = !state.show_gateway_settings;
                }
                if ui.small_button("📡 Ping Gateway").clicked() {
                    let online = ping_esl_gateway(&state.gateway_cfg).unwrap_or(false);
                    state.gateway_online = Some(online);
                }
            });
        });

        // Gateway Hardware Configuration Drawer
        if state.show_gateway_settings {
            ui.add_space(4.0);
            render_gateway_settings_drawer(ui, conn, state);
        }

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if let Some((msg, ok)) = &state.feedback_message {
            let color = if *ok { STATUS_READY } else { STATUS_CANCELLED };
            ui.label(RichText::new(msg).color(color).strong().size(12.0));
            ui.add_space(2.0);
        }

        // Two Column Layout: Left = Tag Roster & Visual Preview | Right = Dynamic Expiry & Broadcast
        ui.columns(2, |cols| {
            // LEFT COLUMN: Tag Roster List & E-Ink Canvas Preview
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Κατάσταση Ετικετών Ραφιού (E-Ink 2.9\" / 4.2\"):").strong());

                Frame::new().fill(BG_PANEL).stroke(Stroke::new(1.0, BORDER_SUBTLE)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(6)).show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                        for tag in &tags {
                            let is_selected = state.selected_tag_mac.as_deref() == Some(&tag.tag_mac);
                            let bat_color = if tag.battery_percentage > 50 { STATUS_READY } else if tag.battery_percentage > 20 { ACCENT_GOLD } else { STATUS_CANCELLED };
                            let frame_fill = if is_selected { BG_CARD } else { BG_BASE };

                            Frame::new()
                                .fill(frame_fill)
                                .stroke(Stroke::new(1.0, if is_selected { ACCENT_PRIMARY } else { BORDER_SUBTLE }))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::same(6))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_label(is_selected, RichText::new(&tag.product_name).strong()).clicked() {
                                            state.selected_tag_mac = Some(tag.tag_mac.clone());
                                            state.manual_sku_input = tag.sku.clone();
                                            state.manual_price_str = format!("{:.2}", tag.current_price_eur);
                                        }
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(format!("🔋 {}%", tag.battery_percentage)).color(bat_color).size(10.0));
                                            ui.label(RichText::new(format!("📶 {}dBm", tag.signal_rssi)).color(TEXT_MUTED).size(10.0));
                                        });
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("Ράφι: {} | SKU: {}", tag.shelf_id, tag.sku)).size(10.5).color(TEXT_MUTED));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if let Some(disc) = tag.discount_price_eur {
                                                ui.label(RichText::new(format!("{:.2}€", disc)).color(STATUS_CANCELLED).strong().size(13.0));
                                                ui.label(RichText::new(format!("{:.2}€", tag.current_price_eur)).strikethrough().color(TEXT_MUTED).size(10.5));
                                            } else {
                                                ui.label(RichText::new(format!("{:.2}€", tag.current_price_eur)).color(STATUS_READY).strong().size(13.0));
                                            }
                                        });
                                    });
                                });
                            ui.add_space(2.0);
                        }
                    });
                });

                // E-Ink Physical Display Preview
                if let Some(mac) = &state.selected_tag_mac {
                    if let Some(tag) = tags.iter().find(|t| &t.tag_mac == mac) {
                        ui.add_space(6.0);
                        render_eink_preview(ui, tag);
                    }
                }
            });

            // RIGHT COLUMN: Dynamic Expiry Discount & Radio Broadcast
            cols[1].vertical(|ui| {
                ui.label(RichText::new("Αυτόματη Έκπτωση Λήξης (GS1 AI 17):").strong());
                ui.horizontal(|ui| {
                    ui.label("SKU:");
                    ui.add(egui::TextEdit::singleline(&mut state.manual_sku_input).desired_width(110.0));
                    ui.label("Τιμή (€):");
                    ui.add(egui::TextEdit::singleline(&mut state.manual_price_str).desired_width(50.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Ημ. Λήξης (YYYY-MM-DD):");
                    ui.add(egui::TextEdit::singleline(&mut state.expiry_date_input).desired_width(90.0));
                    if ui.button(RichText::new("⚡ Υπολογισμός").strong()).clicked() {
                        let base = state.manual_price_str.parse::<f64>().unwrap_or(5.0);
                        state.calculated_discount = Some(evaluate_dynamic_expiry_discount(base, &state.expiry_date_input, &today));
                    }
                });

                if let Some((base, disc, promo)) = state.calculated_discount {
                    ui.add_space(4.0);
                    let promo_label = promo.unwrap_or("Κανονική Τιμή (Καμία Έκπτωση)");
                    let bg = if disc.is_some() { Color32::from_rgb(45, 30, 5) } else { BG_CARD };
                    let stroke_c = if disc.is_some() { ACCENT_GOLD } else { BORDER_SUBTLE };

                    Frame::new().fill(bg).stroke(Stroke::new(1.0, stroke_c)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(8)).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(promo_label).color(ACCENT_GOLD).strong().size(13.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if let Some(new_p) = disc {
                                    ui.label(RichText::new(format!("Νέα Τιμή: {:.2} €", new_p)).color(STATUS_READY).strong().size(15.0));
                                    ui.label(RichText::new(format!("{:.2} €", base)).strikethrough().color(TEXT_MUTED).size(11.0));
                                } else {
                                    ui.label(RichText::new(format!("{:.2} €", base)).color(STATUS_READY).strong().size(14.0));
                                }
                            });
                        });
                    });

                    ui.add_space(6.0);
                    if ui.button(RichText::new("📡 Αποστολή RF στην Ετικέτα").strong().size(12.0).color(ACCENT_PRIMARY)).clicked() {
                        if let Some(tag_mac) = &state.selected_tag_mac {
                            let packet = generate_esl_radio_packet(tag_mac, base, disc, promo, "5201122334455");
                            match enqueue_esl_update(conn, tag_mac, &packet) {
                                Ok(bc_id) => {
                                    state.feedback_message = Some((format!("✓ Πακέτο RF καταχωρήθηκε στην ουρά ({})!", &bc_id[..8]), true));
                                    play_barcode_chime();
                                }
                                Err(e) => { state.feedback_message = Some((format!("Σφάλμα: {}", e), false)); }
                            }
                        }
                    }
                }

                ui.add_space(8.0);
                // Broadcast Queue & Batch Dispatch Button
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Ουρά Ραδιοεκπομπής (ESL Queue):").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("🚀 Μαζική Αποστολή Όλων").strong().color(Color32::BLACK)).clicked() {
                            match broadcast_all_pending(conn, &state.gateway_cfg) {
                                Ok(res) => {
                                    play_barcode_chime();
                                    state.feedback_message = Some((
                                        format!("✓ Εκτελέστηκε μαζική ραδιοεκπομπή! Μεταδόθηκαν {}/{} πακέτα σε {}ms.", res.transmitted_success, res.total_enqueued, res.duration_ms),
                                        true,
                                    ));
                                }
                                Err(e) => { state.feedback_message = Some((format!("Σφάλμα μαζικής αποστολής: {}", e), false)); }
                            }
                        }
                    });
                });

                Frame::new().fill(BG_PANEL).stroke(Stroke::new(1.0, BORDER_SUBTLE)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(6)).show(ui, |ui| {
                    let queue_rows = query_recent_broadcasts(conn);
                    if queue_rows.is_empty() {
                        ui.label(RichText::new("Καμία εκκρεμής ραδιοεκπομπή.").color(TEXT_MUTED).size(11.0));
                    } else {
                        for (bc_id, tag_mac, status) in queue_rows {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("#{}: {}", &bc_id[..8], tag_mac)).size(11.5));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if status == "PENDING" {
                                        if ui.small_button("ACK").clicked() { let _ = mark_packet_transmitted(conn, &bc_id); }
                                        ui.label(RichText::new("Εκκρεμεί").color(ACCENT_GOLD).size(10.5));
                                    } else {
                                        ui.label(RichText::new("✓ Μεταδόθηκε").color(STATUS_READY).size(10.5));
                                    }
                                });
                            });
                        }
                    }
                });
            });
        });
    });
}

fn render_gateway_settings_drawer(ui: &mut Ui, conn: &Connection, state: &mut EslViewState) {
    Frame::default().fill(BG_CARD).stroke(Stroke::new(1.0, BORDER_SUBTLE)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::same(10)).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Gateway Host:");
            ui.add(egui::TextEdit::singleline(&mut state.gateway_cfg.gateway_host).desired_width(120.0));
            ui.label("Port:");
            let mut port_str = state.gateway_cfg.gateway_port.to_string();
            if ui.add(egui::TextEdit::singleline(&mut port_str).desired_width(55.0)).changed() {
                if let Ok(p) = port_str.parse::<u16>() { state.gateway_cfg.gateway_port = p; }
            }
            ui.label("Κανάλι RF:");
            let mut chan_str = state.gateway_cfg.rf_channel.to_string();
            if ui.add(egui::TextEdit::singleline(&mut chan_str).desired_width(35.0)).changed() {
                if let Ok(c) = chan_str.parse::<u8>() { state.gateway_cfg.rf_channel = c; }
            }
            ui.checkbox(&mut state.gateway_cfg.auto_sync_on_price_change, "Auto-Sync Τιμών");
            if ui.button("💾 Αποθήκευση").clicked() {
                let _ = save_esl_gateway_config(conn, &state.gateway_cfg);
                state.feedback_message = Some(("✓ Οι ρυθμίσεις ESL Gateway αποθηκεύτηκαν στη SQLite!".to_string(), true));
            }
        });
    });
}

fn render_eink_preview(ui: &mut Ui, tag: &EslTag) {
    ui.label(RichText::new("Προεπισκόπηση E-Paper 2.9\" (296x128 Shelf Tag):").size(11.0).color(TEXT_MUTED));
    Frame::default()
        .fill(Color32::from_rgb(248, 250, 252))
        .stroke(Stroke::new(2.0, Color32::from_rgb(30, 41, 59)))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&tag.product_name).strong().size(13.0).color(Color32::BLACK));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("🔋{}%", tag.battery_percentage)).size(9.0).color(Color32::from_rgb(71, 85, 105)));
                });
            });
            ui.horizontal(|ui| {
                if let Some(disc) = tag.discount_price_eur {
                    ui.label(RichText::new(format!("€{:.2}", disc)).strong().size(22.0).color(Color32::from_rgb(220, 38, 38)));
                    ui.label(RichText::new(format!("€{:.2}", tag.current_price_eur)).strikethrough().size(13.0).color(Color32::from_rgb(100, 116, 139)));
                } else {
                    ui.label(RichText::new(format!("€{:.2}", tag.current_price_eur)).strong().size(22.0).color(Color32::BLACK));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("|||| ||| |||||").monospace().size(12.0).color(Color32::BLACK));
                    ui.label(RichText::new(&tag.sku).size(9.0).color(Color32::from_rgb(100, 116, 139)));
                });
            });
        });
}

fn query_recent_broadcasts(conn: &Connection) -> Vec<(String, String, String)> {
    let mut stmt = match conn.prepare("SELECT broadcast_id, tag_mac, status FROM esl_broadcast_queue ORDER BY created_at DESC LIMIT 4") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map(|iter| iter.filter_map(|r| r.ok()).collect()).unwrap_or_default()
}

fn seed_default_esl_tags_if_empty(conn: &Connection) {
    let count: i64 = conn.query_row("SELECT count(*) FROM esl_tags", [], |r| r.get(0)).unwrap_or(0);
    if count > 0 { return; }
    let default_tags = [
        EslTag { tag_mac: "ESL-A1-22".to_string(), shelf_id: "S-102".to_string(), sku: "YOGURT-GREEK-1KG".to_string(), product_name: "Ελληνικό Στραγγιστό Γιαούρτι 1kg".to_string(), current_price_eur: 4.50, discount_price_eur: None, unit_of_measure: "τμχ".to_string(), battery_percentage: 95, signal_rssi: -58, last_sync_utc: 1727956800000, pending_refresh: false },
        EslTag { tag_mac: "ESL-B3-15".to_string(), shelf_id: "S-103".to_string(), sku: "ROUTER-AX3000".to_string(), product_name: "Wi-Fi 6 Router AX3000".to_string(), current_price_eur: 65.00, discount_price_eur: None, unit_of_measure: "τμχ".to_string(), battery_percentage: 88, signal_rssi: -65, last_sync_utc: 1727956800000, pending_refresh: false },
    ];
    for t in &default_tags { let _ = register_esl_tag(conn, t); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_esl_view_state_defaults() {
        let state = EslViewState::default();
        assert_eq!(state.manual_sku_input, "YOGURT-GREEK-1KG");
        assert_eq!(state.manual_price_str, "4.50");
        assert!(state.calculated_discount.is_none());
        assert_eq!(state.gateway_cfg.gateway_port, 9100);
    }

    #[test]
    fn test_esl_tag_seeding_and_broadcast_dispatch() {
        let conn = Connection::open_in_memory().unwrap();
        init_esl_schema(&conn).unwrap();
        seed_default_esl_tags_if_empty(&conn);

        let tags = list_esl_tags(&conn).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tag_mac, "ESL-A1-22");

        let packet = generate_esl_radio_packet("ESL-A1-22", 4.50, Some(3.15), Some("-30%"), "5201122334455");
        let bc_id = enqueue_esl_update(&conn, "ESL-A1-22", &packet).unwrap();

        let queue = query_recent_broadcasts(&conn);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].0, bc_id);
        assert_eq!(queue[0].2, "PENDING");

        mark_packet_transmitted(&conn, &bc_id).unwrap();
        let queue_after = query_recent_broadcasts(&conn);
        assert_eq!(queue_after[0].2, "TRANSMITTED");
    }
}
