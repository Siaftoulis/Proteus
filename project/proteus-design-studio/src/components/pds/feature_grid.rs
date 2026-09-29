use eframe::egui;

#[derive(Clone, Default)]
pub struct FeatureItem {
    pub title: String,
    pub description: String,
    pub icon: String,
}

pub struct FeatureGrid {
    pub title: String,
    pub columns: usize,
    pub items: Vec<FeatureItem>,
}

impl FeatureGrid {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            columns: 3,
            items: vec![],
        }
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.clamp(1, 4);
        self
    }

    pub fn items(mut self, items: Vec<FeatureItem>) -> Self {
        self.items = items;
        self
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.add_space(24.0);
            
            // Section Title
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(&self.title)
                        .size(24.0)
                        .strong()
                        .color(ui.visuals().text_color())
                );
            });

            ui.add_space(24.0);

            let available_width = ui.available_width() - 32.0;
            let column_width = (available_width / self.columns as f32) - 16.0;

            let mut iter = self.items.iter().peekable();
            
            while iter.peek().is_some() {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);
                    for _ in 0..self.columns {
                        if let Some(item) = iter.next() {
                            ui.allocate_ui_with_layout(
                                egui::vec2(column_width, 180.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    let (rect, _resp) = ui.allocate_exact_size(
                                        egui::vec2(column_width, 180.0),
                                        egui::Sense::hover(),
                                    );

                                    if ui.is_rect_visible(rect) {
                                        let bg_color = ui.visuals().widgets.noninteractive.bg_fill;
                                        let border_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
                                        
                                        ui.painter().rect(
                                            rect,
                                            egui::CornerRadius::same(8),
                                            bg_color,
                                            egui::Stroke::new(1.0, border_color),
                                            egui::StrokeKind::Middle,
                                        );

                                        let mut child_ui = ui.new_child(
                                            egui::UiBuilder::new()
                                                .max_rect(rect.shrink(20.0))
                                                .layout(egui::Layout::top_down(egui::Align::Min))
                                        );

                                        let icon_glyph = match item.icon.as_str() {
                                            "speed" => "⚡",
                                            "shield" => "🛡",
                                            "analytics" => "📊",
                                            "cloud" => "☁",
                                            "code" => "💻",
                                            "settings" => "⚙",
                                            _ => "✦",
                                        };

                                        child_ui.label(egui::RichText::new(icon_glyph).size(22.0));
                                        child_ui.add_space(12.0);

                                        child_ui.label(
                                            egui::RichText::new(&item.title)
                                                .size(15.0)
                                                .strong()
                                                .color(child_ui.visuals().text_color())
                                        );
                                        child_ui.add_space(8.0);

                                        let text_job = egui::text::LayoutJob::simple(
                                            item.description.clone(),
                                            egui::FontId::proportional(13.0),
                                            child_ui.visuals().text_color().linear_multiply(0.7),
                                            column_width - 40.0,
                                        );
                                        child_ui.label(text_job);
                                    }
                                }
                            );
                            ui.add_space(16.0);
                        } else {
                            ui.allocate_space(egui::vec2(column_width, 180.0));
                            ui.add_space(16.0);
                        }
                    }
                });
                ui.add_space(16.0);
            }
        })
        .response
    }
}
