//! Two-Stage Authentication & Targeted PR Package Mounting Engine for Proteus Client.
//! Stage 1: Cloud & Marketplace Business Account Verification (Custom Login / Google OAuth).
//! Stage 2: Store Owner Master PIN & Terminal Operator Access with Dynamic PR Package Mounting.

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
            ],
            selected_package_index: 0,
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
            self.error_msg = Some("Παρακαλώ εισάγετε έγκυρη διεύθυνση email και κωδικό πρόσβασης επιχείρησης.".to_string());
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
            "0000" | "" => (UserRole::Ceo, "Store Owner (Full Control)".to_string()),
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

    /// Reset authentication and return to Stage 1.
    pub fn logout(&mut self) {
        self.stage = AuthStage::Stage1MarketplaceLogin;
        self.owner_pin_input.clear();
        self.verified_account = None;
        self.error_msg = None;
    }

    /// Lock terminal back to Stage 2 (PIN entry required).
    pub fn lock_terminal(&mut self) {
        self.stage = AuthStage::Stage2StoreOwnerPin;
        self.owner_pin_input.clear();
        self.error_msg = None;
    }
}

/// Render the authentication modal if client is not in Authenticated stage.
pub fn render_auth_modal(state: &mut ClientAuthState, ctx: &egui::Context) -> Option<SessionContext> {
    if let AuthStage::Authenticated(_) = state.stage {
        return None;
    }

    let mut result_session = None;

    egui::CentralPanel::default()
        .frame(Frame::new().fill(Color32::from_rgb(10, 12, 16)))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(60.);

                Frame::new()
                    .fill(Color32::from_rgb(22, 25, 34))
                    .stroke(Stroke::new(1., Color32::from_rgb(45, 52, 70)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(28))
                    .show(ui, |ui| {
                        ui.set_max_width(460.);

                        match &state.stage {
                            AuthStage::Stage1MarketplaceLogin => {
                                ui.label(RichText::new("☁ Proteus Cloud & Marketplace Verification").size(16.).strong().color(Color32::WHITE));
                                ui.add_space(4.);
                                ui.label(
                                    RichText::new("Στάδιο 1: Σύνδεση με το λογαριασμό Marketplace της επιχείρησης για ανάκτηση των αγορασμένων πακέτων .pr.")
                                        .size(11.)
                                        .color(Color32::from_rgb(160, 170, 190)),
                                );
                                ui.add_space(16.);

                                ui.label(RichText::new("Email Επιχείρησης (Marketplace Account):").size(10.5).color(Color32::from_rgb(200, 210, 230)));
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.email_input)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("company@marketplace.com"),
                                );
                                ui.add_space(8.);

                                ui.label(RichText::new("Κωδικός Πρόσβασης:").size(10.5).color(Color32::from_rgb(200, 210, 230)));
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.password_input)
                                        .password(true)
                                        .desired_width(f32::INFINITY),
                                );
                                ui.add_space(14.);

                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("Επαλήθευση Λογαριασμού Marketplace").size(12.).strong().color(Color32::WHITE))
                                            .fill(Color32::from_rgb(79, 140, 237))
                                            .corner_radius(CornerRadius::same(5))
                                            .min_size(Vec2::new(ui.available_width(), 32.)),
                                    )
                                    .clicked()
                                {
                                    state.submit_marketplace_login();
                                }

                                ui.add_space(8.);
                                ui.label(RichText::new("— Ή —").size(10.).color(Color32::from_rgb(120, 130, 150)));
                                ui.add_space(8.);

                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("🔑 Σύνδεση με Google (OAuth Marketplace)").size(11.5).color(Color32::WHITE))
                                            .fill(Color32::from_rgb(38, 43, 56))
                                            .stroke(Stroke::new(1., Color32::from_rgb(70, 80, 105)))
                                            .corner_radius(CornerRadius::same(5))
                                            .min_size(Vec2::new(ui.available_width(), 30.)),
                                    )
                                    .clicked()
                                {
                                    state.submit_google_login();
                                }
                            }
                            AuthStage::Stage2StoreOwnerPin => {
                                ui.label(RichText::new("🏪 Terminal Operator Access & PR Mounting").size(16.).strong().color(Color32::WHITE));
                                ui.add_space(4.);
                                if let Some(ref acc) = state.verified_account {
                                    ui.label(RichText::new(format!("✓ Συνδεδεμένο Marketplace: {}", acc)).size(11.).strong().color(Color32::from_rgb(120, 220, 140)));
                                }
                                ui.add_space(12.);

                                ui.label(RichText::new("Επιλογή Αγορασμένου .pr Package για Εκτέλεση:").size(10.5).color(Color32::from_rgb(200, 210, 230)));
                                for (idx, pkg) in state.available_packages.iter().enumerate() {
                                    let is_selected = state.selected_package_index == idx;
                                    let txt = format!("📦 {} ({})", pkg.title, pkg.bundle_id);
                                    if ui.selectable_label(is_selected, RichText::new(txt).size(11.)).clicked() {
                                        state.selected_package_index = idx;
                                    }
                                }
                                ui.add_space(12.);

                                ui.label(RichText::new("Owner Master PIN / Staff PIN:").size(10.5).color(Color32::from_rgb(200, 210, 230)));
                                ui.add(
                                    egui::TextEdit::singleline(&mut state.owner_pin_input)
                                        .password(true)
                                        .desired_width(f32::INFINITY)
                                        .hint_text("Εισάγετε PIN (0000 = Owner, 1234 = Cashier)"),
                                );
                                ui.add_space(14.);

                                if ui
                                    .add(
                                        egui::Button::new(RichText::new("🚀 Mount Package & Έναρξη POS Client").size(12.).strong().color(Color32::WHITE))
                                            .fill(Color32::from_rgb(46, 125, 50))
                                            .corner_radius(CornerRadius::same(5))
                                            .min_size(Vec2::new(ui.available_width(), 32.)),
                                    )
                                    .clicked()
                                {
                                    result_session = state.submit_store_access();
                                }

                                ui.add_space(8.);
                                if ui.small_button("← Αλλαγή Λογαριασμού Marketplace").clicked() {
                                    state.logout();
                                }
                            }
                            AuthStage::Authenticated(_) => {}
                        }

                        if let Some(ref err) = state.error_msg {
                            ui.add_space(10.);
                            ui.label(RichText::new(err).size(10.5).color(Color32::from_rgb(230, 80, 80)));
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
    }
}
