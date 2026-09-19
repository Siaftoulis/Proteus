//! Screen 11: Enterprise Multi-Store HQ & Anti-SAP Governance Dashboard.
//! Decentralized multi-store management, branch seat quotas, and departmental oversight.

use crm_core::enterprise::{
    calculate_seat_utilization, create_enterprise_store, create_store_department,
    get_primary_enterprise, list_enterprise_stores, list_store_departments, Enterprise,
    EnterpriseStore, StoreDepartment,
};
use egui::{Color32, CornerRadius, Frame, Margin, ProgressBar, RichText, Stroke, Ui, Window};
use rusqlite::Connection;
use std::collections::HashMap;

pub struct EnterpriseHqState {
    pub enterprise: Option<Enterprise>,
    pub stores: Vec<EnterpriseStore>,
    pub departments: HashMap<String, Vec<StoreDepartment>>,
    pub total_quota: i64,
    pub allocated_quota: i64,
    pub show_new_store_modal: bool,
    pub new_store_code: String,
    pub new_store_name: String,
    pub new_store_address: String,
    pub new_store_phone: String,
    pub new_store_seats: String,
    pub show_new_dept_store_id: Option<String>,
    pub new_dept_code: String,
    pub new_dept_name: String,
    pub status_msg: Option<(String, bool)>,
}

impl Default for EnterpriseHqState {
    fn default() -> Self {
        Self {
            enterprise: None,
            stores: Vec::new(),
            departments: HashMap::new(),
            total_quota: 0,
            allocated_quota: 0,
            show_new_store_modal: false,
            new_store_code: "STR-003-PATRA".to_string(),
            new_store_name: "Κατάστημα 03: Πάτρα".to_string(),
            new_store_address: "Κορίνθου 150, Πάτρα".to_string(),
            new_store_phone: "+30 2610 330003".to_string(),
            new_store_seats: "5".to_string(),
            show_new_dept_store_id: None,
            new_dept_code: "WAREHOUSE".to_string(),
            new_dept_name: "Αποθήκη Εμπορευμάτων".to_string(),
            status_msg: None,
        }
    }
}

impl EnterpriseHqState {
    pub fn reload(&mut self, conn: &Connection) {
        if let Ok(Some(ent)) = get_primary_enterprise(conn) {
            if let Ok(stores) = list_enterprise_stores(conn, &ent.enterprise_id) {
                let mut depts = HashMap::new();
                for s in &stores {
                    if let Ok(d_list) = list_store_departments(conn, &s.store_id) {
                        depts.insert(s.store_id.clone(), d_list);
                    }
                }
                self.stores = stores;
                self.departments = depts;
            }
            if let Ok((alloc, tot)) = calculate_seat_utilization(conn, &ent.enterprise_id) {
                self.allocated_quota = alloc;
                self.total_quota = tot;
            }
            self.enterprise = Some(ent);
        }
    }
}

pub fn draw_enterprise_hq_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut EnterpriseHqState,
) {
    if state.enterprise.is_none() {
        state.reload(conn);
    }

    ui.vertical(|ui| {
        ui.heading(RichText::new("🏢 Enterprise Multi-Store HQ (Anti-SAP Governance)").strong().size(22.0));
        ui.add_space(4.0);
        ui.label(RichText::new("Αποκεντρωμένη διαχείριση αλυσίδας καταστημάτων, κατανομή ποσοστώσεων (Seat Quotas) και εποπτεία τμημάτων.")
            .size(12.0)
            .color(crate::theme::TEXT_MUTED));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        if let Some((msg, is_ok)) = &state.status_msg {
            let bg = if *is_ok { Color32::from_rgb(16, 50, 35) } else { Color32::from_rgb(60, 20, 20) };
            let border = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
            Frame::new()
                .fill(bg)
                .stroke(Stroke::new(1.0, border))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.label(RichText::new(msg).color(border).strong());
                });
            ui.add_space(10.0);
        }

        // Top Enterprise Quota Banner
        if let Some(ent) = &state.enterprise {
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&ent.legal_name).strong().size(16.0).color(crate::theme::TEXT_PRIMARY));
                                ui.label(RichText::new(format!("ΑΦΜ: {}", ent.tax_id)).size(12.0).color(crate::theme::TEXT_MUTED));
                                Frame::new()
                                    .fill(Color32::from_rgb(30, 41, 59))
                                    .corner_radius(CornerRadius::same(4))
                                    .inner_margin(Margin::symmetric(6, 2))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(&ent.cloud_tier).size(10.0).color(crate::theme::ACCENT_CYAN));
                                    });
                            });
                            ui.add_space(6.0);

                            let pct = if state.total_quota > 0 {
                                (state.allocated_quota as f32 / state.total_quota as f32).clamp(0.0, 1.0)
                            } else {
                                0.0
                            };
                            let progress_text = format!("{}/{} Θέσεις Κατανεμημένες ({:.0}%)", state.allocated_quota, state.total_quota, pct * 100.0);
                            ui.horizontal(|ui| {
                                ui.add(ProgressBar::new(pct).desired_width(260.0).text(progress_text));
                                ui.label(RichText::new(format!("{} Ελεύθερες", state.total_quota.saturating_sub(state.allocated_quota))).size(11.0).color(Color32::from_rgb(52, 211, 153)));
                            });
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(RichText::new("+ Νέο Υποκατάστημα").strong().color(Color32::WHITE))
                                .fill(crate::theme::ACCENT_PRIMARY))
                                .clicked()
                            {
                                state.show_new_store_modal = true;
                            }
                        });
                    });
                });
            ui.add_space(14.0);
        }

        // Store Branches Section
        ui.label(RichText::new("ΥΠΟΚΑΤΑΣΤΗΜΑΤΑ & ΣΗΜΕΙΑ ΠΩΛΗΣΗΣ (STORE BRANCHES)").strong().color(crate::theme::TEXT_MUTED));
        ui.add_space(8.0);

        for store in state.stores.clone() {
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(14))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("●").color(Color32::from_rgb(52, 211, 153)).size(14.0));
                        ui.label(RichText::new(&store.store_name).strong().size(14.0).color(crate::theme::TEXT_PRIMARY));
                        ui.label(RichText::new(format!("({})", store.store_code)).size(11.0).color(crate::theme::TEXT_MUTED));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("+ Τμήμα").clicked() {
                                state.show_new_dept_store_id = Some(store.store_id.clone());
                            }
                            Frame::new()
                                .fill(crate::theme::BG_BASE)
                                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(8, 3))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(format!("Quota: {}/{} Θέσεις", store.active_seats, store.allocated_seats))
                                        .size(11.0)
                                        .color(crate::theme::ACCENT_CYAN));
                                });
                        });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&store.address).size(11.0).color(crate::theme::TEXT_SECONDARY));
                        ui.label(RichText::new("•").color(crate::theme::TEXT_MUTED));
                        ui.label(RichText::new(&store.phone).size(11.0).color(crate::theme::TEXT_SECONDARY));
                    });

                    ui.add_space(8.0);
                    // Department Chips
                    if let Some(depts) = state.departments.get(&store.store_id) {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("Τμήματα:").size(11.0).color(crate::theme::TEXT_MUTED));
                            for d in depts {
                                let icon = match d.dept_code.as_str() {
                                    "WAREHOUSE" => "📦",
                                    "MOBILE" => "📱",
                                    "SERVICE" => "🛠",
                                    "POS" => "💳",
                                    _ => "🏷",
                                };
                                Frame::new()
                                    .fill(crate::theme::BG_BASE)
                                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                                    .corner_radius(CornerRadius::same(4))
                                    .inner_margin(Margin::symmetric(6, 2))
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(format!("{} {}", icon, d.name)).size(10.0).color(crate::theme::TEXT_PRIMARY));
                                    });
                                ui.add_space(2.0);
                            }
                        });
                    }
                });
            ui.add_space(8.0);
        }
    });

    // Modal: New Store Branch
    if state.show_new_store_modal {
        let mut close_modal = false;
        Window::new("Προσθήκη Νέου Υποκαταστήματος")
            .collapsible(false)
            .resizable(false)
            .default_width(380.0)
            .show(ui.ctx(), |ui| {
                ui.label(RichText::new("Στοιχεία Καταστήματος:").strong());
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Κωδικός:");
                    ui.text_edit_singleline(&mut state.new_store_code);
                });
                ui.horizontal(|ui| {
                    ui.label("Όνομα:");
                    ui.text_edit_singleline(&mut state.new_store_name);
                });
                ui.horizontal(|ui| {
                    ui.label("Διεύθυνση:");
                    ui.text_edit_singleline(&mut state.new_store_address);
                });
                ui.horizontal(|ui| {
                    ui.label("Τηλέφωνο:");
                    ui.text_edit_singleline(&mut state.new_store_phone);
                });
                ui.horizontal(|ui| {
                    ui.label("Θέσεις Quota:");
                    ui.text_edit_singleline(&mut state.new_store_seats);
                });
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("✓ Δημιουργία").strong().color(Color32::WHITE)).fill(crate::theme::ACCENT_PRIMARY)).clicked() {
                        if let Some(ent) = &state.enterprise {
                            let seats = state.new_store_seats.trim().parse::<i64>().unwrap_or(5);
                            let new_store = EnterpriseStore {
                                store_id: format!("str-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)),
                                enterprise_id: ent.enterprise_id.clone(),
                                store_code: state.new_store_code.trim().to_string(),
                                store_name: state.new_store_name.trim().to_string(),
                                address: state.new_store_address.trim().to_string(),
                                phone: state.new_store_phone.trim().to_string(),
                                allocated_seats: seats,
                                active_seats: 0,
                                is_active: true,
                                created_at: format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)),
                            };
                            match create_enterprise_store(conn, &new_store) {
                                Ok(_) => {
                                    state.status_msg = Some((format!("✓ Το κατάστημα '{}' προστέθηκε επιτυχώς!", new_store.store_name), true));
                                    state.reload(conn);
                                    close_modal = true;
                                }
                                Err(e) => {
                                    state.status_msg = Some((format!("❌ Σφάλμα: {}", e), false));
                                }
                            }
                        }
                    }
                    if ui.button("Άκυρο").clicked() {
                        close_modal = true;
                    }
                });
            });
        if close_modal {
            state.show_new_store_modal = false;
        }
    }

    // Modal: New Department
    if let Some(store_id) = state.show_new_dept_store_id.clone() {
        let mut close_dept_modal = false;
        Window::new("Προσθήκη Τμήματος")
            .collapsible(false)
            .resizable(false)
            .default_width(320.0)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Κωδικός:");
                    ui.text_edit_singleline(&mut state.new_dept_code);
                });
                ui.horizontal(|ui| {
                    ui.label("Όνομα:");
                    ui.text_edit_singleline(&mut state.new_dept_name);
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(RichText::new("✓ Προσθήκη").strong().color(Color32::WHITE)).fill(crate::theme::ACCENT_PRIMARY)).clicked() {
                        let dept = StoreDepartment {
                            department_id: format!("dept-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)),
                            store_id,
                            dept_code: state.new_dept_code.trim().to_string(),
                            name: state.new_dept_name.trim().to_string(),
                            created_at: format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)),
                        };
                        match create_store_department(conn, &dept) {
                            Ok(_) => {
                                state.status_msg = Some((format!("✓ Το τμήμα '{}' προστέθηκε!", dept.name), true));
                                state.reload(conn);
                                close_dept_modal = true;
                            }
                            Err(e) => {
                                state.status_msg = Some((format!("❌ Σφάλμα: {}", e), false));
                            }
                        }
                    }
                    if ui.button("Άκυρο").clicked() {
                        close_dept_modal = true;
                    }
                });
            });
        if close_dept_modal {
            state.show_new_dept_store_id = None;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_enterprise_hq_state_reload() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        crm_core::enterprise::init_enterprise_schema(&conn).unwrap();
        crm_core::enterprise::seed_default_enterprise_if_empty(&conn).unwrap();

        let mut state = EnterpriseHqState::default();
        state.reload(&conn);

        assert!(state.enterprise.is_some());
        assert_eq!(state.stores.len(), 2);
        assert_eq!(state.allocated_quota, 25);
        assert_eq!(state.total_quota, 30);
    }
}
