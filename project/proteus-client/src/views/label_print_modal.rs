//! Visual Label Preview & TSPL Print Dispatch Modal for Proteus Client.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use proteus_core::printer_spooler::{
    LabelPrintDispatcher, LabelPrintJob, LabelPrinterConnection, ShelfLabelPayload,
    ShippingWaybillPayload, TsplGenerator, TsplLabelSize,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelModalMode {
    ShelfTag,
    ShippingWaybill,
}

pub struct LabelPrintModalState {
    pub is_open: bool,
    pub mode: LabelModalMode,
    // Connection Target
    pub is_network: bool,
    pub printer_name: String,
    pub printer_host: String,
    pub printer_port: u16,
    // Shelf Tag
    pub sku: String,
    pub name: String,
    pub price_cents: u64,
    pub barcode: String,
    pub vat_rate: u8,
    pub label_size: TsplLabelSize,
    // Shipping Waybill
    pub voucher_id: String,
    pub courier: String,
    pub recipient_name: String,
    pub address: String,
    pub city_zip: String,
    pub phone: String,
    pub cod_cents: Option<u64>,
    pub parcels: u32,
    pub weight_kg: f32,
    pub notes: String,
    // Status Feedback
    pub status_msg: Option<(String, bool)>,
}

impl Default for LabelPrintModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            mode: LabelModalMode::ShelfTag,
            is_network: false,
            printer_name: "TSC TTP-244 Pro".to_string(),
            printer_host: "192.168.1.200".to_string(),
            printer_port: 9100,
            sku: String::new(),
            name: String::new(),
            price_cents: 0,
            barcode: String::new(),
            vat_rate: 24,
            label_size: TsplLabelSize::ShelfCompact,
            voucher_id: String::new(),
            courier: "ACS Courier".to_string(),
            recipient_name: String::new(),
            address: String::new(),
            city_zip: String::new(),
            phone: String::new(),
            cod_cents: None,
            parcels: 1,
            weight_kg: 1.0,
            notes: String::new(),
            status_msg: None,
        }
    }
}

impl LabelPrintModalState {
    #[allow(dead_code)]
    pub fn open_shelf_tag(&mut self, sku: &str, name: &str, price_cents: u64, barcode: &str, vat_rate: u8) {
        self.mode = LabelModalMode::ShelfTag;
        self.sku = sku.to_string();
        self.name = name.to_string();
        self.price_cents = price_cents;
        self.barcode = barcode.to_string();
        self.vat_rate = vat_rate;
        self.label_size = TsplLabelSize::ShelfCompact;
        self.status_msg = None;
        self.is_open = true;
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_shipping_waybill(
        &mut self,
        voucher_id: &str,
        courier: &str,
        recipient: &str,
        address: &str,
        city_zip: &str,
        phone: &str,
        cod_cents: Option<u64>,
        parcels: u32,
        weight_kg: f32,
    ) {
        self.mode = LabelModalMode::ShippingWaybill;
        self.voucher_id = voucher_id.to_string();
        self.courier = courier.to_string();
        self.recipient_name = recipient.to_string();
        self.address = address.to_string();
        self.city_zip = city_zip.to_string();
        self.phone = phone.to_string();
        self.cod_cents = cod_cents;
        self.parcels = parcels;
        self.weight_kg = weight_kg;
        self.label_size = TsplLabelSize::ShippingWaybill;
        self.status_msg = None;
        self.is_open = true;
    }

    pub fn get_connection(&self) -> LabelPrinterConnection {
        if self.is_network {
            LabelPrinterConnection::NetworkTcp {
                host: self.printer_host.trim().to_string(),
                port: self.printer_port,
            }
        } else {
            LabelPrinterConnection::UsbSpooler {
                printer_name: self.printer_name.trim().to_string(),
            }
        }
    }

    pub fn execute_print(&mut self) {
        let conn = self.get_connection();
        let (raw_bytes, doc_title) = match self.mode {
            LabelModalMode::ShelfTag => {
                let payload = ShelfLabelPayload {
                    sku: &self.sku,
                    name: &self.name,
                    price_cents: self.price_cents,
                    barcode: &self.barcode,
                    vat_rate: self.vat_rate,
                    unit: "PCS",
                };
                let bytes = TsplGenerator::build_shelf_label(&payload, self.label_size);
                (bytes, format!("Shelf Label {}", self.sku))
            }
            LabelModalMode::ShippingWaybill => {
                let payload = ShippingWaybillPayload {
                    voucher_id: &self.voucher_id,
                    courier: &self.courier,
                    recipient_name: &self.recipient_name,
                    address: &self.address,
                    city_zip: &self.city_zip,
                    phone: &self.phone,
                    cod_cents: self.cod_cents,
                    parcels: self.parcels,
                    weight_kg: self.weight_kg,
                    notes: &self.notes,
                };
                let bytes = TsplGenerator::build_shipping_waybill(&payload, self.label_size);
                (bytes, format!("Waybill {}", self.voucher_id))
            }
        };

        let job = LabelPrintJob { doc_title, raw_bytes };
        match LabelPrintDispatcher::dispatch(&conn, &job) {
            Ok(()) => {
                self.status_msg = Some(("Η ετικέτα απεστάλη επιτυχώς στον εκτυπωτή.".to_string(), false));
            }
            Err(e) => {
                self.status_msg = Some((format!("Σφάλμα αποστολής: {}", e), true));
            }
        }
    }

    pub fn ping_printer(&mut self) {
        let conn = self.get_connection();
        match LabelPrintDispatcher::ping(&conn) {
            Ok(()) => {
                self.status_msg = Some(("Εκτυπωτής online & έτοιμος.".to_string(), false));
            }
            Err(e) => {
                self.status_msg = Some((format!("Μη διαθέσιμος εκτυπωτής: {}", e), true));
            }
        }
    }
}

pub fn render_label_print_modal(ctx: &egui::Context, state: &mut LabelPrintModalState) {
    if !state.is_open {
        return;
    }

    let mut open = state.is_open;
    egui::Window::new("🏷️ Προεπισκόπηση & Εκτύπωση Ετικέτας (TSPL)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(480.0, 520.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.add_space(4.0);

            // Thermal Canvas Preview Box
            Frame::canvas(ui.style())
                .fill(Color32::from_rgb(255, 255, 255))
                .corner_radius(CornerRadius::same(6))
                .stroke(Stroke::new(1.5, Color32::from_rgb(203, 213, 225)))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    match state.mode {
                        LabelModalMode::ShelfTag => render_shelf_tag_preview(ui, state),
                        LabelModalMode::ShippingWaybill => render_waybill_preview(ui, state),
                    }
                });

            ui.add_space(10.0);
            ui.separator();

            // Hardware Connection Target Selector
            ui.horizontal(|ui| {
                ui.label(RichText::new("Σύνδεση:").strong());
                if ui.selectable_label(!state.is_network, "🖨️ USB Spooler").clicked() {
                    state.is_network = false;
                }
                if ui.selectable_label(state.is_network, "🌐 Network TCP").clicked() {
                    state.is_network = true;
                }
            });

            if !state.is_network {
                ui.horizontal(|ui| {
                    ui.label("Όνομα Συσκευής:");
                    ui.text_edit_singleline(&mut state.printer_name);
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("IP:");
                    ui.text_edit_singleline(&mut state.printer_host);
                    ui.label("Port:");
                    let mut port_str = state.printer_port.to_string();
                    if ui.add(egui::TextEdit::singleline(&mut port_str).desired_width(50.0)).changed() {
                        if let Ok(p) = port_str.parse::<u16>() {
                            state.printer_port = p;
                        }
                    }
                });
            }

            ui.add_space(6.0);

            // Action Buttons
            ui.horizontal(|ui| {
                if ui.button(RichText::new("📡 Ping").color(Color32::from_rgb(147, 197, 253))).clicked() {
                    state.ping_printer();
                }

                if ui.button(RichText::new("🖨️ Άμεση Εκτύπωση").strong().color(Color32::from_rgb(34, 197, 94))).clicked() {
                    state.execute_print();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Κλείσιμο").clicked() {
                        state.is_open = false;
                    }
                });
            });

            // Status message banner
            if let Some((msg, is_err)) = &state.status_msg {
                ui.add_space(6.0);
                let color = if *is_err { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(34, 197, 94) };
                ui.label(RichText::new(msg).color(color).size(12.0));
            }
        });

    state.is_open = open;
}

fn render_shelf_tag_preview(ui: &mut Ui, state: &LabelPrintModalState) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(&state.name).strong().color(Color32::BLACK).size(15.0));
        ui.label(RichText::new(format!("SKU: {} | ΦΠΑ: {}%", state.sku, state.vat_rate)).color(Color32::from_rgb(75, 85, 99)).size(11.0));
        ui.add_space(4.0);

        let price_str = format!("{:.2} €", state.price_cents as f64 / 100.0);
        ui.label(RichText::new(price_str).strong().color(Color32::BLACK).size(26.0));
        ui.add_space(6.0);

        // Barcode simulation
        draw_simulated_barcode(ui, &state.barcode, 36.0);
    });
}

fn render_waybill_preview(ui: &mut Ui, state: &LabelPrintModalState) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(&state.courier).strong().color(Color32::BLACK).size(14.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(format!("Voucher: {}", state.voucher_id)).strong().color(Color32::BLACK).size(12.0));
        });
    });
    ui.separator();

    ui.label(RichText::new(format!("Παραλήπτης: {}", state.recipient_name)).strong().color(Color32::BLACK).size(12.0));
    ui.label(RichText::new(format!("Διεύθυνση: {}, {}", state.address, state.city_zip)).color(Color32::from_rgb(55, 65, 81)).size(11.0));
    ui.label(RichText::new(format!("Τηλ: {}", state.phone)).color(Color32::from_rgb(55, 65, 81)).size(11.0));

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Δέματα: {} | Βάρος: {:.1} kg", state.parcels, state.weight_kg)).color(Color32::from_rgb(55, 65, 81)).size(11.0));
        if let Some(cod) = state.cod_cents {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("Αντικαταβολή: {:.2} €", cod as f64 / 100.0)).strong().color(Color32::BLACK).size(12.0));
            });
        }
    });

    ui.add_space(4.0);
    draw_simulated_barcode(ui, &state.voucher_id, 40.0);
}

fn draw_simulated_barcode(ui: &mut Ui, code: &str, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(320.0, height), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(248, 250, 252));

    let bar_count = 42;
    let bar_width = rect.width() / (bar_count as f32 * 1.5);
    for i in 0..bar_count {
        let x = rect.min.x + (i as f32 * bar_width * 1.5) + 6.0;
        let is_thick = (i % 3 == 0) || (i % 7 == 0);
        let w = if is_thick { bar_width * 1.6 } else { bar_width * 0.8 };
        let bar_rect = Rect::from_min_size(Pos2::new(x, rect.min.y + 2.0), Vec2::new(w, height - 14.0));
        painter.rect_filled(bar_rect, CornerRadius::ZERO, Color32::BLACK);
    }
    let text_pos = Pos2::new(rect.center().x, rect.max.y - 10.0);
    painter.text(text_pos, egui::Align2::CENTER_CENTER, code, egui::FontId::monospace(10.0), Color32::BLACK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_modal_shelf_and_waybill_init() {
        let mut state = LabelPrintModalState::default();
        state.open_shelf_tag("SKU-99", "USB Cable", 550, "5209999", 24);
        assert!(state.is_open);
        assert_eq!(state.mode, LabelModalMode::ShelfTag);
        assert_eq!(state.price_cents, 550);

        state.open_shipping_waybill("VCH-1", "Speedex", "Nikos", "Odos 1", "Athens", "21000", None, 1, 0.5);
        assert_eq!(state.mode, LabelModalMode::ShippingWaybill);
        assert_eq!(state.voucher_id, "VCH-1");
    }
}
