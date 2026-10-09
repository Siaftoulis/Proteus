//! Studio Visual Locale Switcher & i18n Translation Manager (Micro-task 28.1.3).
//! Provides side-by-side key translation management, custom SQLite overrides,
//! date/number formatting inspection, and E.164 phone normalization testbench.

use crate::theme;
use crate::ProteusApp;
use eframe::egui::{self, Color32, RichText, ScrollArea};
use proteus_core::i18n::{
    format_e164_pretty, normalize_e164, DateFormatStyle, NumberFormatter, TranslationResolver,
};

/// State container for the Visual Locale & i18n Manager view.
#[derive(Debug, Clone)]
pub struct I18nManagerState {
    pub active_locale: String,
    pub available_locales: Vec<(String, String)>,
    pub search_query: String,
    pub selected_key: Option<String>,
    pub override_input: String,
    pub status_message: String,
    pub resolver: TranslationResolver,
    pub date_style: DateFormatStyle,
    pub number_formatter: NumberFormatter,
    pub test_phone_input: String,
    pub test_phone_normalized: String,
    pub test_amount_input: f64,
}

impl Default for I18nManagerState {
    fn default() -> Self {
        let active = "el".to_string();
        let resolver = TranslationResolver::new(&active);
        let date_style = DateFormatStyle::for_locale(&active);
        let number_formatter = NumberFormatter::for_locale(&active);
        let default_phone = "698 123 4567".to_string();
        let norm_phone = normalize_e164(&default_phone, "+30")
            .map(|p| format_e164_pretty(&p))
            .unwrap_or_default();

        Self {
            active_locale: active,
            available_locales: vec![
                ("el".into(), "Ελληνικά (Greek)".into()),
                ("en".into(), "English (Global)".into()),
                ("de".into(), "Deutsch (German)".into()),
                ("fr".into(), "Français (French)".into()),
                ("es".into(), "Español (Spanish)".into()),
            ],
            search_query: String::new(),
            selected_key: None,
            override_input: String::new(),
            status_message: "Ready. Select a locale or edit terminology overrides.".into(),
            resolver,
            date_style,
            number_formatter,
            test_phone_input: default_phone,
            test_phone_normalized: norm_phone,
            test_amount_input: 1250.50,
        }
    }
}

impl I18nManagerState {
    pub fn switch_locale(&mut self, locale_code: &str) {
        self.active_locale = locale_code.to_string();
        self.resolver.set_active_locale(locale_code);
        self.date_style = DateFormatStyle::for_locale(locale_code);
        self.number_formatter = NumberFormatter::for_locale(locale_code);
        self.status_message = format!("Active locale switched to [{}]", locale_code);
        self.recalculate_phone();
    }

    pub fn recalculate_phone(&mut self) {
        let default_prefix = match self.active_locale.as_str() {
            "el" => "+30",
            "de" => "+49",
            "fr" => "+33",
            "es" => "+34",
            _ => "+1",
        };
        self.test_phone_normalized = match normalize_e164(&self.test_phone_input, default_prefix) {
            Ok(norm) => format_e164_pretty(&norm),
            Err(err) => format!("Error: {}", err),
        };
    }

    pub fn apply_override(&mut self, key: &str, val: &str) {
        self.resolver.set_override(&self.active_locale, key, val);
        self.status_message = format!("Override saved for key '{}'", key);
    }
}

/// Renders the left sidebar: Locale picker, available languages, and metadata stats.
pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let state = &mut app.i18n_state;

    ui.add_space(8.0);
    ui.label(RichText::new("LOCALE SELECTION").small().strong().color(theme::TEXT_MUTED));
    ui.add_space(4.0);

    let locales = state.available_locales.clone();
    let mut switch_to = None;
    for (code, label) in &locales {
        let is_selected = state.active_locale == *code;
        if ui.selectable_label(is_selected, format!("🌐 {}", label)).clicked() {
            switch_to = Some(code.clone());
        }
    }
    if let Some(code) = switch_to {
        state.switch_locale(&code);
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.label(RichText::new("LOCALE PROFILE").small().strong().color(theme::TEXT_MUTED));
    ui.add_space(4.0);
    ui.label(format!("Active Tag: {}", state.active_locale));
    ui.label(format!("Date Format: {:?}", state.date_style));
    ui.label(format!("Decimal Sep: '{}'", state.number_formatter.decimal_separator));
    ui.label(format!("Thousand Sep: '{}'", state.number_formatter.thousand_separator));
    ui.label(format!("Currency: {}", state.number_formatter.currency_symbol));

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    ui.label(RichText::new("STATUS").small().strong().color(theme::TEXT_MUTED));
    ui.label(RichText::new(&state.status_message).small().color(theme::TEXT));
}

/// Renders the central workspace: Searchable key-value dictionary and override editor.
pub fn show_central(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let state = &mut app.i18n_state;

    ui.horizontal(|ui| {
        ui.heading("🌐 Translation Dictionary & Overrides");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(format!("Locale: {}", state.active_locale.to_uppercase())).strong().color(theme::ACCENT));
        });
    });
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("🔍 Search Keys:");
        ui.text_edit_singleline(&mut state.search_query);
        if ui.button("Clear").clicked() {
            state.search_query.clear();
        }
    });

    ui.add_space(8.0);
    ui.separator();

    let baseline_keys = vec![
        "app_title", "nav_dashboard", "nav_service", "nav_pos", "nav_wms", "nav_settings",
        "btn_save", "btn_cancel", "btn_delete", "btn_search", "ticket_number",
        "customer_name", "customer_phone", "device_model", "reported_fault",
        "status_received", "status_in_progress", "status_ready", "status_delivered",
        "vat_number", "msg_welcome",
    ];

    let query = state.search_query.to_lowercase();
    let filtered_keys: Vec<&str> = baseline_keys
        .into_iter()
        .filter(|k| query.is_empty() || k.contains(&query) || state.resolver.translate(k).to_lowercase().contains(&query))
        .collect();

    ui.label(RichText::new(format!("Showing {} domain terms", filtered_keys.len())).small().color(theme::TEXT_MUTED));
    ui.add_space(4.0);

    ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
        egui::Grid::new("i18n_keys_grid")
            .striped(true)
            .min_col_width(180.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Key").strong());
                ui.label(RichText::new("Active Translation").strong());
                ui.label(RichText::new("Action").strong());
                ui.end_row();

                for key in filtered_keys {
                    let translated = state.resolver.translate(key);
                    ui.label(RichText::new(key).monospace());
                    ui.label(&translated);

                    let is_selected = state.selected_key.as_deref() == Some(key);
                    if ui.selectable_label(is_selected, "Edit").clicked() {
                        state.selected_key = Some(key.to_string());
                        state.override_input = translated.clone();
                    }
                    ui.end_row();
                }
            });
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // Selected key override editor
    if let Some(ref key) = state.selected_key.clone() {
        ui.group(|ui| {
            ui.label(RichText::new(format!("Edit Override for: {}", key)).strong());
            ui.horizontal(|ui| {
                ui.label("Translation:");
                ui.text_edit_singleline(&mut state.override_input);
                if ui.button("💾 Apply & Save").clicked() {
                    let input = state.override_input.clone();
                    state.apply_override(key, &input);
                }
                if ui.button("Close").clicked() {
                    state.selected_key = None;
                }
            });
        });
    }
}

/// Renders the right sidebar: Live formatting preview & E.164 phone test bench.
pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let state = &mut app.i18n_state;

    ui.add_space(8.0);
    ui.label(RichText::new("LIVE FORMATTING PREVIEW").small().strong().color(theme::TEXT_MUTED));
    ui.add_space(6.0);

    let now = chrono::Utc::now();
    ui.group(|ui| {
        ui.label(RichText::new("Date & Time").strong());
        ui.label(format!("Date: {}", state.date_style.format_date(&now)));
        ui.label(format!("DateTime: {}", state.date_style.format_datetime(&now)));
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label(RichText::new("Currency & Numbers").strong());
        ui.horizontal(|ui| {
            ui.label("Amount:");
            ui.add(egui::DragValue::new(&mut state.test_amount_input).speed(10.0));
        });
        let formatted = state.number_formatter.format_currency(state.test_amount_input, None);
        ui.label(RichText::new(format!("Formatted: {}", formatted)).strong().color(theme::ACCENT));
        ui.label(format!("Rate (24%): {}", state.number_formatter.format_percentage(24.0)));
    });

    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label(RichText::new("E.164 Phone Normalizer").strong());
        ui.horizontal(|ui| {
            ui.label("Input:");
            if ui.text_edit_singleline(&mut state.test_phone_input).changed() {
                state.recalculate_phone();
            }
        });
        ui.label(RichText::new(format!("Normalized: {}", state.test_phone_normalized)).monospace().color(Color32::from_rgb(120, 220, 150)));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_manager_state_defaults() {
        let state = I18nManagerState::default();
        assert_eq!(state.active_locale, "el");
        assert_eq!(state.available_locales.len(), 5);
        assert!(state.test_phone_normalized.starts_with("+30"));
    }

    #[test]
    fn test_i18n_manager_locale_switch() {
        let mut state = I18nManagerState::default();
        state.switch_locale("de");
        assert_eq!(state.active_locale, "de");
        assert_eq!(state.number_formatter.currency_symbol, "€");
        assert_eq!(state.date_style, DateFormatStyle::DayMonthYearDot);
    }

    #[test]
    fn test_i18n_manager_live_override() {
        let mut state = I18nManagerState::default();
        state.apply_override("ticket_number", "Καρτέλα #");
        assert_eq!(state.resolver.translate("ticket_number"), "Καρτέλα #");
    }
}
