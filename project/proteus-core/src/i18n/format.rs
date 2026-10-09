use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Date and time formatting conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateFormatStyle {
    DayMonthYearSlash, // DD/MM/YYYY (Greece, UK, France)
    DayMonthYearDot,   // DD.MM.YYYY (Germany, DACH)
    MonthDayYearSlash, // MM/DD/YYYY (USA)
    IsoYearMonthDay,   // YYYY-MM-DD (ISO 8601 standard)
}

impl DateFormatStyle {
    pub fn for_locale(locale_code: &str) -> Self {
        let norm = locale_code.trim().to_lowercase();
        if norm.starts_with("de") {
            DateFormatStyle::DayMonthYearDot
        } else if norm.starts_with("en-us") || norm == "us" {
            DateFormatStyle::MonthDayYearSlash
        } else if norm.starts_with("el") || norm.starts_with("fr") || norm.starts_with("es") || norm.starts_with("en-gb") {
            DateFormatStyle::DayMonthYearSlash
        } else {
            DateFormatStyle::IsoYearMonthDay
        }
    }

    pub fn format_date(&self, dt: &DateTime<Utc>) -> String {
        match self {
            DateFormatStyle::DayMonthYearSlash => dt.format("%d/%m/%Y").to_string(),
            DateFormatStyle::DayMonthYearDot => dt.format("%d.%m.%Y").to_string(),
            DateFormatStyle::MonthDayYearSlash => dt.format("%m/%d/%Y").to_string(),
            DateFormatStyle::IsoYearMonthDay => dt.format("%Y-%m-%d").to_string(),
        }
    }

    pub fn format_datetime(&self, dt: &DateTime<Utc>) -> String {
        match self {
            DateFormatStyle::DayMonthYearSlash => dt.format("%d/%m/%Y %H:%M:%S").to_string(),
            DateFormatStyle::DayMonthYearDot => dt.format("%d.%m.%Y %H:%M:%S").to_string(),
            DateFormatStyle::MonthDayYearSlash => dt.format("%m/%d/%Y %I:%M:%S %p").to_string(),
            DateFormatStyle::IsoYearMonthDay => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    pub fn parse_date(&self, input: &str) -> Option<NaiveDate> {
        let trimmed = input.trim();
        match self {
            DateFormatStyle::DayMonthYearSlash => NaiveDate::parse_from_str(trimmed, "%d/%m/%Y").ok(),
            DateFormatStyle::DayMonthYearDot => NaiveDate::parse_from_str(trimmed, "%d.%m.%Y").ok(),
            DateFormatStyle::MonthDayYearSlash => NaiveDate::parse_from_str(trimmed, "%m/%d/%Y").ok(),
            DateFormatStyle::IsoYearMonthDay => NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok(),
        }
    }
}

/// Localized number and currency formatting engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberFormatter {
    pub thousand_separator: char,
    pub decimal_separator: char,
    pub currency_symbol: String,
    pub symbol_is_suffix: bool,
}

impl NumberFormatter {
    pub fn for_locale(locale_code: &str) -> Self {
        let norm = locale_code.trim().to_lowercase();
        if norm.starts_with("en-us") || norm == "us" {
            Self {
                thousand_separator: ',',
                decimal_separator: '.',
                currency_symbol: "$".into(),
                symbol_is_suffix: false,
            }
        } else if norm.starts_with("de") {
            Self {
                thousand_separator: '.',
                decimal_separator: ',',
                currency_symbol: "€".into(),
                symbol_is_suffix: true,
            }
        } else if norm.starts_with("fr") {
            Self {
                thousand_separator: ' ',
                decimal_separator: ',',
                currency_symbol: "€".into(),
                symbol_is_suffix: true,
            }
        } else {
            // Default Greek / South European convention
            Self {
                thousand_separator: '.',
                decimal_separator: ',',
                currency_symbol: "€".into(),
                symbol_is_suffix: true,
            }
        }
    }

    pub fn format_decimal(&self, value: f64, decimals: usize) -> String {
        let is_negative = value < 0.0;
        let abs_val = value.abs();

        let multiplier = 10f64.powi(decimals as i32);
        let rounded = (abs_val * multiplier).round();
        let int_part = (rounded / multiplier).trunc() as u64;
        let frac_part = (rounded as u64) % (multiplier as u64);

        let int_str = int_part.to_string();
        let mut formatted_int = String::with_capacity(int_str.len() + 4);
        let chars: Vec<char> = int_str.chars().collect();
        let len = chars.len();

        for (idx, &ch) in chars.iter().enumerate() {
            if idx > 0 && (len - idx) % 3 == 0 {
                formatted_int.push(self.thousand_separator);
            }
            formatted_int.push(ch);
        }

        let prefix = if is_negative { "-" } else { "" };
        if decimals > 0 {
            let frac_str = format!("{:0width$}", frac_part, width = decimals);
            format!("{}{}{}{}", prefix, formatted_int, self.decimal_separator, frac_str)
        } else {
            format!("{}{}", prefix, formatted_int)
        }
    }

    pub fn format_currency(&self, amount: f64, custom_symbol: Option<&str>) -> String {
        let sym = custom_symbol.unwrap_or(&self.currency_symbol);
        let num_str = self.format_decimal(amount, 2);

        if self.symbol_is_suffix {
            format!("{} {}", num_str, sym)
        } else {
            if num_str.starts_with('-') {
                format!("-{}{}", sym, &num_str[1..])
            } else {
                format!("{}{}", sym, num_str)
            }
        }
    }

    pub fn format_percentage(&self, rate: f64) -> String {
        format!("{}%", self.format_decimal(rate, 1))
    }
}

/// Phone normalization errors according to ITU-T E.164.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhoneFormatError {
    TooShort,
    TooLong,
    InvalidCharacters,
    Empty,
}

impl std::fmt::Display for PhoneFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhoneFormatError::TooShort => write!(f, "Phone number has fewer than 7 digits"),
            PhoneFormatError::TooLong => write!(f, "Phone number exceeds E.164 15 digits limit"),
            PhoneFormatError::InvalidCharacters => write!(f, "Phone number contains invalid non-digit characters"),
            PhoneFormatError::Empty => write!(f, "Phone number is empty"),
        }
    }
}

impl std::error::Error for PhoneFormatError {}

/// Standard E.164 phone normalizer.
/// Strips non-digit formatting and prepends default international prefix if local.
pub fn normalize_e164(raw: &str, default_country_prefix: &str) -> Result<String, PhoneFormatError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PhoneFormatError::Empty);
    }

    // Convert 00 to +
    let cleaned_input = if trimmed.starts_with("00") {
        format!("+{}", &trimmed[2..])
    } else {
        trimmed.to_string()
    };

    let has_plus = cleaned_input.starts_with('+');
    let mut digits = String::with_capacity(16);

    for (i, ch) in cleaned_input.chars().enumerate() {
        if i == 0 && ch == '+' {
            continue;
        }
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else if ch == ' ' || ch == '-' || ch == '(' || ch == ')' || ch == '.' || ch == '/' {
            continue; // skip acceptable punctuation
        } else {
            return Err(PhoneFormatError::InvalidCharacters);
        }
    }

    if digits.is_empty() {
        return Err(PhoneFormatError::Empty);
    }

    let final_e164 = if has_plus {
        format!("+{}", digits)
    } else {
        let prefix = default_country_prefix.trim().trim_start_matches('+');
        // If local starts with 0 (e.g. 0691234567 in GR or UK), strip leading 0 before appending prefix
        let local_digits = if digits.starts_with('0') {
            &digits[1..]
        } else {
            &digits
        };
        format!("+{}{}", prefix, local_digits)
    };

    let total_digits = final_e164.chars().filter(|c| c.is_ascii_digit()).count();
    if total_digits < 7 {
        return Err(PhoneFormatError::TooShort);
    }
    if total_digits > 15 {
        return Err(PhoneFormatError::TooLong);
    }

    Ok(final_e164)
}

/// Formats an E.164 string with readable grouping.
pub fn format_e164_pretty(e164: &str) -> String {
    if !e164.starts_with('+') {
        return e164.to_string();
    }
    let digits: String = e164.chars().filter(|c| c.is_ascii_digit()).collect();

    // Greece (+30)
    if digits.starts_with("30") && digits.len() == 12 {
        return format!("+30 {} {} {}", &digits[2..5], &digits[5..8], &digits[8..]);
    }
    // US / Canada (+1)
    if digits.starts_with("1") && digits.len() == 11 {
        return format!("+1 ({}) {}-{}", &digits[1..4], &digits[4..7], &digits[7..]);
    }
    // Germany (+49)
    if digits.starts_with("49") && digits.len() >= 11 {
        return format!("+49 {} {} {}", &digits[2..5], &digits[5..8], &digits[8..]);
    }
    // Default fallback chunking (3-3-4)
    format!("+{}", digits)
}

/// Schema manager for tenant formatting preferences.
pub struct FormattingPreferencesStore;

impl FormattingPreferencesStore {
    pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_i18n_formatting_preferences (
                tenant_id TEXT PRIMARY KEY,
                date_style TEXT NOT NULL,
                thousand_sep TEXT NOT NULL,
                decimal_sep TEXT NOT NULL,
                default_phone_prefix TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )", [],
        )?;
        Ok(())
    }

    pub fn save_preferences(
        conn: &Connection,
        tenant_id: &str,
        date_style: DateFormatStyle,
        thousand_sep: char,
        decimal_sep: char,
        phone_prefix: &str,
    ) -> Result<(), rusqlite::Error> {
        Self::init_schema(conn)?;
        let now = Utc::now().to_rfc3339();
        let style_str = match date_style {
            DateFormatStyle::DayMonthYearSlash => "DMY_SLASH",
            DateFormatStyle::DayMonthYearDot => "DMY_DOT",
            DateFormatStyle::MonthDayYearSlash => "MDY_SLASH",
            DateFormatStyle::IsoYearMonthDay => "ISO",
        };

        conn.execute(
            "INSERT INTO system_i18n_formatting_preferences
             (tenant_id, date_style, thousand_sep, decimal_sep, default_phone_prefix, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(tenant_id) DO UPDATE SET
                date_style = excluded.date_style,
                thousand_sep = excluded.thousand_sep,
                decimal_sep = excluded.decimal_sep,
                default_phone_prefix = excluded.default_phone_prefix,
                updated_at = excluded.updated_at",
            params![tenant_id, style_str, thousand_sep.to_string(), decimal_sep.to_string(), phone_prefix, now],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, NaiveDateTime};

    #[test]
    fn test_date_formatting_and_parsing() {
        let style_eu = DateFormatStyle::DayMonthYearSlash;
        let parsed = style_eu.parse_date("25/12/2026").unwrap();
        assert_eq!(parsed.day(), 25);
        assert_eq!(parsed.month(), 12);
        assert_eq!(parsed.year(), 2026);

        let naive = NaiveDateTime::parse_from_str("2026-10-09 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
        assert_eq!(style_eu.format_date(&dt), "09/10/2026");

        let style_de = DateFormatStyle::DayMonthYearDot;
        assert_eq!(style_de.format_date(&dt), "09.10.2026");

        let style_us = DateFormatStyle::MonthDayYearSlash;
        assert_eq!(style_us.format_date(&dt), "10/09/2026");
    }

    #[test]
    fn test_number_and_currency_formatting() {
        let fmt_gr = NumberFormatter::for_locale("el-GR");
        assert_eq!(fmt_gr.format_decimal(1250000.75, 2), "1.250.000,75");
        assert_eq!(fmt_gr.format_currency(1250.50, None), "1.250,50 €");
        assert_eq!(fmt_gr.format_percentage(24.0), "24,0%");

        let fmt_us = NumberFormatter::for_locale("en-US");
        assert_eq!(fmt_us.format_decimal(1250000.75, 2), "1,250,000.75");
        assert_eq!(fmt_us.format_currency(1250.50, None), "$1,250.50");
        assert_eq!(fmt_us.format_currency(-45.99, None), "-$45.99");
    }

    #[test]
    fn test_e164_normalization_greek_and_international() {
        // Greek local with space/dash
        let normalized = normalize_e164("698 123-4567", "+30").unwrap();
        assert_eq!(normalized, "+306981234567");
        assert_eq!(format_e164_pretty(&normalized), "+30 698 123 4567");

        // European 00 prefix
        let normalized_00 = normalize_e164("0030 210 1234567", "+30").unwrap();
        assert_eq!(normalized_00, "+302101234567");

        // Already with +
        let us_phone = normalize_e164("+1 (555) 234-5678", "+30").unwrap();
        assert_eq!(us_phone, "+15552345678");
        assert_eq!(format_e164_pretty(&us_phone), "+1 (555) 234-5678");
    }

    #[test]
    fn test_formatting_preferences_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        FormattingPreferencesStore::init_schema(&conn).unwrap();

        FormattingPreferencesStore::save_preferences(
            &conn,
            "STORE_ATH_01",
            DateFormatStyle::DayMonthYearSlash,
            '.',
            ',',
            "+30",
        ).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM system_i18n_formatting_preferences WHERE tenant_id = 'STORE_ATH_01'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }
}
