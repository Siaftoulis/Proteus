//! Data Types, Enums and Digital Signatures for Shipping Notes & Waybills.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Purpose of transport according to tax and logistics regulations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportPurpose {
    Sale,                    // Πώληση
    Repair,                  // Επισκευή / Service
    TransferBetweenBranches, // Ενδοδιακίνηση
    ReturnToSupplier,        // Επιστροφή σε Προμηθευτή
    Consignment,             // Παρακαταθήκη
    Sample,                  // Δειγματισμός
}

impl TransportPurpose {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Sale => "SALE",
            Self::Repair => "REPAIR",
            Self::TransferBetweenBranches => "BRANCH_TRANSFER",
            Self::ReturnToSupplier => "SUPPLIER_RETURN",
            Self::Consignment => "CONSIGNMENT",
            Self::Sample => "SAMPLE",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sale => "Πώληση",
            Self::Repair => "Επισκευή / Service",
            Self::TransferBetweenBranches => "Ενδοδιακίνηση Υποκαταστημάτων",
            Self::ReturnToSupplier => "Επιστροφή σε Προμηθευτή",
            Self::Consignment => "Παρακαταθήκη",
            Self::Sample => "Δειγματισμός",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "SALE" => Self::Sale,
            "REPAIR" => Self::Repair,
            "BRANCH_TRANSFER" => Self::TransferBetweenBranches,
            "SUPPLIER_RETURN" => Self::ReturnToSupplier,
            "CONSIGNMENT" => Self::Consignment,
            "SAMPLE" => Self::Sample,
            _ => Self::Sale,
        }
    }
}

/// Operational state of a dispatch / waybill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DispatchStatus {
    Draft,      // Πρόχειρο
    Dispatched, // Απεστάλη
    InTransit,  // Σε Διαμετακόμιση
    Delivered,  // Παραδόθηκε
    Cancelled,  // Ακυρώθηκε
}

impl DispatchStatus {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Dispatched => "DISPATCHED",
            Self::InTransit => "IN_TRANSIT",
            Self::Delivered => "DELIVERED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Draft => "Πρόχειρο",
            Self::Dispatched => "Απεστάλη",
            Self::InTransit => "Σε Διαμετακόμιση",
            Self::Delivered => "Παραδόθηκε",
            Self::Cancelled => "Ακυρώθηκε",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "DRAFT" => Self::Draft,
            "DISPATCHED" => Self::Dispatched,
            "IN_TRANSIT" => Self::InTransit,
            "DELIVERED" => Self::Delivered,
            "CANCELLED" => Self::Cancelled,
            _ => Self::Draft,
        }
    }
}

/// Logistics unit of measure for shipped items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShippingUnit {
    Piece,    // Τεμάχιο (τεμ)
    Pack,     // Πακέτο
    Box,      // Κιβώτιο
    Kilogram, // Κιλό (kg)
    Meter,    // Μέτρο (m)
    Pallet,   // Παλέτα
}

impl ShippingUnit {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Piece => "PCS",
            Self::Pack => "PACK",
            Self::Box => "BOX",
            Self::Kilogram => "KG",
            Self::Meter => "MTR",
            Self::Pallet => "PAL",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Piece => "τεμ",
            Self::Pack => "πακέτο",
            Self::Box => "κιβώτιο",
            Self::Kilogram => "kg",
            Self::Meter => "m",
            Self::Pallet => "παλέτα",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "PCS" => Self::Piece,
            "PACK" => Self::Pack,
            "BOX" => Self::Box,
            "KG" => Self::Kilogram,
            "MTR" => Self::Meter,
            "PAL" => Self::Pallet,
            _ => Self::Piece,
        }
    }
}

/// Shipped item line in a digital waybill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShippingNoteItem {
    pub id: String,
    pub note_id: String,
    pub line_number: u32,
    pub sku: String,
    pub description: String,
    pub unit: ShippingUnit,
    pub quantity: f64,
    pub serial_numbers: Vec<String>,
    pub batch_lot: Option<String>,
    pub notes: Option<String>,
}

/// Complete Digital Shipping Note document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShippingNote {
    pub id: String,
    pub note_number: String,
    pub issuer_afm: String,
    pub issuer_name: String,
    pub issuer_address: String,
    pub recipient_afm: String,
    pub recipient_name: String,
    pub recipient_address: String,
    pub vehicle_plate: String,
    pub driver_name: String,
    pub departure_time: String,
    pub estimated_arrival: Option<String>,
    pub actual_arrival: Option<String>,
    pub purpose: TransportPurpose,
    pub gross_weight_kg: Option<f64>,
    pub packages_count: u32,
    pub mydata_mark: Option<String>,
    pub qr_payload: String,
    pub digital_signature_hash: String,
    pub status: DispatchStatus,
    pub recipient_signature_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ShippingNote {
    /// Constructs a new draft shipping note with auto-generated identifiers.
    pub fn new(
        note_number: String,
        issuer_afm: String,
        issuer_name: String,
        issuer_address: String,
        recipient_afm: String,
        recipient_name: String,
        recipient_address: String,
        vehicle_plate: String,
        driver_name: String,
        purpose: TransportPurpose,
        packages_count: u32,
        gross_weight_kg: Option<f64>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let sig = compute_transport_signature(
            &issuer_afm,
            &recipient_afm,
            &vehicle_plate,
            &now,
            purpose.code(),
            packages_count,
        );
        let qr = generate_iapr_qr_payload(
            None,
            &issuer_afm,
            &recipient_afm,
            &now,
            &vehicle_plate,
            packages_count,
            &sig,
        );

        Self {
            id,
            note_number,
            issuer_afm,
            issuer_name,
            issuer_address,
            recipient_afm,
            recipient_name,
            recipient_address,
            vehicle_plate,
            driver_name,
            departure_time: now.clone(),
            estimated_arrival: None,
            actual_arrival: None,
            purpose,
            gross_weight_kg,
            packages_count,
            mydata_mark: None,
            qr_payload: qr,
            digital_signature_hash: sig,
            status: DispatchStatus::Draft,
            recipient_signature_note: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Computes a deterministic SHA-256 seal for the transport document.
pub fn compute_transport_signature(
    issuer_afm: &str,
    recipient_afm: &str,
    vehicle_plate: &str,
    departure_time: &str,
    purpose_code: &str,
    packages_count: u32,
) -> String {
    let mut hasher = Sha256::new();
    let raw = format!(
        "PROTEUS-LOGISTICS|{}|{}|{}|{}|{}|{}",
        issuer_afm.trim(),
        recipient_afm.trim(),
        vehicle_plate.trim().to_uppercase(),
        departure_time.trim(),
        purpose_code,
        packages_count
    );
    hasher.update(raw.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Formats the official IAPR / myDATA & e-CMR standard QR code string.
pub fn generate_iapr_qr_payload(
    mydata_mark: Option<&str>,
    issuer_afm: &str,
    recipient_afm: &str,
    departure_time: &str,
    vehicle_plate: &str,
    packages_count: u32,
    signature_hash: &str,
) -> String {
    let mark_str = mydata_mark.unwrap_or("PENDING_OFFLINE");
    let sig_short = if signature_hash.len() > 16 {
        &signature_hash[..16]
    } else {
        signature_hash
    };
    format!(
        "AADE-CMR|MARK:{}|ISSUER:{}|RECV:{}|DATE:{}|VEH:{}|ITEMS:{}|SIG:{}",
        mark_str,
        issuer_afm.trim(),
        recipient_afm.trim(),
        departure_time.trim(),
        vehicle_plate.trim().to_uppercase(),
        packages_count,
        sig_short
    )
}
