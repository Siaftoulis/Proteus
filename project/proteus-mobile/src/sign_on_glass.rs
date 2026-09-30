//! Touch-Enabled Sign-on-Glass Vector Signature Capture for Proteus Mobile.
//! Enables delivery drivers and field technicians to capture customer signatures
//! directly on mobile touchscreens (smartphones and tablets) with sub-pixel precision.
//! Exports vector paths to compact SVG and SQLite-storable stroke data.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Pos2, Response, Sense, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Interactive vector signature pad widget for mobile devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignOnGlassPad {
    pub strokes: Vec<Vec<(f32, f32)>>,
    #[serde(skip)]
    pub active_stroke: Vec<(f32, f32)>,
    pub width: f32,
    pub height: f32,
    pub stroke_color: [u8; 4],
    pub stroke_width: f32,
}

impl Default for SignOnGlassPad {
    fn default() -> Self {
        Self {
            strokes: Vec::new(),
            active_stroke: Vec::new(),
            width: 320.0,
            height: 160.0,
            stroke_color: [221, 230, 237, 255], // High-contrast title text color
            stroke_width: 2.5,
        }
    }
}

impl SignOnGlassPad {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Checks if any signature strokes have been drawn.
    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty() && self.active_stroke.is_empty()
    }

    /// Clears all drawn strokes.
    pub fn clear(&mut self) {
        self.strokes.clear();
        self.active_stroke.clear();
    }

    /// Exports drawn strokes as an SVG path string for archiving and printing.
    pub fn export_svg(&self) -> String {
        let mut d = String::new();
        for stroke in &self.strokes {
            if stroke.is_empty() {
                continue;
            }
            d.push_str(&format!("M {:.1} {:.1} ", stroke[0].0, stroke[0].1));
            for pt in stroke.iter().skip(1) {
                d.push_str(&format!("L {:.1} {:.1} ", pt.0, pt.1));
            }
        }
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {:.0} {:.0}" width="{:.0}" height="{:.0}"><path d="{}" fill="none" stroke="black" stroke-width="{:.1}" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
            self.width, self.height, self.width, self.height, d.trim(), self.stroke_width
        )
    }

    /// Exports strokes to a compact string for SQLite storage.
    pub fn export_compact_string(&self) -> String {
        let mut out = String::new();
        for (i, stroke) in self.strokes.iter().enumerate() {
            if i > 0 {
                out.push(';');
            }
            for (j, pt) in stroke.iter().enumerate() {
                if j > 0 {
                    out.push(',');
                }
                out.push_str(&format!("{:.1}:{:.1}", pt.0, pt.1));
            }
        }
        out
    }

    /// Renders the touch-sensitive drawing canvas on screen.
    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        let size = Vec2::new(self.width, self.height);
        let (rect, response) = ui.allocate_exact_size(size, Sense::drag());

        // Background card frame
        let bg_color = Color32::from_rgb(20, 28, 40);
        let border_color = if response.has_focus() || response.dragged() {
            Color32::from_rgb(214, 158, 46) // Gold focus
        } else {
            Color32::from_rgb(60, 80, 105)
        };

        ui.painter().rect_filled(rect, CornerRadius::same(6), bg_color);
        ui.painter().rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.5, border_color), egui::StrokeKind::Inside);

        // Guide baseline
        let baseline_y = rect.max.y - 30.0;
        ui.painter().line_segment(
            [Pos2::new(rect.min.x + 20.0, baseline_y), Pos2::new(rect.max.x - 20.0, baseline_y)],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(82, 109, 130, 80)),
        );

        // Watermark indicator when empty
        if self.is_empty() {
            ui.painter().text(
                Pos2::new(rect.center().x, rect.center().y - 10.0),
                egui::Align2::CENTER_CENTER,
                "✍ Υπογράψτε στην οθόνη (Sign Here)",
                egui::FontId::proportional(13.0),
                Color32::from_rgba_unmultiplied(157, 178, 191, 100),
            );
        }

        // Pointer event processing for mobile touch & stylus
        if let Some(pos) = response.interact_pointer_pos() {
            if rect.contains(pos) {
                let local_x = (pos.x - rect.min.x).clamp(0.0, self.width);
                let local_y = (pos.y - rect.min.y).clamp(0.0, self.height);
                self.active_stroke.push((local_x, local_y));
            }
        }

        if response.drag_stopped() && !self.active_stroke.is_empty() {
            let stroke = std::mem::take(&mut self.active_stroke);
            self.strokes.push(stroke);
        }

        // Render confirmed strokes
        let ink_color = Color32::from_rgba_unmultiplied(
            self.stroke_color[0],
            self.stroke_color[1],
            self.stroke_color[2],
            self.stroke_color[3],
        );
        let stroke_spec = Stroke::new(self.stroke_width, ink_color);

        for stroke in &self.strokes {
            for win in stroke.windows(2) {
                let p1 = Pos2::new(rect.min.x + win[0].0, rect.min.y + win[0].1);
                let p2 = Pos2::new(rect.min.x + win[1].0, rect.min.y + win[1].1);
                ui.painter().line_segment([p1, p2], stroke_spec);
            }
        }

        // Render currently active stroke in progress
        for win in self.active_stroke.windows(2) {
            let p1 = Pos2::new(rect.min.x + win[0].0, rect.min.y + win[0].1);
            let p2 = Pos2::new(rect.min.x + win[1].0, rect.min.y + win[1].1);
            ui.painter().line_segment([p1, p2], stroke_spec);
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_on_glass_defaults() {
        let pad = SignOnGlassPad::default();
        assert!(pad.is_empty());
        assert_eq!(pad.width, 320.0);
        assert_eq!(pad.height, 160.0);
    }

    #[test]
    fn test_strokes_and_clearing() {
        let mut pad = SignOnGlassPad::new(300.0, 150.0);
        pad.strokes.push(vec![(10.0, 20.0), (15.0, 25.0), (30.0, 40.0)]);
        assert!(!pad.is_empty());

        let svg = pad.export_svg();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("M 10.0 20.0 L 15.0 25.0 L 30.0 40.0"));

        let compact = pad.export_compact_string();
        assert_eq!(compact, "10.0:20.0,15.0:25.0,30.0:40.0");

        pad.clear();
        assert!(pad.is_empty());
        assert_eq!(pad.export_compact_string(), "");
    }
}
