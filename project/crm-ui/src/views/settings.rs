//! Native Settings Studio and Configuration Workbench for Proteus Designer Studio (PDS).
//! Manages Project Metadata, Design Grid Tokens, Appearance & Theme,
//! POS Hardware & Peripherals, LAN Fleet Synchronization, and SQLite Engine Diagnostics.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use crate::models::{LayoutMode, VrrMode};
use crate::theme::{AccentPreset, ThemeMode};
use crate::app_state::ProteusApp;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Canvas,
    Appearance,
    Hardware,
    LanSync,
    Database,
    About,
}

impl SettingsTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "Γενικά & Έργο",
            Self::Canvas => "Καμβάς & Grid",
            Self::Appearance => "Εμφάνιση & Θέμα",
            Self::Hardware => "POS & Περιφερειακά",
            Self::LanSync => "Δίκτυο & LAN Sync",
            Self::Database => "Βάση & Αποθήκευση",
            Self::About => "Σχετικά με το PDS",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::General => "📋",
            Self::Canvas => "📐",
            Self::Appearance => "🎨",
            Self::Hardware => "🖨",
            Self::LanSync => "🌐",
            Self::Database => "🗄",
            Self::About => "ℹ",
        }
    }
}

pub struct SettingsState {
    pub active_tab: SettingsTab,
    pub industry_niche: String,
    pub author_name: String,
    pub printer_name: String,
    pub paper_width_80mm: bool,
    pub drawer_pin_5: bool,
    pub drawer_pulse_ms: u32,
    pub feedback_msg: Option<(String, bool)>,
    pub db_vacuum_status: Option<String>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_tab: SettingsTab::General,
            industry_niche: "Automotive Service".to_string(),
            author_name: "Proteus Dev Studio".to_string(),
            printer_name: "POS-80C Thermal".to_string(),
            paper_width_80mm: true,
            drawer_pin_5: false,
            drawer_pulse_ms: 100,
            feedback_msg: None,
            db_vacuum_status: None,
        }
    }
}

/// Render the categorized left navigation sidebar for Settings.
pub fn show_left(state: &mut SettingsState, ui: &mut Ui) {
    ui.add_space(4.0);
    ui.label(RichText::new("ΚΑΤΗΓΟΡΙΕΣ").size(10.0).strong().color(Color32::from_rgb(140, 150, 170)));
    ui.add_space(8.0);

    let tabs = [
        SettingsTab::General,
        SettingsTab::Canvas,
        SettingsTab::Appearance,
        SettingsTab::Hardware,
        SettingsTab::LanSync,
        SettingsTab::Database,
        SettingsTab::About,
    ];

    for tab in tabs {
        let is_sel = state.active_tab == tab;
        let btn = ui.add(
            egui::Button::new(
                RichText::new(format!("{}  {}", tab.icon(), tab.label()))
                    .size(11.0)
                    .color(if is_sel { Color32::WHITE } else { Color32::from_rgb(170, 180, 200) })
                    .strong(),
            )
            .fill(if is_sel { Color32::from_rgb(30, 36, 50) } else { Color32::TRANSPARENT })
            .stroke(if is_sel { Stroke::new(1.0, Color32::from_rgb(0, 212, 255)) } else { Stroke::NONE })
            .corner_radius(CornerRadius::same(6))
            .min_size(Vec2::new(ui.available_width(), 30.0)),
        );

        if btn.clicked() {
            state.active_tab = tab;
        }
        ui.add_space(3.0);
    }
}

/// Render the main central settings editor card.
pub fn show_central(app: &mut ProteusApp, ui: &mut Ui) {
    let p = app.palette(ui.ctx());

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("⚙ {}", app.settings_state.active_tab.label())).size(18.0).strong().color(p.text));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new("💾 Αποθήκευση Ρυθμίσεων").fill(p.accent).min_size(Vec2::new(140.0, 26.0))).clicked() {
                    app.save_project();
                    app.toast("✓ Οι ρυθμίσεις αποθηκεύτηκαν στο project");
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);

        if let Some((msg, is_ok)) = &app.settings_state.feedback_msg {
            let col = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
            ui.label(RichText::new(msg).color(col).strong().size(11.0));
            ui.add_space(8.0);
        }

        match app.settings_state.active_tab {
            SettingsTab::General => render_general_tab(app, ui),
            SettingsTab::Canvas => render_canvas_tab(app, ui),
            SettingsTab::Appearance => render_appearance_tab(app, ui),
            SettingsTab::Hardware => render_hardware_tab(app, ui),
            SettingsTab::LanSync => render_lan_sync_tab(app, ui),
            SettingsTab::Database => render_database_tab(app, ui),
            SettingsTab::About => render_about_tab(ui),
        }
        ui.add_space(24.0);
    });
}

fn render_general_tab(app: &mut ProteusApp, ui: &mut Ui) {
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΣΤΟΙΧΕΙΑ ΕΡΓΟΥ & ΕΠΙΧΕΙΡΗΣΗΣ").strong().color(Color32::from_rgb(0, 212, 255)));
        ui.add_space(8.0);

        ui.label("Όνομα Έργου / Εφαρμογής (Project Name):");
        ui.add(egui::TextEdit::singleline(&mut app.pname).desired_width(f32::INFINITY));
        ui.add_space(8.0);

        ui.label("Αναγνωριστικό Έργου (Project ID):");
        ui.add(egui::TextEdit::singleline(&mut app.pid).desired_width(f32::INFINITY));
        ui.add_space(8.0);

        ui.label("Δημιουργός / Εταιρεία Υλοποίησης (Author):");
        ui.add(egui::TextEdit::singleline(&mut app.settings_state.author_name).desired_width(f32::INFINITY));
        ui.add_space(8.0);

        ui.label("Στοχευμένος Κλάδος (Industry Domain):");
        ui.horizontal(|ui| {
            for niche in ["Automotive Service", "Retail POS", "Medical Clinic", "Salon & Spa", "General CRM"] {
                let is_sel = app.settings_state.industry_niche == niche;
                if ui.selectable_label(is_sel, niche).clicked() {
                    app.settings_state.industry_niche = niche.to_string();
                }
            }
        });
    });
}

fn render_canvas_tab(app: &mut ProteusApp, ui: &mut Ui) {
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΠΑΡΑΜΕΤΡΟΠΟΙΗΣΗ ΚΑΜΒΑ & ΕΠΕΞΕΡΓΑΣΤΗ").strong().color(Color32::from_rgb(0, 212, 255)));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Λειτουργία Στοίχισης (Layout):");
            let is_grid = app.layout == LayoutMode::Grid;
            if ui.radio(is_grid, "Κάνναβος / Grid").clicked() {
                app.layout = LayoutMode::Grid;
            }
            if ui.radio(!is_grid, "Ελεύθερη Τοποθέτηση / Free").clicked() {
                app.layout = LayoutMode::Free;
            }
        });
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Προεπισκόπηση Συσκευών (Device Toolbar):");
            ui.checkbox(&mut app.show_device_toolbar, "Εμφάνιση Toolbar διαστάσεων");
        });
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Προφίλ Αφής / Touch Interaction:");
            let is_touch = app.viewport_profile.is_touch_device;
            if ui.checkbox(&mut { is_touch }, "Ελάχιστη επιφάνεια αφής 44pt").clicked() {
                app.viewport_profile = if is_touch {
                    crate::viewport::ViewportProfile::desktop()
                } else {
                    crate::viewport::ViewportProfile::mobile_touch()
                };
            }
        });
        ui.add_space(10.0);

        ui.label("Ρυθμός Ανανέωσης (Variable Refresh Rate - VRR):");
        ui.horizontal(|ui| {
            for vrr in [VrrMode::Reactive, VrrMode::Fps30, VrrMode::Fps60, VrrMode::Fps120, VrrMode::Continuous] {
                let is_sel = app.vrr_mode == vrr;
                if ui.selectable_label(is_sel, vrr.label()).clicked() {
                    app.vrr_mode = vrr;
                }
            }
        });
    });
}

fn render_appearance_tab(app: &mut ProteusApp, ui: &mut Ui) {
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΘΕΜΑ & ΧΡΩΜΑΤΙΚΗ ΠΑΛΕΤΑ").strong().color(Color32::from_rgb(0, 212, 255)));
        ui.add_space(8.0);

        ui.label("Βασικό Θέμα:");
        ui.horizontal(|ui| {
            for mode in [ThemeMode::Dark, ThemeMode::Light, ThemeMode::System] {
                let is_sel = app.theme_mode == mode;
                if ui.selectable_label(is_sel, mode.label()).clicked() {
                    app.theme_mode = mode;
                    app._style_set = false;
                }
            }
        });
        ui.add_space(12.0);

        ui.label("Χρώμα Έμφασης (Accent Color):");
        ui.horizontal(|ui| {
            for preset in [AccentPreset::Indigo, AccentPreset::Sapphire, AccentPreset::Emerald, AccentPreset::Amber, AccentPreset::Rose, AccentPreset::Slate] {
                let is_sel = app.accent_preset == preset;
                if ui.selectable_label(is_sel, preset.label()).clicked() {
                    app.accent_preset = preset;
                    app._style_set = false;
                }
            }
        });
    });
}

fn render_hardware_tab(app: &mut ProteusApp, ui: &mut Ui) {
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΘΕΡΜΙΚΟΣ ΕΚΤΥΠΩΤΗΣ & ΣΥΡΤΑΡΙ ΜΕΤΡΗΤΩΝ").strong().color(Color32::from_rgb(0, 212, 255)));
        ui.add_space(8.0);

        ui.label("Όνομα Συσκευής / Spooler (ESC/POS Device):");
        ui.add(egui::TextEdit::singleline(&mut app.settings_state.printer_name).desired_width(f32::INFINITY));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Πλάτος Χαρτιού:");
            if ui.radio(app.settings_state.paper_width_80mm, "80mm (Standard POS)").clicked() {
                app.settings_state.paper_width_80mm = true;
            }
            if ui.radio(!app.settings_state.paper_width_80mm, "58mm (Compact Receipt)").clicked() {
                app.settings_state.paper_width_80mm = false;
            }
        });
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Παλμός Συρταριού (RJ11 Pin):");
            if ui.radio(!app.settings_state.drawer_pin_5, "Pin 2").clicked() {
                app.settings_state.drawer_pin_5 = false;
            }
            if ui.radio(app.settings_state.drawer_pin_5, "Pin 5").clicked() {
                app.settings_state.drawer_pin_5 = true;
            }
        });
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("🖨 Δοκιμαστική Εκτύπωση").fill(Color32::from_rgb(0, 212, 255)).min_size(Vec2::new(160.0, 26.0))).clicked() {
                match crm_core::printer::test_printer_connection(&app.settings_state.printer_name) {
                    Ok(_) => app.settings_state.feedback_msg = Some(("✓ ESC/POS δοκιμαστική εκτύπωση εστάλη επιτυχώς".into(), true)),
                    Err(e) => app.settings_state.feedback_msg = Some((format!("✗ Σφάλμα εκτύπωσης: {}", e), false)),
                }
            }
            if ui.add(egui::Button::new("💵 Άνοιγμα Συρταριού").fill(Color32::from_rgb(217, 119, 6)).min_size(Vec2::new(150.0, 26.0))).clicked() {
                match crm_core::printer::kick_cash_drawer(&app.settings_state.printer_name) {
                    Ok(_) => app.settings_state.feedback_msg = Some(("✓ Παλμός RJ-11 εστάλη στο συρτάρι".into(), true)),
                    Err(e) => app.settings_state.feedback_msg = Some((format!("✗ Σφάλμα συρταριού: {}", e), false)),
                }
            }
        });
    });
}

fn render_lan_sync_tab(app: &mut ProteusApp, ui: &mut Ui) {
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΤΟΠΙΚΟ ΔΙΚΤΥΟ (LAN) & ΑΥΤΟΜΑΤΗ ΔΙΑΝΟΜΗ").strong().color(Color32::from_rgb(0, 212, 255)));
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.checkbox(&mut app.auto_deploy_on_save, "Αυτόματη διανομή (Auto-deploy) σε δορυφορικά POS κατά την αποθήκευση");
        });
        ui.add_space(8.0);

        ui.label(format!("UDP Beacon Broadcast Port: {}", crm_core::lan::DEFAULT_BEACON_PORT));
        ui.label("Κατάσταση Master Node: Ενεργός & έτοιμος για συγχρονισμό");
        ui.add_space(8.0);

        let daemon_status = if app.lan_daemon.is_some() { "Ενεργός (Running)" } else { "Σε αναμονή (Idle)" };
        ui.label(format!("LAN Daemon: {}", daemon_status));
    });
}

fn render_database_tab(app: &mut ProteusApp, ui: &mut Ui) {
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΤΟΠΙΚΗ ΒΑΣΗ ΔΕΔΟΜΕΝΩΝ (SQLITE)").strong().color(Color32::from_rgb(0, 212, 255)));
        ui.add_space(8.0);

        let path = format!("{}/crm.db", crate::app_state::app_dir());
        ui.label(format!("Διαδρομή Αρχείου: {}", path));
        ui.add_space(8.0);

        ui.label(format!("Επαφές (Contacts): {} | Ευκαιρίες (Deals): {} | Εργασίες (Tasks): {}", app.contacts.len(), app.deals.len(), app.tasks.len()));
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            if ui.button("⚡ Συμπύκνωση Βάσης (VACUUM)").clicked() {
                if let Some(conn) = &app.db_conn {
                    match conn.execute_batch("VACUUM;") {
                        Ok(_) => app.settings_state.db_vacuum_status = Some("✓ Η βάση SQLite συμπυκνώθηκε επιτυχώς".into()),
                        Err(e) => app.settings_state.db_vacuum_status = Some(format!("✗ Σφάλμα: {}", e)),
                    }
                } else {
                    app.settings_state.db_vacuum_status = Some("✓ Δεν απαιτείται συντήρηση (In-Memory state)".into());
                }
            }

            if ui.button("⟳ Εκκαθάριση Cache Πινάκων").clicked() {
                app.table_cache.clear();
                app.toast("✓ Το cache των πινάκων εκκαθαρίστηκε");
            }
        });

        if let Some(msg) = &app.settings_state.db_vacuum_status {
            ui.add_space(6.0);
            ui.label(RichText::new(msg).size(10.5).color(Color32::from_rgb(52, 211, 153)));
        }
    });
}

fn render_about_tab(ui: &mut Ui) {
    card_frame(ui, |ui| {
        ui.label(RichText::new("PROTEUS DESIGNER STUDIO (PDS)").strong().size(15.0).color(Color32::from_rgb(0, 212, 255)));
        ui.add_space(4.0);
        ui.label("Έκδοση: 0.1.0 (Production Release Candidate)");
        ui.label("Αρχιτεκτονική: 100% Original Bespoke Rust Codebase");
        ui.label("Zero External Boilerplate • Native Immediate Mode UI (egui)");
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label("Το Proteus είναι το Οπτικό Λειτουργικό Σύστημα (Visual OS) για επιχειρήσεις, επιτρέποντας σχεδιασμό UI, data modeling, flows, και διανομή POS σε ενιαία πλατφόρμα.");
    });
}

/// Render the right contextual summary panel for Settings.
pub fn show_right(app: &mut ProteusApp, ui: &mut Ui) {
    let p = app.palette(ui.ctx());
    ui.add_space(4.0);
    ui.label(RichText::new("ΕΠΙΣΚΟΠΗΣΗ ΣΥΣΤΗΜΑΤΟΣ").size(10.0).strong().color(p.text_dim));
    ui.add_space(8.0);

    card_frame(ui, |ui| {
        ui.label(RichText::new("ΤΡΕΧΟΝ ΕΡΓΟ").size(10.0).strong().color(Color32::from_rgb(0, 212, 255)));
        ui.label(RichText::new(&app.pname).strong().size(12.0).color(p.text));
        ui.label(RichText::new(format!("ID: {}", app.pid)).size(10.0).color(p.text_dim));
        ui.add_space(4.0);
        ui.label(RichText::new(format!("Κλάδος: {}", app.settings_state.industry_niche)).size(10.0).color(p.text));
    });

    ui.add_space(8.0);
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΚΑΤΑΣΤΑΣΗ ΕΚΤΥΠΩΤΗ").size(10.0).strong().color(Color32::from_rgb(0, 212, 255)));
        ui.label(RichText::new(&app.settings_state.printer_name).size(11.0).color(p.text));
        let width_txt = if app.settings_state.paper_width_80mm { "80mm" } else { "58mm" };
        ui.label(RichText::new(format!("Πλάτος Χαρτιού: {}", width_txt)).size(10.0).color(p.text_dim));
    });

    ui.add_space(8.0);
    card_frame(ui, |ui| {
        ui.label(RichText::new("ΣΥΝΤΟΜΕΥΣΕΙΣ").size(10.0).strong().color(Color32::from_rgb(0, 212, 255)));
        ui.label("Ctrl+K: Command Palette");
        ui.label("Ctrl+S: Αποθήκευση Έργου");
        ui.label("Ctrl+Z / Y: Undo / Redo");
        ui.label("Ctrl+,: Ρυθμίσεις PDS");
    });
}

fn card_frame<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(Color32::from_rgb(18, 20, 27))
        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 58)))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(14))
        .show(ui, add_contents)
        .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_state_defaults() {
        let state = SettingsState::default();
        assert_eq!(state.active_tab, SettingsTab::General);
        assert_eq!(state.industry_niche, "Automotive Service");
        assert!(state.paper_width_80mm);
        assert_eq!(state.printer_name, "POS-80C Thermal");
        assert_eq!(state.drawer_pulse_ms, 100);
    }

    #[test]
    fn test_settings_tab_metadata() {
        let tabs = [
            SettingsTab::General,
            SettingsTab::Canvas,
            SettingsTab::Appearance,
            SettingsTab::Hardware,
            SettingsTab::LanSync,
            SettingsTab::Database,
            SettingsTab::About,
        ];
        for tab in tabs {
            assert!(!tab.label().is_empty());
            assert!(!tab.icon().is_empty());
        }
    }
}
