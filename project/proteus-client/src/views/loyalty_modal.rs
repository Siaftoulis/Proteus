//! Frontline POS Loyalty Points & Gift Card Tender Integration Modal.
//! Bespoke implementation adhering strictly to Rule 1 & Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::loyalty::{
    init_gift_card_schema, init_loyalty_schema, issue_gift_card, lookup_gift_card,
    redeem_gift_card, redeem_points, top_up_gift_card, GiftCard, GiftCardStatus, LoyaltyAccount,
    LoyaltyPolicy, LoyaltyTier,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoyaltyModalTab {
    PointsAndTiers,
    RedeemGiftCard,
    IssueGiftCard,
    TopUpGiftCard,
}

pub struct LoyaltyModalState {
    pub is_open: bool,
    pub active_tab: LoyaltyModalTab,
    pub member_query: String,
    pub active_account: Option<LoyaltyAccount>,
    pub points_to_redeem: String,
    pub gift_card_code: String,
    pub active_gift_card: Option<GiftCard>,
    pub gift_card_amount: String,
    pub gift_card_recipient: String,
    pub gift_card_phone: String,
    pub status_message: Option<(String, bool)>,
    pub generated_code_display: Option<String>,
    pub applied_discount_cents: i64,
    pub applied_gift_card_cents: i64,
    pub schemas_ready: bool,
}

impl Default for LoyaltyModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: LoyaltyModalTab::PointsAndTiers,
            member_query: String::new(),
            active_account: None,
            points_to_redeem: String::new(),
            gift_card_code: String::new(),
            active_gift_card: None,
            gift_card_amount: "50.00".to_string(),
            gift_card_recipient: String::new(),
            gift_card_phone: String::new(),
            status_message: None,
            generated_code_display: None,
            applied_discount_cents: 0,
            applied_gift_card_cents: 0,
            schemas_ready: false,
        }
    }
}

pub fn draw_loyalty_modal(
    ctx: &egui::Context,
    conn: &Connection,
    state: &mut LoyaltyModalState,
    cart_gross_eur: f64,
) {
    if !state.is_open {
        return;
    }

    if !state.schemas_ready {
        let _ = init_loyalty_schema(conn);
        let _ = init_gift_card_schema(conn);
        state.schemas_ready = true;
    }

    egui::Window::new("🎁 Επιβράβευση Πελατών & Δωροκάρτες (Loyalty & Store Credit)")
        .collapsible(false)
        .resizable(false)
        .min_width(520.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(state.active_tab == LoyaltyModalTab::PointsAndTiers, "👤 Πόντοι & Tier").clicked() {
                    state.active_tab = LoyaltyModalTab::PointsAndTiers;
                }
                if ui.selectable_label(state.active_tab == LoyaltyModalTab::RedeemGiftCard, "💳 Εξαργύρωση Gift Card").clicked() {
                    state.active_tab = LoyaltyModalTab::RedeemGiftCard;
                }
                if ui.selectable_label(state.active_tab == LoyaltyModalTab::IssueGiftCard, "✨ Έκδοση Νέας").clicked() {
                    state.active_tab = LoyaltyModalTab::IssueGiftCard;
                }
                if ui.selectable_label(state.active_tab == LoyaltyModalTab::TopUpGiftCard, "🔄 Επαναφόρτιση").clicked() {
                    state.active_tab = LoyaltyModalTab::TopUpGiftCard;
                }
            });
            ui.separator();
            ui.add_space(4.0);

            match state.active_tab {
                LoyaltyModalTab::PointsAndTiers => draw_points_tab(ui, conn, state, cart_gross_eur),
                LoyaltyModalTab::RedeemGiftCard => draw_redeem_gift_tab(ui, conn, state, cart_gross_eur),
                LoyaltyModalTab::IssueGiftCard => draw_issue_gift_tab(ui, conn, state),
                LoyaltyModalTab::TopUpGiftCard => draw_topup_gift_tab(ui, conn, state),
            }

            if let Some((msg, ok)) = &state.status_message {
                ui.add_space(8.0);
                let col = if *ok { crate::theme::STATUS_READY } else { Color32::from_rgb(220, 38, 38) };
                ui.label(RichText::new(msg).color(col).strong());
            }

            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Κλείσιμο").clicked() {
                    state.is_open = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if state.applied_discount_cents > 0 || state.applied_gift_card_cents > 0 {
                        let total_disc = (state.applied_discount_cents + state.applied_gift_card_cents) as f64 / 100.0;
                        ui.label(RichText::new(format!("Εφαρμοσμένη Έκπτωση: {:.2} €", total_disc)).color(crate::theme::STATUS_READY).strong());
                    }
                });
            });
        });
}

fn draw_points_tab(ui: &mut Ui, conn: &Connection, state: &mut LoyaltyModalState, cart_gross_eur: f64) {
    ui.label(RichText::new("Αναζήτηση Μέλους (Τηλέφωνο ή Barcode Κάρτας)").strong());
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.member_query);
        if ui.button("🔍 Αναζήτηση").clicked() {
            match proteus_core::loyalty::find_account_by_phone_or_card(conn, &state.member_query) {
                Ok(Some(acc)) => {
                    state.status_message = Some((format!("Βρέθηκε μέλος: {}", acc.customer_name), true));
                    state.active_account = Some(acc);
                }
                Ok(None) => {
                    state.status_message = Some(("Δεν βρέθηκε μέλος με αυτό το στοιχείο".to_string(), false));
                    state.active_account = None;
                }
                Err(e) => state.status_message = Some((format!("Σφάλμα: {}", e), false)),
            }
        }
    });

    if let Some(acc) = state.active_account.clone() {
        ui.add_space(8.0);
        render_card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&acc.customer_name).size(16.0).strong());
                let (badge_txt, badge_col) = match acc.tier {
                    LoyaltyTier::Bronze => ("Bronze (1.0x)", crate::theme::FROST_GRAY),
                    LoyaltyTier::Silver => ("Silver (1.25x)", Color32::from_rgb(192, 192, 192)),
                    LoyaltyTier::Gold => ("Gold (1.50x)", crate::theme::ACCENT_GOLD),
                    LoyaltyTier::Platinum => ("Platinum (2.0x)", Color32::from_rgb(180, 220, 240)),
                };
                ui.label(RichText::new(badge_txt).color(badge_col).strong());
            });
            ui.label(format!("Τηλέφωνο: {} | Κάρτα: {}", acc.customer_phone, acc.card_number));
            let val_eur = (acc.points_balance * 5) as f64 / 100.0;
            ui.label(RichText::new(format!("Υπόλοιπο Πόντων: {} (Αξία: {:.2} €)", acc.points_balance, val_eur)).strong());

            let preview_spend_cents = (cart_gross_eur * 100.0) as i64;
            let preview_points = (preview_spend_cents * acc.tier.multiplier_basis_points() as i64) / (100 * 10000);
            ui.label(RichText::new(format!("Πρόβλεψη κέρδους για αυτό το καλάθι: +{} πόντοι", preview_points)).size(11.0).color(crate::theme::STATUS_READY));
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Πόντοι προς εξαργύρωση:");
            ui.text_edit_singleline(&mut state.points_to_redeem);
            if ui.button(RichText::new("Εξαργύρωση").strong()).clicked() {
                if let Ok(pts) = state.points_to_redeem.trim().parse::<i64>() {
                    let policy = LoyaltyPolicy::default();
                    match redeem_points(conn, &acc.id, pts, None, "POS redemption", &policy) {
                        Ok((updated_acc, _, discount_cents)) => {
                            state.applied_discount_cents += discount_cents;
                            state.status_message = Some((format!("Εξαργυρώθηκαν {} πόντοι (-{:.2} €)", pts, discount_cents as f64 / 100.0), true));
                            state.active_account = Some(updated_acc);
                            state.points_to_redeem.clear();
                        }
                        Err(e) => state.status_message = Some((format!("Σφάλμα εξαργύρωσης: {}", e), false)),
                    }
                }
            }
        });
    } else {
        ui.add_space(6.0);
        if ui.button("➕ Άμεση Εγγραφή Νέου Μέλους").clicked() && !state.member_query.trim().is_empty() {
            let phone = state.member_query.trim();
            match proteus_core::loyalty::create_or_get_account(conn, phone, "", "Πελάτης Loyalty") {
                Ok(acc) => {
                    state.status_message = Some(("Το μέλος δημιουργήθηκε επιτυχώς".to_string(), true));
                    state.active_account = Some(acc);
                }
                Err(e) => state.status_message = Some((format!("Σφάλμα εγγραφής: {}", e), false)),
            }
        }
    }
}

fn draw_redeem_gift_tab(ui: &mut Ui, conn: &Connection, state: &mut LoyaltyModalState, cart_gross_eur: f64) {
    ui.label(RichText::new("Σάρωση ή Πληκτρολόγηση Κωδικού Δωροκάρτας").strong());
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.gift_card_code);
        if ui.button("🔍 Έλεγχος").clicked() {
            match lookup_gift_card(conn, &state.gift_card_code) {
                Ok(Some(gc)) => {
                    state.status_message = Some((format!("Κάρτα βρέθηκε: {}", gc.masked_code), true));
                    state.active_gift_card = Some(gc);
                }
                Ok(None) => {
                    state.status_message = Some(("Δεν βρέθηκε δωροκάρτα".to_string(), false));
                    state.active_gift_card = None;
                }
                Err(e) => state.status_message = Some((format!("Σφάλμα: {}", e), false)),
            }
        }
    });

    if let Some(gc) = state.active_gift_card.clone() {
        ui.add_space(8.0);
        render_card_frame(ui, |ui| {
            ui.label(RichText::new(&gc.masked_code).size(16.0).strong());
            ui.label(format!("Κατάσταση: {:?}", gc.status));
            let bal_eur = gc.current_balance_cents as f64 / 100.0;
            ui.label(RichText::new(format!("Διαθέσιμο Υπόλοιπο: {:.2} €", bal_eur)).strong().size(14.0).color(crate::theme::STATUS_READY));
        });

        if gc.status == GiftCardStatus::Active && gc.current_balance_cents > 0 {
            ui.add_space(6.0);
            if ui.button(RichText::new("💳 Εφαρμογή Υπολοίπου στο Καλάθι").strong()).clicked() {
                let cart_cents = (cart_gross_eur * 100.0) as i64;
                let needed_cents = (cart_cents - state.applied_gift_card_cents).max(0);
                let redeem_req = if needed_cents > 0 { needed_cents } else { gc.current_balance_cents };
                match redeem_gift_card(conn, &state.gift_card_code, redeem_req, None, Some("POS-01"), "POS Tender") {
                    Ok((updated_card, _, actual_redeemed)) => {
                        state.applied_gift_card_cents += actual_redeemed;
                        state.status_message = Some((format!("Εφαρμόστηκαν {:.2} € από δωροκάρτα!", actual_redeemed as f64 / 100.0), true));
                        state.active_gift_card = Some(updated_card);
                    }
                    Err(e) => state.status_message = Some((format!("Σφάλμα εξαργύρωσης: {}", e), false)),
                }
            }
        }
    }
}

fn draw_issue_gift_tab(ui: &mut Ui, conn: &Connection, state: &mut LoyaltyModalState) {
    ui.label(RichText::new("Ποσό Δωροκάρτας (€):").strong());
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.gift_card_amount);
        for amt in ["20.00", "50.00", "100.00"] {
            if ui.small_button(amt).clicked() {
                state.gift_card_amount = amt.to_string();
            }
        }
    });

    ui.horizontal(|ui| {
        ui.label("Όνομα Παραλήπτη:");
        ui.text_edit_singleline(&mut state.gift_card_recipient);
    });
    ui.horizontal(|ui| {
        ui.label("Τηλέφωνο:");
        ui.text_edit_singleline(&mut state.gift_card_phone);
    });

    ui.add_space(6.0);
    if ui.button(RichText::new("✨ Έκδοση Κρυπτογραφικής Δωροκάρτας").strong().color(crate::theme::ACCENT_GOLD)).clicked() {
        if let Ok(eur) = state.gift_card_amount.trim().parse::<f64>() {
            let cents = (eur * 100.0).round() as i64;
            match issue_gift_card(conn, cents, None, Some(&state.gift_card_recipient), Some(&state.gift_card_phone), Some(365), "Issued via POS") {
                Ok((card, raw_code)) => {
                    state.generated_code_display = Some(raw_code);
                    state.status_message = Some((format!("Εκδόθηκε κάρτα {} αξίας {:.2} €", card.masked_code, eur), true));
                }
                Err(e) => state.status_message = Some((format!("Σφάλμα έκδοσης: {}", e), false)),
            }
        }
    }

    if let Some(code) = &state.generated_code_display {
        ui.add_space(8.0);
        render_card_frame(ui, |ui| {
            ui.label(RichText::new("🎟️ Νέος Κωδικός Δωροκάρτας:").strong());
            ui.label(RichText::new(code).size(18.0).color(crate::theme::STATUS_READY).strong());
            ui.label(RichText::new("Σημειώστε ή εκτυπώστε τον κωδικό για τον πελάτη.").size(11.0));
        });
    }
}

fn draw_topup_gift_tab(ui: &mut Ui, conn: &Connection, state: &mut LoyaltyModalState) {
    ui.horizontal(|ui| {
        ui.label("Κωδικός Δωροκάρτας:");
        ui.text_edit_singleline(&mut state.gift_card_code);
    });
    ui.horizontal(|ui| {
        ui.label("Ποσό Επαναφόρτισης (€):");
        ui.text_edit_singleline(&mut state.gift_card_amount);
    });

    ui.add_space(6.0);
    if ui.button(RichText::new("🔄 Φόρτιση Κάρτας").strong()).clicked() {
        if let Ok(eur) = state.gift_card_amount.trim().parse::<f64>() {
            let cents = (eur * 100.0).round() as i64;
            match top_up_gift_card(conn, &state.gift_card_code, cents, None, Some("POS-01"), "POS TopUp") {
                Ok((card, _)) => {
                    let new_bal = card.current_balance_cents as f64 / 100.0;
                    state.status_message = Some((format!("Η κάρτα {} φορτίστηκε! Νέο υπόλοιπο: {:.2} €", card.masked_code, new_bal), true));
                }
                Err(e) => state.status_message = Some((format!("Σφάλμα φόρτισης: {}", e), false)),
            }
        }
    }
}

fn render_card_frame<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(crate::theme::BG_PANEL)
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .inner_margin(Margin::same(8))
        .show(ui, add_contents)
        .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loyalty_modal_defaults() {
        let state = LoyaltyModalState::default();
        assert!(!state.is_open);
        assert_eq!(state.active_tab, LoyaltyModalTab::PointsAndTiers);
        assert_eq!(state.applied_discount_cents, 0);
        assert_eq!(state.applied_gift_card_cents, 0);
    }
}
