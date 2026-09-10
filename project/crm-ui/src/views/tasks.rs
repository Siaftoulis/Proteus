use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use chrono::Utc;
use crate::models::{
    task_priority_color, task_priority_label, task_status_icon, task_status_label,
    Task, TaskPriority, TaskStatus,
};
use crate::theme;
use crate::ProteusApp;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("TASKS").size(9.).color(theme::TEXT_DIM));
    ui.add_space(2.);
    if ui.add(egui::Button::new(egui::RichText::new("+ New Task").size(10.).color(theme::ACCENT_GREEN)).fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 22.))).clicked() {
        let id = format!("t{}", app.tasks.len() + 1);
        app.tasks.push(Task {
            id: id.clone(),
            title: "New Task".into(),
            description: String::new(),
            contact_id: None,
            deal_id: None,
            assignee: "me".into(),
            due_date: (Utc::now() + chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
            status: TaskStatus::Open,
            priority: TaskPriority::Medium,
            created_by: "me".into(),
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
            completed_at: None,
        });
        app.sel_task = Some(id);
    }
    ui.add_space(4.);
    // Filter: status
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Status:").size(9.).color(theme::TEXT_DIM));
        if ui.selectable_label(app.task_filter_status.is_none(), "All").clicked() { app.task_filter_status = None; }
        for s in [TaskStatus::Open, TaskStatus::InProgress, TaskStatus::Done] {
            if ui.selectable_label(app.task_filter_status.as_ref() == Some(&s), task_status_label(&s)).clicked() {
                app.task_filter_status = Some(s);
            }
        }
    });
    ui.add_space(2.);
    // Filter: priority
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Priority:").size(9.).color(theme::TEXT_DIM));
        if ui.selectable_label(app.task_filter_priority.is_none(), "All").clicked() { app.task_filter_priority = None; }
        for p in [TaskPriority::High, TaskPriority::Medium, TaskPriority::Low] {
            if ui.selectable_label(app.task_filter_priority.as_ref() == Some(&p), task_priority_label(&p)).clicked() {
                app.task_filter_priority = Some(p);
            }
        }
    });
    ui.add_space(4.);
    let filtered: Vec<usize> = app.tasks.iter().enumerate().filter(|(_, t)| {
        let ok_status = app.task_filter_status.as_ref().map_or(true, |s| t.status == *s);
        let ok_priority = app.task_filter_priority.as_ref().map_or(true, |p| t.priority == *p);
        ok_status && ok_priority
    }).map(|(i, _)| i).collect();

    let scroll = egui::ScrollArea::vertical().max_height(ui.available_height() - 120.);
    scroll.show(ui, |ui| {
        for &i in &filtered {
            let t = &app.tasks[i];
            let sel = app.sel_task.as_ref() == Some(&t.id);
            let color = task_priority_color(&t.priority);
            let label = format!("{}  {}  {}", task_status_icon(&t.status), t.title, if t.due_date.len() >= 10 { &t.due_date[..10] } else { "" });
            if ui.add(egui::Button::new(egui::RichText::new(&label).size(10.).color(if sel { theme::ACCENT } else { color })).fill(if sel { Color32::from_rgb(35, 35, 50) } else { theme::WIDGET_BG }).min_size(egui::vec2(ui.available_width(), 20.))).clicked() {
                app.sel_task = Some(t.id.clone());
            }
        }
    });
}

pub fn show_central(app: &mut ProteusApp, pnt: &egui::Painter, r: Rect, mpos: Option<Pos2>, mup: bool, ui: &mut egui::Ui) {
    let scroll = egui::ScrollArea::vertical().max_height(r.height());
    scroll.show_viewport(ui, |_ui, _| {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let mut overdue: Vec<&Task> = vec![];
        let mut today_tasks: Vec<&Task> = vec![];
        let mut upcoming: Vec<&Task> = vec![];
        for t in &app.tasks {
            if t.status == TaskStatus::Done || t.status == TaskStatus::Cancelled { continue; }
            if t.due_date < today { overdue.push(t); }
            else if t.due_date == today { today_tasks.push(t); }
            else { upcoming.push(t); }
        }
        let mut y = r.top() + 20.;
        // Sections
        for (label, items, color) in [
            ("OVERDUE", &overdue, theme::ACCENT_RED),
            ("TODAY", &today_tasks, theme::ACCENT_ORANGE),
            ("UPCOMING", &upcoming, theme::ACCENT_GREEN),
        ] {
            pnt.text(egui::pos2(r.left() + 20., y), egui::Align2::LEFT_TOP,
                &format!("{}  ({})", label, items.len()),
                egui::FontId::proportional(12.), color);
            y += 24.;
            if items.is_empty() {
                pnt.text(egui::pos2(r.left() + 30., y), egui::Align2::LEFT_TOP,
                    "None", egui::FontId::proportional(10.), theme::TEXT_DIM);
                y += 20.;
            }
            for t in items.iter() {
                let card_r = Rect::from_min_size(egui::pos2(r.left() + 20., y), egui::vec2(r.width() - 60., 36.));
                let sel = app.sel_task.as_ref() == Some(&t.id);
                pnt.rect_filled(card_r, 2, if sel { Color32::from_rgb(35, 35, 50) } else { theme::PANEL });
                pnt.rect_stroke(card_r, 2, Stroke::new(if sel { 2. } else { 1. }, if sel { theme::SELECTED } else { theme::BORDER }), egui::StrokeKind::Outside);
                // Priority color bar
                pnt.rect_filled(Rect::from_min_size(card_r.min, egui::vec2(3., card_r.height())), 2, task_priority_color(&t.priority));
                pnt.text(egui::pos2(card_r.left() + 10., card_r.top() + 6.), egui::Align2::LEFT_TOP,
                    &format!("{}  {}", task_status_icon(&t.status), t.title),
                    egui::FontId::proportional(10.), theme::TEXT);
                pnt.text(egui::pos2(card_r.left() + 10., card_r.top() + 20.), egui::Align2::LEFT_TOP,
                    &format!("Due: {}  |  {}", t.due_date, t.assignee),
                    egui::FontId::proportional(8.), theme::TEXT_DIM);
                // Click to select
                if let Some(pos) = mpos {
                    if mup && card_r.contains(pos) {
                        app.sel_task = Some(t.id.clone());
                    }
                }
                y += 40.;
            }
            y += 8.;
        }
        // Quick-add task bar at bottom
        let qr = Rect::from_min_size(egui::pos2(r.left() + 20., r.bottom() - 40.), egui::vec2(r.width() - 60., 28.));
        pnt.rect_filled(qr, 0, theme::WIDGET_BG);
        pnt.rect_stroke(qr, 0, Stroke::new(1., theme::BORDER), egui::StrokeKind::Outside);
    });

    // Quick-add is drawn via painter — handle input separately
    let qr = Rect::from_min_size(egui::pos2(r.left() + 20., r.bottom() - 40.), egui::vec2(r.width() - 60., 28.));
    if let Some(pos) = mpos {
        if mup && qr.contains(pos) {
            let id = format!("t{}", app.tasks.len() + 1);
            app.tasks.push(Task {
                id: id.clone(),
                title: "New Task".into(),
                description: String::new(),
                contact_id: None,
                deal_id: None,
                assignee: "me".into(),
                due_date: (Utc::now() + chrono::Duration::days(7)).format("%Y-%m-%d").to_string(),
                status: TaskStatus::Open,
                priority: TaskPriority::Medium,
                created_by: "me".into(),
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
                completed_at: None,
            });
            app.sel_task = Some(id);
            app.toast("Task created — edit details in right panel");
        }
    }
    // Draw the quick-add text on top
    pnt.text(qr.center(), egui::Align2::CENTER_CENTER,
        "+ Quick Add Task",
        egui::FontId::proportional(10.), theme::TEXT_DIM);
    if app.tasks.is_empty() {
        pnt.text(r.center(), egui::Align2::CENTER_CENTER,
            "No tasks yet. Click '+ Quick Add Task' to create your first one.",
            egui::FontId::proportional(14.), theme::TEXT_DIM);
    }
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    if let Some(id) = &app.sel_task.clone() {
        if let Some(t) = app.tasks.iter_mut().find(|t| &t.id == id) {
            ui.label(egui::RichText::new(&t.title).size(12.).color(theme::TEXT));
            ui.add_space(2.);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Status:").size(9.).color(theme::TEXT_DIM));
                for s in [TaskStatus::Open, TaskStatus::InProgress, TaskStatus::Done, TaskStatus::Cancelled] {
                    if ui.selectable_label(t.status == s, task_status_label(&s)).clicked() {
                        t.status = s.clone();
                        t.updated_at = Utc::now().to_rfc3339();
                        if s == TaskStatus::Done {
                            t.completed_at = Some(Utc::now().to_rfc3339());
                        }
                    }
                }
            });
            ui.add_space(2.);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Priority:").size(9.).color(theme::TEXT_DIM));
                for p in [TaskPriority::Low, TaskPriority::Medium, TaskPriority::High] {
                    let _col = task_priority_color(&p);
                    if ui.selectable_label(t.priority == p, task_priority_label(&p)).clicked() {
                        t.priority = p.clone();
                        t.updated_at = Utc::now().to_rfc3339();
                    }
                }
            });
            ui.add_space(4.);
            ui.label(egui::RichText::new("Due:").size(9.).color(theme::TEXT_DIM));
            let mut due = t.due_date.clone();
            if ui.text_edit_singleline(&mut due).changed() {
                t.due_date = due;
                t.updated_at = Utc::now().to_rfc3339();
            }
            ui.add_space(2.);
            ui.label(egui::RichText::new("Assignee:").size(9.).color(theme::TEXT_DIM));
            let mut assignee = t.assignee.clone();
            if ui.text_edit_singleline(&mut assignee).changed() {
                t.assignee = assignee;
            }
            ui.add_space(4.);
            ui.label(egui::RichText::new("Notes:").size(9.).color(theme::TEXT_DIM));
            let mut desc = t.description.clone();
            if ui.text_edit_multiline(&mut desc).changed() {
                t.description = desc;
            }
            ui.add_space(4.);
            if let Some(cid) = &t.contact_id {
                if let Some(c) = app.contacts.iter().find(|c| &c.id == cid) {
                    ui.label(egui::RichText::new(format!("Contact: {}", c.name)).size(9.).color(theme::ACCENT));
                }
            }
            ui.add_space(4.);
            let del = ui.button(egui::RichText::new("Delete Task").size(10.).color(theme::ACCENT_RED)).clicked();
            if del {
                let did = id.clone();
                app.tasks.retain(|x| x.id != did);
                app.sel_task = None;
            }
        }
    } else {
        ui.label(egui::RichText::new("Select a task").size(10.).color(theme::TEXT_DIM));
    }
}
