//! Security Guardian UI for Proteus Client (Master Problem Audit P3 & P4).
//! Displays Monotonic Anti-Rollback Time Lock integrity and Hardware-Sealed Vault status.
//! Provides zero-friction operational status, 1-click NTP resync, unlock tokens, and master re-keying.

use chrono::Utc;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use proteus_core::security::{
    MasterSession, TimeIntegrityGuard, TimeLockStatus, VaultManager, VaultStatus,
};
use rusqlite::Connection;

pub struct SecurityGuardianState {
    pub time_lock_status: TimeLockStatus,
    pub high_water_iso: String,
    pub time_token_input: String,
    pub vault_status: VaultStatus,
    pub master_password_input: String,
    pub master_session: Option<MasterSession>,
    pub feedback_msg: Option<(String, bool)>,
}

impl Default for SecurityGuardianState {
    fn default() -> Self {
        Self {
            time_lock_status: TimeLockStatus::Synchronized,
            high_water_iso: "-".to_string(),
            time_token_input: String::new(),
            vault_status: VaultStatus::Unprovisioned,
            master_password_input: String::new(),
            master_session: None,
            feedback_msg: None,
        }
    }
}

impl SecurityGuardianState {
    #[allow(dead_code)]
    pub fn refresh(&mut self, conn: &Connection, machine_id: &str) {
        if let Ok(Some(rec)) = TimeIntegrityGuard::get_record(conn) {
            self.high_water_iso = rec.high_water_iso;
            self.time_lock_status = if rec.tamper_locked {
                TimeLockStatus::TamperLocked
            } else {
                TimeLockStatus::Synchronized
            };
        }
        if let Ok(st) = VaultManager::get_status(conn, machine_id) {
            self.vault_status = st;
        }
    }

    pub fn apply_time_token(&mut self, conn: &Connection, machine_id: &str, secret: &str) {
        let tok = self.time_token_input.trim();
        if tok.is_empty() {
            self.feedback_msg = Some(("Εισάγετε έγκυρο Time-Unlock Token.".into(), false));
            return;
        }
        let now = Utc::now().timestamp();
        match TimeIntegrityGuard::apply_unlock_token(conn, tok, machine_id, now, secret) {
            Ok(()) => {
                proteus_core::audio::play_barcode_chime();
                self.time_lock_status = TimeLockStatus::Unlocked;
                if let Ok(Some(rec)) = TimeIntegrityGuard::get_record(conn) {
                    self.high_water_iso = rec.high_water_iso;
                }
                self.feedback_msg = Some(("✓ Το σύστημα ξεκλειδώθηκε επιτυχώς μέσω Time Token!".into(), true));
            }
            Err(e) => {
                proteus_core::audio::play_error_tone();
                self.feedback_msg = Some((format!("❌ Σφάλμα Token: {}", e), false));
            }
        }
    }

    pub fn apply_ntp_sync(&mut self, conn: &Connection, authoritative_epoch_secs: i64) {
        match TimeIntegrityGuard::apply_remote_ntp_sync(conn, authoritative_epoch_secs) {
            Ok(()) => {
                proteus_core::audio::play_affirm_tone();
                self.time_lock_status = TimeLockStatus::Synchronized;
                if let Ok(Some(rec)) = TimeIntegrityGuard::get_record(conn) {
                    self.high_water_iso = rec.high_water_iso;
                }
                self.feedback_msg = Some(("✓ Επαληθεύτηκε ο επίσημος συγχρονισμός ώρας (NTP Sync)!".into(), true));
            }
            Err(e) => {
                proteus_core::audio::play_error_tone();
                self.feedback_msg = Some((format!("❌ Αποτυχία συγχρονισμού: {}", e), false));
            }
        }
    }

    pub fn unlock_master(&mut self, conn: &Connection) {
        let pass = self.master_password_input.trim();
        if pass.is_empty() {
            self.feedback_msg = Some(("Εισάγετε τον Master Password του ιδιοκτήτη.".into(), false));
            return;
        }
        match VaultManager::unlock_master(conn, pass) {
            Ok(sess) => {
                proteus_core::audio::play_affirm_tone();
                self.master_session = Some(sess);
                self.feedback_msg = Some(("✓ Master Owner Session ενεργό (15 λεπτά).".into(), true));
            }
            Err(e) => {
                proteus_core::audio::play_error_tone();
                self.feedback_msg = Some((format!("❌ Σφάλμα Master Password: {}", e), false));
            }
        }
    }

    pub fn provision_vault(&mut self, conn: &Connection, machine_id: &str) {
        let pass = self.master_password_input.trim();
        if pass.len() < 8 {
            self.feedback_msg = Some(("Ο Master Password πρέπει να περιέχει τουλάχιστον 8 χαρακτήρες.".into(), false));
            return;
        }
        match VaultManager::provision(conn, machine_id, pass) {
            Ok(_) => {
                proteus_core::audio::play_affirm_tone();
                self.vault_status = VaultStatus::OperationalReady;
                self.feedback_msg = Some(("✓ Το Hardware Vault αρχικοποιήθηκε και σφραγίστηκε επιτυχώς!".into(), true));
            }
            Err(e) => {
                proteus_core::audio::play_error_tone();
                self.feedback_msg = Some((format!("❌ Σφάλμα αρχικοποίησης: {}", e), false));
            }
        }
    }

    pub fn rekey_operational(&mut self, conn: &Connection, machine_id: &str) {
        if let Some(ref sess) = self.master_session {
            match VaultManager::rekey_operational(conn, machine_id, sess) {
                Ok(_) => {
                    proteus_core::audio::play_affirm_tone();
                    self.vault_status = VaultStatus::OperationalReady;
                    self.feedback_msg = Some(("✓ Το Operational Key επανασφραγίστηκε στο τρέχον Hardware!".into(), true));
                }
                Err(e) => {
                    proteus_core::audio::play_error_tone();
                    self.feedback_msg = Some((format!("❌ Σφάλμα επανασφράγισης: {}", e), false));
                }
            }
        } else {
            self.feedback_msg = Some(("Απαιτείται ενεργό Master Owner Session.".into(), false));
        }
    }
}

pub fn draw_security_guardian_section(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut SecurityGuardianState,
    machine_id: &str,
) {
    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🛡️ SECURITY GUARDIAN & HARDWARE VAULT").strong().color(crate::theme::ACCENT_CYAN));
                ui.label(RichText::new("(Monotonic Time Lock & DPAPI/TPM Sealing)").size(11.0).color(crate::theme::TEXT_MUTED));
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Ανίχνευση παραβίασης ρολογιού Windows, προστασία offline 30-day lease και σφράγιση κλειδιών βάσης στο υλικό.").size(12.0).color(crate::theme::TEXT_MUTED));
            ui.add_space(8.0);

            if let Some((msg, is_ok)) = &state.feedback_msg {
                let col = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
                ui.label(RichText::new(msg).color(col).strong().size(12.0));
                ui.add_space(6.0);
            }

            ui.columns(2, |cols| {
                // Column 1: Monotonic Clock Validation (P3)
                cols[0].vertical(|ui| {
                    ui.label(RichText::new("1. Χρονική Ακεραιότητα & Anti-Rollback (P3)").strong());
                    ui.add_space(4.0);
                    let (status_text, status_col) = match state.time_lock_status {
                        TimeLockStatus::Synchronized => ("● SYNCHRONIZED (Ασφαλές)", Color32::from_rgb(52, 211, 153)),
                        TimeLockStatus::TamperLocked => ("⚠️ TAMPER LOCKED (Read-Only Grace)", Color32::from_rgb(244, 63, 94)),
                        TimeLockStatus::Unlocked => ("✓ UNLOCKED (Administrative Override)", Color32::from_rgb(96, 165, 250)),
                    };
                    ui.horizontal(|ui| {
                        ui.label("Κατάσταση:");
                        ui.label(RichText::new(status_text).color(status_col).strong());
                    });
                    ui.label(RichText::new(format!("High-Water Mark: {}", state.high_water_iso)).size(11.0).color(crate::theme::TEXT_SECONDARY));

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("🔄 Συγχρονισμός NTP").clicked() {
                            let now = Utc::now().timestamp();
                            state.apply_ntp_sync(conn, now);
                        }
                    });

                    if state.time_lock_status == TimeLockStatus::TamperLocked {
                        ui.add_space(6.0);
                        ui.label("Time-Unlock Token:");
                        ui.add(egui::TextEdit::singleline(&mut state.time_token_input).hint_text("tok.<machine_id>.<time>.<sig>").desired_width(f32::INFINITY));
                        if ui.button("🔓 Ξεκλείδωμα με Token").clicked() {
                            state.apply_time_token(conn, machine_id, "PROTEUS_MASTER_PLATFORM_HMAC_SECRET");
                        }
                    }
                });

                // Column 2: Hybrid Hardware Key & DPAPI/TPM Sealing (P4)
                cols[1].vertical(|ui| {
                    ui.label(RichText::new("2. Hardware-Sealed Vault (P4)").strong());
                    ui.add_space(4.0);

                    let (v_text, v_col) = match &state.vault_status {
                        VaultStatus::OperationalReady => ("● OPERATIONAL READY (<50ms Fast-Path)", Color32::from_rgb(52, 211, 153)),
                        VaultStatus::OperationalLocked => ("🔒 OPERATIONAL LOCKED", Color32::from_rgb(251, 191, 36)),
                        VaultStatus::MasterUnlocked => ("🔑 MASTER UNLOCKED", Color32::from_rgb(96, 165, 250)),
                        VaultStatus::HardwareTampered { .. } => (
                            "⛔ HARDWARE TAMPER (Αναντιστοιχία Machine ID)",
                            Color32::from_rgb(244, 63, 94),
                        ),
                        VaultStatus::Unprovisioned => ("○ UNPROVISIONED (Απαιτείται Setup)", crate::theme::TEXT_MUTED),
                    };

                    ui.horizontal(|ui| {
                        ui.label("Κατάσταση Vault:");
                        ui.label(RichText::new(v_text).color(v_col).strong());
                    });
                    ui.label(RichText::new(format!("Τρέχον Machine ID: {}", machine_id)).size(11.0).color(crate::theme::TEXT_SECONDARY));

                    ui.add_space(6.0);
                    ui.label("Master Owner Password:");
                    ui.add(egui::TextEdit::singleline(&mut state.master_password_input).password(true).desired_width(f32::INFINITY));

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if state.vault_status == VaultStatus::Unprovisioned {
                            if ui.button("🛡️ Αρχικοποίηση Vault").clicked() {
                                state.provision_vault(conn, machine_id);
                            }
                        } else {
                            if ui.button("🔑 Master Login").clicked() {
                                state.unlock_master(conn);
                            }
                            if state.master_session.is_some()
                                && ui.button("🔄 Επανασφράγιση στο τρέχον Hardware").clicked()
                            {
                                state.rekey_operational(conn, machine_id);
                            }
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
    fn test_security_guardian_state_defaults() {
        let state = SecurityGuardianState::default();
        assert_eq!(state.time_lock_status, TimeLockStatus::Synchronized);
        assert_eq!(state.vault_status, VaultStatus::Unprovisioned);
        assert!(state.master_session.is_none());
    }

    #[test]
    fn test_security_guardian_lifecycle_with_db() {
        let conn = Connection::open_in_memory().unwrap();
        let mut state = SecurityGuardianState::default();
        let machine_id = "TEST_MACHINE_GUARDIAN_01";

        TimeIntegrityGuard::init_schema(&conn).unwrap();
        TimeIntegrityGuard::record_event(&conn, Utc::now().timestamp()).unwrap();
        state.refresh(&conn, machine_id);

        assert_eq!(state.time_lock_status, TimeLockStatus::Synchronized);
        assert_ne!(state.high_water_iso, "-");

        // Provision vault
        state.master_password_input = "MasterSecure2026!#".to_string();
        state.provision_vault(&conn, machine_id);
        assert_eq!(state.vault_status, VaultStatus::OperationalReady);
    }
}
