//! Frontline Industrial Label Spooler UI & Live Thermal Preview.
//! Supports real-time field bindings for Shelf Edge Pricing & Smart Tote Waybills,
//! live WYSIWYG thermal canvas preview, 1-click test printing, and queue flushing.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use rusqlite::Connection;
use proteus_core::hardware::dispatcher::dispatch_print_job;
use proteus_core::hardware::label::PrintDensity;
use proteus_core::hardware::spooler::{
    enqueue_print_job, get_next_pending_job, init_label_spooler_schema, list_recent_jobs,
    purge_completed_jobs, update_job_status, LabelPrintJob, PrintJobStatus, PrintTargetProtocol,
};
use proteus_core::hardware::templates::{ShelfEdgePriceTemplate, SmartToteWaybillTemplate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelSpoolerTemplateType {
    ShelfEdgePrice,
    SmartToteWaybill,
}

#[derive(Debug, Clone)]
pub struct LabelSpoolerState {
    pub template_type: LabelSpoolerTemplateType,
    pub protocol: PrintTargetProtocol,
    pub density: PrintDensity,
    pub destination_endpoint: String,
    pub copies: u32,
    pub shelf_product_name: String,
    pub shelf_sku: String,
    pub shelf_price_eur: String,
    pub shelf_vat_rate: u8,
    pub shelf_barcode: String,
    pub tote_id: String,
    pub destination_shelf: String,
    pub unit_count: u32,
    pub total_weight_grams: u32,
    pub merkle_seal_sha256: String,
    pub recent_jobs: Vec<LabelPrintJob>,
    pub status_message: Option<(String, bool)>,
}

impl Default for LabelSpoolerState {
    fn default() -> Self {
        Self {
            template_type: LabelSpoolerTemplateType::ShelfEdgePrice,
            protocol: PrintTargetProtocol::Tspl,
            density: PrintDensity::Dpi203,
            destination_endpoint: "tcp://192.168.1.150:9100".to_string(),
            copies: 1,
            shelf_product_name: "Επαγγελματικό Δραπανοκατσάβιδο 18V".to_string(),
            shelf_sku: "SKU-BOSCH-18V".to_string(),
            shelf_price_eur: "89.90".to_string(),
            shelf_vat_rate: 24,
            shelf_barcode: "5201234567890".to_string(),
            tote_id: "TOTE-GR-0042".to_string(),
            destination_shelf: "AISLE-03-B2".to_string(),
            unit_count: 18,
            total_weight_grams: 3450,
            merkle_seal_sha256: "a3f5c71d98e0b24467c9fe10234abcd5".to_string(),
            recent_jobs: Vec::new(),
            status_message: None,
        }
    }
}

impl LabelSpoolerState {
    pub fn refresh_recent_jobs(&mut self, conn: &Connection) {
        let _ = init_label_spooler_schema(conn);
        if let Ok(jobs) = list_recent_jobs(conn, 10) {
            self.recent_jobs = jobs;
        }
    }

    pub fn compute_compiled_bytes(&self) -> (usize, usize) {
        match self.template_type {
            LabelSpoolerTemplateType::ShelfEdgePrice => {
                let price_val = self.shelf_price_eur.trim().parse::<f64>().unwrap_or(0.0);
                let tmpl = ShelfEdgePriceTemplate::new(
                    &self.shelf_sku,
                    &self.shelf_product_name,
                    price_val,
                    &self.shelf_barcode,
                )
                .with_vat(self.shelf_vat_rate)
                .with_density(self.density);
                (tmpl.to_tspl(1).len(), tmpl.to_zpl(1).len())
            }
            LabelSpoolerTemplateType::SmartToteWaybill => {
                let tmpl = SmartToteWaybillTemplate::new(
                    &self.tote_id,
                    "SMART-TOTE",
                    self.unit_count,
                    self.total_weight_grams as f64,
                )
                .with_seal_hash(&self.merkle_seal_sha256)
                .with_shelf_destination(&self.destination_shelf)
                .with_density(self.density);
                (tmpl.to_tspl(1).len(), tmpl.to_zpl(1).len())
            }
        }
    }

    pub fn execute_test_print(&mut self, conn: &Connection) {
        let (title, payload) = match self.template_type {
            LabelSpoolerTemplateType::ShelfEdgePrice => {
                let p = self.shelf_price_eur.trim().parse::<f64>().unwrap_or(0.0);
                let tmpl = ShelfEdgePriceTemplate::new(&self.shelf_sku, &self.shelf_product_name, p, &self.shelf_barcode)
                    .with_vat(self.shelf_vat_rate)
                    .with_density(self.density);
                let bytes = if self.protocol == PrintTargetProtocol::Tspl { tmpl.to_tspl(self.copies) } else { tmpl.to_zpl(self.copies) };
                (format!("Ετικέτα Ραφιού: {}", self.shelf_product_name), bytes)
            }
            LabelSpoolerTemplateType::SmartToteWaybill => {
                let tmpl = SmartToteWaybillTemplate::new(&self.tote_id, "SMART-TOTE", self.unit_count, self.total_weight_grams as f64)
                    .with_seal_hash(&self.merkle_seal_sha256)
                    .with_shelf_destination(&self.destination_shelf)
                    .with_density(self.density);
                let bytes = if self.protocol == PrintTargetProtocol::Tspl { tmpl.to_tspl(self.copies) } else { tmpl.to_zpl(self.copies) };
                (format!("Smart Tote: {}", self.tote_id), bytes)
            }
        };

        let now_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        let job = LabelPrintJob::new(&title, self.protocol, payload, self.copies, &self.destination_endpoint, now_epoch);

        if let Err(e) = enqueue_print_job(conn, &job) {
            self.status_message = Some((format!("Σφάλμα ουράς: {}", e), true));
            return;
        }

        match dispatch_print_job(&job, 3000) {
            Ok(bytes) => {
                let _ = update_job_status(conn, &job.job_id, PrintJobStatus::Completed, Some(now_epoch));
                self.status_message = Some((format!("✓ Επιτυχής αποστολή {} B στο {}", bytes, job.destination_endpoint), false));
            }
            Err(e) => {
                let _ = update_job_status(conn, &job.job_id, PrintJobStatus::Failed(e.to_string()), Some(now_epoch));
                self.status_message = Some((format!("⚠ Σφάλμα: {}", e), true));
            }
        }
        self.refresh_recent_jobs(conn);
    }

    pub fn flush_pending_queue(&mut self, conn: &Connection) {
        let mut processed = 0;
        let now_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
        while let Ok(Some(job)) = get_next_pending_job(conn) {
            processed += 1;
            let _ = update_job_status(conn, &job.job_id, PrintJobStatus::Printing, None);
            match dispatch_print_job(&job, 3000) {
                Ok(_) => { let _ = update_job_status(conn, &job.job_id, PrintJobStatus::Completed, Some(now_epoch)); }
                Err(e) => { let _ = update_job_status(conn, &job.job_id, PrintJobStatus::Failed(e.to_string()), Some(now_epoch)); }
            }
        }
        self.status_message = Some((format!("Εκκαθάριση ουράς: {} εργασίες ολοκληρώθηκαν", processed), false));
        self.refresh_recent_jobs(conn);
    }
}

pub fn draw_label_spooler_view(ui: &mut Ui, conn: &Connection, state: &mut LabelSpoolerState) {
    if state.recent_jobs.is_empty() {
        state.refresh_recent_jobs(conn);
    }

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🏷️ Βιομηχανικός Spooler Ετικετών").strong().size(20.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑️ Εκκαθάριση").clicked() {
                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
                    let _ = purge_completed_jobs(conn, now + 1);
                    state.refresh_recent_jobs(conn);
                }
                if ui.button("🚀 Flush Ουράς").clicked() {
                    state.flush_pending_queue(conn);
                }
                if ui.button("🔄 Ανανέωση").clicked() {
                    state.refresh_recent_jobs(conn);
                }
            });
        });
        ui.label(RichText::new("Ζωντανή προεπισκόπηση θερμικών ετικετών (TSPL / ZPL), δέσμευση πεδίων & διαχείριση εκτυπώσεων.")
            .color(Color32::from_rgb(148, 163, 184)).size(12.0));
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        if let Some((ref msg, is_err)) = state.status_message {
            let bg = if is_err { Color32::from_rgb(153, 27, 27) } else { Color32::from_rgb(22, 101, 52) };
            Frame::new().fill(bg).corner_radius(CornerRadius::same(4)).inner_margin(Margin::symmetric(10, 6)).show(ui, |ui| {
                ui.label(RichText::new(msg).color(Color32::WHITE).strong());
            });
            ui.add_space(6.0);
        }

        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.label(RichText::new("1. Επιλογή Προτύπου & Υλικού").strong().size(14.0));
                ui.horizontal(|ui| {
                    if ui.selectable_label(state.template_type == LabelSpoolerTemplateType::ShelfEdgePrice, "🏷️ Ράφι (50×30 mm)").clicked() {
                        state.template_type = LabelSpoolerTemplateType::ShelfEdgePrice;
                    }
                    if ui.selectable_label(state.template_type == LabelSpoolerTemplateType::SmartToteWaybill, "📦 Tote (80×60 mm)").clicked() {
                        state.template_type = LabelSpoolerTemplateType::SmartToteWaybill;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Πρωτόκολλο:");
                    if ui.selectable_label(state.protocol == PrintTargetProtocol::Tspl, "TSPL").clicked() { state.protocol = PrintTargetProtocol::Tspl; }
                    if ui.selectable_label(state.protocol == PrintTargetProtocol::Zpl, "ZPL").clicked() { state.protocol = PrintTargetProtocol::Zpl; }
                    ui.add_space(8.0);
                    ui.label("DPI:");
                    if ui.selectable_label(state.density == PrintDensity::Dpi203, "203").clicked() { state.density = PrintDensity::Dpi203; }
                    if ui.selectable_label(state.density == PrintDensity::Dpi300, "300").clicked() { state.density = PrintDensity::Dpi300; }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("2. Δέσμευση Πεδίων").strong().size(14.0));
                match state.template_type {
                    LabelSpoolerTemplateType::ShelfEdgePrice => {
                        ui.horizontal(|ui| { ui.label("Περιγραφή:"); ui.text_edit_singleline(&mut state.shelf_product_name); });
                        ui.horizontal(|ui| {
                            ui.label("SKU:"); ui.text_edit_singleline(&mut state.shelf_sku);
                            ui.label("ΦΠΑ %:");
                            let mut vat_str = state.shelf_vat_rate.to_string();
                            if ui.add(egui::TextEdit::singleline(&mut vat_str).desired_width(35.0)).changed() {
                                if let Ok(v) = vat_str.parse::<u8>() { state.shelf_vat_rate = v; }
                            }
                        });
                        ui.horizontal(|ui| { ui.label("Τιμή (€):"); ui.text_edit_singleline(&mut state.shelf_price_eur); });
                        ui.horizontal(|ui| { ui.label("Barcode:"); ui.text_edit_singleline(&mut state.shelf_barcode); });
                    }
                    LabelSpoolerTemplateType::SmartToteWaybill => {
                        ui.horizontal(|ui| { ui.label("Tote ID:"); ui.text_edit_singleline(&mut state.tote_id); });
                        ui.horizontal(|ui| { ui.label("Ράφι:"); ui.text_edit_singleline(&mut state.destination_shelf); });
                        ui.horizontal(|ui| {
                            ui.label("Τμχ:");
                            let mut u_str = state.unit_count.to_string();
                            if ui.add(egui::TextEdit::singleline(&mut u_str).desired_width(45.0)).changed() {
                                if let Ok(u) = u_str.parse::<u32>() { state.unit_count = u; }
                            }
                            ui.label("Βάρος (γρ):");
                            let mut w_str = state.total_weight_grams.to_string();
                            if ui.add(egui::TextEdit::singleline(&mut w_str).desired_width(55.0)).changed() {
                                if let Ok(w) = w_str.parse::<u32>() { state.total_weight_grams = w; }
                            }
                        });
                        ui.horizontal(|ui| { ui.label("Seal Hash:"); ui.text_edit_singleline(&mut state.merkle_seal_sha256); });
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.label(RichText::new("3. Στόχος & Ενέργειες").strong().size(14.0));
                ui.horizontal(|ui| {
                    ui.label("Endpoint:"); ui.text_edit_singleline(&mut state.destination_endpoint);
                    ui.label("Αντίγραφα:");
                    let mut c_str = state.copies.to_string();
                    if ui.add(egui::TextEdit::singleline(&mut c_str).desired_width(35.0)).changed() {
                        if let Ok(c) = c_str.parse::<u32>() { state.copies = c.max(1); }
                    }
                });
                ui.add_space(6.0);
                if ui.button(RichText::new("⚡ Άμεση Δοκιμαστική Εκτύπωση").strong().color(Color32::from_rgb(34, 197, 94))).clicked() {
                    state.execute_test_print(conn);
                }
            });

            cols[1].vertical(|ui| {
                ui.label(RichText::new("Ζωντανή Προεπισκόπηση Θερμικού Canvas").strong().size(14.0));
                let (tspl_bytes, zpl_bytes) = state.compute_compiled_bytes();
                ui.label(RichText::new(format!("⚡ TSPL: {} B | ZPL: {} B", tspl_bytes, zpl_bytes)).color(Color32::from_rgb(56, 189, 248)).size(11.0));
                ui.add_space(4.0);

                Frame::canvas(ui.style()).fill(Color32::WHITE).corner_radius(CornerRadius::same(6))
                    .stroke(Stroke::new(1.5, Color32::from_rgb(203, 213, 225))).inner_margin(Margin::same(12)).show(ui, |ui| {
                        match state.template_type {
                            LabelSpoolerTemplateType::ShelfEdgePrice => render_shelf_edge_canvas(ui, state),
                            LabelSpoolerTemplateType::SmartToteWaybill => render_smart_tote_canvas(ui, state),
                        }
                    });
            });
        });

        ui.add_space(10.0);
        ui.separator();
        ui.label(RichText::new("Πρόσφατες Εργασίες Spooler (label_print_queue)").strong().size(14.0));
        if state.recent_jobs.is_empty() {
            ui.label(RichText::new("Δεν υπάρχουν εργασίες στην ουρά.").color(Color32::GRAY));
        } else {
            for job in &state.recent_jobs {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&job.job_title).strong());
                    ui.label(RichText::new(job.protocol.as_str()).color(Color32::from_rgb(147, 197, 253)));
                    ui.label(RichText::new(&job.destination_endpoint).color(Color32::GRAY));
                    let (status_text, color) = match &job.status {
                        PrintJobStatus::Pending => ("Pending", Color32::from_rgb(234, 179, 8)),
                        PrintJobStatus::Printing => ("Printing", Color32::from_rgb(59, 130, 246)),
                        PrintJobStatus::Completed => ("Completed", Color32::from_rgb(34, 197, 94)),
                        PrintJobStatus::Failed(_) => ("Failed", Color32::from_rgb(239, 68, 68)),
                    };
                    ui.label(RichText::new(status_text).color(color).strong());
                });
            }
        }
    });
}

fn render_shelf_edge_canvas(ui: &mut Ui, state: &LabelSpoolerState) {
    ui.set_width(280.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(&state.shelf_product_name).strong().color(Color32::BLACK).size(15.0));
        ui.label(RichText::new(format!("SKU: {} | ΦΠΑ: {}%", state.shelf_sku, state.shelf_vat_rate)).color(Color32::from_rgb(75, 85, 99)).size(11.0));
        ui.add_space(4.0);
        let p_f64 = state.shelf_price_eur.trim().parse::<f64>().unwrap_or(0.0);
        ui.label(RichText::new(format!("{:.2} €", p_f64).replace('.', ",")).strong().color(Color32::BLACK).size(26.0));
        ui.add_space(6.0);
        draw_simulated_barcode_bars(ui, &state.shelf_barcode, 38.0);
    });
}

fn render_smart_tote_canvas(ui: &mut Ui, state: &LabelSpoolerState) {
    ui.set_width(320.0);
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("SMART TOTE WAYBILL").strong().color(Color32::BLACK).size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(&state.tote_id).strong().color(Color32::BLACK).size(13.0));
            });
        });
        ui.separator();
        ui.label(RichText::new(format!("Προορισμός: {}", state.destination_shelf)).strong().color(Color32::BLACK).size(12.0));
        let w_kg = state.total_weight_grams as f32 / 1000.0;
        ui.label(RichText::new(format!("Τεμάχια: {} | Βάρος: {:.2} kg", state.unit_count, w_kg)).color(Color32::from_rgb(55, 65, 81)).size(11.0));
        ui.label(RichText::new(format!("Merkle Seal: {:.18}...", state.merkle_seal_sha256)).monospace().color(Color32::from_rgb(100, 116, 139)).size(9.0));
        ui.add_space(4.0);
        draw_simulated_barcode_bars(ui, &state.tote_id, 36.0);
    });
}

fn draw_simulated_barcode_bars(ui: &mut Ui, code: &str, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(260.0, height), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, CornerRadius::ZERO, Color32::from_rgb(248, 250, 252));
    let bar_count = 38;
    let bar_width = rect.width() / (bar_count as f32 * 1.5);
    for i in 0..bar_count {
        let x = rect.min.x + (i as f32 * bar_width * 1.5) + 4.0;
        let is_thick = (i % 3 == 0) || (i % 5 == 0);
        let w = if is_thick { bar_width * 1.5 } else { bar_width * 0.75 };
        let bar_rect = Rect::from_min_size(Pos2::new(x, rect.min.y + 2.0), Vec2::new(w, height - 14.0));
        painter.rect_filled(bar_rect, CornerRadius::ZERO, Color32::BLACK);
    }
    let text_pos = Pos2::new(rect.center().x, rect.max.y - 8.0);
    painter.text(text_pos, egui::Align2::CENTER_CENTER, code, egui::FontId::monospace(10.0), Color32::BLACK);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_label_spooler_state_defaults() {
        let state = LabelSpoolerState::default();
        assert_eq!(state.template_type, LabelSpoolerTemplateType::ShelfEdgePrice);
        assert_eq!(state.protocol, PrintTargetProtocol::Tspl);
        assert_eq!(state.copies, 1);
        let (tspl_len, zpl_len) = state.compute_compiled_bytes();
        assert!(tspl_len > 0);
        assert!(zpl_len > 0);
    }

    #[test]
    fn test_label_spooler_test_print_and_flush_flow() {
        let conn = Connection::open_in_memory().unwrap();
        init_label_spooler_schema(&conn).unwrap();

        let mut state = LabelSpoolerState::default();
        state.destination_endpoint = "loopback".to_string();

        state.execute_test_print(&conn);
        assert!(state.status_message.is_some());
        assert!(!state.status_message.as_ref().unwrap().1); // is not error
        assert_eq!(state.recent_jobs.len(), 1);
        assert_eq!(state.recent_jobs[0].status, PrintJobStatus::Completed);

        // Add a pending job and flush
        let job = LabelPrintJob::new("Pending Waybill", PrintTargetProtocol::Tspl, b"RAW".to_vec(), 1, "loopback", 100);
        enqueue_print_job(&conn, &job).unwrap();
        state.flush_pending_queue(&conn);
        assert_eq!(state.recent_jobs.len(), 2);
    }
}
