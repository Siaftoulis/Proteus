//! Customer Service Appointments Subsystem (Ραντεβού Πελατών).
//! Allows front-desk staff to schedule customer intakes, diagnostics, deliveries, and contract appointments.
//! Emits audit events to the centralized system timeline.

use crm_core::audit::{log_audit_event, SystemEvent};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppointmentStatus {
    Scheduled,
    Completed,
    Cancelled,
}

impl AppointmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scheduled => "Προγραμματισμένο",
            Self::Completed => "Ολοκληρώθηκε",
            Self::Cancelled => "Ακυρώθηκε",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: String,
    pub customer_name: String,
    pub customer_phone: String,
    pub appointment_type: String,
    pub scheduled_time: String,
    pub notes: String,
    pub status: AppointmentStatus,
}

pub struct AppointmentsViewState {
    pub customer_name: String,
    pub customer_phone: String,
    pub appointment_type: String,
    pub scheduled_time: String,
    pub notes: String,
    pub appointments: Vec<Appointment>,
    pub status_message: Option<String>,
}

impl Default for AppointmentsViewState {
    fn default() -> Self {
        Self {
            customer_name: String::new(),
            customer_phone: String::new(),
            appointment_type: "Παραλαβή Συσκευής για Έλεγχο".to_string(),
            scheduled_time: "Σήμερα, 15:30".to_string(),
            notes: String::new(),
            appointments: vec![
                Appointment {
                    id: "APT-101".to_string(),
                    customer_name: "Δημήτρης Καρράς".to_string(),
                    customer_phone: "6941122334".to_string(),
                    appointment_type: "Παραλαβή & Διάγνωση MacBook".to_string(),
                    scheduled_time: "Σήμερα, 12:00".to_string(),
                    notes: "Δεν ανάβει μετά από πτώση υγρού.".to_string(),
                    status: AppointmentStatus::Completed,
                },
                Appointment {
                    id: "APT-102".to_string(),
                    customer_name: "Ελένη Βασιλείου".to_string(),
                    customer_phone: "6978899001".to_string(),
                    appointment_type: "Παράδοση Επισκευασμένου iPhone 13".to_string(),
                    scheduled_time: "Σήμερα, 16:30".to_string(),
                    notes: "Έχει εκδοθεί θερμική απόδειξη #1041.".to_string(),
                    status: AppointmentStatus::Scheduled,
                },
            ],
            status_message: None,
        }
    }
}

pub fn draw_appointments_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut AppointmentsViewState,
    operator_name: &str,
    operator_role: &str,
) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("📅 Ραντεβού Πελατών & Υποδοχή (Appointments Subsystem)").strong().size(22.0));
        ui.add_space(4.0);
        ui.label(RichText::new("Προγραμματισμός παραλαβών, επισκέψεων διάγνωσης και ραντεβού παράδοσης. Αυτόματη καταγραφή στο κεντρικό Audit Trail.")
            .size(12.0)
            .color(crate::theme::TEXT_MUTED));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        if let Some(msg) = &state.status_message {
            Frame::new()
                .fill(Color32::from_rgb(16, 50, 35))
                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.label(RichText::new(msg).color(Color32::from_rgb(52, 211, 153)).strong());
                });
            ui.add_space(10.0);
        }

        ui.columns(2, |cols| {
            // Left Column: New Appointment Booking Form
            cols[0].vertical(|ui| {
                Frame::new()
                    .fill(crate::theme::BG_CARD)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.label(RichText::new("ΠΡΟΓΡΑΜΜΑΤΙΣΜΟΣ ΝΕΟΥ ΡΑΝΤΕΒΟΥ").strong().color(crate::theme::TEXT_MUTED));
                        ui.add_space(10.0);

                        ui.label("Ονοματεπώνυμο Πελάτη (*):");
                        ui.add(egui::TextEdit::singleline(&mut state.customer_name).hint_text("π.χ. Γεώργιος Νικολάου"));
                        ui.add_space(6.0);

                        ui.label("Τηλέφωνο Επικοινωνίας (*):");
                        ui.add(egui::TextEdit::singleline(&mut state.customer_phone).hint_text("π.χ. 6900000000"));
                        ui.add_space(6.0);

                        ui.label("Τύπος Ραντεβού:");
                        egui::ComboBox::from_id_salt("apt_type_combo")
                            .selected_text(&state.appointment_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut state.appointment_type, "Παραλαβή Συσκευής για Έλεγχο".to_string(), "Παραλαβή Συσκευής για Έλεγχο");
                                ui.selectable_value(&mut state.appointment_type, "Διάγνωση & Προσφορά Επισκευής".to_string(), "Διάγνωση & Προσφορά Επισκευής");
                                ui.selectable_value(&mut state.appointment_type, "Παράδοση & Εξόφληση".to_string(), "Παράδοση & Εξόφληση");
                                ui.selectable_value(&mut state.appointment_type, "Υπογραφή Συμβολαίου SLA".to_string(), "Υπογραφή Συμβολαίου SLA");
                            });
                        ui.add_space(6.0);

                        ui.label("Ημερομηνία / Ώρα:");
                        ui.add(egui::TextEdit::singleline(&mut state.scheduled_time).hint_text("π.χ. Αύριο, 11:30"));
                        ui.add_space(6.0);

                        ui.label("Σημειώσεις Ραντεβού:");
                        ui.add(egui::TextEdit::multiline(&mut state.notes).desired_rows(2).hint_text("Επιπλέον παρατηρήσεις..."));
                        ui.add_space(12.0);

                        if ui.button(RichText::new("📅 Καταχώρηση Ραντεβού").strong()).clicked() {
                            if state.customer_name.trim().is_empty() || state.customer_phone.trim().is_empty() {
                                state.status_message = Some("Σφάλμα: Παρακαλώ συμπληρώστε όνομα και τηλέφωνο.".to_string());
                            } else {
                                let new_id = format!("APT-{}", 103 + state.appointments.len());
                                let apt = Appointment {
                                    id: new_id.clone(),
                                    customer_name: state.customer_name.clone(),
                                    customer_phone: state.customer_phone.clone(),
                                    appointment_type: state.appointment_type.clone(),
                                    scheduled_time: state.scheduled_time.clone(),
                                    notes: state.notes.clone(),
                                    status: AppointmentStatus::Scheduled,
                                };

                                // Log system event into centralized audit trail
                                let event = SystemEvent::new(
                                    "APPOINTMENT",
                                    &new_id,
                                    "BOOKED",
                                    operator_name,
                                    operator_role,
                                    format!("Προγραμματίστηκε ραντεβού ({}) για: {}", apt.appointment_type, apt.customer_name),
                                    serde_json::to_string(&apt).unwrap_or_default(),
                                );
                                let _ = log_audit_event(conn, &event);

                                state.appointments.push(apt);
                                state.status_message = Some(format!("✓ Το ραντεβού {} καταχωρήθηκε επιτυχώς και καταγράφηκε στο Audit Trail.", new_id));
                                state.customer_name.clear();
                                state.customer_phone.clear();
                                state.notes.clear();
                            }
                        }
                    });
            });

            // Right Column: Appointments List
            cols[1].vertical(|ui| {
                Frame::new()
                    .fill(crate::theme::BG_CARD)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.label(RichText::new("ΠΡΟΓΡΑΜΜΑΤΙΣΜΕΝΑ ΡΑΝΤΕΒΟΥ").strong().color(crate::theme::TEXT_MUTED));
                        ui.add_space(8.0);

                        for apt in state.appointments.iter_mut().rev() {
                            let status_col = match apt.status {
                                AppointmentStatus::Scheduled => Color32::from_rgb(56, 189, 248),
                                AppointmentStatus::Completed => Color32::from_rgb(52, 211, 153),
                                AppointmentStatus::Cancelled => Color32::from_rgb(244, 63, 94),
                            };

                            Frame::new()
                                .fill(crate::theme::BG_BASE)
                                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::same(10))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&apt.id).size(11.0).color(crate::theme::ACCENT_CYAN).strong());
                                        ui.label(RichText::new(&apt.customer_name).strong().color(crate::theme::TEXT_PRIMARY));
                                        ui.label(RichText::new(format!("({})", apt.customer_phone)).size(11.0).color(crate::theme::TEXT_MUTED));

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(RichText::new(apt.status.as_str()).size(11.0).color(status_col).strong());
                                        });
                                    });

                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("🕒").color(crate::theme::TEXT_MUTED));
                                        ui.label(RichText::new(&apt.scheduled_time).size(12.0).color(crate::theme::TEXT_SECONDARY));
                                        ui.label("—");
                                        ui.label(RichText::new(&apt.appointment_type).size(12.0).color(crate::theme::TEXT_PRIMARY));
                                    });

                                    if !apt.notes.is_empty() {
                                        ui.add_space(2.0);
                                        ui.label(RichText::new(&apt.notes).size(11.0).color(crate::theme::TEXT_MUTED));
                                    }

                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        if apt.status == AppointmentStatus::Scheduled {
                                            if ui.small_button("✓ Ολοκλήρωση").clicked() {
                                                apt.status = AppointmentStatus::Completed;
                                            }
                                            if ui.small_button("✕ Ακύρωση").clicked() {
                                                apt.status = AppointmentStatus::Cancelled;
                                            }
                                        }
                                    });
                                });
                            ui.add_space(6.0);
                        }
                    });
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appointments_state_defaults() {
        let state = AppointmentsViewState::default();
        assert_eq!(state.appointments.len(), 2);
        assert_eq!(state.appointments[0].id, "APT-101");
        assert_eq!(state.appointments[0].status, AppointmentStatus::Completed);
    }
}
