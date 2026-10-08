//! Core types for the Serial Number Genealogy & RMA Engine.
//! Tracks the complete physical lifecycle of high-value components and devices:
//! Supplier -> Warehouse -> Installed in Customer Ticket -> RMA Claim -> Supplier Replacement.

use serde::{Deserialize, Serialize};

/// Operational lifecycle stage of a tracked serialized component or device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentLifecycleStage {
    SupplierIntake {
        supplier: String,
        invoice_ref: Option<String>,
    },
    WarehouseStock {
        shelf_location: String,
    },
    InstalledInCustomerDevice {
        customer_name: String,
        device_model: String,
        ticket_number: i64,
    },
    RmaClaimInitiated {
        fault_description: String,
        claimed_by: String,
    },
    RmaReplacedBySupplier {
        replacement_serial: String,
        credit_invoice: Option<String>,
    },
    RmaCreditNoteIssued {
        credit_note_number: String,
        amount_eur: f64,
    },
    Disposed {
        reason: String,
    },
}

impl ComponentLifecycleStage {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SupplierIntake { .. } => "Παραλαβή Προμηθευτή",
            Self::WarehouseStock { .. } => "Απόθεμα Αποθήκης",
            Self::InstalledInCustomerDevice { .. } => "Εγκατεστημένο σε Συσκευή",
            Self::RmaClaimInitiated { .. } => "Αίτηση Εγγύησης / RMA",
            Self::RmaReplacedBySupplier { .. } => "Αντικατάσταση από Προμηθευτή",
            Self::RmaCreditNoteIssued { .. } => "Έκδοση Πιστωτικού Τιμολογίου",
            Self::Disposed { .. } => "Ανακύκλωση / Διαγραφή",
        }
    }
}

/// Dynamic warranty status calculation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarrantyStatus {
    Valid { days_remaining: i64 },
    Expired { days_expired: i64 },
}

/// An individual immutable event along the component's genealogy lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenealogyEvent {
    pub event_id: String,
    pub serial_number: String,
    pub stage: ComponentLifecycleStage,
    pub description: String,
    pub actor: String,
    pub timestamp: i64,
}

/// Full genealogy profile for a serialized part or device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialGenealogy {
    pub serial_number: String,
    pub sku: String,
    pub name: String,
    pub supplier_name: String,
    pub warranty_months: u32,
    pub intake_date: i64,
    pub current_stage: ComponentLifecycleStage,
    pub updated_at: i64,
}

impl SerialGenealogy {
    /// Computes whether the component is currently within its manufacturer warranty window.
    pub fn warranty_status(&self, now_ms: i64) -> WarrantyStatus {
        let warranty_duration_ms = self.warranty_months as i64 * 30 * 24 * 3600 * 1000;
        let expiry_ms = self.intake_date + warranty_duration_ms;
        let diff_ms = expiry_ms - now_ms;
        let diff_days = diff_ms / (24 * 3600 * 1000);

        if diff_ms >= 0 {
            WarrantyStatus::Valid {
                days_remaining: diff_days,
            }
        } else {
            WarrantyStatus::Expired {
                days_expired: -diff_days,
            }
        }
    }
}
