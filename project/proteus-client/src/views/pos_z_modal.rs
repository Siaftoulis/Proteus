//! Daily Z Report & Accounting CPA Export Modal Window.
//! Extracted for clean modularity (<400 lines per file).

use egui::RichText;
use rusqlite::{params, Connection};
use super::pos::RetailPosState;

pub fn draw_z_report_modal(ctx: &egui::Context, conn: &Connection, state: &mut RetailPosState) {
    if !state.show_z_report_modal {
        return;
    }

    egui::Window::new("📊 Ημερήσιο Δελτίο Ζ & Γέφυρα Λογιστηρίου")
        .collapsible(false)
        .resizable(false)
        .min_width(420.0)
        .show(ctx, |ui| {
            ui.label(RichText::new("Σύνολα Σημερινών Συναλλαγών").strong().size(16.0));
            ui.separator();

            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let z_query = conn.query_row(
                r#"
                SELECT count(*), coalesce(sum(total_net_eur), 0.0), coalesce(sum(total_vat_eur), 0.0), coalesce(sum(total_gross_eur), 0.0)
                FROM invoices
                WHERE issue_date = ?1 AND is_cancelled = 0
                "#,
                params![today],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?, r.get::<_, f64>(2)?, r.get::<_, f64>(3)?)),
            ).unwrap_or((0, 0.0, 0.0, 0.0));

            ui.label(format!("Ημερομηνία: {}", today));
            ui.label(format!("Πλήθος Αποδείξεων/Τιμολογίων: {}", z_query.0));
            ui.label(format!("Καθαρή Αξία: {:.2} €", z_query.1));
            ui.label(format!("Σύνολο Φ.Π.Α.: {:.2} €", z_query.2));
            ui.label(RichText::new(format!("Γενικό Σύνολο (Τζίρος): {:.2} €", z_query.3)).strong().size(16.0).color(crate::theme::STATUS_READY));

            ui.add_space(10.0);
            if ui.button(RichText::new("📥 Εξαγωγή για Λογιστή (Excel / JSON)").strong()).clicked() {
                let export_json = serde_json::json!({
                    "report": "Daily_Z_Report",
                    "date": today,
                    "receipts_count": z_query.0,
                    "net_eur": z_query.1,
                    "vat_eur": z_query.2,
                    "gross_eur": z_query.3,
                    "software": "Proteus Sovereign BOS"
                });
                let export_str = serde_json::to_string_pretty(&export_json).unwrap_or_default();
                state.exported_file_path = Some(format!("store_z_report_{}.json", today));
                state.status_feedback = Some((format!("Το αρχείο λογιστηρίου εξήχθη επιτυχώς: store_z_report_{}.json ({:.2}€)", today, z_query.3), true));
                let _ = std::fs::write("store_z_report.json", export_str);
            }

            ui.add_space(8.0);
            if ui.button("Κλείσιμο").clicked() {
                state.show_z_report_modal = false;
            }
        });
}
