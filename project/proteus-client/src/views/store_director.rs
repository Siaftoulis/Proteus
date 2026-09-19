//! Screen 12: Store Director — Delegated Employee Provisioning Dashboard.
//! Enables autonomous store-level employee lifecycle management without central IT overhead,
//! enforcing store seat quotas and recording all changes to the Merkle hash-chain audit log.

use crm_core::enterprise::{
    deactivate_store_user, get_primary_enterprise, list_enterprise_stores,
    list_store_departments, list_store_users, provision_store_user, EnterpriseStore,
    EnterpriseUser, StoreDepartment,
};
use egui::{Color32, CornerRadius, Frame, Margin, ProgressBar, RichText, Stroke, Ui, Window};
use rusqlite::Connection;

pub struct StoreDirectorState {
    pub selected_store_id: Option<String>,
    pub stores: Vec<EnterpriseStore>,
    pub departments: Vec<StoreDepartment>,
    pub users: Vec<EnterpriseUser>,
    pub filter_dept_id: Option<String>,
    pub show_new_user_modal: bool,
    pub new_user_name: String,
    pub new_user_email: String,
    pub new_user_dept_id: Option<String>,
    pub new_user_role: String,
    pub status_msg: Option<(String, bool)>,
}

impl Default for StoreDirectorState {
    fn default() -> Self {
        Self {
            selected_store_id: None,
            stores: Vec::new(),
            departments: Vec::new(),
            users: Vec::new(),
            filter_dept_id: None,
            show_new_user_modal: false,
            new_user_name: String::new(),
            new_user_email: String::new(),
            new_user_dept_id: None,
            new_user_role: "PosCashier".to_string(),
            status_msg: None,
        }
    }
}

impl StoreDirectorState {
    pub fn reload(&mut self, conn: &Connection) {
        if let Ok(Some(ent)) = get_primary_enterprise(conn) {
            if let Ok(stores) = list_enterprise_stores(conn, &ent.enterprise_id) {
                self.stores = stores;
                if self.selected_store_id.is_none() && !self.stores.is_empty() {
                    self.selected_store_id = Some(self.stores[0].store_id.clone());
                }
            }
        }
        if let Some(store_id) = &self.selected_store_id {
            self.departments = list_store_departments(conn, store_id).unwrap_or_default();
            self.users = list_store_users(conn, store_id).unwrap_or_default();
            if self.new_user_dept_id.is_none() && !self.departments.is_empty() {
                self.new_user_dept_id = Some(self.departments[0].department_id.clone());
            }
        }
    }
}

pub fn draw_store_director_view(ui: &mut Ui, conn: &Connection, state: &mut StoreDirectorState) {
    if state.stores.is_empty() {
        state.reload(conn);
    }

    ui.vertical(|ui| {
        // Top Banner / Header
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🏪 Store Director — Τοπική Διαχείριση Προσωπικού").strong().color(crate::theme::ACCENT_CYAN));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Ανανέωση").clicked() {
                    state.reload(conn);
                }
            });
        });
        ui.label(RichText::new("Αυτόνομη ανάθεση θέσεων υπαλλήλων ανά κατάστημα με επιβολή ορίων θέσεων (Seat Quotas) & Merkle Audit Trail.").size(12.0).color(crate::theme::TEXT_MUTED));
        ui.add_space(8.0);

        if let Some((msg, success)) = &state.status_msg {
            let (bg, border, fg) = if *success {
                (Color32::from_rgb(16, 50, 35), Color32::from_rgb(52, 211, 153), Color32::from_rgb(52, 211, 153))
            } else {
                (Color32::from_rgb(60, 20, 20), Color32::from_rgb(248, 113, 113), Color32::from_rgb(248, 113, 113))
            };
            Frame::new().fill(bg).stroke(Stroke::new(1.0, border)).corner_radius(CornerRadius::same(6)).inner_margin(Margin::symmetric(12, 8)).show(ui, |ui| {
                ui.label(RichText::new(msg).color(fg).size(12.0).strong());
            });
            ui.add_space(8.0);
        }

        // Store Selector Dropdown & Info Card
        let current_store = state.stores.iter().find(|s| Some(&s.store_id) == state.selected_store_id.as_ref()).cloned();
        
        Frame::new().fill(crate::theme::BG_PANEL).stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE)).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(14)).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Επιλογή Καταστήματος:").strong().size(13.0));
                let current_name = current_store.as_ref().map(|s| s.store_name.clone()).unwrap_or_else(|| "Επιλέξτε...".into());
                let mut switch_store_id = None;
                egui::ComboBox::from_id_salt("store_dir_selector").selected_text(current_name).show_ui(ui, |ui| {
                    for s in &state.stores {
                        let is_sel = state.selected_store_id.as_ref() == Some(&s.store_id);
                        if ui.selectable_label(is_sel, &s.store_name).clicked() {
                            switch_store_id = Some(s.store_id.clone());
                        }
                    }
                });
                if let Some(sid) = switch_store_id {
                    state.selected_store_id = Some(sid);
                    state.reload(conn);
                }

                if let Some(ref st) = current_store {
                    ui.add_space(16.0);
                    let ratio = if st.allocated_seats > 0 { st.active_seats as f32 / st.allocated_seats as f32 } else { 0.0 };
                    let bar_color = if ratio >= 1.0 { Color32::from_rgb(239, 68, 68) } else if ratio > 0.8 { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(52, 211, 153) };
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Θέσεις Εργασίας: {} / {} κατειλημμένες", st.active_seats, st.allocated_seats)).strong().size(12.0));
                            if st.active_seats >= st.allocated_seats {
                                ui.label(RichText::new("⚠️ ΠΛΗΡΕΣ").color(Color32::from_rgb(239, 68, 68)).strong().size(11.0));
                            }
                        });
                        ui.add(ProgressBar::new(ratio.min(1.0)).fill(bar_color).desired_width(180.0));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_add = st.active_seats < st.allocated_seats;
                        let btn = egui::Button::new(RichText::new("+ Νέος Υπάλληλος").strong().size(12.0).color(Color32::WHITE))
                            .fill(if can_add { crate::theme::ACCENT_PRIMARY } else { Color32::from_rgb(70, 70, 70) });
                        if ui.add_enabled(can_add, btn).clicked() {
                            state.show_new_user_modal = true;
                            state.status_msg = None;
                        }
                    });
                }
            });
        });

        ui.add_space(12.0);

        // Department Filter Tabs
        let mut change_dept = None;
        ui.horizontal(|ui| {
            ui.label(RichText::new("Φίλτρο Τμήματος:").size(12.0).color(crate::theme::TEXT_MUTED));
            let is_all = state.filter_dept_id.is_none();
            if ui.selectable_label(is_all, "Όλα τα Τμήματα").clicked() {
                change_dept = Some(None);
            }
            for d in &state.departments {
                let is_sel = state.filter_dept_id.as_deref() == Some(&d.department_id);
                let label = format!("{} {}", dept_icon(&d.dept_code), d.name);
                if ui.selectable_label(is_sel, label).clicked() {
                    change_dept = Some(Some(d.department_id.clone()));
                }
            }
        });
        if let Some(new_filter) = change_dept {
            state.filter_dept_id = new_filter;
        }

        ui.add_space(10.0);

        // Employee Roster Table
        let filtered_users: Vec<EnterpriseUser> = state.users.iter().filter(|u| {
            if let Some(ref dept_id) = state.filter_dept_id {
                u.department_id.as_deref() == Some(dept_id.as_str())
            } else {
                true
            }
        }).cloned().collect();

        let mut deactivate_action: Option<(String, String)> = None;

        Frame::new().fill(crate::theme::BG_CARD).stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE)).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(12)).show(ui, |ui| {
            if filtered_users.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("Δεν βρέθηκαν υπάλληλοι για τα επιλεγμένα κριτήρια.").color(crate::theme::TEXT_MUTED));
                    ui.add_space(20.0);
                });
            } else {
                egui::Grid::new("store_users_grid").striped(true).min_col_width(120.0).spacing([16.0, 10.0]).show(ui, |ui| {
                    ui.label(RichText::new("Ονοματεπώνυμο").strong().size(12.0).color(crate::theme::TEXT_SECONDARY));
                    ui.label(RichText::new("Email").strong().size(12.0).color(crate::theme::TEXT_SECONDARY));
                    ui.label(RichText::new("Τμήμα").strong().size(12.0).color(crate::theme::TEXT_SECONDARY));
                    ui.label(RichText::new("Ρόλος").strong().size(12.0).color(crate::theme::TEXT_SECONDARY));
                    ui.label(RichText::new("Κατάσταση").strong().size(12.0).color(crate::theme::TEXT_SECONDARY));
                    ui.label(RichText::new("Ενέργειες").strong().size(12.0).color(crate::theme::TEXT_SECONDARY));
                    ui.end_row();

                    for u in &filtered_users {
                        ui.label(RichText::new(&u.full_name).strong().color(Color32::WHITE));
                        ui.label(RichText::new(&u.email).size(12.0).color(crate::theme::TEXT_MUTED));
                        
                        let dept_name = u.department_id.as_ref().and_then(|did| {
                            state.departments.iter().find(|d| &d.department_id == did).map(|d| format!("{} {}", dept_icon(&d.dept_code), d.name))
                        }).unwrap_or_else(|| "—".into());
                        ui.label(RichText::new(dept_name).size(12.0));

                        ui.label(RichText::new(role_display_name(&u.role_code)).size(12.0).color(crate::theme::ACCENT_CYAN));

                        if u.is_active {
                            ui.label(RichText::new("● Ενεργός").color(Color32::from_rgb(52, 211, 153)).size(12.0));
                            if ui.button(RichText::new("Απενεργοποίηση").size(11.0).color(Color32::from_rgb(248, 113, 113))).clicked() {
                                deactivate_action = Some((u.user_id.clone(), u.full_name.clone()));
                            }
                        } else {
                            ui.label(RichText::new("○ Ανενεργός").color(crate::theme::TEXT_MUTED).size(12.0));
                            ui.label(RichText::new("—").color(crate::theme::TEXT_MUTED));
                        }
                        ui.end_row();
                    }
                });
            }
        });

        if let Some((uid, uname)) = deactivate_action {
            match deactivate_store_user(conn, &uid) {
                Ok(_) => {
                    state.status_msg = Some((format!("✓ Ο χρήστης '{}' απενεργοποιήθηκε επιτυχώς (Merkle Logged).", uname), true));
                    state.reload(conn);
                }
                Err(e) => {
                    state.status_msg = Some((format!("❌ {}", e), false));
                }
            }
        }
    });

    // New User Provisioning Modal
    if state.show_new_user_modal {
        if let Some(ref store) = state.stores.iter().find(|s| Some(&s.store_id) == state.selected_store_id.as_ref()).cloned() {
            Window::new("➕ Ανάθεση Νέου Υπαλλήλου (Store Provisioning)")
                .collapsible(false)
                .resizable(false)
                .default_width(420.0)
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("Κατάστημα: {}", store.store_name)).strong().color(crate::theme::ACCENT_CYAN));
                        ui.label(RichText::new(format!("Διαθέσιμες Θέσεις: {} / {}", store.active_seats, store.allocated_seats)).size(12.0).color(crate::theme::TEXT_MUTED));
                        ui.add_space(8.0);

                        ui.label("Ονοματεπώνυμο:");
                        ui.text_edit_singleline(&mut state.new_user_name);

                        ui.label("Email Υπαλλήλου:");
                        ui.text_edit_singleline(&mut state.new_user_email);

                        ui.label("Τμήμα Εργασίας:");
                        let cur_dept = state.new_user_dept_id.as_ref().and_then(|id| {
                            state.departments.iter().find(|d| &d.department_id == id).map(|d| d.name.clone())
                        }).unwrap_or_else(|| "Χωρίς Τμήμα".into());
                        egui::ComboBox::from_id_salt("modal_dept_select").selected_text(cur_dept).show_ui(ui, |ui| {
                            for d in &state.departments {
                                let is_s = state.new_user_dept_id.as_ref() == Some(&d.department_id);
                                if ui.selectable_label(is_s, &d.name).clicked() {
                                    state.new_user_dept_id = Some(d.department_id.clone());
                                }
                            }
                        });

                        ui.label("Ρόλος / Αρμοδιότητα:");
                        let roles = [
                            ("PosCashier", "Ταμίας (POS)"),
                            ("FloorSales", "Πωλητής Καταστήματος"),
                            ("TechnicianSpecialist", "Τεχνικός Επισκευών"),
                            ("WarehouseClerk", "Υπεύθυνος Αποθήκης"),
                            ("StoreManager", "Υποδιευθυντής Καταστήματος"),
                        ];
                        let cur_role_label = roles.iter().find(|(c, _)| *c == state.new_user_role).map(|(_, l)| *l).unwrap_or(&state.new_user_role);
                        egui::ComboBox::from_id_salt("modal_role_select").selected_text(cur_role_label).show_ui(ui, |ui| {
                            for (code, label) in roles {
                                if ui.selectable_label(state.new_user_role == code, label).clicked() {
                                    state.new_user_role = code.to_string();
                                }
                            }
                        });

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("Ακύρωση").clicked() {
                                state.show_new_user_modal = false;
                            }
                            let submit_btn = egui::Button::new(RichText::new("✓ Ανάθεση Θέσης (Provision)").strong().color(Color32::WHITE)).fill(crate::theme::ACCENT_PRIMARY);
                            if ui.add(submit_btn).clicked() {
                                if state.new_user_name.trim().is_empty() || state.new_user_email.trim().is_empty() {
                                    state.status_msg = Some(("❌ Συμπληρώστε όνομα και email.".into(), false));
                                } else {
                                    let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
                                    let new_user = EnterpriseUser {
                                        user_id: format!("usr-{}", now_ms),
                                        enterprise_id: store.enterprise_id.clone(),
                                        store_id: store.store_id.clone(),
                                        department_id: state.new_user_dept_id.clone(),
                                        full_name: state.new_user_name.trim().to_string(),
                                        email: state.new_user_email.trim().to_string(),
                                        role_code: state.new_user_role.clone(),
                                        is_active: true,
                                        created_at: format!("{}", now_ms),
                                    };
                                    match provision_store_user(conn, &new_user) {
                                        Ok(_) => {
                                            state.status_msg = Some((format!("✓ Επιτυχής καταχώριση & ανάθεση θέσης στον υπάλληλο '{}' (Merkle Verified).", new_user.full_name), true));
                                            state.new_user_name.clear();
                                            state.new_user_email.clear();
                                            state.show_new_user_modal = false;
                                            state.reload(conn);
                                        }
                                        Err(e) => {
                                            state.status_msg = Some((format!("❌ {}", e), false));
                                        }
                                    }
                                }
                            }
                        });
                    });
                });
        }
    }
}

fn dept_icon(code: &str) -> &'static str {
    match code {
        "WAREHOUSE" => "📦",
        "MOBILE" => "📱",
        "SERVICE" => "🛠",
        "POS" => "💳",
        _ => "🏢",
    }
}

fn role_display_name(code: &str) -> &str {
    match code {
        "PosCashier" => "Ταμίας (POS)",
        "FloorSales" => "Πωλητής",
        "TechnicianSpecialist" => "Τεχνικός",
        "WarehouseClerk" => "Αποθηκάριος",
        "StoreManager" => "Υποδιευθυντής",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_director_state_reload() {
        let conn = Connection::open_in_memory().unwrap();
        crm_core::merkle::init_merkle_schema(&conn).unwrap();
        crm_core::enterprise::init_enterprise_schema(&conn).unwrap();
        crm_core::enterprise::seed_default_enterprise_if_empty(&conn).unwrap();

        let mut state = StoreDirectorState::default();
        assert!(state.selected_store_id.is_none());

        state.reload(&conn);
        assert!(!state.stores.is_empty());
        assert!(state.selected_store_id.is_some());
        assert!(!state.departments.is_empty());
    }
}
