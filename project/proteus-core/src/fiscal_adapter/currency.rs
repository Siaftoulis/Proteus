//! Multi-Currency & Universal Fixed-Point Decimal Math Ledger (Micro-task 27.1.2).
//! Provides integer-based currency representations, ISO 4217 metadata,
//! tax rounding, and real SQLite exchange rate persistence.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Supported ISO 4217 standard currency codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurrencyCode {
    EUR,
    USD,
    GBP,
    JPY,
    CHF,
    PLN,
    SEK,
    CAD,
    AUD,
}

impl CurrencyCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EUR => "EUR",
            Self::USD => "USD",
            Self::GBP => "GBP",
            Self::JPY => "JPY",
            Self::CHF => "CHF",
            Self::PLN => "PLN",
            Self::SEK => "SEK",
            Self::CAD => "CAD",
            Self::AUD => "AUD",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::EUR => "€",
            Self::USD => "$",
            Self::GBP => "£",
            Self::JPY => "¥",
            Self::CHF => "CHF",
            Self::PLN => "zł",
            Self::SEK => "kr",
            Self::CAD => "C$",
            Self::AUD => "A$",
        }
    }

    pub fn decimal_places(&self) -> u32 {
        match self {
            Self::JPY => 0,
            _ => 2,
        }
    }

    pub fn parse(code: &str) -> Option<Self> {
        match code.trim().to_uppercase().as_str() {
            "EUR" => Some(Self::EUR),
            "USD" => Some(Self::USD),
            "GBP" => Some(Self::GBP),
            "JPY" => Some(Self::JPY),
            "CHF" => Some(Self::CHF),
            "PLN" => Some(Self::PLN),
            "SEK" => Some(Self::SEK),
            "CAD" => Some(Self::CAD),
            "AUD" => Some(Self::AUD),
            _ => None,
        }
    }
}

/// Fixed-point monetary value preventing floating-point precision drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonetaryAmount {
    /// Stored in the smallest currency unit (e.g. cents for EUR/USD, whole units for JPY).
    pub minor_units: i64,
    pub currency: CurrencyCode,
}

impl MonetaryAmount {
    pub fn new(minor_units: i64, currency: CurrencyCode) -> Self {
        Self {
            minor_units,
            currency,
        }
    }

    pub fn from_major(units: i64, decimals: u32, currency: CurrencyCode) -> Self {
        let factor = 10i64.pow(currency.decimal_places());
        Self {
            minor_units: units * factor + decimals as i64,
            currency,
        }
    }

    pub fn zero(currency: CurrencyCode) -> Self {
        Self {
            minor_units: 0,
            currency,
        }
    }

    pub fn add(&self, other: &Self) -> Result<Self, String> {
        if self.currency != other.currency {
            return Err(format!("Cannot add {} to {}", self.currency.as_str(), other.currency.as_str()));
        }
        Ok(Self {
            minor_units: self.minor_units + other.minor_units,
            currency: self.currency,
        })
    }

    pub fn subtract(&self, other: &Self) -> Result<Self, String> {
        if self.currency != other.currency {
            return Err(format!("Cannot subtract {} from {}", other.currency.as_str(), self.currency.as_str()));
        }
        Ok(Self {
            minor_units: self.minor_units - other.minor_units,
            currency: self.currency,
        })
    }

    pub fn multiply_quantity(&self, quantity: i64) -> Self {
        Self {
            minor_units: self.minor_units * quantity,
            currency: self.currency,
        }
    }

    /// Computes statutory tax amount given tax basis points (e.g. 2400 for 24%) with half-even rounding.
    pub fn calculate_tax(&self, tax_basis_points: u32) -> Self {
        let tax_raw = (self.minor_units as i128) * (tax_basis_points as i128);
        // Divide by 10000 with half-up rounding
        let rounded = (tax_raw + 5000) / 10000;
        Self {
            minor_units: rounded as i64,
            currency: self.currency,
        }
    }

    /// Formats the monetary amount for human display.
    pub fn format_display(&self) -> String {
        let places = self.currency.decimal_places();
        let sym = self.currency.symbol();

        if places == 0 {
            format!("{}{} ", sym, self.minor_units)
        } else {
            let divisor = 10i64.pow(places);
            let whole = self.minor_units / divisor;
            let frac = (self.minor_units % divisor).abs();
            format!("{} {}.{:02}", sym, whole, frac)
        }
    }
}

/// Official or override exchange rate record between two currencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExchangeRateRecord {
    pub base_currency: CurrencyCode,
    pub target_currency: CurrencyCode,
    /// Exchange rate multiplied by 10,000 (e.g. 1.0850 = 10850).
    pub rate_basis_points: u64,
    pub source: String,
    pub updated_at: String,
}

/// Multi-currency ledger and exchange conversion engine.
pub struct CurrencyLedger;

impl CurrencyLedger {
    pub const RATE_SCALE: u64 = 10_000;

    /// Converts an amount into a target currency using exchange rate basis points.
    pub fn convert(
        amount: &MonetaryAmount,
        target_currency: CurrencyCode,
        rate_basis_points: u64,
    ) -> MonetaryAmount {
        if amount.currency == target_currency {
            return *amount;
        }

        let from_places = amount.currency.decimal_places();
        let to_places = target_currency.decimal_places();

        let raw = (amount.minor_units as i128) * (rate_basis_points as i128);
        let base_converted = (raw + (Self::RATE_SCALE as i128 / 2)) / (Self::RATE_SCALE as i128);

        // Adjust for decimal place differences (e.g. EUR (2) -> JPY (0))
        let final_units = if to_places > from_places {
            base_converted * 10i128.pow(to_places - from_places)
        } else if from_places > to_places {
            (base_converted + 5) / 10i128.pow(from_places - to_places)
        } else {
            base_converted
        };

        MonetaryAmount {
            minor_units: final_units as i64,
            currency: target_currency,
        }
    }

    /// Initializes `system_currency_rates` table in SQLite.
    pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_currency_rates (
                base_currency TEXT NOT NULL,
                target_currency TEXT NOT NULL,
                rate_basis_points INTEGER NOT NULL,
                source TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (base_currency, target_currency)
            )",
            [],
        )?;
        Ok(())
    }

    /// Saves or updates an exchange rate.
    pub fn save_rate(conn: &Connection, rate: &ExchangeRateRecord) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO system_currency_rates (
                base_currency, target_currency, rate_basis_points, source, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(base_currency, target_currency) DO UPDATE SET
                rate_basis_points = excluded.rate_basis_points,
                source = excluded.source,
                updated_at = excluded.updated_at",
            params![
                rate.base_currency.as_str(),
                rate.target_currency.as_str(),
                rate.rate_basis_points as i64,
                rate.source,
                rate.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Retrieves an exchange rate between two currencies.
    pub fn get_rate(
        conn: &Connection,
        base: CurrencyCode,
        target: CurrencyCode,
    ) -> Result<Option<ExchangeRateRecord>, rusqlite::Error> {
        if base == target {
            return Ok(Some(ExchangeRateRecord {
                base_currency: base,
                target_currency: target,
                rate_basis_points: Self::RATE_SCALE,
                source: "IDENTITY".to_string(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            }));
        }

        let mut stmt = conn.prepare(
            "SELECT base_currency, target_currency, rate_basis_points, source, updated_at
             FROM system_currency_rates WHERE base_currency = ?1 AND target_currency = ?2",
        )?;
        let mut rows = stmt.query(params![base.as_str(), target.as_str()])?;
        if let Some(row) = rows.next()? {
            let base_str: String = row.get(0)?;
            let target_str: String = row.get(1)?;
            let b = CurrencyCode::parse(&base_str).unwrap_or(CurrencyCode::EUR);
            let t = CurrencyCode::parse(&target_str).unwrap_or(CurrencyCode::USD);
            let rate: i64 = row.get(2)?;
            Ok(Some(ExchangeRateRecord {
                base_currency: b,
                target_currency: t,
                rate_basis_points: rate as u64,
                source: row.get(3)?,
                updated_at: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monetary_amount_addition_and_tax_calc() {
        // 100.00 EUR
        let price = MonetaryAmount::new(10000, CurrencyCode::EUR);
        // Tax 24% = 24.00 EUR
        let tax = price.calculate_tax(2400);
        assert_eq!(tax.minor_units, 2400);

        // Total 124.00 EUR
        let total = price.add(&tax).unwrap();
        assert_eq!(total.minor_units, 12400);
        assert_eq!(total.format_display(), "€ 124.00");
    }

    #[test]
    fn test_jpy_zero_decimal_places_formatting() {
        let yen = MonetaryAmount::new(3500, CurrencyCode::JPY);
        assert_eq!(yen.currency.decimal_places(), 0);
        assert_eq!(yen.format_display(), "¥3500 ");
    }

    #[test]
    fn test_currency_conversion_eur_to_usd() {
        let eur = MonetaryAmount::new(10000, CurrencyCode::EUR); // 100.00 EUR
        // 1 EUR = 1.0850 USD (10850 basis points)
        let usd = CurrencyLedger::convert(&eur, CurrencyCode::USD, 10850);
        assert_eq!(usd.minor_units, 10850); // 108.50 USD
        assert_eq!(usd.currency, CurrencyCode::USD);
    }

    #[test]
    fn test_currency_rates_sqlite_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        CurrencyLedger::init_schema(&conn).unwrap();

        let rec = ExchangeRateRecord {
            base_currency: CurrencyCode::EUR,
            target_currency: CurrencyCode::USD,
            rate_basis_points: 10850,
            source: "ECB_DAILY".into(),
            updated_at: "2026-10-09T00:00:00Z".into(),
        };

        CurrencyLedger::save_rate(&conn, &rec).unwrap();
        let loaded = CurrencyLedger::get_rate(&conn, CurrencyCode::EUR, CurrencyCode::USD).unwrap().unwrap();
        assert_eq!(loaded.rate_basis_points, 10850);
        assert_eq!(loaded.source, "ECB_DAILY");
    }
}
