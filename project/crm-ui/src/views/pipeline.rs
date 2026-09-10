use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use chrono::Utc;
use crate::models::{Deal, Task, TaskPriority, TaskStatus, KANBAN_STAGES};
use crate::theme;
use crate::ProteusApp;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("DEALS").size(9.).color(theme::TEXT_DIM));
    ui.add_space(4.);
    ui.label(egui::RichText::new(format!("Total: {}", app.deals.len())).size(10.).color(theme::TEXT_DIM));
    ui.add_space(4.);
    if ui.add(egui::Button::new(egui::RichText::new("+ New Deal").size(10.).color(theme::ACCENT_GREEN)).fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 22.))).clicked() {
        let id = format!("d{}", app.deals.len() + 1);
        app.deals.push(Deal {
            id: id.clone(),
            title: "New Deal".into(),
            value: 0.0,
            stage: "New".into(),
            contact_id: None,
            expected_close: String::new(),
            notes: String::new(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        });
        app.sel_deal = Some(id);
    }
    ui.add_space(4.);
    let scroll = egui::ScrollArea::vertical().max_height(ui.available_height());
    scroll.show(ui, |ui| {
        for d in &app.deals {
            let sel = app.sel_deal.as_ref() == Some(&d.id);
            let resp = ui.add(
                egui::Button::new(egui::RichText::new(format!("{}\n${:.0} — {}", d.title, d.value, d.stage)).size(10.).color(if sel { theme::ACCENT } else { theme::TEXT }))
                    .fill(if sel { Color32::from_rgb(35, 35, 50) } else { theme::WIDGET_BG })
                    .min_size(egui::vec2(ui.available_width(), 36.))
            );
            if resp.clicked() {
                app.sel_deal = Some(d.id.clone());
            }
        }
    });
}

pub fn show_central(app: &mut ProteusApp, pnt: &egui::Painter, r: Rect, mpos: Option<Pos2>, mup: bool) {
    let stage_width = (r.width() - 48.) / KANBAN_STAGES.len() as f32;
    let header_h = 36.;
    let mpos_canvas = mpos;
    let mup_canvas = mup;

    for (si, stage) in KANBAN_STAGES.iter().enumerate() {
        let cx = r.left() + 8. + si as f32 * (stage_width + 8.);
        let col_r = Rect::from_min_size(egui::pos2(cx, r.top() + 8.), egui::vec2(stage_width, r.height() - 16.));
        pnt.rect_filled(col_r, 4, theme::PANEL);
        pnt.rect_stroke(col_r, 4, Stroke::new(1., theme::BORDER), egui::StrokeKind::Outside);

        // Stage header
        let stage_total: f64 = app.deals.iter().filter(|d| d.stage == *stage).map(|d| d.value).sum();
        let stage_count = app.deals.iter().filter(|d| d.stage == *stage).count();
        pnt.text(egui::pos2(cx + 4., col_r.top() + 8.), egui::Align2::LEFT_CENTER,
            &format!("{}  (${:.0})", stage, stage_total),
            egui::FontId::proportional(10.), theme::ACCENT);
        pnt.text(egui::pos2(cx + stage_width - 4., col_r.top() + 8.), egui::Align2::RIGHT_CENTER,
            &stage_count.to_string(),
            egui::FontId::proportional(10.), theme::TEXT_DIM);

        // Deals in this stage
        let stage_deals: Vec<(usize, Deal)> = app.deals.iter().enumerate()
            .filter(|(_, d)| d.stage == *stage)
            .map(|(i, d)| (i, d.clone()))
            .collect();

        let mut y = col_r.top() + header_h + 4.;
        for (deal_idx, d) in &stage_deals {
            if y + 60. > col_r.bottom() { break; }
            let card_r = Rect::from_min_size(egui::pos2(cx + 4., y), egui::vec2(stage_width - 8., 56.));
            let is_sel = app.sel_deal.as_ref() == Some(&d.id);

            // Click to select
            if let Some(pos) = mpos_canvas {
                if card_r.contains(pos) && mup_canvas {
                    app.sel_deal = Some(d.id.clone());
                }
            }

            pnt.rect_filled(card_r, 3, if is_sel { Color32::from_rgb(45, 45, 60) } else { theme::WIDGET_BG });
            pnt.rect_stroke(card_r, 3, Stroke::new(1., if is_sel { theme::ACCENT } else { theme::BORDER }), egui::StrokeKind::Outside);

            pnt.text(egui::pos2(cx + 8., y + 4.), egui::Align2::LEFT_TOP,
                &d.title, egui::FontId::proportional(10.), theme::TEXT);
            pnt.text(egui::pos2(cx + 8., y + 18.), egui::Align2::LEFT_TOP,
                &format!("${:.0}", d.value), egui::FontId::proportional(11.), theme::ACCENT_GREEN);

            if let Some(cid) = &d.contact_id {
                if let Some(c) = app.contacts.iter().find(|c| &c.id == cid) {
                    pnt.text(egui::pos2(cx + 8., y + 32.), egui::Align2::LEFT_TOP,
                        &c.name, egui::FontId::proportional(8.), theme::TEXT_DIM);
                }
            }

            // Quick stage move arrows
            let left_arrow_r = Rect::from_min_size(egui::pos2(card_r.right() - 28., y + 36.), egui::vec2(12., 14.));
            let right_arrow_r = Rect::from_min_size(egui::pos2(card_r.right() - 14., y + 36.), egui::vec2(12., 14.));

            if si > 0 {
                pnt.text(left_arrow_r.center(), egui::Align2::CENTER_CENTER, "◀", egui::FontId::proportional(8.), theme::TEXT_DIM);
                if let Some(pos) = mpos_canvas {
                    if left_arrow_r.contains(pos) && mup_canvas {
                        if let Some(deal) = app.deals.get_mut(*deal_idx) {
                            deal.stage = KANBAN_STAGES[si - 1].to_string();
                            deal.updated_at = Utc::now().to_rfc3339();
                        }
                    }
                }
            }
            if si < KANBAN_STAGES.len() - 1 {
                pnt.text(right_arrow_r.center(), egui::Align2::CENTER_CENTER, "▶", egui::FontId::proportional(8.), theme::TEXT_DIM);
                if let Some(pos) = mpos_canvas {
                    if right_arrow_r.contains(pos) && mup_canvas {
                        if let Some(deal) = app.deals.get_mut(*deal_idx) {
                            deal.stage = KANBAN_STAGES[si + 1].to_string();
                            deal.updated_at = Utc::now().to_rfc3339();
                        }
                    }
                }
            }

            y += 62.;
        }
    }

    // Move deal between stages: click stage header with a deal selected
    if let Some(sel_id) = &app.sel_deal.clone() {
        if mup_canvas {
            if let Some(pos) = mpos_canvas {
                for (si, stage) in KANBAN_STAGES.iter().enumerate() {
                    let cx = r.left() + 8. + si as f32 * (stage_width + 8.);
                    let header_r = Rect::from_min_size(egui::pos2(cx + 4., r.top() + 8.), egui::vec2(stage_width - 8., 28.));
                    if header_r.contains(pos) {
                        let (prev, deal_title, deal_contact) = if let Some(d) = app.deals.iter_mut().find(|d| &d.id == sel_id) {
                            let p = d.stage.clone();
                            d.stage = stage.to_string();
                            d.updated_at = Utc::now().to_rfc3339();
                            (p, d.title.clone(), d.contact_id.clone())
                        } else { continue; };
                        app.toast(format!("{} → {}", prev, stage));
                        let tid = format!("t{}", app.tasks.len() + 1);
                        app.tasks.push(Task {
                            id: tid,
                            title: format!("Follow-up: {} moved to {}", deal_title, stage),
                            description: String::new(),
                            contact_id: deal_contact,
                            deal_id: Some(sel_id.clone()),
                            assignee: "me".into(),
                            due_date: (Utc::now() + chrono::Duration::days(3)).format("%Y-%m-%d").to_string(),
                            status: TaskStatus::Open,
                            priority: TaskPriority::Medium,
                            created_by: "me".into(),
                            created_at: Utc::now().to_rfc3339(),
                            updated_at: Utc::now().to_rfc3339(),
                            completed_at: None,
                        });
                    }
                }
            }
        }
    }

    // Hint text
    pnt.text(egui::pos2(r.center().x, r.bottom() - 20.), egui::Align2::CENTER_CENTER,
        "Select a deal card, then click a stage header to move it",
        egui::FontId::proportional(9.), theme::TEXT_DIM);
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    if let Some(id) = &app.sel_deal.clone() {
        if let Some(d) = app.deals.iter_mut().find(|d| &d.id == id) {
            ui.label(egui::RichText::new(&d.title).size(12.).color(theme::TEXT));
            ui.label(egui::RichText::new(format!("${:.0}", d.value)).size(11.).color(theme::ACCENT_GREEN));
            ui.add_space(4.);
            ui.label(egui::RichText::new(format!("Stage: {}", d.stage)).size(10.).color(theme::ACCENT));
            if let Some(cid) = &d.contact_id {
                if let Some(c) = app.contacts.iter().find(|c| &c.id == cid) {
                    ui.label(egui::RichText::new(&c.name).size(9.).color(theme::TEXT));
                }
            }
            ui.add_space(4.);
            ui.label(egui::RichText::new("Expected Close:").size(9.).color(theme::TEXT_DIM));
            ui.label(egui::RichText::new(&d.expected_close).size(10.).color(theme::TEXT));
            ui.separator();
            ui.label(egui::RichText::new("Notes").size(9.).color(theme::TEXT_DIM));
            let mut notes = d.notes.clone();
            if ui.text_edit_multiline(&mut notes).changed() {
                d.notes = notes;
            }
            ui.add_space(8.);
            let delete = ui.button(egui::RichText::new("Delete Deal").size(10.).color(theme::ACCENT_RED)).clicked();
            if delete {
                let did = id.clone();
                app.deals.retain(|x| x.id != did);
                app.sel_deal = None;
            }
        }
    } else {
        ui.label(egui::RichText::new("Select a deal").size(10.).color(theme::TEXT_DIM));
    }
}
