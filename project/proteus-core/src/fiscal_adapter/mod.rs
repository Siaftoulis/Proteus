//! Global Multi-Jurisdiction Fiscal Adapter & Universal ERP Bridge (Phase 27).
//! Provides country-specific tax rules, fiscal signature strategies, and accounting hooks.

pub mod currency;
pub mod jurisdiction;
pub mod types;

pub use currency::{CurrencyCode, CurrencyLedger, ExchangeRateRecord, MonetaryAmount};
pub use jurisdiction::JurisdictionEngine;
pub use types::{
    FiscalDocumentKind, FiscalEntityProfile, FiscalJurisdiction, FiscalSignatureMechanism,
    TaxRateBracket,
};
