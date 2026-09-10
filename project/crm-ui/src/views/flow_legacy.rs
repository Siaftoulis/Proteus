use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use crate::models::UiFlowNode;
use crate::theme;
use crate::ProteusApp;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("NODES").size(9.).color(theme::TEXT_DIM));
    ui.add_space(4.);
    let nodes: &[(&str, &str, &str, Color32, Option<serde_json::Value>)] = &[
        (">", "On Click", "trigger", theme::HEADER_TRIGGER, None),
        (">", "Create Record", "action", theme::HEADER_ACTION, None),
        (">", "Update Record", "action", theme::HEADER_ACTION, None),
        (">", "Send Email", "action", theme::HEADER_ACTION, None),
        (">", "API Call", "action", theme::HEADER_ACTION, None),
        ("?", "Check Field", "condition", theme::HEADER_CONDITION, None),
        ("?", "Compare Dates", "condition", theme::HEADER_CONDITION, None),
        ("&", "AND Gate", "logicalGate", theme::HEADER_GATE, Some(serde_json::json!({"gateType":"and"}))),
        ("|", "OR Gate", "logicalGate", theme::HEADER_GATE, Some(serde_json::json!({"gateType":"or"}))),
    ];
    for (icon, label, nt, color, extra) in nodes {
        if ui.add(
            egui::Button::new(egui::RichText::new(format!("{} {}", icon, label)).size(11.).color(*color))
                .fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 24.))
        ).clicked() {
            app.add_fn(nt, label, extra.clone());
        }
    }
}

pub fn show_central(app: &mut ProteusApp, pnt: &egui::Painter, r: Rect, mpos: Option<Pos2>, mdown: bool, mup: bool) {
    let to_c = |pos: Pos2| egui::pos2(pos.x - r.left(), pos.y - r.top());

    let node_handles = |n: &UiFlowNode| -> (Pos2, Pos2) {
        (egui::pos2(r.left() + n.x + 100., r.top() + n.y),
         egui::pos2(r.left() + n.x + 100., r.top() + n.y + 60.))
    };
    let handle_hit = |pos: Pos2, center: Pos2| -> bool {
        (pos - center).length() < 20.
    };

    let hovered_handle = mpos.and_then(|pos| {
        app.fns.iter().find_map(|n| {
            let (top, bot) = node_handles(n);
            if handle_hit(pos, top) || handle_hit(pos, bot) { Some(n.id.clone()) }
            else { None }
        })
    });

    // Draw edges
    for (_, src, tgt) in &app.fes {
        let s = app.fns.iter().find(|n| n.id == *src);
        let d = app.fns.iter().find(|n| n.id == *tgt);
        if let (Some(sn), Some(dn)) = (s, d) {
            let (_, sp) = node_handles(sn);
            let (dp, _) = node_handles(dn);
            pnt.line_segment([sp, dp], Stroke::new(1.5, theme::ACCENT));
            let dir = (dp - sp).normalized();
            let per = egui::vec2(-dir.y, dir.x);
            pnt.line_segment([dp, dp - dir * 8. + per * 4.], Stroke::new(1.5, theme::ACCENT));
            pnt.line_segment([dp, dp - dir * 8. - per * 4.], Stroke::new(1.5, theme::ACCENT));
        }
    }

    // Temporary connection line
    if let (Some(src_id), Some(pos)) = (&app.flow_con, mpos) {
        if let Some(sn) = app.fns.iter().find(|n| n.id == *src_id) {
            let (_, sp) = node_handles(sn);
            let mid = egui::pos2((sp.x + pos.x) / 2., sp.y.max(pos.y).min(sp.y + (pos.y - sp.y).abs() / 2.));
            pnt.line_segment([sp, mid], Stroke::new(2., theme::ACCENT));
            pnt.line_segment([mid, pos], Stroke::new(2., theme::ACCENT_ORANGE));
            pnt.circle_stroke(pos, 12., Stroke::new(1.5, theme::ACCENT_ORANGE));
        }
    }

    // Node drag
    if mpos.is_some() && hovered_handle.is_none() {
        if mdown && app.fdrag.is_none() && app.flow_con.is_none() {
            let c = to_c(mpos.unwrap());
            for n in app.fns.iter().rev() {
                if Rect::from_min_size(egui::pos2(n.x, n.y), egui::vec2(200., 60.)).contains(c) {
                    app.sel_fn = Some(n.id.clone());
                    app.fdrag = Some((n.id.clone(), egui::vec2(n.x, n.y), egui::vec2(c.x, c.y)));
                    break;
                }
            }
        }
    }
    if let Some((id, sp, off)) = app.fdrag.clone() {
        if mdown && app.flow_con.is_none() {
            if let Some(pos) = mpos {
                let c = to_c(pos);
                if let Some(n) = app.fns.iter_mut().find(|n| n.id == id) {
                    n.x = (sp.x + c.x - off.x).max(0.);
                    n.y = (sp.y + c.y - off.y).max(0.);
                }
            }
        } else {
            app.fdrag = None;
        }
    }

    // Handle clicks on node handles
    if mup {
        if let Some(pos) = mpos {
            for n in &app.fns {
                let (top_h, bot_h) = node_handles(n);
                let on_handle = handle_hit(pos, top_h) || handle_hit(pos, bot_h);
                if on_handle {
                    if app.flow_con.is_none() {
                        app.flow_con = Some(n.id.clone());
                        app.toast("Click another node to connect");
                    } else if let Some(src) = &app.flow_con {
                        if *src != n.id {
                            let eid = format!("e{}", app.fes.len() + 1);
                            app.fes.push((eid, src.clone(), n.id.clone()));
                            app.toast("Connected ✓");
                        }
                        app.flow_con = None;
                    }
                    return;
                }
            }
            if app.flow_con.is_some() {
                app.flow_con = None;
                app.toast("Connection cancelled");
            }
        }
    }

    // Draw nodes
    let mut fns_sorted: Vec<_> = app.fns.iter().enumerate().collect();
    fns_sorted.sort_by_key(|(_, n)| if app.sel_fn.as_ref() == Some(&n.id) { 1 } else { 0 });
    for (_, n) in &fns_sorted {
        let nr = Rect::from_min_size(egui::pos2(r.left() + n.x, r.top() + n.y), egui::vec2(200., 60.));
        let sel = app.sel_fn.as_ref() == Some(&n.id);
        let hcol = match n.nt.as_str() {
            "trigger" => theme::HEADER_TRIGGER,
            "action" => theme::HEADER_ACTION,
            "condition" => theme::HEADER_CONDITION,
            "logicalGate" => theme::HEADER_GATE,
            _ => theme::BORDER,
        };
        pnt.rect_filled(nr, 0, theme::WIDGET_BG);
        pnt.rect_stroke(nr, 0, Stroke::new(if sel { 2. } else { 1. }, if sel { theme::SELECTED } else { theme::BORDER }), egui::StrokeKind::Outside);

        let hr = Rect::from_min_size(nr.min, egui::vec2(200., 20.));
        pnt.rect_filled(hr, 0, hcol);
        pnt.text(egui::pos2(hr.left() + 6., hr.center().y), egui::Align2::LEFT_CENTER, &n.nt, egui::FontId::proportional(9.), Color32::from_rgb(0, 0, 0));
        pnt.text(egui::pos2(nr.left() + 6., nr.top() + 34.), egui::Align2::LEFT_CENTER, &n.label, egui::FontId::proportional(10.), theme::TEXT);

        let (th, bh) = node_handles(n);
        let is_hovered = hovered_handle.as_deref() == Some(&n.id);
        let hr_col = if is_hovered { theme::ACCENT } else { hcol };
        pnt.circle_filled(th, 6., if is_hovered { theme::ACCENT_DIM } else { theme::BORDER });
        pnt.circle_stroke(th, 6., Stroke::new(1.5, hr_col));
        pnt.circle_filled(bh, 6., if is_hovered { theme::ACCENT_DIM } else { theme::BORDER });
        pnt.circle_stroke(bh, 6., Stroke::new(1.5, hr_col));

        if is_hovered {
            pnt.text(egui::pos2(th.x, th.y - 12.), egui::Align2::CENTER_CENTER, "● click to connect", egui::FontId::proportional(7.), theme::ACCENT);
        }
    }
    if app.fns.is_empty() {
        pnt.text(r.center(), egui::Align2::CENTER_CENTER, "Add flow nodes from the left panel", egui::FontId::proportional(14.), theme::TEXT_DIM);
    }
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    if let Some(id) = &app.sel_fn {
        if let Some(n) = app.fns.iter().find(|n| n.id == *id) {
            ui.label(egui::RichText::new(&n.nt).size(10.).color(theme::ACCENT));
            ui.label(egui::RichText::new(&n.label).size(11.).color(theme::TEXT));
        }
    } else {
        ui.label(egui::RichText::new("No selection").size(10.).color(theme::TEXT_DIM));
    }
}
