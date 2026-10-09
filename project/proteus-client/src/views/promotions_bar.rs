//! Frontline POS Promotional Campaign Banner & Coupon Application Widget.
//! Bespoke implementation adhering strictly to Rule 1 & Rule 5 (Zero Mock Data).

use chrono::Utc;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::invoicing::InvoiceLine;
use proteus_core::promotions::{
    evaluate_cart, generate_single_use_coupon, init_coupon_schema, init_promotions_schema,
    list_active_rules, redeem_coupon, CartEvaluationResult, CartItem, CouponError,
    CouponRedemption, DiscountType,
};

pub struct PromotionsBarState {
    pub coupon_input: String,
    pub applied_coupon: Option<CouponRedemption>,
    pub coupon_code_applied: Option<String>,
    pub coupon_discount_cents: i64,
    pub status_message: Option<(String, bool)>,
    pub is_modal_open: bool,
    pub new_coupon_prefix: String,
    pub new_coupon_discount_cents: String,
    pub generated_coupon_display: Option<String>,
    pub schemas_ready: bool,
}

impl Default for PromotionsBarState {
    fn default() -> Self {
        Self {
            coupon_input: String::new(),
            applied_coupon: None,
            coupon_code_applied: None,
            coupon_discount_cents: 0,
            status_message: None,
            is_modal_open: false,
            new_coupon_prefix: "PROMO".to_string(),
            new_coupon_discount_cents: "5.00".to_string(),
            generated_coupon_display: None,
            schemas_ready: false,
        }
    }
}

pub fn convert_invoice_lines_to_cart_items(lines: &[InvoiceLine]) -> Vec<CartItem> {
    lines
        .iter()
        .map(|l| {
            let cents = (l.net_unit_price * 100.0).round() as i64;
            CartItem::new(&format!("SKU-{}", l.line_number), &l.description, "Retail", l.quantity as u32, cents)
        })
        .collect()
}

pub fn draw_promotions_banner(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut PromotionsBarState,
    cart_lines: &[InvoiceLine],
    customer_phone: &str,
) -> (i64, CartEvaluationResult) {
    if !state.schemas_ready {
        let _ = init_promotions_schema(conn);
        let _ = init_coupon_schema(conn);
        state.schemas_ready = true;
    }

    let cart_items = convert_invoice_lines_to_cart_items(cart_lines);
    let rules = list_active_rules(conn).unwrap_or_default();
    let now_iso = Utc::now().to_rfc3339();
    let eval = evaluate_cart(&cart_items, &rules, &now_iso);

    Frame::new()
        .fill(crate::theme::BG_PANEL)
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .inner_margin(Margin::same(6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🏷️ Προσφορές & Κουπόνια:").strong().size(13.0));

                if !eval.applied_promotions.is_empty() {
                    for promo in &eval.applied_promotions {
                        let disc_eur = promo.discount_cents as f64 / 100.0;
                        let text = format!("{} (-{:.2}€)", promo.rule_name, disc_eur);
                        ui.label(RichText::new(text).color(crate::theme::STATUS_READY).strong().size(12.0));
                    }
                } else {
                    ui.label(RichText::new("Καμία ενεργή αυτόματη έκπτωση").size(11.0).color(crate::theme::FROST_GRAY));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("⚙️ Διαχείριση").size(11.0)).clicked() {
                        state.is_modal_open = true;
                    }
                });
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Κωδικός:").size(12.0));
                let code_edit = ui.add(egui::TextEdit::singleline(&mut state.coupon_input).desired_width(120.0));
                
                if let Some(code) = &state.coupon_code_applied {
                    ui.label(RichText::new(format!("Εφαρμοσμένο: {} (-{:.2}€)", code, state.coupon_discount_cents as f64 / 100.0)).color(crate::theme::ACCENT_GOLD).strong());
                    if ui.button("✕").clicked() {
                        state.coupon_code_applied = None;
                        state.applied_coupon = None;
                        state.coupon_discount_cents = 0;
                        state.status_message = None;
                    }
                } else if ui.button(RichText::new("Εφαρμογή").strong()).clicked() || (code_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    let code = state.coupon_input.trim();
                    if !code.is_empty() {
                        let remaining_cents = (eval.final_gross_cents).max(0);
                        match redeem_coupon(conn, code, customer_phone, None, remaining_cents, &now_iso) {
                            Ok((red, disc)) => {
                                state.coupon_code_applied = Some(code.to_uppercase());
                                state.applied_coupon = Some(red);
                                state.coupon_discount_cents = disc;
                                state.status_message = Some((format!("Κουπόνι {} εφαρμόστηκε (-{:.2}€)", code.to_uppercase(), disc as f64 / 100.0), true));
                                state.coupon_input.clear();
                            }
                            Err(e) => {
                                let err_msg = match e {
                                    CouponError::NotFound => "Το κουπόνι δεν βρέθηκε".to_string(),
                                    CouponError::Inactive => "Το κουπόνι είναι ανενεργό".to_string(),
                                    CouponError::Expired => "Το κουπόνι έχει λήξει".to_string(),
                                    CouponError::MinSpendNotMet { min_spend_cents, .. } => format!("Απαιτείται ελάχιστη παραγγελία {:.2}€", min_spend_cents as f64 / 100.0),
                                    CouponError::CustomerUsageLimitExceeded { .. } => "Έχετε ήδη χρησιμοποιήσει αυτό το κουπόνι".to_string(),
                                    CouponError::TotalUsageLimitExceeded(_) => "Εξαντλήθηκαν οι χρήσεις του κουπονιού".to_string(),
                                    CouponError::Database(d) => format!("Σφάλμα βάσης: {}", d),
                                };
                                state.status_message = Some((err_msg, false));
                            }
                        }
                    }
                }

                if let Some((msg, ok)) = &state.status_message {
                    let col = if *ok { crate::theme::STATUS_READY } else { Color32::from_rgb(239, 68, 68) };
                    ui.label(RichText::new(msg).color(col).size(11.0).strong());
                }
            });
        });

    let total_promo_discount = eval.total_discount_cents + state.coupon_discount_cents;
    (total_promo_discount, eval)
}

pub fn draw_promotions_modal(ctx: &egui::Context, conn: &Connection, state: &mut PromotionsBarState) {
    if !state.is_modal_open {
        return;
    }

    egui::Window::new("🏷️ Διαχείριση Προσφορών & Έκδοση Κουπονιών")
        .collapsible(false)
        .resizable(false)
        .min_width(450.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("Έκδοση Νέου Μοναδικού Κουπονιού").strong().size(15.0));
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Πρόθεμα:");
                ui.text_edit_singleline(&mut state.new_coupon_prefix);
                ui.label("Έκπτωση (€):");
                ui.text_edit_singleline(&mut state.new_coupon_discount_cents);
            });

            ui.add_space(6.0);
            if ui.button(RichText::new("✨ Παραγωγή Κουπονιού").strong().color(crate::theme::ACCENT_GOLD)).clicked() {
                if let Ok(eur) = state.new_coupon_discount_cents.trim().parse::<f64>() {
                    let cents = (eur * 100.0).round() as i64;
                    match generate_single_use_coupon(conn, &state.new_coupon_prefix, DiscountType::FixedAmountCents(cents), cents, Some(30)) {
                        Ok((_, raw_code)) => {
                            state.generated_coupon_display = Some(raw_code);
                        }
                        Err(e) => {
                            state.status_message = Some((format!("Σφάλμα δημιουργίας: {}", e), false));
                        }
                    }
                }
            }

            if let Some(code) = &state.generated_coupon_display {
                ui.add_space(8.0);
                Frame::new()
                    .fill(crate::theme::BG_PANEL)
                    .stroke(Stroke::new(1.0, crate::theme::ACCENT_GOLD))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.label(RichText::new("🎟️ Νέος Κωδικός Κουπονιού:").strong());
                        ui.label(RichText::new(code).size(18.0).color(crate::theme::STATUS_READY).strong());
                    });
            }

            ui.add_space(10.0);
            ui.separator();
            if ui.button("Κλείσιμο").clicked() {
                state.is_modal_open = false;
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promotions_bar_state_defaults() {
        let state = PromotionsBarState::default();
        assert!(state.coupon_input.is_empty());
        assert_eq!(state.coupon_discount_cents, 0);
        assert!(!state.is_modal_open);
    }

    #[test]
    fn test_convert_invoice_lines_to_cart_items() {
        let lines = vec![
            InvoiceLine::new(1, "Service Screen", 1.0, 50.0, proteus_core::invoicing::VatCategory::Vat24),
            InvoiceLine::new(2, "Screen Glass", 2.0, 10.0, proteus_core::invoicing::VatCategory::Vat24),
        ];
        let items = convert_invoice_lines_to_cart_items(&lines);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].unit_price_cents, 5000);
        assert_eq!(items[1].quantity, 2);
        assert_eq!(items[1].total_cents(), 2000);
    }
}
