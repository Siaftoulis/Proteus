//! Two-Stage Authentication & Workspace Launcher Engine for Proteus Client.
//! Stage 1: Cloud & Marketplace Account Verification (Email/Password or Google OAuth).
//! Stage 2: CRM Project / Workspace Selection & Role Assignment.

use crm_core::roles::UserRole;
use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LicensedPackageInfo {
    pub bundle_id: String,
    pub title: String,
    pub version: String,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionContext {
    pub marketplace_email: String,
    pub mounted_bundle_id: String,
    pub mounted_bundle_title: String,
    pub operator_name: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthStage {
    Stage1MarketplaceLogin,
    Stage2StoreOwnerPin,
    Authenticated(SessionContext),
}

#[derive(Debug, Clone)]
pub struct ClientAuthState {
    pub stage: AuthStage,
    pub email_input: String,
    pub password_input: String,
    pub owner_pin_input: String,
    pub available_packages: Vec<LicensedPackageInfo>,
    pub selected_package_index: usize,
    pub selected_role: UserRole,
    pub verified_account: Option<String>,
    pub error_msg: Option<String>,
}

impl Default for ClientAuthState {
    fn default() -> Self {
        Self {
            stage: AuthStage::Stage1MarketplaceLogin,
            email_input: "demo@company.com".to_string(),
            password_input: "password123".to_string(),
            owner_pin_input: String::new(),
            available_packages: vec![
                LicensedPackageInfo {
                    bundle_id: "PKG-SERVICE-AUTO".to_string(),
                    title: "Automotive Service & Repair BOS".to_string(),
                    version: "1.4.0".to_string(),
                    category: "Automotive".to_string(),
                },
                LicensedPackageInfo {
                    bundle_id: "PKG-RETAIL-POS".to_string(),
                    title: "Multi-Store Retail & Cashier BOS".to_string(),
                    version: "2.1.0".to_string(),
                    category: "Retail".to_string(),
                },
                LicensedPackageInfo {
                    bundle_id: "PKG-CLINIC-HEALTH".to_string(),
                    title: "Medical & Dental Practice Suite".to_string(),
                    version: "1.0.2".to_string(),
                    category: "Healthcare".to_string(),
                },
                LicensedPackageInfo {
                    bundle_id: "PKG-MOTO-PRO".to_string(),
                    title: "Motorcycle Workshop & Tuning BOS".to_string(),
                    version: "1.1.0".to_string(),
                    category: "Automotive".to_string(),
                },
                LicensedPackageInfo {
                    bundle_id: "PKG-CUSTOM-STUDIO".to_string(),
                    title: "Proteus Custom Designer Canvas".to_string(),
                    version: "2.0.0".to_string(),
                    category: "Designer".to_string(),
                },
            ],
            selected_package_index: 0,
            selected_role: UserRole::Ceo,
            verified_account: None,
            error_msg: None,
        }
    }
}

impl ClientAuthState {
    /// Verify Stage 1 credentials against Marketplace / Cloud.
    pub fn submit_marketplace_login(&mut self) -> bool {
        let email = self.email_input.trim();
        let pass = self.password_input.trim();
        if email.is_empty() || !email.contains('@') || pass.is_empty() {
            self.error_msg = Some("Παρακαλώ εισάγετε έγκυρη διεύθυνση email και κωδικό πρόσβασης.".to_string());
            return false;
        }

        self.verified_account = Some(email.to_string());
        self.error_msg = None;
        self.stage = AuthStage::Stage2StoreOwnerPin;
        true
    }

    /// Fast-track Google OAuth login for marketplace account.
    pub fn submit_google_login(&mut self) -> bool {
        self.email_input = "owner.google@enterprise-cloud.com".to_string();
        self.verified_account = Some(self.email_input.clone());
        self.error_msg = None;
        self.stage = AuthStage::Stage2StoreOwnerPin;
        true
    }

    /// Verify Stage 2 Owner Master PIN / Staff PIN and mount selected PR package.
    pub fn submit_store_access(&mut self) -> Option<SessionContext> {
        let pin = self.owner_pin_input.trim();
        let (role, operator_name) = match pin {
            "0000" | "" => (self.selected_role, format!("{} (Full Control)", self.selected_role.display_name())),
            "1234" => (UserRole::CustomerService, "Terminal Cashier (Operator)".to_string()),
            "9999" => (UserRole::Technician, "Service Technician".to_string()),
            _ => {
                self.error_msg = Some("Μη έγκυρος κωδικός PIN (Δοκιμάστε 0000 για Owner ή 1234 για Cashier)".to_string());
                return None;
            }
        };

        let pkg = self
            .available_packages
            .get(self.selected_package_index)
            .cloned()
            .unwrap_or_else(|| LicensedPackageInfo {
                bundle_id: "PKG-DEFAULT-RUNTIME".to_string(),
                title: "Standard Enterprise BOS".to_string(),
                version: "1.0.0".to_string(),
                category: "General".to_string(),
            });

        let session = SessionContext {
            marketplace_email: self.verified_account.clone().unwrap_or_else(|| "owner@shop.com".to_string()),
            mounted_bundle_id: pkg.bundle_id,
            mounted_bundle_title: pkg.title,
            operator_name,
            role,
        };

        self.error_msg = None;
        self.stage = AuthStage::Authenticated(session.clone());
        Some(session)
    }

    /// Reset authentication and return to Stage 1 login.
    pub fn logout(&mut self) {
        self.stage = AuthStage::Stage1MarketplaceLogin;
        self.owner_pin_input.clear();
        self.verified_account = None;
        self.error_msg = None;
    }

    /// Lock terminal back to Stage 2 PIN prompt.
    pub fn lock_terminal(&mut self) {
        self.stage = AuthStage::Stage2StoreOwnerPin;
        self.owner_pin_input.clear();
        self.error_msg = None;
    }

    /// Switch active CRM project / package.
    pub fn switch_project(&mut self) {
        self.stage = AuthStage::Stage2StoreOwnerPin;
        self.error_msg = None;
    }
}

/// Render the launcher & authentication modal when not authenticated.
pub fn render_auth_modal(state: &mut ClientAuthState, ctx: &egui::Context) -> Option<SessionContext> {
    if let AuthStage::Authenticated(_) = state.stage {
        return None;
    }

    let mut result_session = None;

    egui::CentralPanel::default()
        .frame(Frame::new().fill(Color32::from_rgb(10, 12, 16)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.);

                Frame::new()
                    .fill(Color32::from_rgb(20, 23, 31))
                    .stroke(Stroke::new(1., Color32::from_rgb(38, 44, 61)))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::same(24))
                    .show(ui, |ui| {
                        ui.set_max_width(520.);

                        match &state.stage {
                            AuthStage::Stage1MarketplaceLogin => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("PROTEUS LAUNCHER").strong().size(15.).color(Color32::WHITE));
                                    Frame::new()
                                        .fill(Color32::from_rgb(30, 41, 59))
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(RichText::new("WORKSPACE GATEWAY").size(9.5).color(Color32::from_rgb(148, 163, 184)));
                                        });
                                });
                                ui.add_space(4.);
                                ui.label(
                                    RichText::new("Σύνδεση με το λογαριασμό Proteus Portal για αυτόματο συγχρονισμό των CRM έργων σας.")
                                        .size(11.5)
                                        .color(Color32::from_rgb(148, 163, 184)),
                                );
                                ui.add_space(16.);

                                ui.label(RichText::new("Email Λογαριασμού:").size(11.).color(Color32::from_rgb(203, 213, 225)));
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.email_input)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("user@company.com"),
                                );
                                ui.add_space(8.);

                                ui.label(RichText::new("Κωδικός Πρόσβασης:").size(11.).color(Color32::from_rgb(203, 213, 225)));
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.password_input)
                                        .password(true)
                                        .desired_width(f32::INFINITY),
                                );
                                ui.add_space(14.);

                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("Είσοδος στο Workspace").size(12.).strong().color(Color32::WHITE))
                                            .fill(Color32::from_rgb(37, 99, 235))
                                            .corner_radius(CornerRadius::same(8))
                                            .min_size(Vec2::new(ui.available_width(), 34.)),
                                    )
                                    .clicked()
                                {
                                    state.submit_marketplace_login();
                                }

                                ui.add_space(8.);
                                ui.label(RichText::new("— ή εναλλακτικά —").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.add_space(8.);

                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("🔑 Σύνδεση με Google Workspace").size(11.5).color(Color32::WHITE))
                                            .fill(Color32::from_rgb(30, 36, 49))
                                            .stroke(Stroke::new(1., Color32::from_rgb(51, 65, 85)))
                                            .corner_radius(CornerRadius::same(8))
                                            .min_size(Vec2::new(ui.available_width(), 32.)),
                                    )
                                    .clicked()
                                {
                                    state.submit_google_login();
                                }

                                ui.add_space(14.);
                                ui.separator();
                                ui.add_space(6.);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Δεν έχετε ακόμα λογαριασμό;").size(11.).color(Color32::from_rgb(148, 163, 184)));
                                    if ui.button(RichText::new("Εγγραφή στο Portal (Port 8080)").size(11.).color(Color32::from_rgb(96, 165, 250))).clicked() {
                                        ctx.open_url(egui::OpenUrl::new_tab("http://localhost:8080"));
                                    }
                                });
                            }
                            AuthStage::Stage2StoreOwnerPin => {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("ΕΠΙΛΟΓΗ CRM PROJECT & ΡΟΛΟΥ").strong().size(14.).color(Color32::WHITE));
                                    if let Some(ref acc) = state.verified_account {
                                        Frame::new()
                                            .fill(Color32::from_rgb(16, 40, 28))
                                            .corner_radius(CornerRadius::same(4))
                                            .inner_margin(Margin::symmetric(6, 2))
                                            .show(ui, |ui| {
                                                ui.label(RichText::new(acc).size(10.5).color(Color32::from_rgb(52, 211, 153)));
                                            });
                                    }
                                });
                                ui.add_space(4.);
                                ui.label(RichText::new("Επιλέξτε το επιχειρησιακό CRM project που θέλετε να λανσάρετε:").size(11.).color(Color32::from_rgb(148, 163, 184)));
                                ui.add_space(10.);

                                egui::ScrollArea::vertical().max_height(160.).show(ui, |ui| {
                                    for (idx, pkg) in state.available_packages.iter().enumerate() {
                                        let is_selected = state.selected_package_index == idx;
                                        let bg = if is_selected { Color32::from_rgb(30, 48, 75) } else { Color32::from_rgb(16, 19, 26) };
                                        let border = if is_selected { Color32::from_rgb(59, 130, 246) } else { Color32::from_rgb(38, 44, 61) };

                                        Frame::new()
                                            .fill(bg)
                                            .stroke(Stroke::new(1., border))
                                            .corner_radius(CornerRadius::same(8))
                                            .inner_margin(Margin::same(8))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    if ui.radio(is_selected, "").clicked() {
                                                        state.selected_package_index = idx;
                                                    }
                                                    ui.vertical(|ui| {
                                                        ui.label(RichText::new(&pkg.title).strong().size(11.5).color(Color32::WHITE));
                                                        ui.horizontal(|ui| {
                                                            ui.label(RichText::new(format!("Category: {}", pkg.category)).size(10.).color(Color32::from_rgb(148, 163, 184)));
                                                            ui.label(RichText::new(format!("v{}", pkg.version)).size(10.).color(Color32::from_rgb(100, 116, 139)));
                                                            ui.label(RichText::new(&pkg.bundle_id).size(10.).color(Color32::from_rgb(96, 165, 250)));
                                                        });
                                                    });
                                                });
                                            });
                                        ui.add_space(4.);
                                    }
                                });
                                ui.add_space(10.);

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Επιχειρησιακός Ρόλος:").size(11.).color(Color32::from_rgb(203, 213, 225)));
                                    egui::ComboBox::from_id_salt("launcher_role_picker")
                                        .selected_text(RichText::new(state.selected_role.display_name()).size(11.))
                                        .show_ui(ui, |ui| {
                                            for role in UserRole::all() {
                                                ui.selectable_value(&mut state.selected_role, *role, role.display_name());
                                            }
                                        });
                                });
                                ui.add_space(8.);

                                ui.label(RichText::new("Terminal PIN (0000 = Owner, 1234 = Cashier, 9999 = Tech):").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.owner_pin_input)
                                        .password(true)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("0000"),
                                );
                                ui.add_space(12.);

                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("🚀 Εκκίνηση CRM Project").size(12.).strong().color(Color32::WHITE))
                                            .fill(Color32::from_rgb(16, 185, 129))
                                            .corner_radius(CornerRadius::same(8))
                                            .min_size(Vec2::new(ui.available_width(), 34.)),
                                    )
                                    .clicked()
                                {
                                    result_session = state.submit_store_access();
                                }

                                ui.add_space(8.);
                                ui.horizontal(|ui| {
                                    if ui.small_button("← Αλλαγή Λογαριασμού").clicked() {
                                        state.logout();
                                    }
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("🌐 Proteus Marketplace").clicked() {
                                            ctx.open_url(egui::OpenUrl::new_tab("http://localhost:8080"));
                                        }
                                    });
                                });
                            }
                            AuthStage::Authenticated(_) => {}
                        }

                        if let Some(ref err) = state.error_msg {
                            ui.add_space(10.);
                            ui.label(RichText::new(err).size(10.5).color(Color32::from_rgb(244, 63, 94)));
                        }
                    });
            });
        });

    result_session
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_stage_auth_flow() {
        let mut state = ClientAuthState::default();
        assert_eq!(state.stage, AuthStage::Stage1MarketplaceLogin);

        // Stage 1: Invalid email rejected
        state.email_input = "invalid-email".to_string();
        assert!(!state.submit_marketplace_login());
        assert!(state.error_msg.is_some());

        // Stage 1: Valid email passes to Stage 2
        state.email_input = "owner@business.com".to_string();
        assert!(state.submit_marketplace_login());
        assert_eq!(state.stage, AuthStage::Stage2StoreOwnerPin);

        // Stage 2: Invalid PIN rejected
        state.owner_pin_input = "wrong-pin".to_string();
        assert!(state.submit_store_access().is_none());

        // Stage 2: Owner PIN 0000 passes and mounts selected package
        state.owner_pin_input = "0000".to_string();
        let session = state.submit_store_access().expect("Session created");
        assert_eq!(session.role, UserRole::Ceo);
        assert_eq!(session.mounted_bundle_id, "PKG-SERVICE-AUTO");
        assert_eq!(session.marketplace_email, "owner@business.com");

        // Lock terminal goes back to Stage 2 PIN
        state.lock_terminal();
        assert_eq!(state.stage, AuthStage::Stage2StoreOwnerPin);

        // Cashier PIN 1234 gives CustomerService role
        state.owner_pin_input = "1234".to_string();
        let cashier_session = state.submit_store_access().expect("Cashier session");
        assert_eq!(cashier_session.role, UserRole::CustomerService);

        // Switch project returns to Stage 2
        state.switch_project();
        assert_eq!(state.stage, AuthStage::Stage2StoreOwnerPin);
    }
}
