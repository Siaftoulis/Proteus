use eframe::egui;

#[derive(Clone, Copy, PartialEq)]
pub enum HeroAlign {
    Left,
    Center,
}

pub struct HeroSection {
    pub title: String,
    pub subtitle: String,
    pub cta_text: String,
    pub align: HeroAlign,
}

impl HeroSection {
    pub fn new(title: impl Into<String>, subtitle: impl Into<String>, cta_text: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            cta_text: cta_text.into(),
            align: HeroAlign::Center,
        }
    }

    pub fn align(mut self, align: HeroAlign) -> Self {
        self.align = align;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let align_enum = match self.align {
            HeroAlign::Left => egui::Align::Min,
            HeroAlign::Center => egui::Align::Center,
        };

        ui.vertical(|ui| {
            let layout = egui::Layout::top_down(align_enum);
            ui.with_layout(layout, |ui| {
                ui.add_space(48.0);
                
                let title_color = ui.visuals().text_color();
                let title_font = egui::FontId::proportional(48.0);
                ui.label(egui::RichText::new(&self.title).font(title_font).strong().color(title_color));
                
                ui.add_space(16.0);
                
                let subtitle_color = ui.visuals().widgets.noninteractive.fg_stroke.color;
                let subtitle_font = egui::FontId::proportional(20.0);
                let mut job = egui::text::LayoutJob::simple(
                    self.subtitle.clone(),
                    subtitle_font,
                    subtitle_color,
                    ui.available_width().min(800.0),
                );
                job.halign = align_enum;
                ui.label(job);
                
                ui.add_space(32.0);
                
                // In egui, the visuals selection stroke is usually the theme accent color
                let accent_color = ui.visuals().selection.stroke.color;
                
                let btn = egui::Button::new(
                    egui::RichText::new(&self.cta_text)
                        .size(16.0)
                        .strong()
                        .color(egui::Color32::WHITE)
                )
                .fill(accent_color)
                .corner_radius(egui::CornerRadius::same(6))
                .min_size(egui::vec2(160.0, 48.0));

                let resp = ui.add(btn);
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                ui.add_space(48.0);
            })
        }).response
    }
}
