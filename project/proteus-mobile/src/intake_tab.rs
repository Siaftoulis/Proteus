//! Mobile Fast Intake Form Tab (Micro-task 29.1.3).
//! Zero-privilege mobile device service intake with outbox priority enqueueing.
//! Fully shielded against virtual keyboard occlusion via focus-driven scroll cushioning
//! and strictly adhering to 44.0pt touch target accessibility.

use crate::touch_controls::{touch_button, touch_checkbox, touch_segmented_picker, touch_text_input};
use crate::{MobileAppState, MobileTab, TEXT_BODY, TEXT_TITLE};
use egui::{FontId, RichText, Ui};

pub const INTAKE_CATEGORIES: &[(&str, &str)] = &[
    ("Smartphone", "📱 Phone"),
    ("Tablet", "📟 Tablet"),
    ("Laptop", "💻 Laptop"),
    ("Desktop", "🖥 PC"),
];

/// Renders the fast intake form tab wrapped in the virtual keyboard cushion.
pub fn render_intake_tab(ui: &mut Ui, state: &mut MobileAppState) {
    ui.label(RichText::new("Γρήγορη Παραλαβή Συσκευής").strong().font(FontId::proportional(16.0)).color(TEXT_TITLE));
    ui.add_space(4.0);

    let mut cushion = std::mem::take(&mut state.keyboard_cushion);
    cushion.show_cushioned_scroll(ui, "intake_form_scroll", |ui, c| {
        // 1. Device Category Segmented Picker (44pt ergonomics)
        ui.label(RichText::new("Κατηγορία Συσκευής:").font(FontId::proportional(13.0)).color(TEXT_BODY));
        ui.add_space(3.0);
        let categories = [
            ("Smartphone".to_string(), "📱 Phone"),
            ("Tablet".to_string(), "📟 Tablet"),
            ("Laptop".to_string(), "💻 Laptop"),
            ("Desktop".to_string(), "🖥 PC"),
        ];
        touch_segmented_picker(ui, &categories, &mut state.intake_category);
        ui.add_space(10.0);

        // 2. Customer Name Input (44pt touch text edit with clear button)
        ui.label(RichText::new("Ονοματεπώνυμο Πελάτη:").font(FontId::proportional(13.0)).color(TEXT_BODY));
        ui.add_space(3.0);
        touch_text_input(ui, c, &mut state.intake_customer, "π.χ. Γιάννης Παπαδόπουλος");
        ui.add_space(10.0);

        // 3. Phone Number Input (44pt touch text edit with clear button)
        ui.label(RichText::new("Τηλέφωνο Επικοινωνίας:").font(FontId::proportional(13.0)).color(TEXT_BODY));
        ui.add_space(3.0);
        touch_text_input(ui, c, &mut state.intake_phone, "π.χ. 6981234567");
        ui.add_space(10.0);

        // 4. Device Model Input (44pt touch text edit with clear button)
        ui.label(RichText::new("Μοντέλο / Συσκευή:").font(FontId::proportional(13.0)).color(TEXT_BODY));
        ui.add_space(3.0);
        touch_text_input(ui, c, &mut state.intake_device, "π.χ. Samsung Galaxy S23");
        ui.add_space(10.0);

        // 5. Reported Fault Description (Multiline with cushion focus protection)
        ui.label(RichText::new("Περιγραφή Βλάβης / Σημειώσεις:").font(FontId::proportional(13.0)).color(TEXT_BODY));
        ui.add_space(3.0);
        c.cushioned_multiline_edit(ui, &mut state.intake_problem, 64.0);
        ui.add_space(10.0);

        // 6. Urgent / Priority Rush Toggle (44pt touch checkbox)
        touch_checkbox(ui, &mut state.intake_is_urgent, "⚡ Επείγον / Άμεση Προτεραιότητα (+20%)");
        ui.add_space(16.0);

        // 7. 44pt Primary Submit Button
        let submit_resp = touch_button(ui, "💾 Καταχώρηση στη SQLite", true);
        if submit_resp.clicked() {
            if state.intake_customer.trim().is_empty() || state.intake_device.trim().is_empty() {
                state.status_message = Some(("Παρακαλώ συμπληρώστε τουλάχιστον όνομα και συσκευή.".into(), false));
            } else {
                match state.submit_ticket() {
                    Ok(id) => {
                        state.status_message = Some((format!("✓ Η εντολή #{} καταχωρήθηκε επιτυχώς!", id), true));
                        state.intake_customer.clear();
                        state.intake_phone.clear();
                        state.intake_device.clear();
                        state.intake_problem.clear();
                        state.intake_is_urgent = false;
                        state.active_tab = MobileTab::Tickets;
                    }
                    Err(e) => {
                        state.status_message = Some((e, false));
                    }
                }
            }
        }
    });
    state.keyboard_cushion = cushion;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intake_categories_definition() {
        assert_eq!(INTAKE_CATEGORIES.len(), 4);
        assert_eq!(INTAKE_CATEGORIES[0].0, "Smartphone");
        assert_eq!(INTAKE_CATEGORIES[1].0, "Tablet");
        assert_eq!(INTAKE_CATEGORIES[2].0, "Laptop");
        assert_eq!(INTAKE_CATEGORIES[3].0, "Desktop");
    }

    #[test]
    fn test_intake_touch_dimensions_comply_with_44pt() {
        assert!(crate::touch_controls::MIN_TOUCH_DIMENSION >= 44.0);
    }

    #[test]
    fn test_intake_tab_successful_submission() {
        let mut state = MobileAppState::default();
        state.intake_customer = "Νίκος Αντωνίου".into();
        state.intake_device = "iPhone 14 Pro".into();
        state.intake_phone = "6941122334".into();
        state.intake_problem = "Σπασμένη οθόνη αφής".into();
        state.intake_is_urgent = true;

        let res = state.submit_ticket();
        assert!(res.is_ok());
        let ticket_id = res.unwrap();
        assert!(!ticket_id.is_empty());
    }
}
