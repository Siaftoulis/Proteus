//! Frontline Real-Time Scale Telemetry & Weighed Product Ingestion Bar for Touch POS.
//! Connects native industrial scales (CAS, Dibal, Mettler-Toledo, Avery Weigh-Tronix)
//! directly to the cashier cart with zero latency and metrological stability validation (OIML R76 / A.1155).
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use proteus_core::audio::{play_barcode_chime, play_error_tone};
use proteus_core::hardware::{LiveScaleState, ScaleBroadcastHub, WeighedCartItem};
use proteus_core::invoicing::{InvoiceLine, VatCategory};

/// Interactive state for the Touch POS scale telemetry bar.
#[derive(Debug, Clone)]
pub struct PosScaleBarState {
    pub scale_hub: ScaleBroadcastHub,
    pub item_name_input: String,
    pub unit_price_input: String,
    pub selected_vat: VatCategory,
    pub weighed_items_history: Vec<WeighedCartItem>,
}

impl Default for PosScaleBarState {
    fn default() -> Self {
        Self {
            scale_hub: ScaleBroadcastHub::new(),
            item_name_input: "Ζυγιζόμενο Προϊόν".to_string(),
            unit_price_input: "2.50".to_string(),
            selected_vat: VatCategory::Vat13, // Standard agricultural/food VAT in Greece
            weighed_items_history: Vec::new(),
        }
    }
}

/// Renders the real-time scale telemetry HUD and 1-click weighed product ingestion interface.
pub fn draw_scale_telemetry_bar(
    ui: &mut Ui,
    state: &mut PosScaleBarState,
    cart_lines: &mut Vec<InvoiceLine>,
    status_feedback: &mut Option<(String, bool)>,
) {
    let reading = state.scale_hub.latest_reading();
    let weight_kg = (reading.net_weight_grams as f64) / 1000.0;

    Frame::new()
        .fill(crate::theme::BG_PANEL)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Left: Real-time Weight Display & Status Indicator
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚖️").size(18.0));
                        ui.label(
                            RichText::new(format!("{:.3} kg", weight_kg))
                                .strong()
                                .size(20.0)
                                .color(if reading.is_overweight {
                                    Color32::from_rgb(239, 68, 68)
                                } else if reading.is_stable && reading.net_weight_grams > 0 {
                                    crate::theme::STATUS_READY
                                } else {
                                    crate::theme::ACCENT_GOLD
                                }),
                        );

                        // Stability & Metrological Certification Badge
                        if reading.is_overweight {
                            ui.label(
                                RichText::new("⚠️ OVERLOAD")
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(239, 68, 68)),
                            );
                        } else if reading.is_stable && reading.net_weight_grams > 0 {
                            ui.label(
                                RichText::new("🟢 ΣΤΑΘΕΡΟ (OIML R76)")
                                    .size(11.0)
                                    .strong()
                                    .color(crate::theme::STATUS_READY),
                            );
                        } else if reading.is_zero {
                            ui.label(
                                RichText::new("⚪ ΜΗΔΕΝ")
                                    .size(11.0)
                                    .color(crate::theme::TEXT_MUTED),
                            );
                        } else {
                            ui.label(
                                RichText::new("🟡 ΣΕ ΚΙΝΗΣΗ")
                                    .size(11.0)
                                    .color(crate::theme::ACCENT_GOLD),
                            );
                        }
                    });

                    if reading.tare_weight_grams > 0 {
                        ui.label(
                            RichText::new(format!(
                                "Απόβαρο (Tare): {} g | Μικτό: {} g",
                                reading.tare_weight_grams, reading.gross_weight_grams
                            ))
                            .size(10.0)
                            .color(crate::theme::TEXT_MUTED),
                        );
                    }
                });

                ui.separator();

                // Center-Right: Ingestion Inputs & Touch Button
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Περιγραφή:").size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.item_name_input)
                                .desired_width(130.0),
                        );

                        ui.label(RichText::new("Τιμή/kg:").size(12.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut state.unit_price_input)
                                .desired_width(55.0),
                        );

                        let can_ingest =
                            reading.is_stable && reading.net_weight_grams > 0 && !reading.is_overweight;
                        let ingest_btn = egui::Button::new(
                            RichText::new("➕ Εισαγωγή στο Καλάθι")
                                .strong()
                                .size(12.0)
                                .color(if can_ingest {
                                    Color32::WHITE
                                } else {
                                    crate::theme::TEXT_MUTED
                                }),
                        )
                        .min_size(Vec2::new(140.0, 28.0));

                        if ui.add_enabled(can_ingest, ingest_btn).clicked() {
                            let price_per_kg = state.unit_price_input.parse::<f64>().unwrap_or(0.0);
                            if price_per_kg <= 0.0 {
                                play_error_tone();
                                *status_feedback = Some((
                                    "Η τιμή κιλού πρέπει να είναι θετικός αριθμός.".to_string(),
                                    false,
                                ));
                                return;
                            }

                            match state.scale_hub.ingest_weighed_product(&state.item_name_input, price_per_kg) {
                                Ok(weighed_item) => {
                                    let next_line_num = cart_lines.len() as u32 + 1;
                                    let cart_line = InvoiceLine::new(
                                        next_line_num,
                                        weighed_item.description.clone(),
                                        weighed_item.quantity_kg,
                                        weighed_item.unit_price_per_kg,
                                        state.selected_vat,
                                    );
                                    state.weighed_items_history.push(weighed_item.clone());
                                    cart_lines.push(cart_line);
                                    play_barcode_chime();
                                    *status_feedback = Some((
                                        format!(
                                            "✓ Προστέθηκε ζυγιζόμενο: {} ({:.2}€)",
                                            weighed_item.description, weighed_item.line_total_eur
                                        ),
                                        true,
                                    ));
                                }
                                Err(err) => {
                                    play_error_tone();
                                    *status_feedback = Some((format!("⚠️ {}", err), false));
                                }
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
    fn test_pos_scale_bar_state_default() {
        let state = PosScaleBarState::default();
        assert_eq!(state.item_name_input, "Ζυγιζόμενο Προϊόν");
        assert_eq!(state.unit_price_input, "2.50");
        assert_eq!(state.selected_vat, VatCategory::Vat13);
        assert_eq!(state.scale_hub.latest_reading().net_weight_grams, 0);
    }

    #[test]
    fn test_ingest_into_cart_lines_logic() {
        let state = PosScaleBarState::default();
        state.scale_hub.publish_reading(LiveScaleState {
            gross_weight_grams: 1800,
            tare_weight_grams: 0,
            net_weight_grams: 1800,
            unit: "kg".to_string(),
            is_stable: true,
            is_settled: true,
            is_zero: false,
            is_overweight: false,
            consecutive_stable_count: 3,
        });

        let mut cart_lines = Vec::new();
        let res = state.scale_hub.ingest_weighed_product("Μπανάνες", 1.50);
        assert!(res.is_ok());

        let weighed = res.unwrap();
        let line = InvoiceLine::new(
            1,
            weighed.description,
            weighed.quantity_kg,
            weighed.unit_price_per_kg,
            state.selected_vat,
        );
        cart_lines.push(line);

        assert_eq!(cart_lines.len(), 1);
        assert_eq!(cart_lines[0].quantity, 1.800);
        assert_eq!(cart_lines[0].net_unit_price, 1.50);
        // 1.800 * 1.50 = 2.70 net. VAT 13% = 0.35. Gross = 3.05.
        assert_eq!(cart_lines[0].net_total, 2.70);
        assert_eq!(cart_lines[0].vat_amount, 0.35);
        assert_eq!(cart_lines[0].gross_total, 3.05);
    }
}
