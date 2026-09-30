//! Screen: Cold Chain HACCP Telemetry & IoT Sensor Monitor for Proteus Client.
//! Real-time temperature & humidity tracking for refrigerated transport and cold rooms.
//! Automated breach detection, operator corrective actions, and Merkle audit certification.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::Utc;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::cold_chain::{
    generate_haccp_certificate, ingest_telemetry_reading, init_cold_chain_schema,
    list_active_breaches, list_cold_sensors, list_recent_readings, register_cold_sensor,
    resolve_breach_event, BreachSeverity, ColdChainSensor, ColdStorageType,
    HaccpComplianceCertificate, TelemetryReading,
};

use crate::theme::{
    ACCENT_GOLD, BG_CARD, BG_PANEL, BORDER_SUBTLE, STATUS_CANCELLED, STATUS_READY,
    TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY,
};

pub struct ColdChainViewState {
    pub selected_target_filter: String,
    pub show_register_modal: bool,
    pub new_sensor_id: String,
    pub new_target_id: String,
    pub new_storage_type: ColdStorageType,

    pub show_ingest_modal: bool,
    pub ingest_sensor_id: String,
    pub ingest_temp_str: String,
    pub ingest_hum_str: String,
    pub ingest_door_open: bool,

    pub show_resolve_modal: bool,
    pub resolve_breach_id: String,
    pub resolve_note: String,

    pub certificate_modal: Option<HaccpComplianceCertificate>,
    pub feedback_message: Option<(String, bool)>,
}

impl Default for ColdChainViewState {
    fn default() -> Self {
        Self {
            selected_target_filter: "ALL".to_string(),
            show_register_modal: false,
            new_sensor_id: String::new(),
            new_target_id: String::new(),
            new_storage_type: ColdStorageType::Chilled,

            show_ingest_modal: false,
            ingest_sensor_id: String::new(),
            ingest_temp_str: "3.5".to_string(),
            ingest_hum_str: "75.0".to_string(),
            ingest_door_open: false,

            show_resolve_modal: false,
            resolve_breach_id: String::new(),
            resolve_note: String::new(),

            certificate_modal: None,
            feedback_message: None,
        }
    }
}

pub fn draw_cold_chain_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut ColdChainViewState,
    operator_name: &str,
) {
    let _ = init_cold_chain_schema(conn);

    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("❄️ Τηλεμετρία Ψυχρής Αλυσίδας & HACCP").strong().size(18.0).color(TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("➕ Νέος Αισθητήρας").color(TEXT_PRIMARY)).clicked() {
                    state.show_register_modal = true;
                }
                if ui.button(RichText::new("📡 Εισαγωγή Τηλεμετρίας").color(TEXT_PRIMARY)).clicked() {
                    state.show_ingest_modal = true;
                }
                if ui.button(RichText::new("📜 Έκδοση Πιστοποιητικού HACCP").color(ACCENT_GOLD)).clicked() {
                    let now = Utc::now().timestamp();
                    let start = now - (7 * 86400); // 7 days window
                    let target = if state.selected_target_filter == "ALL" {
                        "ALL_FACILITIES"
                    } else {
                        &state.selected_target_filter
                    };
                    if let Ok(cert) = generate_haccp_certificate(conn, target, start, now + 60) {
                        state.certificate_modal = Some(cert);
                    }
                }
            });
        });

        ui.label(RichText::new("Συνεχής καταγραφή θερμοκρασιών/υγρασίας ψυκτικών θαλάμων & φορτηγών με κρυπτογραφική σφραγίδα Merkle.").size(12.0).color(TEXT_SECONDARY));
        ui.add_space(8.0);

        // Feedback banner
        if let Some((msg, is_err)) = &state.feedback_message {
            let color = if *is_err { STATUS_CANCELLED } else { STATUS_READY };
            ui.label(RichText::new(msg).color(color).size(12.0));
            ui.add_space(4.0);
        }

        // Active Breaches / Excursions Alert Panel
        let active_breaches = list_active_breaches(conn).unwrap_or_default();
        if !active_breaches.is_empty() {
            egui::Frame::new()
                .fill(Color32::from_rgb(45, 20, 25))
                .stroke(Stroke::new(1.5, STATUS_CANCELLED))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🚨 ΕΝΕΡΓΕΣ ΑΠΟΚΛΙΣΕΙΣ ΘΕΡΜΟΚΡΑΣΙΑΣ (HACCP BREACHES)").strong().color(STATUS_CANCELLED));
                        ui.label(RichText::new(format!("({} ενεργά συμβάντα)", active_breaches.len())).color(TEXT_SECONDARY));
                    });
                    ui.add_space(4.0);

                    for b in &active_breaches {
                        ui.horizontal(|ui| {
                            let sev_color = match b.severity {
                                BreachSeverity::MinorWarning => ACCENT_GOLD,
                                BreachSeverity::MajorExcursion => Color32::from_rgb(239, 68, 68),
                                BreachSeverity::CriticalSpoilage => Color32::from_rgb(220, 38, 38),
                            };
                            ui.label(RichText::new(format!("[{}]", b.severity.display_name())).color(sev_color).strong());
                            ui.label(RichText::new(format!("Στόχος: {} ({})", b.target_id, b.sensor_id)).color(TEXT_PRIMARY));
                            ui.label(RichText::new(format!("{:.1}°C (Όριο: {:.1}°C)", b.excursion_temp, b.threshold_limit)).color(STATUS_CANCELLED).strong());
                            ui.label(RichText::new(format!("Έναρξη: {}", b.started_at)).color(TEXT_MUTED).size(11.0));

                            if ui.button(RichText::new("Καταγραφή Ενέργειας").size(11.0)).clicked() {
                                state.resolve_breach_id = b.breach_id.clone();
                                state.show_resolve_modal = true;
                            }
                        });
                    }
                });
            ui.add_space(10.0);
        }

        // Filter tabs
        let sensors = list_cold_sensors(conn).unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Φίλτρο Στόχου:").color(TEXT_MUTED));
            if ui.selectable_label(state.selected_target_filter == "ALL", "Όλοι οι Στόχοι").clicked() {
                state.selected_target_filter = "ALL".to_string();
            }
            let mut seen_targets = std::collections::HashSet::new();
            for s in &sensors {
                if seen_targets.insert(s.target_id.clone()) {
                    let is_sel = state.selected_target_filter == s.target_id;
                    if ui.selectable_label(is_sel, &s.target_id).clicked() {
                        state.selected_target_filter = s.target_id.clone();
                    }
                }
            }
        });
        ui.add_space(8.0);

        // Sensors Cards Grid
        let filtered_sensors: Vec<&ColdChainSensor> = if state.selected_target_filter == "ALL" {
            sensors.iter().collect()
        } else {
            sensors.iter().filter(|s| s.target_id == state.selected_target_filter).collect()
        };

        if filtered_sensors.is_empty() {
            ui.label(RichText::new("Δεν έχουν καταχωρηθεί αισθητήρες ψυχρής αλυσίδας στη SQLite.").color(TEXT_MUTED));
        } else {
            ui.label(RichText::new("Ενεργές Μονάδες Ψύξης & Αισθητήρες").strong().color(TEXT_PRIMARY));
            ui.add_space(4.0);

            for s in &filtered_sensors {
                let recent = list_recent_readings(conn, Some(&s.target_id), 1).unwrap_or_default();
                let last_reading = recent.first();

                Frame::new()
                    .fill(BG_CARD)
                    .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&s.target_id).strong().size(13.5).color(TEXT_PRIMARY));
                            ui.label(RichText::new(format!("({})", s.sensor_id)).color(TEXT_MUTED).size(11.5));
                            ui.label(RichText::new(s.storage_type.display_name()).color(TEXT_SECONDARY).size(11.5));

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if let Some(r) = last_reading {
                                    let is_breach = r.temperature_celsius > s.max_temp_celsius || r.temperature_celsius < s.min_temp_celsius;
                                    let temp_color = if is_breach { STATUS_CANCELLED } else { STATUS_READY };
                                    ui.label(RichText::new(format!("{:.1}°C", r.temperature_celsius)).strong().size(15.0).color(temp_color));

                                    if let Some(h) = r.humidity_pct {
                                        ui.label(RichText::new(format!("Υγρασία: {:.0}%", h)).size(11.5).color(TEXT_SECONDARY));
                                    }
                                    let door_str = if r.door_open { "🚪 ΑΝΟΙΧΤΗ" } else { "🚪 Κλειστή" };
                                    let door_col = if r.door_open { STATUS_CANCELLED } else { STATUS_READY };
                                    ui.label(RichText::new(door_str).color(door_col).size(11.0));
                                } else {
                                    ui.label(RichText::new("Αναμονή σήματος...").color(TEXT_MUTED).size(12.0));
                                }
                            });
                        });

                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Όρια: {:.1}°C έως {:.1}°C", s.min_temp_celsius, s.max_temp_celsius)).size(11.0).color(TEXT_MUTED));
                            if let Some(r) = last_reading {
                                ui.label(RichText::new(format!("Τελευταία μέτρηση: {}", r.recorded_at)).size(11.0).color(TEXT_MUTED));
                                let short_hash = if r.merkle_hash.len() > 12 { &r.merkle_hash[..12] } else { &r.merkle_hash };
                                ui.label(RichText::new(format!("SHA-256: {}...", short_hash)).size(10.5).color(TEXT_MUTED));
                            }
                        });
                    });
                ui.add_space(4.0);
            }
        }

        ui.add_space(10.0);

        // Recent Telemetry Logs Table
        ui.label(RichText::new("Τελευταία Τηλεμετρικά Πακέτα (Merkle Audit Trail)").strong().color(TEXT_PRIMARY));
        let logs: Vec<TelemetryReading> = list_recent_readings(
            conn,
            if state.selected_target_filter == "ALL" { None } else { Some(&state.selected_target_filter) },
            8,
        ).unwrap_or_default();

        if logs.is_empty() {
            ui.label(RichText::new("Δεν υπάρχουν ακόμη καταγεγραμμένα πακέτα.").size(11.5).color(TEXT_MUTED));
        } else {
            Frame::new()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::same(6))
                .show(ui, |ui| {
                    for l in logs {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&l.recorded_at).size(11.0).color(TEXT_MUTED));
                            ui.label(RichText::new(&l.target_id).strong().size(11.5).color(TEXT_PRIMARY));
                            ui.label(RichText::new(&l.sensor_id).size(11.0).color(TEXT_SECONDARY));
                            ui.label(RichText::new(format!("{:.1}°C", l.temperature_celsius)).strong().size(12.0).color(TEXT_PRIMARY));
                            if let Some(hum) = l.humidity_pct {
                                ui.label(RichText::new(format!("{:.0}% RH", hum)).size(11.0).color(TEXT_MUTED));
                            }
                            let door_txt = if l.door_open { "Ανοιχτή" } else { "Κλειστή" };
                            ui.label(RichText::new(format!("Πόρτα: {}", door_txt)).size(11.0).color(TEXT_MUTED));
                            let short_h = if l.merkle_hash.len() > 10 { &l.merkle_hash[..10] } else { &l.merkle_hash };
                            ui.label(RichText::new(format!("Hash: {}...", short_h)).size(10.5).color(TEXT_MUTED));
                        });
                    }
                });
        }
    });

    // Modals
    draw_sensor_register_modal(ui.ctx(), conn, state);
    draw_ingest_telemetry_modal(ui.ctx(), conn, state);
    draw_resolve_breach_modal(ui.ctx(), conn, state, operator_name);
    draw_haccp_certificate_modal(ui.ctx(), state);
}

fn draw_sensor_register_modal(ctx: &egui::Context, conn: &Connection, state: &mut ColdChainViewState) {
    if !state.show_register_modal {
        return;
    }

    egui::Window::new("➕ Καταχώριση Νέου Αισθητήρα Ψυχρής Αλυσίδας")
        .collapsible(false)
        .resizable(false)
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("ID Αισθητήρα (π.χ. SEN-BLE-01):").size(12.0).color(TEXT_SECONDARY));
            ui.text_edit_singleline(&mut state.new_sensor_id);

            ui.label(RichText::new("Όχημα ή Ψυκτικός Θάλαμος (π.χ. VAN-9988):").size(12.0).color(TEXT_SECONDARY));
            ui.text_edit_singleline(&mut state.new_target_id);

            ui.label(RichText::new("Κατηγορία Ψύξης:").size(12.0).color(TEXT_SECONDARY));
            egui::ComboBox::from_id_salt("reg_storage_type")
                .selected_text(state.new_storage_type.display_name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut state.new_storage_type, ColdStorageType::DeepFreeze, ColdStorageType::DeepFreeze.display_name());
                    ui.selectable_value(&mut state.new_storage_type, ColdStorageType::Chilled, ColdStorageType::Chilled.display_name());
                    ui.selectable_value(&mut state.new_storage_type, ColdStorageType::ControlledAmbient, ColdStorageType::ControlledAmbient.display_name());
                    ui.selectable_value(&mut state.new_storage_type, ColdStorageType::PharmaCold, ColdStorageType::PharmaCold.display_name());
                });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Αποθήκευση").clicked() {
                    if !state.new_sensor_id.is_empty() && !state.new_target_id.is_empty() {
                        let s = ColdChainSensor::new(&state.new_sensor_id, &state.new_target_id, state.new_storage_type);
                        if register_cold_sensor(conn, &s).is_ok() {
                            state.feedback_message = Some((format!("Ο αισθητήρας {} καταχωρήθηκε επιτυχώς.", s.sensor_id), false));
                            state.show_register_modal = false;
                            state.new_sensor_id.clear();
                            state.new_target_id.clear();
                        }
                    }
                }
                if ui.button("Ακύρωση").clicked() {
                    state.show_register_modal = false;
                }
            });
        });
}

fn draw_ingest_telemetry_modal(ctx: &egui::Context, conn: &Connection, state: &mut ColdChainViewState) {
    if !state.show_ingest_modal {
        return;
    }

    egui::Window::new("📡 Εισαγωγή / Προσομοίωση Μέτρησης Τηλεμετρίας")
        .collapsible(false)
        .resizable(false)
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("Επιλογή Αισθητήρα:").size(12.0).color(TEXT_SECONDARY));
            let sensors = list_cold_sensors(conn).unwrap_or_default();
            egui::ComboBox::from_id_salt("ingest_sensor_select")
                .selected_text(if state.ingest_sensor_id.is_empty() { "Επιλέξτε Αισθητήρα" } else { &state.ingest_sensor_id })
                .show_ui(ui, |ui| {
                    for s in &sensors {
                        ui.selectable_value(&mut state.ingest_sensor_id, s.sensor_id.clone(), format!("{} ({})", s.sensor_id, s.target_id));
                    }
                });

            ui.label(RichText::new("Θερμοκρασία (°C):").size(12.0).color(TEXT_SECONDARY));
            ui.text_edit_singleline(&mut state.ingest_temp_str);

            ui.label(RichText::new("Υγρασία (% RH):").size(12.0).color(TEXT_SECONDARY));
            ui.text_edit_singleline(&mut state.ingest_hum_str);

            ui.checkbox(&mut state.ingest_door_open, "Πόρτα Θαλάμου Ανοιχτή");

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Καταγραφή στη SQLite").clicked() {
                    if !state.ingest_sensor_id.is_empty() {
                        let temp: f64 = state.ingest_temp_str.parse().unwrap_or(3.0);
                        let hum: Option<f64> = state.ingest_hum_str.parse().ok();
                        match ingest_telemetry_reading(conn, &state.ingest_sensor_id, temp, hum, state.ingest_door_open, Some(95)) {
                            Ok((_reading, breach_opt)) => {
                                if let Some(b) = breach_opt {
                                    state.feedback_message = Some((format!("⚠️ ΚΑΤΑΓΡΑΦΗΚΕ ΑΠΟΚΛΙΣΗ HACCP: {:.1}°C ({})", b.excursion_temp, b.severity.display_name()), true));
                                } else {
                                    state.feedback_message = Some((format!("Μέτρηση {:.1}°C καταχωρήθηκε κανονικά.", temp), false));
                                }
                                state.show_ingest_modal = false;
                            }
                            Err(e) => {
                                state.feedback_message = Some((format!("Σφάλμα: {}", e), true));
                            }
                        }
                    }
                }
                if ui.button("Κλείσιμο").clicked() {
                    state.show_ingest_modal = false;
                }
            });
        });
}

fn draw_resolve_breach_modal(ctx: &egui::Context, conn: &Connection, state: &mut ColdChainViewState, operator_name: &str) {
    if !state.show_resolve_modal {
        return;
    }

    egui::Window::new("🛠 Καταγραφή Διορθωτικής Ενέργειας HACCP")
        .collapsible(false)
        .resizable(false)
        .default_width(380.0)
        .show(ctx, |ui| {
            ui.label(RichText::new(format!("Συμβάν: {}", state.resolve_breach_id)).strong().color(TEXT_PRIMARY));
            ui.label(RichText::new("Περιγραφή Διορθωτικής Ενέργειας (Corrective Action):").size(12.0).color(TEXT_SECONDARY));
            ui.text_edit_multiline(&mut state.resolve_note);

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Ολοκλήρωση Επίλυσης").clicked() {
                    if !state.resolve_note.is_empty() {
                        if resolve_breach_event(conn, &state.resolve_breach_id, operator_name, &state.resolve_note).is_ok() {
                            state.feedback_message = Some(("Το συμβάν επιλύθηκε και αρχειοθετήθηκε στο audit log.".to_string(), false));
                            state.show_resolve_modal = false;
                            state.resolve_note.clear();
                        }
                    }
                }
                if ui.button("Ακύρωση").clicked() {
                    state.show_resolve_modal = false;
                }
            });
        });
}

fn draw_haccp_certificate_modal(ctx: &egui::Context, state: &mut ColdChainViewState) {
    let mut close = false;
    if let Some(cert) = &state.certificate_modal {
        egui::Window::new("📜 Πιστοποιητικό Συμμόρφωσης HACCP & Merkle Audit")
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label(RichText::new(format!("Αριθμός Πιστοποιητικού: {}", cert.certificate_id)).strong().size(13.0).color(ACCENT_GOLD));
                ui.label(RichText::new(format!("Στόχος Επιθεώρησης: {}", cert.target_id)).color(TEXT_PRIMARY));
                ui.label(RichText::new(format!("Χρονικό Παράθυρο: {} έως {}", cert.time_window_start, cert.time_window_end)).size(11.5).color(TEXT_MUTED));
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Σύνολο Μετρήσεων:").color(TEXT_SECONDARY));
                    ui.label(RichText::new(cert.total_readings.to_string()).strong().color(TEXT_PRIMARY));
                    ui.label(RichText::new("Εντός Ορίων:").color(TEXT_SECONDARY));
                    ui.label(RichText::new(format!("{:.1}%", cert.in_spec_percentage)).strong().color(STATUS_READY));
                });

                let status_text = if cert.is_compliant { "✅ ΠΛΗΡΗΣ ΣΥΜΜΟΡΦΩΣΗ HACCP" } else { "❌ ΑΠΑΙΤΕΙΤΑΙ ΕΛΕΓΧΟΣ ΑΠΟΚΛΙΣΕΩΝ" };
                let status_color = if cert.is_compliant { STATUS_READY } else { STATUS_CANCELLED };
                ui.label(RichText::new(status_text).strong().size(13.0).color(status_color));
                ui.add_space(6.0);

                ui.label(RichText::new("Κρυπτογραφικό Merkle Root (Απαραβίαστο):").size(11.0).color(TEXT_MUTED));
                ui.label(RichText::new(&cert.merkle_root_hash).size(10.5).color(TEXT_SECONDARY));

                ui.add_space(8.0);
                if ui.button("Κλείσιμο").clicked() {
                    close = true;
                }
            });
    }

    if close {
        state.certificate_modal = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_cold_chain_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_cold_chain_view_state_defaults() {
        let state = ColdChainViewState::default();
        assert_eq!(state.selected_target_filter, "ALL");
        assert!(!state.show_register_modal);
        assert!(!state.show_ingest_modal);
        assert!(state.certificate_modal.is_none());
    }

    #[test]
    fn test_cold_chain_live_sqlite_integration() {
        let conn = setup_test_db();
        let sensor = ColdChainSensor::new("SEN-TEST-01", "VAN-TEST-88", ColdStorageType::Chilled);
        register_cold_sensor(&conn, &sensor).unwrap();

        let mut state = ColdChainViewState::default();
        state.selected_target_filter = "VAN-TEST-88".to_string();
        assert_eq!(state.selected_target_filter, "VAN-TEST-88");

        // Ingest reading
        let (reading, breach) = ingest_telemetry_reading(&conn, &sensor.sensor_id, 3.2, Some(70.0), false, Some(99)).unwrap();
        assert!(breach.is_none());
        assert_eq!(reading.temperature_celsius, 3.2);

        let logs = list_recent_readings(&conn, Some("VAN-TEST-88"), 5).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].reading_id, reading.reading_id);

        let cert = generate_haccp_certificate(&conn, "VAN-TEST-88", 0, Utc::now().timestamp() + 100).unwrap();
        assert_eq!(cert.total_readings, 1);
        assert!(cert.is_compliant);
    }
}
