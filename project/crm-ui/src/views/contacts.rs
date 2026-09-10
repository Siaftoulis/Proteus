use eframe::egui::{self, Color32, Rect};
use chrono::Utc;
use crate::models::{Contact, ContactNote};
use crate::theme;
use crate::ProteusApp;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("CONTACTS").size(9.).color(theme::TEXT_DIM));
    ui.add_space(2.);
    ui.add(egui::TextEdit::singleline(&mut app.search_query).hint_text("Search...").desired_width(ui.available_width()));
    ui.add_space(4.);
    if ui.add(egui::Button::new(egui::RichText::new("+ New Contact").size(10.).color(theme::ACCENT_GREEN)).fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 22.))).clicked() {
        let id = format!("c{}", app.contacts.len() + 1);
        let contact = Contact {
            id: id.clone(),
            name: "New Contact".into(),
            email: String::new(),
            phone: String::new(),
            company: String::new(),
            tags: vec![],
            notes: vec![],
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };
        app.sync_contact_to_db(&contact);
        app.contacts.push(contact);
        app.sel_contact = Some(id.clone());
        app.editing_contact = Some(id);
        app.edit_name = "New Contact".into();
        app.edit_email.clear();
        app.edit_phone.clear();
        app.edit_company.clear();
        app.edit_tags.clear();
    }
    ui.add_space(2.);
    ui.horizontal(|ui| {
        let btn_w = (ui.available_width() - 4.) / 2.;
        if ui.add(egui::Button::new(egui::RichText::new("📥 Export CSV").size(9.).color(theme::TEXT)).fill(theme::WIDGET_BG).min_size(egui::vec2(btn_w, 20.))).on_hover_text("Export contacts to CSV file & clipboard").clicked() {
            let csv = crate::csv_utils::export_contacts_csv(&app.contacts);
            let _ = std::fs::write("contacts_export.csv", &csv);
            ui.ctx().copy_text(csv);
            app.toast("Exported contacts to contacts_export.csv & clipboard!");
        }
        if ui.add(egui::Button::new(egui::RichText::new("📤 Import CSV").size(9.).color(theme::TEXT)).fill(theme::WIDGET_BG).min_size(egui::vec2(btn_w, 20.))).on_hover_text("Import contacts from contacts_import.csv or contacts_export.csv").clicked() {
            let path = if std::path::Path::new("contacts_import.csv").exists() {
                "contacts_import.csv"
            } else if std::path::Path::new("contacts_export.csv").exists() {
                "contacts_export.csv"
            } else {
                ""
            };
            if !path.is_empty() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let imported = crate::csv_utils::import_contacts_csv(&content);
                    let count = imported.len();
                    for c in imported {
                        app.sync_contact_to_db(&c);
                        app.contacts.retain(|x| x.id != c.id);
                        app.contacts.push(c);
                    }
                    app.toast(format!("Imported {} contacts from {}!", count, path));
                }
            } else {
                app.toast("Place contacts_import.csv in app directory to import");
            }
        }
    });
    ui.add_space(4.);
    let q = app.search_query.to_lowercase();
    let filtered: Vec<usize> = app.contacts.iter().enumerate()
        .filter(|(_, c)| q.is_empty() || c.name.to_lowercase().contains(&q) || c.email.to_lowercase().contains(&q) || c.company.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect();

    let scroll = egui::ScrollArea::vertical().max_height(ui.available_height());
    scroll.show(ui, |ui| {
        for &i in &filtered {
            let c = &app.contacts[i];
            let sel = app.sel_contact.as_ref() == Some(&c.id);
            let resp = ui.add(
                egui::Button::new(egui::RichText::new(format!("{}\n{}", c.name, c.email)).size(10.).color(if sel { theme::ACCENT } else { theme::TEXT }))
                    .fill(if sel { Color32::from_rgb(35, 35, 50) } else { theme::WIDGET_BG })
                    .min_size(egui::vec2(ui.available_width(), 36.))
            );
            if resp.clicked() {
                app.sel_contact = Some(c.id.clone());
                app.editing_contact = None;
            }
        }
    });
}

pub fn show_central(app: &mut ProteusApp, pnt: &egui::Painter, r: Rect, ui: &mut egui::Ui) {
    let scroll = egui::ScrollArea::vertical().max_height(r.height());
    scroll.show_viewport(ui, |_ui, _| {
        if let Some(id) = &app.sel_contact.clone() {
            if let Some(c) = app.contacts.iter().find(|c| &c.id == id) {
                pnt.text(egui::pos2(r.left() + 20., r.top() + 20.), egui::Align2::LEFT_TOP,
                    &format!("{} — {} — {}", c.name, c.email, c.phone),
                    egui::FontId::proportional(16.), theme::TEXT);
                pnt.text(egui::pos2(r.left() + 20., r.top() + 44.), egui::Align2::LEFT_TOP,
                    &format!("Company: {}  |  Created: {}  |  Updated: {}", c.company, &c.created_at[..10.min(c.created_at.len())], &c.updated_at[..10.min(c.updated_at.len())]),
                    egui::FontId::proportional(10.), theme::TEXT_DIM);
                pnt.text(egui::pos2(r.left() + 20., r.top() + 64.), egui::Align2::LEFT_TOP,
                    &format!("Tags: {}", c.tags.join(", ")),
                    egui::FontId::proportional(10.), theme::ACCENT_ORANGE);
                pnt.text(egui::pos2(r.left() + 20., r.top() + 90.), egui::Align2::LEFT_TOP,
                    "Notes Timeline:",
                    egui::FontId::proportional(11.), theme::TEXT_DIM);
                for (i, n) in c.notes.iter().enumerate() {
                    let ny = r.top() + 110. + i as f32 * 28.;
                    pnt.rect_filled(Rect::from_min_size(egui::pos2(r.left() + 20., ny), egui::vec2(r.width() - 40., 24.)), 2, theme::PANEL);
                    pnt.text(egui::pos2(r.left() + 28., ny + 4.), egui::Align2::LEFT_TOP, &n.text, egui::FontId::proportional(10.), theme::TEXT);
                    pnt.text(egui::pos2(r.left() + 28., ny + 16.), egui::Align2::LEFT_TOP, &n.created_at[..10.min(n.created_at.len())], egui::FontId::proportional(7.), theme::TEXT_DIM);
                }
            } else {
                pnt.text(r.center(), egui::Align2::CENTER_CENTER, "Contact not found", egui::FontId::proportional(14.), theme::TEXT_DIM);
            }
        } else {
            pnt.text(r.center(), egui::Align2::CENTER_CENTER, "Select a contact from the left panel", egui::FontId::proportional(14.), theme::TEXT_DIM);
        }
    });
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let sel_id = app.sel_contact.clone();
    let mut contact_to_sync: Option<Contact> = None;
    let del_clicked = if let Some(ref id) = sel_id {
        if let Some(c) = app.contacts.iter_mut().find(|c| &c.id == id) {
            let editing = app.editing_contact.as_deref() == Some(&c.id);
            if ui.button(egui::RichText::new(if editing { "Done Editing" } else { "Edit" }).size(10.).color(theme::ACCENT)).clicked() {
                if editing {
                    app.editing_contact = None;
                    c.updated_at = Utc::now().to_rfc3339();
                    contact_to_sync = Some(c.clone());
                } else {
                    app.editing_contact = Some(c.id.clone());
                    app.edit_name = c.name.clone();
                    app.edit_email = c.email.clone();
                    app.edit_phone = c.phone.clone();
                    app.edit_company = c.company.clone();
                    app.edit_tags = c.tags.join(", ");
                }
            }
            let ret = ui.button(egui::RichText::new("Delete").size(10.).color(theme::ACCENT_RED)).clicked();
            ui.add_space(4.);
            if editing {
                ui.horizontal(|ui| { ui.label("Email:"); ui.text_edit_singleline(&mut app.edit_email); });
                ui.horizontal(|ui| { ui.label("Phone:"); ui.text_edit_singleline(&mut app.edit_phone); });
                ui.horizontal(|ui| { ui.label("Co:"); ui.text_edit_singleline(&mut app.edit_company); });
                ui.horizontal(|ui| { ui.label("Tags:"); ui.text_edit_singleline(&mut app.edit_tags); });
                if ui.button(egui::RichText::new("Save").size(10.).color(theme::ACCENT_GREEN)).clicked() {
                    c.name = app.edit_name.clone();
                    c.email = app.edit_email.clone();
                    c.phone = app.edit_phone.clone();
                    c.company = app.edit_company.clone();
                    c.tags = app.edit_tags.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    c.updated_at = Utc::now().to_rfc3339();
                    contact_to_sync = Some(c.clone());
                    app.editing_contact = None;
                }
            } else {
                ui.label(egui::RichText::new(&c.name).size(12.).color(theme::TEXT));
                ui.label(egui::RichText::new(&c.email).size(10.).color(theme::ACCENT));
                ui.label(egui::RichText::new(&c.phone).size(10.).color(theme::TEXT));
                ui.label(egui::RichText::new(&c.company).size(10.).color(theme::TEXT));
                ui.add_space(4.);
                if !c.tags.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        for t in &c.tags {
                            ui.label(egui::RichText::new(t).size(9.).color(theme::ACCENT_ORANGE).background_color(theme::WIDGET_BG));
                        }
                    });
                }
                ui.add_space(4.);
                ui.label(egui::RichText::new("Notes Timeline").size(9.).color(theme::TEXT_DIM));
                ui.separator();
                let scroll = egui::ScrollArea::vertical().max_height(ui.available_height() - 60.);
                scroll.show(ui, |ui| {
                    for n in &c.notes {
                        ui.add_space(2.);
                        ui.label(egui::RichText::new(&n.text).size(9.).color(theme::TEXT));
                        ui.label(egui::RichText::new(&n.created_at[..10.min(n.created_at.len())]).size(7.).color(theme::TEXT_DIM));
                    }
                });
                ui.add_space(4.);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        egui::vec2(ui.available_width() - 40., 18.),
                        egui::TextEdit::singleline(&mut app.new_note_text).hint_text("Add note...").desired_width(f32::INFINITY),
                    );
                    if ui.button(egui::RichText::new("+").size(12.).color(theme::ACCENT_GREEN)).clicked() && !app.new_note_text.is_empty() {
                        c.notes.push(ContactNote {
                            id: format!("n{}", c.notes.len() + 1),
                            text: app.new_note_text.clone(),
                            created_at: Utc::now().to_rfc3339(),
                        });
                        c.updated_at = Utc::now().to_rfc3339();
                        contact_to_sync = Some(c.clone());
                        app.new_note_text.clear();
                    }
                });
            }
            ret
        } else { false }
    } else { false };

    if let Some(contact) = contact_to_sync {
        app.sync_contact_to_db(&contact);
    }

    if del_clicked {
        let did = app.sel_contact.clone().unwrap();
        app.delete_contact_from_db(&did);
        app.contacts.retain(|x| x.id != did);
        app.sel_contact = None;
        app.editing_contact = None;
    }
}
