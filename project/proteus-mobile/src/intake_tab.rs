//! Mobile Fast Intake Form Tab.
//! Zero-privilege mobile device service intake with outbox priority enqueueing.

use crate::{MobileAppState, MobileTab, ACCENT_GOLD, BG_PANEL, MIN_TOUCH_TARGET, TEXT_BODY, TEXT_TITLE};
use egui::{CornerRadius, RichText, Stroke, Ui, Vec2};

pub fn render_intake_tab(ui: &mut Ui, state: &mut MobileAppState) {
    ui.label(RichText::new("Γρήγορη Παραλαβή Συσκευής").strong().size(14.0).color(TEXT_TITLE));
    ui.add_space(6.0);

    ui.label(RichText::new("Ονοματεπώνυμο:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 38.0), egui::TextEdit::singleline(&mut state.intake_customer));

    ui.label(RichText::new("Τηλέφωνο:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 38.0), egui::TextEdit::singleline(&mut state.intake_phone));

    ui.label(RichText::new("Μοντέλο / Συσκευή:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 38.0), egui::TextEdit::singleline(&mut state.intake_device));

    ui.label(RichText::new("Περιγραφή Βλάβης:").size(11.5).color(TEXT_BODY));
    ui.add_sized(Vec2::new(ui.available_width(), 60.0), egui::TextEdit::multiline(&mut state.intake_problem));

    ui.add_space(10.0);
    let submit_btn = ui.add_sized(
        Vec2::new(ui.available_width(), MIN_TOUCH_TARGET),
        egui::Button::new(RichText::new("💾 Καταχώρηση στη SQLite").strong().color(TEXT_TITLE))
            .fill(BG_PANEL)
            .stroke(Stroke::new(1.0, ACCENT_GOLD))
            .corner_radius(CornerRadius::same(6)),
    );

    if submit_btn.clicked() {
        match state.submit_ticket() {
            Ok(id) => {
                state.status_message = Some((format!("✓ Η εντολή {} καταχωρήθηκε επιτυχώς!", id), true));
                state.active_tab = MobileTab::Tickets;
            }
            Err(e) => {
                state.status_message = Some((e, false));
            }
        }
    }
}
