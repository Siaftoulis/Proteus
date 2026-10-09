pub mod dictionary;
pub mod format;

pub use dictionary::{
    build_baseline_dictionaries, SupportedLocale, TranslationDictionary, TranslationResolver,
};
pub use format::{
    format_e164_pretty, normalize_e164, DateFormatStyle, FormattingPreferencesStore,
    NumberFormatter, PhoneFormatError,
};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Legacy locale enum for existing API compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Locale {
    ElGr,
    EnUs,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Locale::ElGr => "el-GR",
            Locale::EnUs => "en-US",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Locale::ElGr => "Ελληνικά",
            Locale::EnUs => "English (US)",
        }
    }

    pub fn from_code(code: &str) -> Self {
        let normalized = code.trim().to_lowercase();
        if normalized.starts_with("el") || normalized.contains("gr") {
            Locale::ElGr
        } else {
            Locale::EnUs
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::ElGr
    }
}

/// Static bilingual dictionary of core domain terminology.
static BASE_DICTIONARY: &[(&str, &str, &str)] = &[
    ("nav_dashboard", "Πίνακας Ελέγχου", "Dashboard"),
    ("nav_pos", "Ταμείο POS", "Point of Sale"),
    ("nav_service", "Επισκευές", "Service Bench"),
    ("nav_wms", "Αποθήκη WMS", "Warehouse WMS"),
    ("nav_settings", "Ρυθμίσεις", "Settings"),
    ("nav_compliance", "Συμμόρφωση & Φορολογία", "Compliance & Tax"),
    ("btn_save", "Αποθήκευση", "Save"),
    ("btn_cancel", "Ακύρωση", "Cancel"),
    ("btn_delete", "Διαγραφή", "Delete"),
    ("btn_sync", "Συγχρονισμός", "Sync"),
    ("btn_export", "Εξαγωγή", "Export"),
    ("btn_search", "Αναζήτηση", "Search"),
    ("btn_confirm", "Επιβεβαίωση", "Confirm"),
    ("ticket_number", "Αριθμός Εντολής", "Ticket #"),
    ("customer_name", "Ονοματεπώνυμο Πελάτη", "Customer Name"),
    ("customer_phone", "Τηλέφωνο", "Phone Number"),
    ("device_model", "Μοντέλο Συσκευής", "Device Model"),
    ("reported_fault", "Δηλωμένη Βλάβη", "Reported Fault"),
    ("technician_notes", "Σημειώσεις Τεχνικού", "Technician Notes"),
    ("status_received", "Παραλήφθηκε", "Received"),
    ("status_in_progress", "Σε Εξέλιξη", "In Progress"),
    ("status_waiting_parts", "Αναμονή Ανταλλακτικών", "Waiting for Parts"),
    ("status_ready", "Έτοιμο προς Παράδοση", "Ready for Pickup"),
    ("status_delivered", "Παραδόθηκε", "Delivered"),
    ("status_cancelled", "Ακυρώθηκε", "Cancelled"),
    ("vat_number", "Α.Φ.Μ.", "Tax ID / VAT"),
    ("tax_office", "Δ.Ο.Υ.", "Tax Authority Office"),
    ("invoice_type", "Τύπος Παραστατικού", "Invoice Type"),
    ("mark_number", "ΜΑΡΚ", "MARK"),
    ("qr_verification", "Επαλήθευση QR", "QR Verification"),
    ("digital_work_card", "Ψηφιακή Κάρτα Εργασίας", "Digital Work Card"),
    ("clock_in", "Έναρξη Βάρδιας", "Clock In"),
    ("clock_out", "Λήξη Βάρδιας", "Clock Out"),
    ("batch_number", "Αριθμός Παρτίδας", "Lot / Batch #"),
    ("haccp_temperature", "Θερμοκρασία HACCP", "HACCP Temperature"),
    ("esl_label", "Ηλεκτρονική Ετικέτα Ραφιού", "Electronic Shelf Label"),
    ("msg_welcome", "Καλωσήρθατε, {name}!", "Welcome, {name}!"),
    ("msg_items_synced", "Συγχρονίστηκαν {count} εγγραφές.", "Synced {count} records."),
];

/// Initializes the i18n translation overrides table in SQLite.
pub fn init_i18n_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS i18n_translation_overrides (
            locale TEXT NOT NULL,
            key TEXT NOT NULL,
            custom_text TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (locale, key)
        );
        CREATE INDEX IF NOT EXISTS idx_i18n_locale ON i18n_translation_overrides(locale);",
    )
}

/// Persists a custom translation override in SQLite.
pub fn save_translation_override(
    conn: &Connection,
    locale: Locale,
    key: &str,
    custom_text: &str,
) -> rusqlite::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO i18n_translation_overrides (locale, key, custom_text, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(locale, key) DO UPDATE SET
            custom_text = excluded.custom_text,
            updated_at = excluded.updated_at;",
        params![locale.code(), key, custom_text, now],
    )?;
    Ok(())
}

/// Formats an amount into localized currency string.
pub fn format_currency(amount: f64, locale: Locale, symbol: Option<&str>) -> String {
    let sym = symbol.unwrap_or(match locale {
        Locale::ElGr => "€",
        Locale::EnUs => "$",
    });

    let is_negative = amount < 0.0;
    let abs_amount = amount.abs();
    let int_part = abs_amount.trunc() as u64;
    let frac_part = ((abs_amount.fract() * 100.0).round() as u64).min(99);

    let int_str = int_part.to_string();
    let mut formatted_int = String::with_capacity(int_str.len() + 4);
    let chars: Vec<char> = int_str.chars().collect();
    let len = chars.len();

    let (thousands_sep, decimal_sep) = match locale {
        Locale::ElGr => ('.', ','),
        Locale::EnUs => (',', '.'),
    };

    for (idx, &ch) in chars.iter().enumerate() {
        if idx > 0 && (len - idx) % 3 == 0 {
            formatted_int.push(thousands_sep);
        }
        formatted_int.push(ch);
    }

    let prefix = if is_negative { "-" } else { "" };
    match locale {
        Locale::ElGr => format!("{}{}{}{:02} {}", prefix, formatted_int, decimal_sep, frac_part, sym),
        Locale::EnUs => format!("{}{}{}{}{:02}", prefix, sym, formatted_int, decimal_sep, frac_part),
    }
}

/// Interpolates named `{param}` keys into a template string.
pub fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for &(key, val) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, val);
    }
    result
}

/// Sovereign localization engine with zero-allocation base lookups and dynamic DB overrides.
pub struct I18nEngine {
    current_locale: Locale,
    overrides: HashMap<(String, String), String>,
}

impl I18nEngine {
    pub fn new(locale: Locale) -> Self {
        Self {
            current_locale: locale,
            overrides: HashMap::new(),
        }
    }

    pub fn current_locale(&self) -> Locale {
        self.current_locale
    }

    pub fn set_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    pub fn load_overrides(&mut self, conn: &Connection) -> rusqlite::Result<usize> {
        init_i18n_schema(conn)?;
        let mut stmt = conn.prepare(
            "SELECT locale, key, custom_text FROM i18n_translation_overrides WHERE locale = ?1",
        )?;
        let rows = stmt.query_map(params![self.current_locale.code()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut count = 0;
        for row in rows {
            let (loc, key, text) = row?;
            self.overrides.insert((loc, key), text);
            count += 1;
        }
        Ok(count)
    }

    pub fn insert_override(&mut self, key: &str, text: &str) {
        self.overrides.insert((self.current_locale.code().to_string(), key.to_string()), text.to_string());
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        let cache_key = (self.current_locale.code().to_string(), key.to_string());
        if let Some(custom) = self.overrides.get(&cache_key) {
            return custom.as_str();
        }

        for &(entry_key, el_text, en_text) in BASE_DICTIONARY {
            if entry_key == key {
                return match self.current_locale {
                    Locale::ElGr => el_text,
                    Locale::EnUs => en_text,
                };
            }
        }

        key
    }

    pub fn t_with(&self, key: &str, params: &[(&str, &str)]) -> String {
        let template = self.t(key);
        interpolate(template, params)
    }

    pub fn format_amount(&self, amount: f64) -> String {
        format_currency(amount, self.current_locale, None)
    }
}

impl Default for I18nEngine {
    fn default() -> Self {
        Self::new(Locale::ElGr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_code_parsing() {
        assert_eq!(Locale::from_code("el-GR"), Locale::ElGr);
        assert_eq!(Locale::from_code("en-US"), Locale::EnUs);
    }

    #[test]
    fn test_default_dictionary_translations() {
        let el_engine = I18nEngine::new(Locale::ElGr);
        assert_eq!(el_engine.t("nav_dashboard"), "Πίνακας Ελέγχου");
        let en_engine = I18nEngine::new(Locale::EnUs);
        assert_eq!(en_engine.t("nav_dashboard"), "Dashboard");
    }

    #[test]
    fn test_currency_formatting() {
        let el_val = format_currency(1250.50, Locale::ElGr, None);
        assert_eq!(el_val, "1.250,50 €");
        let en_val = format_currency(1250.50, Locale::EnUs, None);
        assert_eq!(en_val, "$1,250.50");
    }
}
