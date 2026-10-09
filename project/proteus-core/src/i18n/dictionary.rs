use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard ISO supported locales and extensible custom tags.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLocale {
    En,
    El,
    De,
    Fr,
    Es,
    Custom(String),
}

impl SupportedLocale {
    pub fn code(&self) -> &str {
        match self {
            SupportedLocale::En => "en",
            SupportedLocale::El => "el",
            SupportedLocale::De => "de",
            SupportedLocale::Fr => "fr",
            SupportedLocale::Es => "es",
            SupportedLocale::Custom(ref s) => s.as_str(),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            SupportedLocale::En => "English (Global)",
            SupportedLocale::El => "Ελληνικά",
            SupportedLocale::De => "Deutsch",
            SupportedLocale::Fr => "Français",
            SupportedLocale::Es => "Español",
            SupportedLocale::Custom(ref s) => s.as_str(),
        }
    }

    pub fn from_code(code: &str) -> Self {
        let norm = code.trim().to_lowercase();
        let primary = norm.split(|c| c == '-' || c == '_').next().unwrap_or(&norm);
        match primary {
            "en" => SupportedLocale::En,
            "el" | "gr" => SupportedLocale::El,
            "de" => SupportedLocale::De,
            "fr" => SupportedLocale::Fr,
            "es" => SupportedLocale::Es,
            _ => SupportedLocale::Custom(code.trim().to_string()),
        }
    }
}

/// A declarative translation dictionary for a specific locale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationDictionary {
    pub locale: String,
    pub display_name: String,
    pub translations: HashMap<String, String>,
    pub plurals: HashMap<String, (String, String)>,
}

impl TranslationDictionary {
    pub fn new(locale: &str, display_name: &str) -> Self {
        Self {
            locale: locale.to_string(),
            display_name: display_name.to_string(),
            translations: HashMap::new(),
            plurals: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, text: &str) {
        self.translations.insert(key.to_string(), text.to_string());
    }

    pub fn insert_plural(&mut self, key: &str, one: &str, other: &str) {
        self.plurals.insert(key.to_string(), (one.to_string(), other.to_string()));
    }

    pub fn from_json_str(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn to_json_str(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Baseline built-in dictionary generator for key domains.
pub fn build_baseline_dictionaries() -> HashMap<String, TranslationDictionary> {
    let mut map = HashMap::new();

    // 1. English (Default)
    let mut en = TranslationDictionary::new("en", "English");
    en.insert("app_title", "Proteus Autonomous OS");
    en.insert("nav_dashboard", "Dashboard");
    en.insert("nav_service", "Service Bench");
    en.insert("nav_pos", "Point of Sale");
    en.insert("nav_wms", "Warehouse WMS");
    en.insert("nav_settings", "Settings");
    en.insert("btn_save", "Save");
    en.insert("btn_cancel", "Cancel");
    en.insert("btn_delete", "Delete");
    en.insert("btn_search", "Search");
    en.insert("ticket_number", "Ticket #");
    en.insert("customer_name", "Customer Name");
    en.insert("customer_phone", "Phone Number");
    en.insert("device_model", "Device Model");
    en.insert("reported_fault", "Reported Fault");
    en.insert("status_received", "Received");
    en.insert("status_in_progress", "In Progress");
    en.insert("status_ready", "Ready for Pickup");
    en.insert("status_delivered", "Delivered");
    en.insert("vat_number", "Tax ID / VAT");
    en.insert("msg_welcome", "Welcome, {name}!");
    en.insert_plural("plural_ticket", "{count} ticket", "{count} tickets");
    map.insert("en".to_string(), en);

    // 2. Greek
    let mut el = TranslationDictionary::new("el", "Ελληνικά");
    el.insert("app_title", "Proteus Αυτόνομο Λειτουργικό");
    el.insert("nav_dashboard", "Πίνακας Ελέγχου");
    el.insert("nav_service", "Επισκευές");
    el.insert("nav_pos", "Ταμείο POS");
    el.insert("nav_wms", "Αποθήκη WMS");
    el.insert("nav_settings", "Ρυθμίσεις");
    el.insert("btn_save", "Αποθήκευση");
    el.insert("btn_cancel", "Ακύρωση");
    el.insert("btn_delete", "Διαγραφή");
    el.insert("btn_search", "Αναζήτηση");
    el.insert("ticket_number", "Αριθμός Εντολής");
    el.insert("customer_name", "Ονοματεπώνυμο Πελάτη");
    el.insert("customer_phone", "Τηλέφωνο");
    el.insert("device_model", "Μοντέλο Συσκευής");
    el.insert("reported_fault", "Δηλωμένη Βλάβη");
    el.insert("status_received", "Παραλήφθηκε");
    el.insert("status_in_progress", "Σε Εξέλιξη");
    el.insert("status_ready", "Έτοιμο προς Παράδοση");
    el.insert("status_delivered", "Παραδόθηκε");
    el.insert("vat_number", "Α.Φ.Μ.");
    el.insert("msg_welcome", "Καλωσήρθατε, {name}!");
    el.insert_plural("plural_ticket", "{count} δελτίο", "{count} δελτία");
    map.insert("el".to_string(), el);

    // 3. German
    let mut de = TranslationDictionary::new("de", "Deutsch");
    de.insert("app_title", "Proteus Autonomes OS");
    de.insert("nav_dashboard", "Übersicht");
    de.insert("nav_service", "Reparaturwerkstatt");
    de.insert("nav_pos", "Kasse POS");
    de.insert("nav_wms", "Lagerverwaltung");
    de.insert("nav_settings", "Einstellungen");
    de.insert("btn_save", "Speichern");
    de.insert("btn_cancel", "Abbrechen");
    de.insert("btn_delete", "Löschen");
    de.insert("btn_search", "Suchen");
    de.insert("ticket_number", "Auftrags-Nr.");
    de.insert("customer_name", "Kundenname");
    de.insert("customer_phone", "Telefonnummer");
    de.insert("device_model", "Gerätemodell");
    de.insert("reported_fault", "Fehlerbeschreibung");
    de.insert("status_received", "Eingegangen");
    de.insert("status_in_progress", "In Bearbeitung");
    de.insert("status_ready", "Abholbereit");
    de.insert("status_delivered", "Ausgeliefert");
    de.insert("vat_number", "USt-IdNr.");
    de.insert("msg_welcome", "Willkommen, {name}!");
    de.insert_plural("plural_ticket", "{count} Auftrag", "{count} Aufträge");
    map.insert("de".to_string(), de);

    map
}

/// Sovereign multi-tier translation resolver with database override caching and fallback.
#[derive(Debug, Clone)]
pub struct TranslationResolver {
    active_locale: String,
    fallback_locale: String,
    dictionaries: HashMap<String, TranslationDictionary>,
    overrides: HashMap<(String, String), String>,
}

impl TranslationResolver {
    pub fn new(active_locale: &str) -> Self {
        let dictionaries = build_baseline_dictionaries();
        Self {
            active_locale: active_locale.to_string(),
            fallback_locale: "en".to_string(),
            dictionaries,
            overrides: HashMap::new(),
        }
    }

    pub fn active_locale(&self) -> &str {
        &self.active_locale
    }

    pub fn set_active_locale(&mut self, locale: &str) {
        self.active_locale = locale.to_string();
    }

    pub fn register_dictionary(&mut self, dict: TranslationDictionary) {
        self.dictionaries.insert(dict.locale.clone(), dict);
    }

    pub fn set_override(&mut self, locale: &str, key: &str, val: &str) {
        self.overrides.insert((locale.to_string(), key.to_string()), val.to_string());
    }

    /// Translates key across (1) overrides -> (2) active dict -> (3) fallback dict -> (4) raw key.
    pub fn translate(&self, key: &str) -> String {
        // 1. Direct runtime override
        if let Some(val) = self.overrides.get(&(self.active_locale.clone(), key.to_string())) {
            return val.clone();
        }

        // 2. Active dictionary
        if let Some(dict) = self.dictionaries.get(&self.active_locale) {
            if let Some(val) = dict.translations.get(key) {
                return val.clone();
            }
        }

        // 3. Fallback dictionary (usually 'en')
        if self.active_locale != self.fallback_locale {
            if let Some(dict) = self.dictionaries.get(&self.fallback_locale) {
                if let Some(val) = dict.translations.get(key) {
                    return val.clone();
                }
            }
        }

        // 4. Raw key
        key.to_string()
    }

    /// Translates key and interpolates named `{param}` arguments.
    pub fn translate_with_params(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut text = self.translate(key);
        for &(param_key, param_val) in params {
            let pattern = format!("{{{}}}", param_key);
            text = text.replace(&pattern, param_val);
        }
        text
    }

    /// Resolves plural forms based on count.
    pub fn translate_plural(&self, key: &str, count: i64, params: &[(&str, &str)]) -> String {
        let count_str = count.to_string();
        let mut effective_template = None;

        // Try active dictionary plurals
        if let Some(dict) = self.dictionaries.get(&self.active_locale) {
            if let Some((one, other)) = dict.plurals.get(key) {
                effective_template = Some(if count == 1 { one } else { other });
            }
        }

        // Try fallback dictionary plurals
        if effective_template.is_none() {
            if let Some(dict) = self.dictionaries.get(&self.fallback_locale) {
                if let Some((one, other)) = dict.plurals.get(key) {
                    effective_template = Some(if count == 1 { one } else { other });
                }
            }
        }

        let mut res = effective_template.cloned().unwrap_or_else(|| key.to_string());
        res = res.replace("{count}", &count_str);
        for &(k, v) in params {
            res = res.replace(&format!("{{{}}}", k), v);
        }
        res
    }

    /// Initializes SQLite tables for internationalization.
    pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_i18n_locales (
                locale_code TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                is_active INTEGER NOT NULL,
                is_default INTEGER NOT NULL
            )", [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_i18n_translations (
                locale_code TEXT NOT NULL,
                translation_key TEXT NOT NULL,
                translation_val TEXT NOT NULL,
                is_override INTEGER NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (locale_code, translation_key)
            )", [],
        )?;
        Ok(())
    }

    /// Loads active overrides from SQLite into memory.
    pub fn load_db_overrides(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        Self::init_schema(conn)?;
        let mut stmt = conn.prepare(
            "SELECT locale_code, translation_key, translation_val FROM system_i18n_translations WHERE is_override = 1",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let mut loaded = 0;
        for item in rows {
            let (loc, key, val) = item?;
            self.overrides.insert((loc, key), val);
            loaded += 1;
        }
        Ok(loaded)
    }

    /// Persists a custom translation override directly to SQLite.
    pub fn persist_override(
        conn: &Connection,
        locale: &str,
        key: &str,
        val: &str,
    ) -> Result<(), rusqlite::Error> {
        Self::init_schema(conn)?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO system_i18n_translations (locale_code, translation_key, translation_val, is_override, updated_at)
             VALUES (?1, ?2, ?3, 1, ?4)
             ON CONFLICT(locale_code, translation_key) DO UPDATE SET
                translation_val = excluded.translation_val,
                updated_at = excluded.updated_at",
            params![locale, key, val, now],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_locale_normalization() {
        assert_eq!(SupportedLocale::from_code("en-US"), SupportedLocale::En);
        assert_eq!(SupportedLocale::from_code("el_GR"), SupportedLocale::El);
        assert_eq!(SupportedLocale::from_code("de"), SupportedLocale::De);
        assert_eq!(SupportedLocale::from_code("fr-FR"), SupportedLocale::Fr);
        assert_eq!(SupportedLocale::from_code("es"), SupportedLocale::Es);
        assert_eq!(SupportedLocale::from_code("it-IT"), SupportedLocale::Custom("it-IT".into()));
    }

    #[test]
    fn test_resolver_multi_lingual_and_fallback() {
        let mut resolver = TranslationResolver::new("el");
        assert_eq!(resolver.translate("ticket_number"), "Αριθμός Εντολής");
        assert_eq!(resolver.translate("nav_service"), "Επισκευές");

        // Switch to German
        resolver.set_active_locale("de");
        assert_eq!(resolver.translate("ticket_number"), "Auftrags-Nr.");
        assert_eq!(resolver.translate("nav_service"), "Reparaturwerkstatt");

        // Missing key in German falls back to English baseline
        resolver.set_active_locale("de");
        assert_eq!(resolver.translate("non_existent"), "non_existent");
    }

    #[test]
    fn test_resolver_plurals_and_params() {
        let resolver = TranslationResolver::new("el");
        assert_eq!(
            resolver.translate_plural("plural_ticket", 1, &[]),
            "1 δελτίο"
        );
        assert_eq!(
            resolver.translate_plural("plural_ticket", 5, &[]),
            "5 δελτία"
        );

        let welcome = resolver.translate_with_params("msg_welcome", &[("name", "Κώστα")]);
        assert_eq!(welcome, "Καλωσήρθατε, Κώστα!");
    }

    #[test]
    fn test_resolver_sqlite_override_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        TranslationResolver::init_schema(&conn).unwrap();

        TranslationResolver::persist_override(&conn, "el", "ticket_number", "Καρτέλα Επισκευής").unwrap();

        let mut resolver = TranslationResolver::new("el");
        let count = resolver.load_db_overrides(&conn).unwrap();
        assert_eq!(count, 1);
        assert_eq!(resolver.translate("ticket_number"), "Καρτέλα Επισκευής");
    }
}
