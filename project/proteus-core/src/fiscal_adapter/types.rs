//! Multi-Jurisdiction Fiscal Adapter types and tax models (Micro-task 27.1.1).
//! Models global tax regimes, digital signatures, and fiscal profiles.

use serde::{Deserialize, Serialize};

/// Supported tax and compliance jurisdictions across the Proteus ecosystem.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FiscalJurisdiction {
    /// Greece: AADE myDATA & A.1155 POS-ERP interconnected regime.
    GreeceAade,
    /// Germany: KassenSichV TSE (Technical Security System) & DSFinV-K.
    GermanyKassenSichV,
    /// France: NF525 certified cash register ledger & anti-fraud chain.
    FranceNf525,
    /// United States: Destination-based / origin-based sales tax regimes.
    UsSalesTax { state_code: String },
    /// Global: Jurisdiction-agnostic standard fiscal receipt & reverse-charge VAT.
    GlobalGeneric,
}

impl FiscalJurisdiction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::GreeceAade => "Greece (AADE myDATA / A.1155)",
            Self::GermanyKassenSichV => "Germany (KassenSichV TSE)",
            Self::FranceNf525 => "France (NF525 Anti-Fraud)",
            Self::UsSalesTax { .. } => "United States (Sales Tax)",
            Self::GlobalGeneric => "Global Standard (Universal)",
        }
    }

    pub fn country_code(&self) -> &'static str {
        match self {
            Self::GreeceAade => "GR",
            Self::GermanyKassenSichV => "DE",
            Self::FranceNf525 => "FR",
            Self::UsSalesTax { .. } => "US",
            Self::GlobalGeneric => "XX",
        }
    }

    pub fn signature_mechanism(&self) -> FiscalSignatureMechanism {
        match self {
            Self::GreeceAade => FiscalSignatureMechanism::AadeQrCodeSignature,
            Self::GermanyKassenSichV => FiscalSignatureMechanism::TseHardwareModule,
            Self::FranceNf525 => FiscalSignatureMechanism::Ed25519MerkleChain,
            Self::UsSalesTax { .. } => FiscalSignatureMechanism::None,
            Self::GlobalGeneric => FiscalSignatureMechanism::Sha256Digest,
        }
    }
}

/// Digital verification and audit signature strategy required by law.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiscalSignatureMechanism {
    None,
    Sha256Digest,
    Ed25519MerkleChain,
    AadeQrCodeSignature,
    TseHardwareModule,
}

/// Standardized classification of business transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiscalDocumentKind {
    RetailReceipt,
    ServiceInvoice,
    ShippingDeliveryNote,
    CreditNote,
    ProformaReceipt,
}

impl FiscalDocumentKind {
    pub fn code(&self) -> &'static str {
        match self {
            Self::RetailReceipt => "REC",
            Self::ServiceInvoice => "INV",
            Self::ShippingDeliveryNote => "DEL",
            Self::CreditNote => "CRD",
            Self::ProformaReceipt => "PRO",
        }
    }
}

/// Individual tax rate bracket specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaxRateBracket {
    pub id: String,
    pub name: String,
    /// Percentage in basis points (e.g. 2400 = 24.00%).
    pub basis_points: u32,
    pub category_code: String,
    pub is_exempt: bool,
}

impl TaxRateBracket {
    pub fn new(id: &str, name: &str, basis_points: u32, category: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            basis_points,
            category_code: category.to_string(),
            is_exempt: basis_points == 0,
        }
    }

    pub fn decimal_rate(&self) -> f64 {
        self.basis_points as f64 / 10000.0
    }
}

/// Registered legal entity fiscal profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiscalEntityProfile {
    pub profile_id: String,
    pub tax_id: String,
    pub legal_name: String,
    pub trade_name: String,
    pub jurisdiction: FiscalJurisdiction,
    pub registered_address: String,
    pub city: String,
    pub postal_code: String,
    pub country_code: String,
    pub tax_office: String,
    pub is_vat_registered: bool,
    pub default_currency: String,
    pub updated_at: String,
}
