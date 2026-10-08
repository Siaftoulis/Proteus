//! Types and Enums for Cold Chain HACCP Telemetry.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use serde::{Deserialize, Serialize};

/// Cold storage category and regulatory temperature boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColdStorageType {
    /// Deep Freeze (-25.0°C to -18.0°C): Ice cream, frozen meat, poultry, fish
    DeepFreeze,
    /// Chilled (0.0°C to +4.0°C): Fresh dairy, meat cuts, prepared salads
    Chilled,
    /// Controlled Ambient (+15.0°C to +25.0°C): Confectionery, dry goods, olive oil
    ControlledAmbient,
    /// Pharma Cold Chain (+2.0°C to +8.0°C): Vaccines, insulin, biologicals
    PharmaCold,
}

impl ColdStorageType {
    pub fn default_range(&self) -> (f64, f64) {
        match self {
            Self::DeepFreeze => (-25.0, -18.0),
            Self::Chilled => (0.0, 4.0),
            Self::ControlledAmbient => (15.0, 25.0),
            Self::PharmaCold => (2.0, 8.0),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DeepFreeze => "Κατάψυξη (-25°C έως -18°C)",
            Self::Chilled => "Συντήρηση (0°C έως +4°C)",
            Self::ControlledAmbient => "Ελεγχόμενη (+15°C έως +25°C)",
            Self::PharmaCold => "Φάρμακα / Βιολογικά (+2°C έως +8°C)",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeepFreeze => "DEEP_FREEZE",
            Self::Chilled => "CHILLED",
            Self::ControlledAmbient => "CONTROLLED_AMBIENT",
            Self::PharmaCold => "PHARMA_COLD",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "DEEP_FREEZE" => Self::DeepFreeze,
            "CHILLED" => Self::Chilled,
            "CONTROLLED_AMBIENT" => Self::ControlledAmbient,
            "PHARMA_COLD" => Self::PharmaCold,
            _ => Self::Chilled,
        }
    }
}

/// HACCP breach excursion severity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreachSeverity {
    /// Minor deviation (excursion <= 2.0°C)
    MinorWarning,
    /// Major deviation (excursion > 2.0°C and <= 5.0°C)
    MajorExcursion,
    /// Critical spoilage danger (excursion > 5.0°C)
    CriticalSpoilage,
}

impl BreachSeverity {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MinorWarning => "Ήπια Απόκλιση",
            Self::MajorExcursion => "Σημαντική Υπέρβαση",
            Self::CriticalSpoilage => "Κρίσιμος Κίνδυνος Αλλοίωσης",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MinorWarning => "MINOR_WARNING",
            Self::MajorExcursion => "MAJOR_EXCURSION",
            Self::CriticalSpoilage => "CRITICAL_SPOILAGE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "MINOR_WARNING" => Self::MinorWarning,
            "MAJOR_EXCURSION" => Self::MajorExcursion,
            "CRITICAL_SPOILAGE" => Self::CriticalSpoilage,
            _ => Self::MinorWarning,
        }
    }
}

/// Operational status of an identified HACCP temperature breach.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreachStatus {
    Active,
    Acknowledged,
    Resolved,
    QuarantineMandated,
}

impl BreachStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Active => "Ενεργό Συμβάν",
            Self::Acknowledged => "Σε Διερεύνηση",
            Self::Resolved => "Επιλύθηκε",
            Self::QuarantineMandated => "Εντολή Καραντίνας Προϊόντων",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Acknowledged => "ACKNOWLEDGED",
            Self::Resolved => "RESOLVED",
            Self::QuarantineMandated => "QUARANTINE_MANDATED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ACTIVE" => Self::Active,
            "ACKNOWLEDGED" => Self::Acknowledged,
            "RESOLVED" => Self::Resolved,
            "QUARANTINE_MANDATED" => Self::QuarantineMandated,
            _ => Self::Active,
        }
    }
}

/// Registered IoT cold-chain sensor metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColdChainSensor {
    pub sensor_id: String,
    pub target_id: String,
    pub storage_type: ColdStorageType,
    pub min_temp_celsius: f64,
    pub max_temp_celsius: f64,
    pub min_humidity_pct: Option<f64>,
    pub max_humidity_pct: Option<f64>,
    pub is_active: bool,
    pub last_reading_epoch: Option<i64>,
}

impl ColdChainSensor {
    pub fn new(
        sensor_id: impl Into<String>,
        target_id: impl Into<String>,
        storage_type: ColdStorageType,
    ) -> Self {
        let (min, max) = storage_type.default_range();
        Self {
            sensor_id: sensor_id.into(),
            target_id: target_id.into(),
            storage_type,
            min_temp_celsius: min,
            max_temp_celsius: max,
            min_humidity_pct: None,
            max_humidity_pct: None,
            is_active: true,
            last_reading_epoch: None,
        }
    }
}

/// Immutable telemetry packet recorded from an IoT sensor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryReading {
    pub reading_id: String,
    pub sensor_id: String,
    pub target_id: String,
    pub temperature_celsius: f64,
    pub humidity_pct: Option<f64>,
    pub door_open: bool,
    pub battery_level_pct: Option<u8>,
    pub recorded_at: String,
    pub recorded_epoch: i64,
    pub merkle_hash: String,
}

/// Formal HACCP excursion incident record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HaccpBreachEvent {
    pub breach_id: String,
    pub sensor_id: String,
    pub target_id: String,
    pub severity: BreachSeverity,
    pub status: BreachStatus,
    pub excursion_temp: f64,
    pub threshold_limit: f64,
    pub started_at: String,
    pub resolved_at: Option<String>,
    pub corrective_action_note: Option<String>,
    pub operator_name: Option<String>,
    pub merkle_hash: String,
}

/// Regulatory compliance certificate generated over an audited time window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HaccpComplianceCertificate {
    pub certificate_id: String,
    pub target_id: String,
    pub time_window_start: String,
    pub time_window_end: String,
    pub total_readings: usize,
    pub in_spec_count: usize,
    pub breach_count: usize,
    pub in_spec_percentage: f64,
    pub merkle_root_hash: String,
    pub is_compliant: bool,
}
